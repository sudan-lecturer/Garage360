# Garage360 — Master Development Plan
**Version:** 1.0.0
**Based on:** SRS v4.0.0
**Status:** Planning Complete — Ready for Development

---

## Part 1 — What We Are Building

Garage360 is a multi-tenant workshop management SaaS. One Docker stack, deployable for any number of independent workshops, each with complete database isolation. A React PWA frontend (mobile-first, installable) talks to a Rust API (Axum), which routes every request to the correct per-tenant PostgreSQL database.

### The Problem It Solves
A vehicle service center today manages jobs on paper, estimates in WhatsApp, stock in Excel, and HR in a folder. Garage360 replaces all of it with one system that works on a phone in the workshop bay.

### Non-Negotiables
- Every tenant's data is in its own PostgreSQL database — no shared tables
- Mobile-first: every screen functional at 375px
- Offline read capability via service worker
- Feature flags control module visibility per tenant — disabled = invisible
- No migrations — fresh schema deployed per new tenant database

---

## Part 2 — System Architecture (Final)

```
Internet
    │
    ▼
┌─────────────┐
│    Nginx    │  SSL termination, static file serving, reverse proxy
└──────┬──────┘
       │
   ┌───┴────┐
   │        │
   ▼        ▼
React    Rust API
 PWA     (Axum)
(Vite)   :8080
         │
    ┌────┴────────────────────┐
    │     Tenant Router        │
    │  JWT → DB pool lookup    │
    └────┬──────┬─────────┬───┘
         │      │         │
    [tenant_a] [tenant_b] [tenant_N]
    PostgreSQL  PostgreSQL  PostgreSQL
         │
    ┌────┴─────────────────┐
    │   Control DB         │  Tenant registry, feature flags, super admin
    │   PostgreSQL         │
    └──────────────────────┘
         │
    ┌────┴─────────────────┐
    │   Redis              │  JWT blocklist, session cache, flag cache, rate limiting
    └──────────────────────┘
         │
    ┌────┴─────────────────┐
    │   MinIO              │  Photos, signatures, documents, exports (S3-compatible)
    └──────────────────────┘
```

### Docker Services (Final List)
| Service | Image | Role |
|---|---|---|
| `nginx` | nginx:alpine | Reverse proxy, SSL, static files |
| `api` | Custom Rust build | Axum API, all business logic |
| `control-db` | postgres:16-alpine | Tenant registry + feature flags |
| `redis` | redis:7-alpine | Cache, sessions, rate limiting |
| `minio` | minio/minio | Object storage (photos, PDFs, exports) |

No migration-runner service. Each new tenant database is provisioned and seeded with the complete schema via the API's tenant provisioning endpoint, which runs the schema creation SQL directly. No incremental migrations needed — fresh schema every time.

---

## Part 3 — Technology Decisions (Final, No Changes)

### Backend
| Concern | Choice |
|---|---|
| Language | Rust |
| HTTP Framework | Axum |
| Async Runtime | Tokio |
| Database Driver | sqlx (compile-time query checking) |
| Auth | jsonwebtoken (JWT HS256) |
| Password Hashing | argon2 (argon2id) |
| Validation | validator |
| Serialisation | serde + serde_json |
| Error Handling | thiserror + anyhow |
| Logging | tracing + tracing-subscriber |
| Config | dotenvy + config |
| Redis | redis (async) |
| UUID | uuid v7 (time-sortable PKs) |
| Excel Read | calamine |
| Excel Write | rust_xlsxwriter |
| Object Storage | object_store (S3-compatible) |
| Email | lettre |
| SMS | reqwest (HTTP to SMS gateway API) |
| PDF Generation | printpdf |
| Testing | tokio-test, sqlx test transactions |

### Frontend
| Concern | Choice |
|---|---|
| Language | TypeScript 5 (strict) |
| Framework | React 19 |
| Build | Vite 6 |
| Routing | React Router v7 |
| Global State | Zustand |
| Server State | TanStack Query v5 |
| Forms | React Hook Form + Zod |
| UI Components | shadcn/ui (customised) |
| Icons | lucide-react |
| Styling | Tailwind CSS v4 |
| PWA | vite-plugin-pwa |
| Charts | Recharts |
| Tables | TanStack Table v8 |
| Excel Export | xlsx (SheetJS) |
| HTTP Client | Axios + typed hooks |
| i18n | react-i18next |
| Camera | react-webcam |
| Signature Pad | react-signature-canvas |

---

## Part 4 — Database (Final)

### Strategy
- Fresh schema per tenant. No migrations. Schema SQL runs once on tenant creation.
- All PKs: UUID v7 (generated in Rust via `uuid::Uuid::now_v7()`)
- All money: `NUMERIC(10,2)`
- All quantities: `NUMERIC(10,3)`
- Timestamps: `TIMESTAMPTZ`
- Soft deletes: `is_active` on root entities only
- Append-only tables: `audit_logs`, `job_card_activities`, `po_status_history`, `payroll_entries`, `asset_defects`

### Complete Table List (45 tables, Tenant DB)
**Auth & Locations:** `users`, `locations`, `tenant_settings`
**CRM:** `customers`, `vehicles`
**Job Core:** `job_cards`, `job_card_items`, `job_card_activities`, `job_card_approvals`
**Intake:** `intake_checklist_templates`, `intake_checklists`, `intake_checklist_responses`, `intake_photos`, `customer_signatures`
**Bays:** `service_bays`
**Change Requests:** `job_change_requests`, `job_change_request_items`
**Inventory:** `inventory_items`, `stock_alerts`, `stock_adjustments`
**Purchasing:** `suppliers`, `purchase_orders`, `purchase_order_items`, `po_approvals`, `po_status_history`, `goods_receipt_notes`, `grn_items`, `qa_inspections`
**Billing:** `invoices`, `invoice_line_items`
**DVI:** `dvi_templates`, `dvi_results`
**Assets:** `assets`, `asset_inspection_templates`, `asset_inspections`, `asset_defects`
**HR:** `employees`, `payroll_periods`, `payroll_entries`, `payroll_deduction_configs`, `leave_types`, `leave_requests`, `attendance_records`
**Reports:** `saved_reports`
**Audit:** `audit_logs`

### Core Types
`employment_type` ENUM ('FULL_TIME', 'PART_TIME', 'CONTRACT', 'INTERN')
`deduction_type` ENUM ('PCT_OF_GROSS', 'FIXED_AMOUNT', 'SLAB')

### Control DB (4 tables)
`tenants`, `feature_flags`, `super_admin_users`, `control_audit_logs`

---

## Part 5 — Module Inventory (Complete)

Every module listed with its status, feature flag, and roles that can access it.

| # | Module | Feature Flag | Always On | Roles |
|---|---|---|---|---|
| 1 | Auth | — | Yes | All |
| 2 | Dashboard | — | Yes | All staff |
| 3 | Customer Management | — | Yes | All staff |
| 4 | Vehicle Management | — | Yes | All staff |
| 5 | Job Card — Intake | `jobs.intake_inspection` | Default on | Manager, Mechanic |
| 6 | Job Card — Core Lifecycle | — | Yes | Manager, Account Manager, Mechanic, Cashier |
| 7 | Service Bay Management | `jobs.bay_management` | Default on | Manager, Admin, Owner |
| 8 | Mid-Service Change Requests | `jobs.mid_service_approval` | Default on | Mechanic, Manager, Account Manager |
| 9 | Job Card Activity Timeline | — | Yes | All staff (read) |
| 10 | Inventory Management | — | Yes | Manager, Admin, Owner |
| 11 | Inventory Addition Form | — | Yes | Manager, Admin, Owner |
| 12 | Purchase Orders + GRN + QA | `module.purchases` | Default on | Manager, Admin, Owner |
| 13 | Billing & Invoicing | — | Yes | Account Manager, Cashier, Admin, Owner |
| 14 | Digital Vehicle Inspection (DVI) | `module.dvi` | Default on | Mechanic, Manager |
| 15 | Asset Management | `module.assets` | Default off | All staff |
| 16 | HR & Payroll | `module.hr` | Default off | HR Officer, Admin, Owner |
| 17 | Reports & Analytics | `module.reports` | Default on | Manager, Admin, Owner |
| 18 | Excel Import/Export | `export.excel` / `import.excel` | Default on | All staff (export), Manager+ (import) |
| 19 | Settings & Configuration | — | Yes | Admin, Owner |
| 20 | Customer Portal | `module.customer_portal` | Default off | Customer |
| 21 | Super Admin Control Panel | — | Yes | Super Admin only |

---

## Part 6 — Job Card Lifecycle (Final, Definitive)

This is the most complex flow in the system. Every transition is server-enforced.

```
[Customer arrives at workshop]
         │
         ▼
    ┌─────────┐
    │ INTAKE  │  Service manager creates job card. Completes intake checklist
    │         │  (keys, tyres, engine, odometer, lights, belongings, damage history).
    │         │  Optional: vehicle photos captured. Customer signs digitally.
    └────┬────┘
         │  Requires: checklist complete + signature captured
         ▼
    ┌─────────┐
    │  AUDIT  │  Manager records complaint and diagnosis. Assigns mechanic.
    │         │  Assigns service bay. Sets preferred customer contact channel.
    └────┬────┘
         │  Requires: complaint, mechanic assigned, bay assigned
         ▼
    ┌─────────┐
    │  QUOTE  │  Account Manager builds estimate: parts from inventory + labour.
    │         │  System shows stock status inline. Subtotal, tax, total calculated.
    └────┬────┘
         │  Requires: at least one line item
         ▼
    ┌──────────────┐
    │   APPROVAL   │  Customer notified (email/SMS/phone). Approval recorded.
    │  (skippable) │  [If jobs.approval_workflow = false, skip to IN_SERVICE]
    └────┬─────────┘
         │  Requires: approval recorded by staff
         ▼
    ┌────────────┐
    │ IN_SERVICE │  Mechanic works on vehicle in assigned bay. Bay shows OCCUPIED.
    │            │  ┌─────────────────────────────────────────┐
    │            │  │ Mid-service change request loop:         │
    │            │  │ Mechanic adds item → customer notified   │
    │            │  │ → approved: item added to job            │
    │            │  │ → declined: noted on timeline            │
    │            │  └─────────────────────────────────────────┘
    └────┬───────┘
         │  Requires: mechanic marks work complete
         ▼
    ┌─────────┐
    │   QA    │  QA technician reviews completed work.
    │         │  Intake checklist shown as "before" reference.
    │         │  All complaint items + change request items checked.
    │         │  ┌──────────────────────────────────────────┐
    │         │  │ FAIL → returns to IN_SERVICE             │
    │         │  │ with defect notes. qa_cycles incremented. │
    │         │  └──────────────────────────────────────────┘
    └────┬────┘
         │  Requires: QA PASS
         ▼
    ┌─────────┐
    │ BILLING │  Invoice generated from job items. Customer pays.
    │         │  Invoice PDF generated and emailed.
    └────┬────┘
         │
         ▼
    ┌───────────┐
    │ COMPLETED │  Job closed. Bay freed. Vehicle odometer updated.
    └───────────┘
         
    [CANCELLED reachable from any stage — requires reason + Manager+]
```

Every status change, assignment, approval, notification, and note is written to `job_card_activities` — immutable, append-only.

---

## Part 7 — User Roles & Permissions Matrix

| Permission | OWNER | ADMIN | MANAGER | ACCOUNT_MGR | MECHANIC | CASHIER | HR_OFFICER | CUSTOMER |
|---|---|---|---|---|---|---|---|---|
| Create tenant settings | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Manage users & roles | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Create / edit bays | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Create job card (intake) | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| Assign mechanic / bay | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Build estimate (QUOTE) | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Record approval | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Update job (IN_SERVICE) | ✓ | ✓ | ✓ | ✗ | own jobs only | ✗ | ✗ | ✗ |
| Submit change request | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| Approve change request | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Submit QA | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| Generate invoice | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Record payment | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ |
| Manage inventory | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Manual stock adjustment | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Create / approve PO | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Submit GRN + QA | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| View reports | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Manage assets | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Daily asset inspection | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| Report asset defect | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| Manage employees (HR) | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| Run payroll | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| View own payslip | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ |
| Export any list to Excel | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ |
| Import from Excel | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| View own job history (portal) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |

---

## Part 8 — Feature Flags (Complete Registry)

Super Admin sets global defaults. Tenant Owner/Admin can override per tenant.
Resolution order: Tenant override → Global default → `false`

| Flag Key | Global Default | Description |
|---|---|---|
| `module.dvi` | `true` | Digital Vehicle Inspection |
| `module.purchases` | `true` | Purchase Orders + GRN + QA |
| `module.reports` | `true` | Report builder |
| `module.hr` | `false` | HR records + payroll |
| `module.assets` | `false` | Asset management |
| `module.customer_portal` | `false` | Customer self-service portal |
| `module.loyalty` | `false` | Loyalty points |
| `jobs.intake_inspection` | `true` | Intake checklist |
| `jobs.intake_signature` | `true` | Customer digital signature |
| `jobs.bay_management` | `true` | Service bay tracking |
| `jobs.approval_workflow` | `true` | Quote approval step |
| `jobs.mid_service_approval` | `true` | Change request approvals |
| `jobs.dvi_required` | `false` | Block billing until QA complete |
| `jobs.notification_email` | `true` | Email customer notifications |
| `jobs.notification_sms` | `false` | SMS customer notifications |
| `inventory.low_stock_alerts` | `true` | Low stock push notifications |
| `purchases.approval_required` | `true` | PO approval before sending |
| `purchases.qa_required` | `true` | QA before stock-in |
| `billing.vat` | `false` | VAT/tax line on invoices |
| `billing.multi_currency` | `false` | Multi-currency |
| `hr.payroll` | `false` | Payroll (requires module.hr) |
| `hr.leave_management` | `false` | Leave management |
| `hr.attendance` | `false` | Clock-in/out |
| `assets.daily_inspection` | `true` | Daily inspection checklist |
| `export.excel` | `true` | Excel export on all lists |
| `import.excel` | `true` | Excel import for core entities |

---

## Part 9 — API Route Map (Complete)

All tenant API routes: `GET/POST/PUT/DELETE /api/v1/{resource}`
Control plane routes: `/control/v1/{resource}` (Super Admin only)

### Control Plane
```
POST   /control/v1/tenants                       Provision new tenant + DB
GET    /control/v1/tenants                       List tenants
GET    /control/v1/tenants/:id                   Tenant detail + health
PUT    /control/v1/tenants/:id                   Update tenant
DELETE /control/v1/tenants/:id                   Deactivate tenant
PUT    /control/v1/feature-flags/:key            Set global default
PUT    /control/v1/tenants/:id/feature-flags/:key  Set tenant override
```

### Auth
```
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
POST   /api/v1/auth/logout
POST   /api/v1/auth/change-password
GET    /api/v1/auth/me
GET    /api/v1/feature-flags
```

### Customers
```
GET    /api/v1/customers
POST   /api/v1/customers
GET    /api/v1/customers/search
GET    /api/v1/customers/export
POST   /api/v1/customers/import
GET    /api/v1/customers/:id
PUT    /api/v1/customers/:id
DELETE /api/v1/customers/:id
GET    /api/v1/customers/:id/vehicles
GET    /api/v1/customers/:id/jobs
GET    /api/v1/customers/:id/invoices
```

### Vehicles
```
GET    /api/v1/vehicles
POST   /api/v1/vehicles
GET    /api/v1/vehicles/search
GET    /api/v1/vehicles/export
POST   /api/v1/vehicles/import
GET    /api/v1/vehicles/:id
PUT    /api/v1/vehicles/:id
DELETE /api/v1/vehicles/:id
```

### Jobs
```
GET    /api/v1/jobs
POST   /api/v1/jobs
GET    /api/v1/jobs/search
GET    /api/v1/jobs/export
GET    /api/v1/jobs/:id
PUT    /api/v1/jobs/:id
POST   /api/v1/jobs/:id/cancel
POST   /api/v1/jobs/:id/transition
PUT    /api/v1/jobs/:id/assign-mechanic
PUT    /api/v1/jobs/:id/assign-bay
PUT    /api/v1/jobs/:id/assign-account-manager
PUT    /api/v1/jobs/:id/estimated-completion
GET    /api/v1/jobs/:id/items
POST   /api/v1/jobs/:id/items
PUT    /api/v1/jobs/:id/items/:itemId
DELETE /api/v1/jobs/:id/items/:itemId
GET    /api/v1/jobs/:id/activities
POST   /api/v1/jobs/:id/activities/note
GET    /api/v1/jobs/:id/change-requests
POST   /api/v1/jobs/:id/change-requests
PUT    /api/v1/jobs/:id/change-requests/:crId
GET    /api/v1/jobs/:id/approvals
POST   /api/v1/jobs/:id/approvals
GET    /api/v1/jobs/:id/qa/context
POST   /api/v1/jobs/:id/qa/submit
GET    /api/v1/jobs/:id/qa/history
```

### Intake
```
GET    /api/v1/jobs/:id/intake/template
POST   /api/v1/jobs/:id/intake/checklist
POST   /api/v1/jobs/:id/intake/photos
DELETE /api/v1/jobs/:id/intake/photos/:photoId
POST   /api/v1/jobs/:id/intake/signature
GET    /api/v1/jobs/:id/intake/report
POST   /api/v1/jobs/:id/intake/complete
```

### Bays
```
GET    /api/v1/bays/board
GET    /api/v1/settings/bays
POST   /api/v1/settings/bays
PUT    /api/v1/settings/bays/:id
PUT    /api/v1/settings/bays/:id/status
DELETE /api/v1/settings/bays/:id
```

### Inventory
```
GET    /api/v1/inventory
POST   /api/v1/inventory
GET    /api/v1/inventory/search
GET    /api/v1/inventory/low-stock
GET    /api/v1/inventory/export
POST   /api/v1/inventory/import
GET    /api/v1/inventory/:id
PUT    /api/v1/inventory/:id
DELETE /api/v1/inventory/:id
POST   /api/v1/inventory/:id/adjust
```

### Purchases
```
GET    /api/v1/purchases
POST   /api/v1/purchases
GET    /api/v1/purchases/in-transit
GET    /api/v1/purchases/export
GET    /api/v1/purchases/:id
PUT    /api/v1/purchases/:id
GET    /api/v1/purchases/:id/history
POST   /api/v1/purchases/:id/submit
POST   /api/v1/purchases/:id/approve
POST   /api/v1/purchases/:id/reject
POST   /api/v1/purchases/:id/send
POST   /api/v1/purchases/:id/transit
POST   /api/v1/purchases/:id/grn
GET    /api/v1/purchases/:id/grn/:grnId
POST   /api/v1/purchases/:id/grn/:grnId/qa
```

### Billing
```
GET    /api/v1/invoices
GET    /api/v1/invoices/export
GET    /api/v1/invoices/:id
POST   /api/v1/invoices
PUT    /api/v1/invoices/:id
POST   /api/v1/invoices/:id/issue
POST   /api/v1/invoices/:id/payment
POST   /api/v1/invoices/:id/void
GET    /api/v1/invoices/:id/pdf
```

### DVI
```
GET    /api/v1/dvi/templates
POST   /api/v1/dvi/templates
PUT    /api/v1/dvi/templates/:id
DELETE /api/v1/dvi/templates/:id
POST   /api/v1/dvi/results
GET    /api/v1/dvi/results/:id
PUT    /api/v1/dvi/results/:id
DELETE /api/v1/dvi/results/:id
```

### Assets
```
GET    /api/v1/assets
POST   /api/v1/assets
GET    /api/v1/assets/due-inspection
GET    /api/v1/assets/defects/open
GET    /api/v1/assets/export
GET    /api/v1/assets/:id
PUT    /api/v1/assets/:id
POST   /api/v1/assets/:id/inspect
GET    /api/v1/assets/:id/inspections
POST   /api/v1/assets/:id/defects
PUT    /api/v1/assets/:id/defects/:defId
```

### HR
```
GET    /api/v1/hr/employees
POST   /api/v1/hr/employees
GET    /api/v1/hr/employees/export
POST   /api/v1/hr/employees/import
GET    /api/v1/hr/employees/:id
PUT    /api/v1/hr/employees/:id
GET    /api/v1/hr/payroll/periods
POST   /api/v1/hr/payroll/periods
POST   /api/v1/hr/payroll/periods/:id/run
POST   /api/v1/hr/payroll/periods/:id/approve
GET    /api/v1/hr/payroll/periods/:id/export
GET    /api/v1/hr/leave/requests
POST   /api/v1/hr/leave/requests
PUT    /api/v1/hr/leave/requests/:id
GET    /api/v1/hr/attendance
POST   /api/v1/hr/attendance/clock-in
POST   /api/v1/hr/attendance/clock-out
```

### Reports
```
POST   /api/v1/reports/generate
POST   /api/v1/reports/export
GET    /api/v1/reports/saved
POST   /api/v1/reports/saved
DELETE /api/v1/reports/saved/:id
```

### Import / Export
```
GET    /api/v1/export/template/:entity
POST   /api/v1/import/:entity
POST   /api/v1/import/:entity/confirm
GET    /api/v1/import/:entity/status/:id
```

### Settings
```
GET    /api/v1/settings/profile
PUT    /api/v1/settings/profile
GET    /api/v1/settings/locations
POST   /api/v1/settings/locations
PUT    /api/v1/settings/locations/:id
GET    /api/v1/settings/users
POST   /api/v1/settings/users
PUT    /api/v1/settings/users/:id
DELETE /api/v1/settings/users/:id
GET    /api/v1/settings/intake-template
PUT    /api/v1/settings/intake-template
GET    /api/v1/settings/feature-flags
PUT    /api/v1/settings/feature-flags/:key
GET    /api/v1/settings/notification-preferences
PUT    /api/v1/settings/notification-preferences
```

### Customer Portal (No auth required)
```
GET    /portal/jobs/:token
POST   /portal/jobs/:token/approve
POST   /portal/jobs/:token/callback
```

---

## Part 10 — Frontend Screen Inventory

Every screen the user sees. Grouped by module.

### Auth
- Login
- Forgot Password

### Dashboard
- Main dashboard (KPIs: open jobs, stock alerts, bays status, goods in transit, recent activity)
- Bay Board (live bay status for all bays at location)

### Customers
- Customer list (search, filter, export)
- Customer detail (profile + vehicles + job history + invoices)
- Create customer (individual / organisation toggle)
- Edit customer

### Vehicles
- Vehicle list (search by license plate, make, model)
- Vehicle detail (profile + job history)
- Create vehicle
- Edit vehicle

### Jobs
- Job list (kanban view + list view, filter by status/mechanic/bay/date)
- Job detail (full view: intake, timeline, items, QA, invoice)
- **Intake flow** (multi-step: checklist → photos → signature → confirm)
- AUDIT form (complaint, diagnosis, mechanic assignment, bay selection)
- QUOTE builder (parts picker + labour entries + estimate total)
- Change request modal (submit + approve/decline)
- QA screen (intake reference panel + work checklist + final checks)
- Job search results

### Inventory
- Inventory list (stock status badges, filter by category, low stock filter)
- Inventory detail (item + stock movement history)
- **Add inventory item** (dedicated form with barcode scan)
- Edit inventory item
- Manual stock adjustment modal
- Low stock alerts list

### Purchases
- Purchase order list (filter by status, supplier)
- PO detail (timeline view of lifecycle stages)
- Create PO
- Edit PO (DRAFT only)
- In-transit board (widget + full view)
- GRN entry form (received quantities)
- QA inspection form (per GRN item, photo capture)

### Billing
- Invoice list (filter by status, customer, date)
- Invoice detail + PDF preview
- Create invoice (from job card)
- Record payment

### DVI
- DVI template list (per location)
- DVI template editor (configurable checklist builder)
- DVI result entry (mechanic view, mobile-optimised)
- DVI result detail

### Assets (flag-gated)
- Asset list (filter by category, status, location)
- Asset detail (profile + inspection history + defect history)
- Create asset
- Daily inspection checklist (mobile-first, per asset)
- Defect report form (photo capture, severity picker)
- Open defects board (Manager/Admin view)

### HR (flag-gated)
- Employee list
- Employee detail (profile + payroll history + leave balance)
- Create / edit employee
- Payroll period list
- Payroll period detail + run + approve + export
- Leave request list (manager view)
- Leave request approval (manager approve/reject modal)
- Leave request form (employee view)
- Attendance view (calendar / table)

### Reports
- Report builder (select type → configure filters → preview → export)
- Saved reports list

### Settings
- Workshop profile
- Locations (CRUD)
- Users & roles (CRUD)
- Intake checklist template editor
- DVI template management
- Asset inspection template management
- Bay management
- Feature flags (tenant-level overrides, Owner/Admin only)
- Notification preferences
- Leave types configuration

### Super Admin (separate app shell)
- Tenant list
- Tenant detail + provision
- Global feature flag management
- Platform analytics

---

## Part 11 — Agent Skills Build Plan

These skills will be built sequentially. Each skill is a reusable agent that generates production-ready code for Garage360 following the conventions in this plan.

| # | Skill Name | Purpose | Depends On | Priority |
|---|---|---|---|---|
| 1 | `schema-architect` | Generates the complete PostgreSQL schema SQL | — | ✅ Done |
| 2 | `rust-module-builder` | Generates a complete Axum module (handler + service + repo + types) for any entity | schema-architect | P0 — Next |
| 3 | `state-machine-builder` | Generates the job card transition validator in Rust | rust-module-builder | P0 |
| 4 | `notification-builder` | Email (lettre) + SMS (reqwest to gateway) + push notification patterns | rust-module-builder | P0 |
| 5 | `react-module-builder` | Generates a complete React module (hooks + types + components + routes) | rust-module-builder | P1 |
| 6 | `rbac-enforcer` | Generates Axum RBAC middleware + frontend role/flag guards | rust-module-builder | P1 |
| 7 | `portal-builder` | Generates customer portal pages (access, approve, callback) | react-module-builder | P1 |
| 8 | `docker-composer` | Generates docker-compose.yml, Dockerfiles, nginx.conf | — | P1 |
| 9 | `pwa-configurator` | Vite PWA config, service worker strategy, manifest, offline handling | react-module-builder | P2 |
| 10 | `excel-io-builder` | calamine read + rust_xlsxwriter write patterns + SheetJS frontend | rust-module-builder | P2 |
| 11 | `search-builder` | PostgreSQL FTS tsvector patterns + Axum search endpoint | rust-module-builder | P2 |
| 12 | `pdf-builder` | Intake report PDF + invoice PDF in Rust (printpdf) | rust-module-builder | P2 |
| 13 | `test-writer` | Rust integration tests + React Testing Library tests | All of above | P3 |

---

## Part 12 — Development Sequence

The recommended order to build Garage360 from scratch.

### Phase 0 — Infrastructure (Week 1)
1. Set up repository structure (`api/`, `web/`, `docker/`)
2. Write `docker-compose.yml` and all Dockerfiles (skill: `docker-composer`)
3. Bare Axum app with health check endpoint
4. Bare Vite + React + TypeScript + Tailwind setup
5. Control DB schema + tenant provisioning endpoint
6. JWT auth middleware skeleton

### Phase 1 — Core Auth & Tenant Shell (Week 1-2)
7. Auth module: login, refresh, logout (Rust)
8. Tenant router middleware (Rust)
9. Feature flag resolution middleware (Rust)
10. Login screen + auth state (React)
11. App shell with feature-flag-aware sidebar (React)

### Phase 2 — CRM Foundation (Week 2-3)
12. Customers module: Individual + Org CRUD + FTS search (Rust)
13. Vehicles module: CRUD + license plate search (Rust)
14. Customer screens: list + detail + create forms (React)
15. Vehicle screens (React)

### Phase 3 — Job Card Core (Week 3-5)
16. Service bays CRUD (Rust + React settings screen)
17. Job card creation + INTAKE status (Rust)
18. Intake checklist + photos + signature (Rust + React mobile flow)
19. Job card state machine — all transitions (Rust)
20. Job card list + kanban view (React)
21. AUDIT form: complaint, mechanic assignment, bay selection (React)
22. QUOTE builder: parts picker + labour (React + Rust)
23. APPROVAL: notification + approval recording (Rust + React)
24. IN_SERVICE + change request flow (Rust + React)
25. QA screen with intake reference panel (Rust + React)
26. Job card activity timeline (Rust + React)
27. Bay board (React real-time view)

### Phase 4 — Inventory & Purchasing (Week 5-6)
28. Inventory CRUD + stock threshold alerts (Rust + React)
29. Inventory addition form + barcode scan (React)
30. Purchase order full lifecycle + GRN + QA + stock-in (Rust)
31. PO list + detail + in-transit board (React)

### Phase 5 — Billing (Week 6-7)
32. Invoice generation from job card (Rust)
33. Invoice PDF (Rust)
34. Payment recording (Rust)
35. Invoice screens (React)

### Phase 6 — DVI (Week 7)
36. DVI template editor (Rust + React)
37. DVI result entry — mechanic mobile view (React)

### Phase 7 — Assets (Week 7-8)
38. Asset CRUD (Rust + React)
39. Daily inspection flow (React mobile)
40. Defect reporting + notification (Rust + React)

### Phase 8 — HR & Payroll (Week 8-9)
41. Employee records (Rust + React)
42. Payroll calculation + approval + export (Rust + React)
43. Leave management (Rust + React)
44. Attendance (Rust + React)

### Phase 9 — Reports & Excel (Week 9-10)
45. Report builder backend (Rust dynamic queries)
46. Report builder UI (React)
47. Excel export on all list views (Rust + React)
48. Excel import with validation preview (Rust + React)

### Phase 10 — PWA & Polish (Week 10-11)
49. Service worker + offline support (vite-plugin-pwa)
50. Push notification setup
51. Performance optimisation
52. Industrial Brutalism design system final pass
53. Mobile UX final pass (all screens at 375px)

### Phase 11 — Super Admin Panel (Week 11-12)
54. Tenant provisioning flow
55. Global feature flag management
56. Platform analytics

---

## Part 13 — Repository Structure (Final)

```
garage360/
├── api/                          Rust Axum backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── db/
│   │   │   ├── control.rs        Control DB pool
│   │   │   └── tenant.rs         Per-tenant pool registry (LRU, idle timeout)
│   │   ├── middleware/
│   │   │   ├── auth.rs
│   │   │   ├── tenant.rs
│   │   │   ├── feature_flags.rs
│   │   │   └── rbac.rs
│   │   ├── modules/
│   │   │   ├── auth/
│   │   │   ├── customers/
│   │   │   ├── vehicles/
│   │   │   ├── jobs/
│   │   │   │   ├── intake/
│   │   │   │   ├── bays/
│   │   │   │   ├── change_requests/
│   │   │   │   ├── activities/
│   │   │   │   └── qa/
│   │   │   ├── inventory/
│   │   │   ├── purchases/
│   │   │   ├── billing/
│   │   │   ├── dvi/
│   │   │   ├── assets/
│   │   │   ├── hr/
│   │   │   ├── reports/
│   │   │   ├── import_export/
│   │   │   └── settings/
│   │   ├── control/
│   │   │   ├── tenants/
│   │   │   └── feature_flags/
│   │   ├── background/           Tokio background tasks (import, notifications)
│   │   ├── notifications/        Email + SMS dispatch
│   │   ├── storage/              MinIO/S3 abstraction
│   │   ├── pdf/                  Intake report + invoice PDF generation
│   │   ├── search/               FTS helpers
│   │   └── errors.rs
│   ├── schema/
│   │   └── tenant_schema.sql     Complete schema SQL (run once per new tenant)
│   └── Cargo.toml
│
├── web/                          React TypeScript PWA
│   ├── public/
│   │   ├── manifest.json
│   │   └── icons/
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── api/
│   │   ├── store/
│   │   ├── hooks/
│   │   ├── components/
│   │   │   ├── ui/               shadcn/ui base (Industrial Brutalism theme)
│   │   │   └── shared/
│   │   ├── modules/              (one folder per module)
│   │   ├── layouts/
│   │   └── styles/
│   ├── vite.config.ts
│   └── package.json
│
├── docker/
│   ├── api.Dockerfile
│   ├── web.Dockerfile
│   └── nginx.conf
│
├── docker-compose.yml
├── docker-compose.dev.yml
├── .env.example
└── README.md
```

---

## Part 14 — Locked Decisions

All decisions confirmed. No open questions remain before development starts.

### D1 — SMS: Sparrow SMS (Nepal)
Sparrow SMS integrated via REST API (`api.sparrowsms.com/v2/sms/`). API token + sender ID stored encrypted in `tenant_settings`. Configurable by Admin in Settings → Notifications. Feature flag `jobs.notification_sms` defaults `false` — Admin enables after configuring credentials.

### D2 — Email: Admin-configurable SMTP
Admin enters SMTP credentials (host, port, username, password, from name, from email, TLS flag) in Settings → Email Configuration. "Send Test Email" button validates on save. Credentials stored encrypted in `tenant_settings`, cached in Redis (5-min TTL). Rust uses `lettre` crate, transport built from tenant settings at send time. If unconfigured, email notifications silently skipped + in-app alert to Admin.

### D3 — Currency: NPR default, foreign currency on POs reconciled to NPR
All financial values stored in NPR (`NUMERIC(10,2)`). Purchase orders for imported parts can record `purchase_currency` (TEXT) + `exchange_rate_to_npr` (`NUMERIC(10,6)`). On GRN receipt, cost is reconciled: `unit_cost_npr = unit_cost_foreign × exchange_rate`. Inventory `cost_price` always in NPR. Invoices, reports, dashboard KPIs always display Rs. symbol. Feature flag `billing.multi_currency` — when `true`, PO form shows foreign currency + exchange rate fields. Tenant settings: `currency_symbol` (default "Rs."), `currency_code` (default "NPR").

### D4 — Timezone: Asia/Kathmandu (UTC+5:45), configurable per tenant
All DB timestamps stored as UTC (`TIMESTAMPTZ`). All display and report generation uses `Asia/Kathmandu`. Rust uses `chrono-tz` crate; timezone loaded from `tenant_settings` at request time. Frontend uses `date-fns-tz` with tenant timezone from feature context. Report date range filters: frontend sends local dates, API converts to UTC before querying. Nepal's UTC+5:45 non-standard offset tested explicitly. Tenant setting: `timezone` (default `"Asia/Kathmandu"`).

### D5 — Photos: WebP-compressed, thumbnails generated, bulk-download as .zip
Photos compressed to WebP (max 1200px longest side) on upload. Thumbnail also generated (300px). Stored in MinIO: `originals/intake/{job_id}/{uuid}.webp` + `thumbnails/intake/{job_id}/{uuid}_thumb.webp`. Served via pre-signed URLs (15-min TTL). Bulk download: `GET /jobs/:id/intake/photos/download` streams a `.zip` of all job photos. Rust crates: `image` (compression), `zip` (archive). Max 5MB original per photo, 10 photos per job.

### D6 — Signature & document retention: Configurable per tenant, weekly cleanup
Tenant setting: `document_retention_days` (NULL = keep forever, integer = delete after N days). Applies to: intake signatures, intake photos, DVI result photos, asset defect photos. Weekly Tokio background task (via `tokio-cron-scheduler`) deletes MinIO objects for expired records, sets `file_deleted_at TIMESTAMPTZ` on the DB row. DB metadata record kept even after file deletion. Settings screen: Settings → Documents → Retention Policy.

### D7 — Mechanics can create customers: Allowed, UI de-emphasised
`MECHANIC` role permitted on `POST /customers` and `POST /vehicles`. In UI, intake flow shows "Quick Add Customer" as a text link (not primary button). Mechanic cannot edit or delete customers. All mechanic-created customer records audit-logged with `performed_by_role` noted.

### D8 — Bay conflict: Optimistic locking (409 on conflict)
`PUT /jobs/:id/assign-bay` uses a serializable sqlx transaction with `FOR UPDATE` on the target `service_bays` row. If bay is already `OCCUPIED` or `RESERVED`, returns `409` with current occupant job number. Frontend re-fetches bay board on 409 and shows toast: "Bay [code] just taken by [Job No] — please select another."

### D9 — Customer portal: Online estimate + change request approval
When `module.customer_portal` is enabled, customers receive a short-lived signed approval token (32 bytes, cryptographically random, 48-hour expiry, single-use) via email or SMS. Token link opens a minimal portal page (no auth shell) showing estimate line items and total. Customer taps **Approve** or **Request Callback**. On approval, token immediately invalidated, `job_card_approvals` record created with `channel = EMAIL_LINK` or `SMS_LINK`. Same mechanism applies to mid-service change requests. New routes: `GET /portal/jobs/:token`, `POST /portal/jobs/:token/approve`, `POST /portal/jobs/:token/callback`. New columns on `job_card_approvals`: `portal_token TEXT UNIQUE`, `portal_token_expires_at TIMESTAMPTZ`.

### D10 — Payroll: Configurable deduction components per tenant
Deduction components defined in Settings → HR → Payroll Configuration. Each component: name, type (`PCT_OF_GROSS` | `FIXED_AMOUNT` | `SLAB`), value (`JSONB` — for slabs: `[{from, to, rate}]`), applies-to (employment type filter), active flag. Nepal defaults seeded on tenant creation: Income Tax (slab-based brackets), Provident Fund 10% (full-time only), SSF 1% (full-time only). Payroll run: Rust loads active configs, calculates per employee, stores breakdown in `payroll_entries.deductions JSONB`. New table: `payroll_deduction_configs`.

---

## Part 15 — Schema Additions from Decisions

The following schema additions are required for specific features. These tables/types are now included in Part 4's complete table list.

### `job_card_approvals` additions (D9):
```sql
-- Add to existing job_card_approvals table:
portal_token            TEXT UNIQUE,
portal_token_expires_at TIMESTAMPTZ,
portal_token_used_at    TIMESTAMPTZ
```

### `customer_signatures` additions (D6):
```sql
-- Add to existing customer_signatures table:
file_deleted_at     TIMESTAMPTZ   -- set by retention cleanup task, NULL = file exists
```

### Schema Versioning
Every tenant database has a `schema_version` in `tenant_settings` (key = `"schema_version"`, value = semantic version e.g., `"1.0.0"`). On tenant provisioning, insert: `INSERT INTO tenant_settings (key, value, is_encrypted) VALUES ('schema_version', '1.0.0', FALSE);`
