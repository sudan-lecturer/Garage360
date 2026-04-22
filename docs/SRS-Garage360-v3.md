# Software Requirements Specification — Garage360
**Version:** 3.0.0
**Status:** Active
**Authors:** Product & Engineering Team
**Last Updated:** 2025

---

## Document Control

| Version | Change Summary |
|---|---|
| 1.0.0 | Initial SRS — Next.js monolith |
| 2.0.0 | Replatformed to Rust + React PWA, multi-tenant, Docker |
| 3.0.0 | Added: Customer types, HR/Payroll, stock thresholds, PO full lifecycle + QA, Asset Management, configurable reports, Excel import/export, inventory addition form |

---

## Table of Contents

1. Introduction & Goals
2. Architecture Overview
3. Tech Stack
4. Multi-Tenancy Design
5. Feature Flag System
6. UI Design System — Industrial Brutalism
7. PWA Requirements
8. Authentication & RBAC
9. Module Specifications (detailed)
   - 9.1 Customer Management
   - 9.2 Vehicle Management
   - 9.3 Job Card Lifecycle
   - 9.4 Inventory Management
   - 9.5 Purchase Order Management
   - 9.6 Billing & Invoicing
   - 9.7 Digital Vehicle Inspection (DVI)
   - 9.8 Asset Management *(new)*
   - 9.9 HR & Payroll *(new)*
   - 9.10 Reports & Analytics
   - 9.11 Excel Import / Export *(new)*
   - 9.12 Settings & Configuration
10. Database Schema Design
11. Backend Module Structure (Rust)
12. Frontend Module Structure (React)
13. Docker Architecture
14. Security Requirements
15. Development Workflow
16. Agent Skills Plan

---

## 1. Introduction & Goals

### 1.1 Project Overview
**Garage360** is a multi-tenant, cloud-native SaaS platform for vehicle service centers and garages. It delivers a complete workshop operating system covering CRM (individual and corporate clients), job card lifecycle, digital vehicle inspection, inventory management with stock intelligence, a full purchase order workflow including QA sign-off, billing, HR and payroll, organisational asset tracking, configurable analytics, and Excel-based data import/export — all under a single deployable product with per-tenant data isolation.

### 1.2 Design Principles
- **Mobile-first, floor-ready:** Mechanics and cashiers use the system on phones and tablets on the workshop floor. Every screen works at 375px width before desktop.
- **One codebase, any workshop:** Deploy once for SaaS or hand a Docker stack to a client for on-premise. Same code, different environment variables.
- **Data isolation is non-negotiable:** Each workshop (tenant) owns its own PostgreSQL database. No shared tables, no shared connection pool, no possible cross-contamination.
- **Feature flags over feature branches:** Super Admin enables/disables entire modules per tenant. Disabled modules are invisible — not greyed out.
- **Excel as a first-class citizen:** Every significant list can be exported. Core entities can be bulk-imported. The workshop world runs on Excel.

### 1.3 Out of Scope (v3.0)
- Payment gateway integration (third-party, future phase)
- External parts marketplace procurement
- Fleet management beyond individual vehicle records
- Scheduled email reports (noted as roadmap item)
- Multi-currency (flag exists, implementation future phase)

---

## 2. Architecture Overview

### 2.1 System Topology

```
┌─────────────────────────────────────────────────────────┐
│                      Docker Host                         │
│                                                         │
│  ┌──────────────────┐      ┌───────────────────────┐   │
│  │   React PWA      │      │   Rust API (Axum)     │   │
│  │   Nginx:80/443   │─────▶│   Port 8080           │   │
│  └──────────────────┘      └───────────┬───────────┘   │
│                                        │                │
│              ┌─────────────────────────▼─────────────┐  │
│              │           Tenant Router               │  │
│              │  JWT → tenant_id → DB pool lookup     │  │
│              └────┬───────────────┬──────────────┬───┘  │
│                   │               │              │      │
│           ┌───────▼──┐    ┌───────▼──┐   ┌──────▼───┐  │
│           │tenant_a  │    │tenant_b  │   │tenant_N  │  │
│           │(postgres)│    │(postgres)│   │(postgres)│  │
│           └──────────┘    └──────────┘   └──────────┘  │
│                                                         │
│  ┌────────────────────┐    ┌───────────────────────┐   │
│  │  Control DB        │    │  Redis                │   │
│  │  (postgres)        │    │  sessions + cache     │   │
│  │  tenant registry   │    │  + flag cache         │   │
│  │  feature flags     │    │  + rate limiting      │   │
│  │  super admin       │    │                       │   │
│  └────────────────────┘    └───────────────────────┘   │
│                                                         │
│  ┌────────────────────┐                                 │
│  │  migration-runner  │  (one-shot, runs on deploy)     │
│  └────────────────────┘                                 │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Database Strategy — Separate Database per Tenant
Each tenant is provisioned with a dedicated PostgreSQL database (`tenant_{slug}`). A central **Control DB** holds only: tenant registry, feature flags, and Super Admin accounts. Zero workshop business data in the Control DB.

**Tenant lifecycle:**
1. Super Admin creates tenant → API provisions `tenant_{slug}` database
2. Migration runner executes all migrations against the new DB
3. Tenant JWT signing key generated and stored (encrypted) in Control DB
4. Tenant is immediately operational

**Why this model was chosen:**
- Hard compliance boundary — data residency and regulatory requirements are trivially satisfied
- Independent backup/restore per tenant
- On-premise deployment is the same Docker stack with one tenant configured
- A breach of one tenant's data cannot expose another tenant's data

**Operational trade-offs accepted:**
- Schema migrations run against every tenant DB (handled by migration-runner service)
- Super Admin cross-tenant reports aggregate across connections (API side, not DB side)
- Connection pool management: one pool per active tenant, idle timeout after 10 minutes

### 2.3 Service Decomposition

| Service | Runtime | Responsibility |
|---|---|---|
| `api` | Rust (Axum) | All business logic, REST API, auth, tenant routing |
| `web` | React 19 + TypeScript (Nginx) | PWA frontend |
| `control-db` | PostgreSQL 16 | Tenant registry, feature flags, super admin |
| `tenant-db-N` | PostgreSQL 16 | All data for one workshop |
| `redis` | Redis 7 | JWT blocklist, rate limiting, flag cache, sessions |
| `migration-runner` | Rust binary | Runs sqlx migrations against all tenant DBs |
| `nginx` | Nginx Alpine | Reverse proxy, SSL termination, static files |

---

## 3. Tech Stack

### 3.1 Backend — Rust

| Concern | Crate | Reason |
|---|---|---|
| HTTP framework | `axum` | Tower-native, ergonomic, async-first |
| Async runtime | `tokio` | Industry standard |
| Database | `sqlx` | Compile-time query checking, async, PostgreSQL native |
| Migrations | `sqlx migrate` | Built-in, versioned, per-DB execution |
| Auth / JWT | `jsonwebtoken` | HS256/RS256 encode/decode |
| Password hashing | `argon2` | argon2id, industry standard |
| Validation | `validator` | Struct-level derive macros |
| Serialisation | `serde` + `serde_json` | Universal |
| Error handling | `thiserror` + `anyhow` | Typed in lib, dynamic in bin |
| Config | `dotenvy` + `config` | Layered env → file → defaults |
| Logging | `tracing` + `tracing-subscriber` | Structured, async-aware |
| Redis | `redis` (async feature) | Cache, sessions, rate limiting |
| UUID | `uuid` v7 | Time-sortable PKs |
| Excel read | `calamine` | Parse .xlsx uploads for import |
| Excel write | `rust_xlsxwriter` | Generate .xlsx exports |
| File storage | `object_store` | S3-compatible storage for documents/photos |
| Notifications | `lettre` (email) | Staff and customer notifications |
| Testing | `tokio-test`, sqlx test transactions | Unit + integration |

### 3.2 Frontend — React + TypeScript

| Concern | Library | Reason |
|---|---|---|
| Framework | React 19 | Concurrent features |
| Language | TypeScript 5 strict | Full type safety |
| Build | Vite 6 | Fast HMR, PWA plugin |
| Routing | React Router v7 | Nested routes |
| State | Zustand | Lightweight global state |
| Server state | TanStack Query v5 | Cache, optimistic updates |
| Forms | React Hook Form + Zod | Type-safe validation |
| UI primitives | shadcn/ui (customised) | Accessible, unstyled |
| Icons | lucide-react | Consistent set |
| Styling | Tailwind CSS v4 | Utility-first, design tokens |
| PWA | vite-plugin-pwa | SW, manifest, offline |
| Charts | Recharts | React-native charts |
| Tables | TanStack Table v8 | Virtualised, sortable, filterable |
| Excel export | `xlsx` (SheetJS) | Client-side xlsx generation |
| HTTP | Axios + typed hooks | Interceptors, retry |
| i18n | react-i18next | Multi-language ready |
| Camera | `react-webcam` | DVI + QA photo capture |

### 3.3 Infrastructure

| Component | Technology |
|---|---|
| Containerisation | Docker + Docker Compose |
| Web/proxy | Nginx Alpine |
| Database | PostgreSQL 16 Alpine |
| Cache | Redis 7 Alpine |
| File storage | MinIO (self-hosted S3-compatible) or AWS S3 |
| CI/CD | GitHub Actions |
| SSL | Let's Encrypt / Certbot or bring-your-own |

---

## 4. Multi-Tenancy Design

### 4.1 Tenant Identification
JWT payload on every request:
```json
{
  "sub": "user-uuid-v7",
  "tenant_id": "tenant-uuid-v7",
  "tenant_slug": "workshop-alpha",
  "role": "MECHANIC",
  "location_id": "location-uuid-v7",
  "exp": 1234567890
}
```

Axum middleware: extracts `tenant_id` → looks up DB URL from Redis (5-min TTL, fallback to Control DB) → injects `TenantDb` extractor. Every handler receiving `TenantDb` is cryptographically scoped to that tenant's database.

### 4.2 Control Plane API (Super Admin only — `/control/v1/`)
```
POST   /tenants                    Create + provision tenant DB
GET    /tenants                    List all tenants
GET    /tenants/:id                Tenant detail + DB health
PUT    /tenants/:id                Update name, plan, status
DELETE /tenants/:id                Soft-deactivate (DB preserved)
POST   /tenants/:id/migrate        Trigger migration run
GET    /feature-flags              List all global flags
PUT    /feature-flags/:key         Set global default
GET    /tenants/:id/feature-flags  List tenant overrides
PUT    /tenants/:id/feature-flags/:key  Set tenant override
```

### 4.3 Control DB Schema
```sql
tenants           — id, slug, name, db_url_encrypted, plan, is_active, created_at, updated_at
feature_flags     — id, tenant_id (nullable=global), flag_key, is_enabled, updated_at
super_admin_users — id, email, password_hash, is_active, created_at
control_audit_logs — id, admin_id, action, target_tenant_id, metadata, ip_address, created_at
```

---

## 5. Feature Flag System

### 5.1 Resolution
Priority: Tenant override → Global default → Hard-coded `false`

### 5.2 Complete Flag Registry

| Flag Key | Default | Description |
|---|---|---|
| `module.dvi` | `true` | Digital Vehicle Inspection |
| `module.purchases` | `true` | Purchase Orders + GRN + QA |
| `module.reports` | `true` | Analytics and report builder |
| `module.hr` | `false` | HR records + payroll |
| `module.assets` | `false` | Organisational asset management |
| `module.customer_portal` | `false` | Customer self-service portal |
| `module.loyalty` | `false` | Loyalty points |
| `billing.vat` | `false` | VAT line on invoices |
| `billing.multi_currency` | `false` | Multi-currency support |
| `jobs.approval_workflow` | `true` | Quote approval step |
| `jobs.dvi_required` | `false` | Block billing until DVI complete |
| `inventory.low_stock_alerts` | `true` | Low stock notifications |
| `purchases.qa_required` | `true` | QA sign-off mandatory before stocking |
| `purchases.approval_required` | `true` | PO approval before sending to supplier |
| `hr.payroll` | `false` | Payroll calculation (requires module.hr) |
| `hr.leave_management` | `false` | Leave requests and balances |
| `hr.attendance` | `false` | Clock-in/out attendance tracking |
| `assets.daily_inspection` | `true` | Daily inspection checklist (requires module.assets) |
| `export.excel` | `true` | Excel export on all list views |
| `import.excel` | `true` | Excel import for core entities |

### 5.3 Frontend Consumption
```
GET /api/v1/feature-flags  →  { "module.hr": false, "module.dvi": true, ... }
```
Zustand stores the flag map. Components gate with `useFeatureFlag('module.hr')`. Disabled modules are absent from navigation — not rendered at all.

---

## 6. UI Design System — Industrial Brutalism

### 6.1 Color Tokens
```css
--color-base:           #121416;   /* page background */
--color-surface:        #1E2022;   /* panels, cards */
--color-surface-alt:    #252829;   /* elevated surfaces */
--color-border:         #2E3235;   /* dividers */
--color-primary:        #FFD100;   /* Safety Yellow — primary CTA */
--color-secondary:      #FF5800;   /* High-Vis Orange — alerts */
--color-text-primary:   #F0F0F0;
--color-text-secondary: #9CA3AF;
--color-text-muted:     #6B7280;
--color-success:        #22C55E;
--color-danger:         #EF4444;
--color-warning:        #F59E0B;
--color-info:           #3B82F6;
```

### 6.2 Universal Stock Color Coding
Applied consistently across inventory list, job item picker, PO suggestions, and dashboard widgets:

| Status | Color | Condition | Badge |
|---|---|---|---|
| Healthy | `#22C55E` (green) | qty ≥ reorder_point | `IN STOCK` |
| Warning | `#F59E0B` (amber) | qty < reorder_point AND qty > min_stock | `LOW STOCK` |
| Critical | `#EF4444` (red) | qty ≤ min_stock | `CRITICAL` |
| Out of Stock | `#6B7280` (gray) | qty = 0 | `OUT OF STOCK` |

This four-state system is implemented as a computed field (`stock_status`) returned by all inventory API responses. The frontend never calculates it — the Rust service layer returns the enum value.

### 6.3 Typography
| Role | Font | Weight | Style |
|---|---|---|---|
| Headers / Nav | Space Grotesk | 700 | Uppercase |
| Body | Work Sans | 400/500 | Normal |
| Technical data | IBM Plex Mono | 400 | VINs, part nos, amounts, codes |

### 6.4 Component Rules
- Zero drop shadows — tonal shifts only
- 2px border radius on interactive elements
- Hover: bottom border 2px `--color-primary`
- Focus: outline 2px `--color-primary` offset 2px
- Currency/numeric: always IBM Plex Mono
- Status badges: filled pill, uppercase monospace label
- Tables: sharp rows, alternating `--color-surface` / `--color-surface-alt`

---

## 7. PWA Requirements

### 7.1 Capabilities
- Installable (Android, iOS, desktop Chrome)
- Offline: read-only access to job cards list, inventory list, and asset inspection checklists via service worker cache
- Background sync for form submissions queued while offline
- Push notifications: job status changes, stock alerts, PO approvals needed, asset defect reports, DVI completion
- App manifest: name "Garage360", theme `#121416`, display `standalone`, icons 192px + 512px

### 7.2 Mobile-First Navigation
- `< 768px`: bottom tab bar (max 5 tabs, overflow in "More" sheet)
- `768px – 1024px`: collapsible side rail
- `> 1024px`: full sidebar with section groups

### 7.3 Mobile-Specific UX
- Touch targets: minimum 44×44px
- Tables degrade to card lists on mobile
- Forms use native inputs (date pickers, number keypads)
- Swipe gestures on job card kanban columns
- Camera API: DVI photo capture, QA inspection photos, asset defect photos
- Barcode scan on inventory addition form (via device camera)

### 7.4 Performance Targets
| Metric | Target |
|---|---|
| Lighthouse PWA score | ≥ 90 |
| First Contentful Paint | < 1.5s |
| Time to Interactive | < 3.0s |
| Initial bundle (gzipped) | < 200KB |
| Offline core views | Job list, Inventory list, Asset checklist |

---

## 8. Authentication & RBAC

### 8.1 Auth Flow
1. `POST /api/v1/auth/login` — argon2id verify, check `is_active` on user + tenant
2. Issue access token (15 min) + refresh token (7 days, stored in Redis)
3. Refresh: single-use rotation, old token immediately invalidated
4. Logout: access token added to Redis blocklist (TTL = remaining expiry)

### 8.2 Roles

| Role | Scope | Key Permissions |
|---|---|---|
| `SUPER_ADMIN` | Control plane | Tenant management, global flags, platform-wide analytics |
| `OWNER` | Tenant | All permissions including HR sensitive data, payroll, asset CRUD |
| `ADMIN` | Tenant | All except payroll private fields, tenant billing settings |
| `MANAGER` | Tenant | Jobs, inventory, purchasing approval (if delegated), staff scheduling |
| `MECHANIC` | Tenant | Assigned jobs, DVI, asset daily inspection, defect reporting |
| `CASHIER` | Tenant | Invoicing, payment receipt, read-only job view |
| `HR_OFFICER` | Tenant | Employee records, payroll (flag-gated), leave management |
| `CUSTOMER` | Tenant | Own vehicles, job history, portal (if flag enabled) |

### 8.3 RBAC Enforcement
- Route-level: Axum `require_roles!([OWNER, ADMIN, MANAGER])` macro on each handler
- Resource-level: mechanics only update their assigned jobs; employees see only their own payroll
- Field-level: bank account details and salary figures visible only to OWNER, ADMIN, HR_OFFICER
- Frontend hides UI; API enforces — both layers mandatory

---

## 9. Module Specifications

---

### 9.1 Customer Management

#### Business Rules
- A customer is either an **Individual** or an **Organisation** — determined at creation, cannot be changed.
- Both types can own vehicles and have job cards.
- An Organisation has a primary contact person (the human the workshop talks to) plus company-level details.
- A customer may optionally be linked to a portal `User` account for self-service access.

#### Individual Customer Fields
| Field | Type | Required | Notes |
|---|---|---|---|
| customer_type | enum | Yes | INDIVIDUAL |
| full_name | text | Yes | |
| phone | text | Yes | Primary contact number |
| email | text | No | |
| id_number | text | No | National ID / Passport |
| address | text | No | |
| notes | text | No | |
| loyalty_pts | integer | Auto | Default 0 |

#### Organisation Customer Fields
| Field | Type | Required | Notes |
|---|---|---|---|
| customer_type | enum | Yes | ORGANISATION |
| company_name | text | Yes | |
| tax_id | text | No | VAT / Company registration number |
| billing_address | text | No | Invoice address |
| contact_address | text | No | Physical location if different |
| primary_contact_name | text | Yes | Person to speak to |
| primary_contact_phone | text | Yes | |
| primary_contact_email | text | No | |
| secondary_contact_name | text | No | |
| secondary_contact_phone | text | No | |
| notes | text | No | |
| loyalty_pts | integer | Auto | Default 0 |

#### Search Requirements
Full-text search across the following fields, all queryable from a single search bar and also available as individual column filters:

| Search Field | Source Table |
|---|---|
| Customer name (individual or company) | customers |
| Phone number | customers |
| Email | customers |
| License plate number | vehicles |
| Vehicle make + model | vehicles |
| VIN | vehicles |
| Job card number | job_cards |
| Job date range | job_cards |
| Tax ID / company registration | customers |

Implementation: PostgreSQL `tsvector` generated column on `customers` (name, phone, email, tax_id) and `vehicles` (license_plate, vin, make, model). Full-text search with `to_tsquery`. API endpoint: `GET /api/v1/customers/search?q=&license_plate=&date_from=&date_to=&vehicle_model=`.

#### API Routes
```
GET    /customers                  List + filter + search
POST   /customers                  Create (individual or org)
GET    /customers/:id              Customer detail + vehicles + job history
PUT    /customers/:id              Update
DELETE /customers/:id              Soft delete (is_active = false)
GET    /customers/:id/vehicles     Customer vehicles
GET    /customers/:id/jobs         Customer job history
GET    /customers/:id/invoices     Customer invoice history
GET    /customers/search           Full-text + field search
GET    /customers/export           Export to Excel
POST   /customers/import           Import from Excel
```

---

### 9.2 Vehicle Management

#### Business Rules
- **License plate is the primary identifier** — mandatory on all vehicles.
- VIN is optional (some older/imported vehicles may not have one, or it may not be known at intake).
- A vehicle always belongs to one customer (Individual or Organisation).
- Odometer is recorded at each job intake (odometerIn) and completion (odometerOut) — history is maintained.
- License plate must be unique within the tenant.

#### Vehicle Fields
| Field | Type | Required |
|---|---|---|
| license_plate | text | **Yes** — unique, primary identifier |
| customer_id | uuid | Yes |
| vin | text | No — unique if provided |
| make | text | No |
| model | text | No |
| year | integer | No |
| color | text | No |
| fuel_type | enum | No — PETROL, DIESEL, ELECTRIC, HYBRID, LPG, OTHER |
| transmission | enum | No — MANUAL, AUTOMATIC, CVT, OTHER |
| engine | text | No — e.g. "2.0L Turbo" |
| odometer | integer | No — current reading (km), updated each job |
| last_service_date | date | Auto — updated on job completion |
| notes | text | No |

#### API Routes
```
GET    /vehicles                   List + filter (make, model, license plate, customer)
POST   /vehicles                   Create
GET    /vehicles/:id               Vehicle detail + job history
PUT    /vehicles/:id               Update
GET    /vehicles/search            Quick search by license plate, VIN, make/model
GET    /vehicles/export            Export to Excel
POST   /vehicles/import            Import from Excel
```

---

### 9.3 Job Card Lifecycle

#### Status Flow
```
AUDIT → QUOTE → APPROVAL* → IN_SERVICE → INSPECTION* → BILLING → COMPLETED
                                                      ↘ CANCELLED  (any stage)
```
`*APPROVAL` skipped if `jobs.approval_workflow = false`
`*INSPECTION` skipped if `module.dvi = false`

All transitions are validated server-side. Invalid transitions return `422`.

#### Job Card Fields (additions to v2.0)
All fields from v2.0 remain. Additions:
- `customer_type_snapshot` — Individual/Organisation at time of job (for reporting)
- `license_plate_snapshot` — License plate at intake (in case vehicle record changes)

#### API Routes (additions)
```
GET    /jobs                       List + filter (status, mechanic, location, date, customer, license plate)
GET    /jobs/search                Search by job no, customer name, license plate
GET    /jobs/export                Export to Excel (respects current filters)
```

---

### 9.4 Inventory Management

#### Business Rules
- Every inventory item belongs to a Location (stock is location-scoped).
- Three stock levels defined per item: `min_stock_level` (critical threshold), `reorder_point` (warning threshold), `max_stock_level` (optional, for overstock alerts).
- `stock_status` is a computed value returned by the API — never stored, always calculated at query time from `stock_qty` vs thresholds.
- Stock changes happen via: job card item consumption (decreases stock), purchase order GRN + QA pass (increases stock), manual stock adjustment (requires Manager+ role, audit-logged).
- Universal color coding (defined in §6.2) applies everywhere stock status is displayed.
- A dedicated **Inventory Addition Form** allows adding new items to the catalogue (separate from editing existing items).

#### Inventory Item Fields
| Field | Type | Required | Notes |
|---|---|---|---|
| location_id | uuid | Yes | |
| part_no | text | No | Manufacturer part number |
| barcode | text | No | Scannable on mobile |
| name | text | Yes | |
| description | text | No | |
| category | text | No | Brakes, Filters, Lubricants, etc. |
| unit | text | Yes | each, litre, kg, metre |
| cost_price | numeric(10,2) | Yes | |
| retail_price | numeric(10,2) | Yes | |
| stock_qty | numeric(10,3) | Auto | Default 0 |
| min_stock_level | numeric(10,3) | Yes | Triggers CRITICAL alert |
| reorder_point | numeric(10,3) | Yes | Triggers WARNING alert |
| max_stock_level | numeric(10,3) | No | Optional overstock alert |
| is_active | boolean | Auto | Default true |

#### Stock Alerts
- Generated every time `stock_qty` changes (on job item save or GRN stock-in)
- Alert stored in `stock_alerts` table: item_id, alert_type (CRITICAL/LOW/OVERSTOCK), notified_at, acknowledged_by, acknowledged_at
- Push notification + in-app badge sent to all MANAGER and OWNER users at the location
- Dashboard widget: "Stock Alerts" shows count by severity with color coding

#### Inventory Addition Form
Dedicated route: `/inventory/add`
- Fields: all item fields above
- Barcode scan button (mobile camera) auto-fills `barcode` field
- Category autocomplete from existing categories
- Part number lookup (future: supplier catalogue integration)
- On submit: creates item with `stock_qty = 0`; stock added via PO/GRN flow or manual adjustment

#### API Routes
```
GET    /inventory                  List + filter + stock_status computed
POST   /inventory                  Create (inventory addition form)
GET    /inventory/:id              Item detail + stock movement history
PUT    /inventory/:id              Update item details
POST   /inventory/:id/adjust       Manual stock adjustment (Manager+, audit-logged)
GET    /inventory/low-stock        Items at WARNING or CRITICAL status
GET    /inventory/export           Export to Excel (all fields + stock_status)
POST   /inventory/import           Import from Excel (new items only)
GET    /inventory/search           Search by part_no, barcode, name, category
```

---

### 9.5 Purchase Order Management

#### Business Rules
- PO raised by Manager+, must be approved by Owner/Admin before sending (if `purchases.approval_required = true`).
- Once sent to supplier, goods are tracked as "in transit" until GRN created.
- GRN (Goods Receipt Note) records actual quantities received vs ordered — partial receipts allowed.
- Every GRN line item must pass QA before stock is added to inventory (if `purchases.qa_required = true`).
- QA result per line: PASS, FAIL, or PARTIAL_PASS. Failed items trigger supplier return workflow.
- Only QA-PASS items are added to inventory stock. This is enforced server-side.
- Full traceability: every state change records user + timestamp.

#### PO Lifecycle
```
DRAFT → PENDING_APPROVAL → APPROVED → SENT → IN_TRANSIT → PARTIALLY_RECEIVED
                         ↘ REJECTED                    → RECEIVED → QA_IN_PROGRESS
                                                                   → QA_COMPLETE → STOCKED
                                                       ↘ CANCELLED (before SENT)
```

#### Tables Involved
- `purchase_orders` — header record
- `purchase_order_items` — line items with ordered/received/qa quantities
- `po_approvals` — approval events (approved_by, approved_at, notes)
- `goods_receipt_notes` — one GRN per delivery (partial or full)
- `grn_items` — line items per GRN with received_qty
- `qa_inspections` — QA result per GRN item (pass/fail, notes, photos)
- `po_status_history` — every state change with user + timestamp

#### Goods in Transit Dashboard Widget
Visible to Manager+. Shows all POs in `IN_TRANSIT` status with:
- Supplier name, PO number, expected delivery date
- Days since dispatch
- Items and quantities expected
- Color coded: green (on time), amber (due today), red (overdue)

#### API Routes
```
GET    /purchases                  List POs + filter by status, supplier, date
POST   /purchases                  Create PO (DRAFT)
GET    /purchases/:id              PO detail + items + approval + GRN history
PUT    /purchases/:id              Update PO (DRAFT only)
POST   /purchases/:id/submit       Submit for approval
POST   /purchases/:id/approve      Approve PO (Owner/Admin)
POST   /purchases/:id/reject       Reject with reason
POST   /purchases/:id/send         Mark as sent to supplier
POST   /purchases/:id/transit      Mark goods dispatched (with expected date)
POST   /purchases/:id/grn          Create GRN (goods received)
GET    /purchases/:id/grn/:grnId   GRN detail
POST   /purchases/:id/grn/:grnId/qa  Submit QA results (triggers stock update on pass)
GET    /purchases/in-transit       In-transit dashboard data
GET    /purchases/export           Export to Excel
```

---

### 9.6 Billing & Invoicing

#### Changes from v2.0
- No functional changes to invoice structure
- Excel export added on invoice list and individual invoice
- Customer type (Individual/Organisation) shown on invoice header

#### API Routes (additions)
```
GET    /invoices/export            Export invoices to Excel
```

---

### 9.7 Digital Vehicle Inspection (DVI)
*(No significant changes from v2.0 — flag-gated as `module.dvi`)*

Photo capture enhanced: photos stored in object storage (MinIO/S3), URL stored in `dvi_results.results` JSON array.

---

### 9.8 Asset Management *(new — flag: `module.assets`)*

#### Business Rules
- Assets are workshop equipment and tools: lifts, compressors, diagnostic machines, welders, hand tools, power tools.
- Each asset belongs to a Location. Optionally assigned to a specific user.
- A daily inspection checklist is assigned to each asset (or asset category). Mechanics complete it at start of shift.
- Any defect found (during inspection or during use) is reported immediately — triggers notification to Admin/Owner.
- Defect status is tracked through to resolution.
- Assets go through a lifecycle: OPERATIONAL → UNDER_MAINTENANCE → OPERATIONAL, or → DECOMMISSIONED.

#### Asset Fields
| Field | Type | Required | Notes |
|---|---|---|---|
| asset_code | text | Yes | Unique identifier e.g. "LIFT-01" |
| name | text | Yes | e.g. "2-Post Vehicle Lift" |
| category | enum | Yes | LIFT, COMPRESSOR, DIAGNOSTIC, WELDING, HAND_TOOL, POWER_TOOL, OTHER |
| serial_number | text | No | Manufacturer serial |
| location_id | uuid | Yes | |
| assigned_user_id | uuid | No | Responsible person |
| purchase_date | date | No | |
| purchase_cost | numeric(10,2) | No | |
| warranty_expiry | date | No | |
| status | enum | Auto | OPERATIONAL, UNDER_MAINTENANCE, DECOMMISSIONED |
| notes | text | No | |
| last_inspection_date | date | Auto | Updated on inspection |

#### Daily Inspection
- Inspection template per asset category (customisable per tenant)
- Template is a JSON checklist: `[{ id, label, type: "pass_fail"|"text"|"numeric", required }]`
- At start of each shift, system generates inspection assignments for all OPERATIONAL assets
- Mechanic completes on mobile — each item marked pass/fail, with optional notes
- Photos can be attached to any item
- Completed inspection: timestamped, user-stamped, stored as `asset_inspections` record

#### Defect Reporting
- Any user can file a defect report (not just during inspection)
- Fields: asset, description, severity (LOW/MEDIUM/HIGH/CRITICAL), photo(s)
- Immediate push notification to all Admin/Owner users at that location
- Defect status lifecycle: `REPORTED → ACKNOWLEDGED → IN_REPAIR → RESOLVED`
- Admin/Owner must acknowledge within system (creates accountability trail)
- High/Critical severity: asset automatically set to `UNDER_MAINTENANCE`

#### API Routes
```
GET    /assets                     List assets + filter by category, location, status
POST   /assets                     Create asset
GET    /assets/:id                 Asset detail + inspection history + defect history
PUT    /assets/:id                 Update
POST   /assets/:id/inspect         Submit daily inspection
GET    /assets/:id/inspections     Inspection history
POST   /assets/:id/defects         Report defect
PUT    /assets/:id/defects/:defId  Update defect status
GET    /assets/due-inspection      Assets with no inspection today
GET    /assets/defects/open        All open defects (Manager+ view)
GET    /assets/export              Export to Excel
```

---

### 9.9 HR & Payroll *(new — flag: `module.hr`)*

#### Business Rules
- Every workshop staff member has an Employee record linked to their User account.
- Payroll is calculated monthly. HR Officer or Admin triggers payroll run for a period.
- Salary data and bank details are sensitive — visible only to OWNER, ADMIN, HR_OFFICER.
- Leave requests submitted by employee, approved by Manager/Admin/Owner.
- Attendance can be manual entry or clock-in/clock-out (flag: `hr.attendance`).
- All payroll summaries exportable to Excel.

#### Employee Record Fields
| Field | Category | Required |
|---|---|---|
| user_id | Link | Yes — one-to-one with User |
| employee_code | Identifier | Auto-generated |
| department | Text | No |
| designation | Text | No — e.g. "Senior Mechanic" |
| employment_type | Enum | Yes — FULL_TIME, PART_TIME, CONTRACT, INTERN |
| hire_date | Date | Yes |
| end_date | Date | No — for contracts |
| salary_type | Enum | Yes — FIXED_MONTHLY, HOURLY |
| base_salary | numeric(10,2) | Yes |
| hourly_rate | numeric(10,2) | If HOURLY |
| bank_name | Text | No — sensitive |
| bank_account_no | Text | No — encrypted at rest |
| bank_routing_code | Text | No — encrypted at rest |
| emergency_contact_name | Text | No |
| emergency_contact_phone | Text | No |
| emergency_contact_relation | Text | No |
| national_id | Text | No — sensitive |
| notes | Text | No |

#### Payroll Calculation
- Monthly payroll period
- Inputs: base salary, worked days/hours (from attendance if enabled), approved overtime, approved deductions
- Deductions: tax (configured per tenant), insurance, advances, other
- Output per employee: gross pay, total deductions, net pay
- Payroll run creates `payroll_entries` records — immutable once approved
- Approved payroll exports to Excel: employee name, code, designation, gross, deductions breakdown, net pay

#### Leave Management (flag: `hr.leave_management`)
- Leave types configured per tenant: Annual Leave, Sick Leave, Unpaid Leave, Emergency Leave
- Leave balance tracked per employee per type per year
- Employee submits request → Manager/Admin approves or rejects with reason
- Approved leave automatically deducted from balance
- Leave calendar view for Manager (who is out on what days)

#### Attendance (flag: `hr.attendance`)
- Clock-in / clock-out via the PWA (records timestamp + location)
- Manual entry by Manager for missed records
- Late arrival / early departure flagged (compared to shift schedule)
- Attendance report feeds into payroll calculation

#### API Routes
```
GET    /hr/employees               List employees
POST   /hr/employees               Create employee record
GET    /hr/employees/:id           Employee detail
PUT    /hr/employees/:id           Update (sensitive fields require Owner/Admin)
GET    /hr/employees/:id/payroll   Payroll history for employee
GET    /hr/payroll/periods         List payroll periods
POST   /hr/payroll/periods         Create payroll period
POST   /hr/payroll/periods/:id/run Calculate payroll for period
POST   /hr/payroll/periods/:id/approve Approve and lock payroll
GET    /hr/payroll/periods/:id/export  Export payroll to Excel
GET    /hr/leave/requests          List leave requests (Manager view)
POST   /hr/leave/requests          Submit leave request
PUT    /hr/leave/requests/:id      Approve/reject
GET    /hr/attendance              Attendance records + filter
POST   /hr/attendance/clock-in     Clock in
POST   /hr/attendance/clock-out    Clock out
GET    /hr/employees/export        Export employee list to Excel
```

---

### 9.10 Reports & Analytics

#### Business Rules
- Users select what type of report to generate (no fixed report pages).
- Report builder: choose report type → configure filters → preview → export.
- Saved configurations allow one-click re-run.
- All reports exportable to Excel.

#### Available Report Types

| Report Type | Key Metrics | Filters Available |
|---|---|---|
| Revenue Summary | Total invoiced, collected, outstanding | Date range, location, customer type |
| Job Card Analysis | Jobs by status, avg completion time, jobs per mechanic | Date range, location, mechanic, status |
| Technician Productivity | Jobs completed, labour hours, revenue per technician | Date range, location, mechanic |
| Inventory Movement | Stock in, stock out, adjustments, closing balance | Date range, location, category, item |
| Stock Valuation | Current stock at cost and retail price | Location, category |
| Purchase Spend | Total spend by supplier, by category | Date range, supplier, location |
| Goods in Transit | POs in transit (current snapshot) | Location, supplier |
| Customer Activity | Jobs and spend per customer | Date range, customer type |
| Asset Status | Assets by status, defect history, inspection compliance | Location, category |
| HR Payroll Summary | Total payroll, by department (if module.hr enabled) | Period, department |
| Leave Summary | Leave taken by type and employee (if hr.leave enabled) | Period, department |

#### Saved Reports
- Users save a report configuration with a name
- Saved configs stored in `saved_reports` table: user_id, name, report_type, filter_config (JSON)
- Visible only to the user who saved them (or all if shared)
- "My Reports" section on reports page

#### API Routes
```
POST   /reports/generate           Generate report (type + filters in body)
GET    /reports/saved              List user's saved configurations
POST   /reports/saved              Save a configuration
DELETE /reports/saved/:id          Delete saved config
POST   /reports/export             Generate + export to Excel
```

---

### 9.11 Excel Import / Export *(new)*

#### Export — Available on Every List View
Every major list view has an "Export to Excel" button. Export respects current active filters — what the user sees is what they export.

| Entity | Key Exported Fields |
|---|---|
| Customers | Type, name/company, phone, email, loyalty pts, vehicle count, last job date |
| Vehicles | License plate, VIN, make, model, year, customer name, last service date |
| Job Cards | Job no, date, customer, license plate, mechanic, status, total value |
| Inventory Items | Part no, barcode, name, category, unit, cost price, retail price, stock qty, stock status, min level, reorder point |
| Purchase Orders | PO no, supplier, status, total, ordered date, received date |
| Invoices | Invoice no, job no, customer, status, subtotal, tax, total, payment method, paid date |
| Employees | Code, name, designation, department, employment type, hire date |
| Payroll (period) | Employee, gross pay, deductions, net pay |
| Assets | Code, name, category, status, location, last inspection date, open defects |

#### Import — Core Entities
Import is available for: Customers, Vehicles, Inventory Items, Employees.

**Import flow:**
1. User downloads template Excel file (correct column headers, example row)
2. User fills template, uploads file
3. System parses and validates all rows
4. Preview screen: first 10 rows shown, all validation errors highlighted
5. Row-level error display: "Row 14: license_plate is required"
6. User fixes and re-uploads, or proceeds with valid rows only
7. Duplicate detection: license plate (vehicles), email + phone (customers), part_no + barcode (inventory), employee_code (employees)
8. Confirmed import runs in a background task — user notified on completion with count (imported, skipped, failed)

#### Rust Implementation
- Read: `calamine` crate for .xlsx parsing
- Write: `rust_xlsxwriter` for .xlsx generation
- Templates: generated on-the-fly with correct headers and one example row
- Background tasks: `tokio::spawn` for import jobs with Redis progress tracking

#### API Routes
```
GET    /export/template/:entity    Download blank template Excel
POST   /import/:entity             Upload + validate Excel (returns preview + errors)
POST   /import/:entity/confirm     Confirm and run import
GET    /import/:entity/status/:id  Check import job progress
```

---

### 9.12 Settings & Configuration

#### Tenant Settings
- Workshop profile: name, address, phone, email, logo, tax ID
- Locations: CRUD for workshop branches
- Tax / VAT configuration (if `billing.vat = true`)
- Leave types configuration (if `hr.leave_management = true`)
- Asset inspection templates (if `module.assets = true`)
- DVI templates (if `module.dvi = true`)
- Notification preferences per role

#### User Management
- User CRUD within tenant
- Role assignment
- Password reset by Admin
- Deactivate user (preserves all historical records)

---

## 10. Database Schema Design

### 10.1 Conventions
- All PKs: UUID v7 (time-sortable)
- All money: `NUMERIC(10,2)` — no exceptions
- All quantities: `NUMERIC(10,3)` (for litres, kg, fractional units)
- Percentages: `NUMERIC(5,2)`
- Soft deletes: `is_active BOOLEAN DEFAULT true` on root entities
- Timestamps: `created_at TIMESTAMPTZ DEFAULT now()`, `updated_at TIMESTAMPTZ` (trigger-managed)
- Append-only tables: `audit_logs`, `po_status_history`, `payroll_entries` — no UPDATE or DELETE ever

### 10.2 Complete Table Inventory (Tenant DB)

**Auth & Access:** `users`, `sessions` (if not Redis-only)

**CRM:** `customers` (with customer_type discriminator + org fields), `vehicles` (license_plate primary)

**Workshop:** `job_cards`, `job_card_items`

**Inventory:** `inventory_items`, `stock_alerts`, `stock_adjustments`

**Purchasing:** `suppliers`, `purchase_orders`, `purchase_order_items`, `po_approvals`, `po_status_history`, `goods_receipt_notes`, `grn_items`, `qa_inspections`

**Billing:** `invoices`, `invoice_line_items`

**DVI:** `dvi_templates`, `dvi_results`

**Assets:** `assets`, `asset_inspection_templates`, `asset_inspections`, `asset_defects`

**HR:** `employees`, `payroll_periods`, `payroll_entries`, `leave_types`, `leave_requests`, `attendance_records`

**Reports:** `saved_reports`

**Audit:** `audit_logs`

### 10.3 Key Indexes
```sql
-- Customers: full-text search
CREATE INDEX customers_fts_idx ON customers USING gin(search_vector);
-- search_vector is a generated column:
-- to_tsvector('english', coalesce(full_name,'') || ' ' || coalesce(company_name,'') || ' ' || coalesce(phone,'') || ' ' || coalesce(email,''))

-- Vehicles: primary lookups
CREATE UNIQUE INDEX vehicles_license_plate_idx ON vehicles(license_plate);
CREATE INDEX vehicles_fts_idx ON vehicles USING gin(search_vector);
-- search_vector covers license_plate, vin, make, model

-- Job cards: operational queries
CREATE INDEX job_cards_location_status_idx ON job_cards(location_id, status);
CREATE INDEX job_cards_customer_id_idx ON job_cards(customer_id);
CREATE INDEX job_cards_mechanic_id_idx ON job_cards(mechanic_id);
CREATE INDEX job_cards_created_at_idx ON job_cards(created_at);

-- Inventory: stock management
CREATE INDEX inventory_location_category_idx ON inventory_items(location_id, category);
CREATE INDEX inventory_part_no_idx ON inventory_items(part_no);
CREATE INDEX inventory_barcode_idx ON inventory_items(barcode);

-- Purchase orders: status tracking
CREATE INDEX po_location_status_idx ON purchase_orders(location_id, status);
CREATE INDEX po_supplier_id_idx ON purchase_orders(supplier_id);

-- Audit logs: compliance queries
CREATE INDEX audit_logs_module_action_idx ON audit_logs(module, action);
CREATE INDEX audit_logs_created_at_idx ON audit_logs(created_at);
CREATE INDEX audit_logs_user_id_idx ON audit_logs(user_id);

-- Assets
CREATE INDEX assets_location_status_idx ON assets(location_id, status);

-- HR
CREATE INDEX attendance_user_date_idx ON attendance_records(user_id, work_date);
CREATE INDEX payroll_entries_period_idx ON payroll_entries(period_id);
```

---

## 11. Backend Module Structure (Rust)

```
api/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── db/
│   │   ├── control.rs           — Control DB pool
│   │   └── tenant.rs            — Per-tenant pool registry (LRU cache, idle timeout)
│   ├── middleware/
│   │   ├── auth.rs              — JWT extraction + validation + blocklist check
│   │   ├── tenant.rs            — TenantDb injector
│   │   ├── feature_flags.rs     — Flag resolution + FeatureFlags extractor
│   │   └── rbac.rs              — require_roles! macro
│   ├── modules/
│   │   ├── auth/                — login, refresh, logout
│   │   ├── customers/           — CRUD + search (individual + org)
│   │   ├── vehicles/            — CRUD + search by license plate
│   │   ├── jobs/                — job card lifecycle + state machine
│   │   ├── inventory/           — CRUD + stock alerts + adjustments
│   │   ├── purchases/           — PO lifecycle + GRN + QA + stock-in
│   │   ├── billing/             — invoice generation + payment
│   │   ├── dvi/                 — templates + results + photos
│   │   ├── assets/              — asset CRUD + daily inspection + defects
│   │   ├── hr/                  — employees + payroll + leave + attendance
│   │   ├── reports/             — report builder + saved configs
│   │   ├── import_export/       — Excel import/export for all entities
│   │   └── settings/            — tenant config + users + locations
│   ├── control/
│   │   ├── tenants/
│   │   └── feature_flags/
│   ├── jobs/                    — Background job runners (import, notifications)
│   ├── notifications/           — Push + email notification dispatch
│   ├── storage/                 — Object storage abstraction (MinIO/S3)
│   ├── search/                  — Full-text search helpers
│   └── errors.rs
├── migrations/                  — sqlx migration files
│   ├── 001_initial_schema.sql
│   ├── 002_customers_fts.sql
│   ├── 003_vehicles_license_plate.sql
│   ├── 004_hr_module.sql
│   ├── 005_assets_module.sql
│   └── ...
└── Cargo.toml
```

Each module:
```
module/
├── mod.rs       — Router registration
├── handlers.rs  — Axum handlers (HTTP layer only)
├── service.rs   — Business logic (no HTTP types)
├── repo.rs      — sqlx queries
└── types.rs     — Request/response structs (serde + validator)
```

---

## 12. Frontend Module Structure (React)

```
web/src/
├── modules/
│   ├── auth/
│   ├── dashboard/
│   ├── customers/           — Individual + Org creation forms, search UI
│   ├── vehicles/            — License plate as hero field
│   ├── jobs/
│   ├── inventory/
│   │   ├── components/
│   │   │   ├── StockBadge.tsx         — Universal color-coded badge
│   │   │   ├── InventoryAddForm.tsx   — Dedicated add form with barcode scan
│   │   │   └── StockAlertWidget.tsx   — Dashboard widget
│   ├── purchases/           — PO lifecycle timeline, in-transit widget, QA form
│   ├── billing/
│   ├── dvi/
│   ├── assets/              — Asset list, daily inspection checklist, defect report
│   ├── hr/                  — Employee list, payroll, leave calendar
│   ├── reports/             — Report builder UI, saved configs
│   ├── import-export/       — Import modal with preview, export button component
│   └── settings/
├── components/
│   ├── ui/                  — shadcn/ui base (customised to Industrial Brutalism)
│   └── shared/
│       ├── ExportButton.tsx           — Reusable export to Excel button
│       ├── ImportModal.tsx            — Reusable import flow modal
│       ├── StockBadge.tsx             — Shared stock status badge
│       ├── SearchBar.tsx              — Multi-field search bar
│       └── NotificationBell.tsx       — Push notification handler
```

---

## 13. Docker Architecture

### 13.1 Services

```yaml
services:
  nginx:              # SSL termination, proxy, static files
  api:                # Rust Axum — PORT 8080
  control-db:         # PostgreSQL 16 — control plane only
  redis:              # JWT blocklist, cache, rate limiting
  minio:              # S3-compatible object storage (photos, documents, exports)
  migration-runner:   # One-shot: migrates all tenant DBs on deploy

networks:
  public:    # nginx only
  internal:  # all other services

volumes:
  control-db-data:
  redis-data:
  minio-data:
  tenant-db-data-{n}:   # one volume per provisioned tenant
```

### 13.2 Deployment Modes

| Mode | Config | Use Case |
|---|---|---|
| Cloud SaaS | All tenants on shared host | Default, most cost-effective |
| On-premise single tenant | `MAX_TENANTS=1` env var | Workshop wants own server |
| On-premise multi-tenant | N tenants, client's infra | Franchise or group |

Same Docker stack, same images, different environment variables.

---

## 14. Security Requirements

| Requirement | Implementation |
|---|---|
| Passwords | argon2id, per-user salt |
| JWT | HS256, per-tenant signing secret, 15-min access token |
| Refresh tokens | Single-use, Redis-stored, rotation on every use |
| SQL injection | sqlx compile-time parameterised queries — no string interpolation ever |
| Rate limiting | Redis-based, per-IP + per-user, on all auth endpoints |
| Tenant isolation | Middleware-enforced DB routing — no shared tables |
| Sensitive fields | Bank details + national ID + salary encrypted at rest (AES-256) |
| File access | Object storage URLs are pre-signed, short-lived (15 min TTL) |
| Audit logging | All mutations logged — append-only, immutable |
| HTTPS | Nginx SSL; HTTP → HTTPS redirect |
| RBAC | Both API middleware and frontend flag gating |

---

## 15. Development Workflow

### 15.1 Repository Structure
```
garage360/
├── api/                — Rust Axum backend
├── web/                — React TypeScript PWA
├── docker/             — Dockerfiles, nginx.conf, minio setup
├── docker-compose.yml
├── docker-compose.dev.yml
├── .env.example
└── README.md
```

### 15.2 Local Dev
```bash
docker compose -f docker-compose.dev.yml up
# API: cargo-watch hot reload
# Web: Vite HMR
# MinIO: http://localhost:9001 (console)

# Run migrations on dev tenant
docker compose exec api cargo sqlx migrate run --database-url $DEV_TENANT_DB_URL

# Seed dev data
docker compose exec api cargo run --bin seed-dev
```

---

## 16. Agent Skills Plan

| # | Skill | Status | Priority |
|---|---|---|---|
| 1 | `schema-architect` | Needs update (sqlx, UUID v7, new tables, license plate, HR, assets, FTS indexes) | P0 |
| 2 | `rust-module-builder` | Build next | P0 |
| 3 | `react-module-builder` | After rust skill | P1 |
| 4 | `docker-composer` | Generates full docker-compose + Dockerfiles + nginx | P1 |
| 5 | `pwa-configurator` | Vite PWA config, SW strategy, manifest | P2 |
| 6 | `rbac-enforcer` | Axum middleware + frontend flag/role gating | P1 |
| 7 | `excel-io-builder` | Import/export patterns for calamine + rust_xlsxwriter + SheetJS | P2 |
| 8 | `test-writer` | Rust integration tests + RTL tests | P2 |
| 9 | `notification-builder` | Push notifications + email dispatch patterns | P3 |
| 10 | `search-builder` | PostgreSQL FTS with tsvector + API search patterns | P2 |
