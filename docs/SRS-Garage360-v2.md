# Software Requirements Specification — Garage360
**Version:** 2.0.0
**Status:** Active
**Last Updated:** 2025

---

## 1. Introduction

### 1.1 Project Overview
**Garage360** is a multi-tenant, cloud-native SaaS platform for vehicle service centers and garages. It provides a full workshop operating system: CRM, job card lifecycle management, digital vehicle inspection (DVI), inventory management, purchasing, billing, and analytics — all under a single deployable product that multiple independent workshops can operate simultaneously with complete data isolation.

### 1.2 Goals
- One codebase, deployable for any number of independent workshop tenants
- Each tenant's data is fully isolated (separate PostgreSQL database per tenant)
- Super Admin can provision tenants and control feature availability globally or per tenant
- Mobile-first Progressive Web App usable on the workshop floor on any device
- Containerised via Docker for consistent deployments across cloud and on-premise

### 1.3 Scope
The system covers all internal workshop operations and a customer-facing portal. It does not include payments processing (integrated via third-party), vehicle parts procurement from external marketplaces, or fleet management beyond individual vehicle tracking.

---

## 2. Architecture Overview

### 2.1 System Topology

```
┌─────────────────────────────────────────────────────┐
│                   Docker Host                        │
│                                                     │
│  ┌─────────────────┐     ┌──────────────────────┐  │
│  │  React PWA      │     │   Rust API (Axum)    │  │
│  │  (Nginx)        │────▶│   Port 8080          │  │
│  │  Port 80/443    │     │                      │  │
│  └─────────────────┘     └──────────┬───────────┘  │
│                                     │               │
│  ┌──────────────────────────────────▼────────────┐  │
│  │              Tenant Router                    │  │
│  │  Resolves tenant from JWT → picks DB pool     │  │
│  └──┬──────────────────┬──────────────────┬─────┘  │
│     │                  │                  │         │
│  ┌──▼──────┐     ┌─────▼────┐     ┌──────▼──────┐  │
│  │ tenant_ │     │ tenant_  │     │  tenant_N   │  │
│  │ alpha   │     │ beta     │     │  (postgres) │  │
│  │(postgres│     │(postgres)│     │             │  │
│  └─────────┘     └──────────┘     └─────────────┘  │
│                                                     │
│  ┌─────────────────┐     ┌──────────────────────┐  │
│  │  Control DB     │     │   Redis (cache +     │  │
│  │  (postgres)     │     │   sessions)          │  │
│  │  Super Admin    │     │                      │  │
│  └─────────────────┘     └──────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 2.2 Database Strategy — Separate Database per Tenant

Each tenant (workshop) is provisioned with its own isolated PostgreSQL database. A central **Control DB** stores only tenant metadata, feature flags, and Super Admin accounts. No workshop data ever touches the Control DB.

**Tenant lifecycle:**
1. Super Admin creates a tenant via the control panel
2. API provisions a new PostgreSQL database (`tenant_{slug}`)
3. Runs migrations on that database automatically
4. Issues a tenant JWT signing key and stores metadata in Control DB
5. Tenant can immediately log in

**Isolation guarantees:**
- A bug that exposes one tenant's data cannot reach another tenant's database
- Each tenant DB can be backed up, restored, or migrated independently
- A tenant can be given their own dedicated server (on-premise) by simply running the same Docker stack with a single tenant configured
- Regulatory compliance (data residency) is achievable by running the tenant DB in the required region

**Trade-offs accepted:**
- Cross-tenant analytics must be done by the API aggregating across DB connections (Super Admin reports only)
- Schema migrations must run against every tenant DB — handled by the `migration-runner` service in Docker
- Connection pool must be managed carefully — use one pool per active tenant, with idle timeouts

### 2.3 Service Decomposition

| Service | Language / Runtime | Responsibility |
|---|---|---|
| `api` | Rust (Axum) | All business logic, REST API, auth, tenant routing |
| `web` | React 19 + TypeScript (Vite) | PWA frontend, served by Nginx |
| `control-db` | PostgreSQL 16 | Super Admin data, tenant registry, global feature flags |
| `tenant-db-N` | PostgreSQL 16 | All data for one tenant |
| `redis` | Redis 7 | JWT blocklist, rate limiting, feature flag cache |
| `migration-runner` | Rust binary | Runs `sqlx migrate` against all tenant DBs on deploy |
| `nginx` | Nginx | Reverse proxy, SSL termination, static file serving |

---

## 3. Tech Stack

### 3.1 Backend — Rust

| Concern | Crate | Reason |
|---|---|---|
| HTTP framework | `axum` | Ergonomic, tower-native, async-first |
| Async runtime | `tokio` | Industry standard, axum dependency |
| Database ORM | `sqlx` | Compile-time checked queries, async, PostgreSQL native |
| Migrations | `sqlx migrate` | Built into sqlx, runs at startup |
| Auth / JWT | `jsonwebtoken` | JWT encode/decode, HS256/RS256 |
| Password hashing | `argon2` | Secure, modern hashing |
| Validation | `validator` | Struct-level field validation with derive macros |
| Serialisation | `serde` + `serde_json` | Universal Rust serialisation |
| Error handling | `thiserror` + `anyhow` | Typed errors in lib, dynamic errors in bin |
| Env config | `dotenvy` + `config` | Layered config (env → file → defaults) |
| Logging | `tracing` + `tracing-subscriber` | Structured async-aware logging |
| Redis client | `redis` (async) | JWT blocklist, caching |
| UUID | `uuid` (v7) | Time-sortable UUIDs for all PKs |
| Feature flags | Custom middleware | Resolves global + tenant-level flags |
| Testing | `tokio-test`, `sqlx` test transactions | Unit + integration tests |

### 3.2 Frontend — React + TypeScript

| Concern | Library | Reason |
|---|---|---|
| Framework | React 19 | Latest concurrent features |
| Language | TypeScript 5 | Strict typing throughout |
| Build tool | Vite 6 | Fast HMR, optimised PWA builds |
| Routing | React Router v7 | File-based + nested routing |
| State management | Zustand | Lightweight, no boilerplate |
| Server state | TanStack Query v5 | Caching, background refetch, optimistic updates |
| Forms | React Hook Form + Zod | Type-safe validation |
| UI components | shadcn/ui (customised) | Accessible, unstyled primitives |
| Icons | lucide-react | Consistent icon set |
| Styling | Tailwind CSS v4 | Utility-first, design token support |
| PWA | vite-plugin-pwa | Service worker, manifest, offline support |
| Charts | Recharts | Composable, React-native charts |
| Tables | TanStack Table v8 | Virtualised, sortable, filterable |
| HTTP client | Axios + custom hooks | Typed API layer with interceptors |
| i18n | react-i18next | Multi-language ready |

### 3.3 Infrastructure

| Component | Technology |
|---|---|
| Containerisation | Docker + Docker Compose |
| Web server / proxy | Nginx (Alpine) |
| Database | PostgreSQL 16 (Alpine) |
| Cache / sessions | Redis 7 (Alpine) |
| CI/CD | GitHub Actions |
| Secrets | Docker secrets / environment injection |
| SSL | Let's Encrypt via Certbot (optional) or bring-your-own cert |

---

## 4. Multi-Tenancy Design

### 4.1 Tenant Identification

Every API request carries a JWT. The JWT payload contains:
```json
{
  "sub": "user-uuid",
  "tenant_id": "tenant-uuid",
  "tenant_slug": "workshop-alpha",
  "role": "MECHANIC",
  "exp": 1234567890
}
```

Axum middleware extracts `tenant_id`, looks up the tenant's DB connection string from the Control DB (cached in Redis for 5 minutes), and injects a `TenantDb` extractor into the request. Every handler that touches business data receives this extractor — they cannot accidentally query the wrong DB.

### 4.2 Tenant Provisioning API (Super Admin only)

```
POST /control/tenants          — Create tenant + provision DB + run migrations
GET  /control/tenants          — List all tenants
GET  /control/tenants/:id      — Tenant detail + health
PUT  /control/tenants/:id      — Update name, plan, status
DELETE /control/tenants/:id    — Soft-deactivate (DB preserved)
POST /control/tenants/:id/migrate — Manually trigger migration run
```

### 4.3 Control DB Schema (simplified)

```sql
tenants           — id, slug, name, db_url (encrypted), plan, is_active, created_at
feature_flags     — id, tenant_id (nullable), flag_key, is_enabled, created_at
super_admin_users — id, email, password_hash, created_at
audit_logs        — id, admin_id, action, target_tenant_id, metadata, created_at
```

---

## 5. Feature Flag System

### 5.1 Resolution Logic

Feature flags are resolved in this priority order:
1. Tenant-specific override (if exists) → use it
2. Global default → use it
3. Hard-coded fallback → `false` (disabled)

### 5.2 Available Feature Flags

| Flag Key | Default | Description |
|---|---|---|
| `module.dvi` | `true` | Digital Vehicle Inspection module |
| `module.purchases` | `true` | Purchase Orders / GRN module |
| `module.reports` | `true` | Analytics and reporting |
| `module.customer_portal` | `false` | Customer self-service portal |
| `module.loyalty` | `false` | Loyalty points system |
| `billing.vat` | `false` | VAT/tax line on invoices |
| `billing.multi_currency` | `false` | Multi-currency support |
| `jobs.approval_workflow` | `true` | Quote approval step in job lifecycle |
| `jobs.dvi_required` | `false` | Block billing until DVI is complete |
| `inventory.low_stock_alerts` | `true` | Notify manager on low stock |

### 5.3 Frontend Flag Consumption

The frontend fetches the active flag set on login:
```
GET /api/v1/feature-flags   → { "module.dvi": true, "module.purchases": false, ... }
```
Zustand stores them. React components gate rendering:
```tsx
const { flags } = useFeatureFlags();
if (!flags['module.dvi']) return null;
```
Disabled modules are completely absent from the sidebar — not greyed out, not visible.

---

## 6. UI Design System — Industrial Brutalism

The interface design language remains **Industrial Brutalism** with the following specification:

### 6.1 Color Tokens
```css
--color-base:        #121416;   /* page background */
--color-surface:     #1E2022;   /* panels, cards */
--color-surface-alt: #252829;   /* elevated surfaces */
--color-border:      #2E3235;   /* dividers, outlines */
--color-primary:     #FFD100;   /* Safety Yellow — primary CTA */
--color-secondary:   #FF5800;   /* High-Vis Orange — alerts, warnings */
--color-text-primary:   #F0F0F0;
--color-text-secondary: #9CA3AF;
--color-text-muted:     #6B7280;
--color-success:     #22C55E;
--color-danger:      #EF4444;
--color-info:        #3B82F6;
```

### 6.2 Typography
| Role | Font | Weight | Style |
|---|---|---|---|
| Headers / Nav | Space Grotesk | 700 | Uppercase |
| Body text | Work Sans | 400/500 | Normal |
| Technical data | IBM Plex Mono | 400 | (VINs, part numbers, amounts) |

### 6.3 Component Rules
- Zero drop shadows — tonal shifts only for elevation
- 2px border radius on all interactive elements
- Hover states: bottom border 2px `--color-primary`
- Focus states: outline 2px `--color-primary`, offset 2px
- Sharp table rows, no rounded cells
- Status badges: filled pill, monospace text
- All currency / numeric output: IBM Plex Mono

---

## 7. PWA Requirements

### 7.1 Progressive Web App Capabilities
- Installable on Android, iOS, and desktop Chrome
- Offline support for read operations (job card list, inventory list) via service worker cache
- Background sync for form submissions when offline
- Push notifications for: job status changes, low stock alerts, approval requests
- App manifest: name, icons (192px, 512px), theme color `#121416`, display `standalone`

### 7.2 Mobile-First Design Principles
- Base breakpoint: 375px (iPhone SE)
- Touch targets: minimum 44×44px
- Navigation: bottom tab bar on mobile (`< 768px`), side rail on tablet, full sidebar on desktop
- Tables degrade to card lists on mobile
- Forms use native mobile inputs (date pickers, number pads)
- Swipe gestures on job card kanban columns
- Camera access for DVI photo capture on mobile

### 7.3 Performance Targets
| Metric | Target |
|---|---|
| Lighthouse PWA score | ≥ 90 |
| First Contentful Paint | < 1.5s |
| Time to Interactive | < 3.0s |
| Offline functionality | Core read views |
| Bundle size (initial) | < 200KB gzipped |

---

## 8. Authentication & RBAC

### 8.1 Auth Flow
1. User POSTs credentials to `POST /api/v1/auth/login`
2. Rust validates password (argon2), checks `is_active`, checks tenant `is_active`
3. Issues short-lived access token (15min) + long-lived refresh token (7 days, stored in Redis)
4. Refresh token rotation: each use issues a new refresh token; old one is immediately invalidated
5. Logout: access token added to Redis blocklist (TTL = remaining expiry)

### 8.2 Roles and Permissions

| Role | Scope | Key Permissions |
|---|---|---|
| `SUPER_ADMIN` | Control DB | Tenant management, global flags, platform analytics |
| `OWNER` | Tenant | All permissions within tenant |
| `ADMIN` | Tenant | All except owner-level billing and tenant settings |
| `MANAGER` | Tenant | Job cards, inventory, purchasing, staff management |
| `MECHANIC` | Tenant | Assigned job cards, DVI, limited inventory view |
| `CASHIER` | Tenant | Billing, invoices, payment receipting |
| `CUSTOMER` | Tenant | Own vehicles, job history, portal (if enabled) |

### 8.3 RBAC Enforcement
- Enforced in Axum middleware via a `require_role!` macro
- Route-level: `GET /api/v1/jobs` — requires `[OWNER, ADMIN, MANAGER, MECHANIC, CASHIER]`
- Resource-level: Mechanics can only update jobs assigned to them
- All role checks happen in the API — the frontend hides UI but the API enforces it

---

## 9. Backend Module Structure (Rust)

```
api/
├── src/
│   ├── main.rs                  — Axum app setup, router, middleware stack
│   ├── config.rs                — Environment config struct
│   ├── db/
│   │   ├── mod.rs               — DB pool management
│   │   ├── control.rs           — Control DB pool (Super Admin)
│   │   └── tenant.rs            — Per-tenant pool registry + resolver
│   ├── middleware/
│   │   ├── auth.rs              — JWT extraction + validation
│   │   ├── tenant.rs            — Tenant DB injection
│   │   └── feature_flags.rs     — Flag resolution + injection
│   ├── modules/
│   │   ├── auth/                — Login, refresh, logout
│   │   ├── customers/           — CRM endpoints
│   │   ├── vehicles/            — Vehicle endpoints
│   │   ├── jobs/                — Job card lifecycle
│   │   ├── inventory/           — Parts catalogue
│   │   ├── purchases/           — Purchase orders
│   │   ├── billing/             — Invoice generation
│   │   ├── dvi/                 — Inspection templates + results
│   │   ├── reports/             — Analytics queries
│   │   └── settings/            — Tenant + location config
│   ├── control/
│   │   ├── tenants/             — Tenant CRUD + provisioning
│   │   └── feature_flags/       — Global + per-tenant flag management
│   ├── errors.rs                — AppError enum + IntoResponse impl
│   └── models/                  — Shared structs, DB row types
├── migrations/                  — sqlx migration files (run per tenant DB)
└── Cargo.toml
```

Each module follows this internal structure:
```
module/
├── mod.rs       — Router registration
├── handlers.rs  — Axum handler functions
├── service.rs   — Business logic (pure functions, no HTTP types)
├── repo.rs      — sqlx queries (database layer)
└── types.rs     — Request/response structs with serde + validator
```

---

## 10. Frontend Module Structure (React)

```
web/
├── public/
│   ├── manifest.json            — PWA manifest
│   └── icons/                   — PWA icons (192, 512)
├── src/
│   ├── main.tsx
│   ├── App.tsx                  — Root router + providers
│   ├── api/
│   │   ├── client.ts            — Axios instance + interceptors
│   │   └── endpoints/           — Typed API functions per module
│   ├── store/
│   │   ├── auth.store.ts        — User + token state
│   │   └── flags.store.ts       — Feature flag state
│   ├── hooks/
│   │   ├── useFeatureFlags.ts
│   │   ├── useAuth.ts
│   │   └── useTenant.ts
│   ├── components/
│   │   ├── ui/                  — shadcn/ui base components (customised)
│   │   └── shared/              — App-specific shared components
│   ├── modules/
│   │   ├── auth/
│   │   ├── dashboard/
│   │   ├── customers/
│   │   ├── vehicles/
│   │   ├── jobs/
│   │   ├── inventory/
│   │   ├── purchases/
│   │   ├── billing/
│   │   ├── dvi/
│   │   ├── reports/
│   │   └── settings/
│   ├── layouts/
│   │   ├── AppLayout.tsx        — Sidebar + header shell
│   │   ├── MobileLayout.tsx     — Bottom nav shell
│   │   └── AuthLayout.tsx       — Centred auth pages
│   └── styles/
│       ├── globals.css          — CSS variables (design tokens)
│       └── tailwind.config.ts
├── vite.config.ts               — Vite + PWA plugin config
└── package.json
```

Each frontend module follows this structure:
```
module/
├── index.tsx          — Route entry + layout
├── components/        — Module-specific components
├── hooks/             — TanStack Query hooks for this module
├── types.ts           — TypeScript interfaces matching API types
└── routes.tsx         — React Router route definitions
```

---

## 11. Docker Architecture

### 11.1 docker-compose.yml Structure

```yaml
services:
  nginx:        # Reverse proxy + static file server
  api:          # Rust Axum backend
  web:          # React PWA (build output served by nginx)
  control-db:   # PostgreSQL — Super Admin control plane
  redis:        # Session store + cache
  migration-runner:  # One-shot: runs migrations on all tenant DBs

networks:
  internal:     # api ↔ databases ↔ redis (not exposed)
  public:       # nginx only (exposed)

volumes:
  control-db-data:
  redis-data:
  tenant-db-data-N:   # One volume per provisioned tenant
```

### 11.2 Environment Configuration

All configuration via environment variables. No hardcoded values in source.

```env
# Control plane
CONTROL_DATABASE_URL=postgres://...
REDIS_URL=redis://redis:6379

# API
JWT_SECRET=<secret>
JWT_EXPIRY_SECONDS=900
REFRESH_TOKEN_EXPIRY_SECONDS=604800
API_PORT=8080
RUST_LOG=info

# Tenant provisioning
TENANT_DB_HOST=tenant-db
TENANT_DB_PORT=5432
TENANT_DB_SUPERUSER=postgres
TENANT_DB_SUPERUSER_PASSWORD=<secret>

# Frontend (build-time)
VITE_API_BASE_URL=https://api.yourdomain.com
VITE_APP_NAME=Garage360
```

### 11.3 Deployment Modes

| Mode | Description | Use Case |
|---|---|---|
| **Cloud SaaS** | Single host, all tenants share infrastructure | Default — most cost-effective |
| **On-Premise Single Tenant** | Same Docker stack, one tenant configured | Workshop wants own server |
| **On-Premise Multi-Tenant** | Same Docker stack, N tenants on client's infra | Enterprise / franchise group |

The Docker stack is identical across all modes — only environment variables differ.

---

## 12. Core Modules & API Routes

### 12.1 Route Overview

All tenant routes prefixed `/api/v1/`. Control routes prefixed `/control/v1/`.

| Module | Key Routes | Feature Flag |
|---|---|---|
| Auth | `/auth/login`, `/auth/refresh`, `/auth/logout` | Always on |
| Dashboard | `/dashboard/summary` | Always on |
| Customers | `/customers` CRUD | Always on |
| Vehicles | `/vehicles` CRUD | Always on |
| Jobs | `/jobs` CRUD + status transitions | Always on |
| Job Items | `/jobs/:id/items` CRUD | Always on |
| Inventory | `/inventory` CRUD | Always on |
| DVI Templates | `/dvi/templates` CRUD | `module.dvi` |
| DVI Results | `/dvi/results` CRUD | `module.dvi` |
| Purchases | `/purchases` CRUD | `module.purchases` |
| Billing | `/invoices` CRUD + payment | Always on |
| Reports | `/reports/*` | `module.reports` |
| Settings | `/settings/locations`, `/settings/users` | Always on |
| Feature Flags | `/feature-flags` (read) | Always on |

---

## 13. Database Schema Design

### 13.1 Tenant DB

All tenant data lives in isolated PostgreSQL databases. Schema uses UUIDs v7 (time-sortable) for all primary keys. All monetary values use `NUMERIC(10,2)`.

Core models: `users`, `locations`, `customers`, `vehicles`, `job_cards`, `job_card_items`, `inventory_items`, `suppliers`, `purchase_orders`, `purchase_order_items`, `invoices`, `invoice_line_items`, `dvi_templates`, `dvi_results`, `audit_logs`.

Full schema maintained in `api/migrations/` as versioned `sqlx` migration files.

### 13.2 Key Design Decisions
- UUIDs v7 for PKs — time-sortable, no sequential enumeration risk
- `NUMERIC(10,2)` for all money — no floating point errors
- Soft deletes via `is_active` on root entities
- All FK constraints include explicit `ON DELETE` rules
- Composite indexes on high-traffic filter combinations (`location_id + status`, `customer_id + created_at`)
- `audit_logs` is append-only — no updates or deletes ever

---

## 14. Job Card Lifecycle

```
AUDIT → QUOTE → APPROVAL → IN_SERVICE → INSPECTION → BILLING → COMPLETED
                                                    ↘ CANCELLED (any stage)
```

State transitions are enforced in the Rust service layer, not just the frontend. An invalid transition (e.g. jumping from `AUDIT` to `BILLING`) returns `422 Unprocessable Entity`. The `jobs.approval_workflow` feature flag, when off, skips the `APPROVAL` state entirely.

---

## 15. Security Requirements

| Requirement | Implementation |
|---|---|
| Passwords | argon2id with per-user salt |
| JWT signing | HS256 with per-tenant secret (stored in Control DB) |
| Refresh token rotation | Single-use, invalidated on use, stored in Redis |
| SQL injection | sqlx compile-time query checking; parameterised queries only |
| Rate limiting | Redis-based, per IP + per user, on all auth endpoints |
| Tenant isolation | Middleware-enforced DB routing; no cross-tenant query path exists |
| Secrets | Docker secrets / env injection; never in source code or images |
| Audit logging | All mutations write to `audit_logs`; append-only |
| HTTPS | Nginx SSL termination; HTTP redirected to HTTPS in production |

---

## 16. Development Workflow

### 16.1 Repository Structure
```
garage360/
├── api/           — Rust Axum backend
├── web/           — React TypeScript PWA
├── docker/        — Dockerfiles, nginx config
├── docker-compose.yml
├── docker-compose.dev.yml
├── .env.example
└── README.md
```

### 16.2 Local Development
```bash
# Start all services (dev mode with hot reload)
docker compose -f docker-compose.dev.yml up

# API: cargo-watch hot reload on save
# Web: Vite HMR
# DBs: persistent volumes

# Run migrations manually
docker compose exec api cargo sqlx migrate run

# Seed a dev tenant
docker compose exec api cargo run --bin seed-dev
```

### 16.3 Linting & Quality
- Rust: `clippy` (pedantic), `rustfmt`
- TypeScript: `eslint` (strict), `prettier`
- Pre-commit hooks via `husky`: lint + format on staged files
- All PRs require passing CI: tests + lint + build

---

## 17. Agent Skills Plan

The following skills will be built to assist re-development of this system:

| # | Skill | Status |
|---|---|---|
| 1 | `schema-architect` | Complete (needs update for sqlx + tenant model) |
| 2 | `rust-module-builder` | Next — generates Axum handler/service/repo/types per module |
| 3 | `react-module-builder` | Generates React module with hooks, types, components |
| 4 | `docker-composer` | Generates docker-compose, Dockerfiles, nginx config |
| 5 | `pwa-configurator` | Vite PWA config, manifest, service worker strategy |
| 6 | `rbac-enforcer` | Axum middleware + frontend flag/role gating |
| 7 | `test-writer` | Rust integration tests + React Testing Library tests |
