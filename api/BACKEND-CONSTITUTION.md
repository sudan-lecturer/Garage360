# Garage360 — Backend Constitution

**Version:** 1.0
**Status:** Active
**Scope:** Rust API (`/api`)

---

## 1. Core Principles

### 1.1 Product Reality
Garage360 backend is a multi-tenant workshop management API built with Rust and Axum.

Non-negotiables:
- Every tenant has its own PostgreSQL database
- No shared tenant tables for business data
- No incremental migrations for tenant databases
- Tenant databases are provisioned from a complete schema
- Feature flags control module visibility and behavior per tenant
- All server-enforced workflow rules live in the API, not the frontend

### 1.2 Build and Runtime Rules
- Use Docker for backend build, run, and test workflows
- Prefer `docker-compose -f docker-compose.dev.yml ...` commands from the repo root
- Keep the API deployable as one service in the shared stack
- Health endpoints must remain available at `/health/liveness` and `/health/readiness`

### 1.3 Technology Baseline
| Concern | Choice |
|---------|--------|
| Language | Rust |
| HTTP | Axum |
| Async runtime | Tokio |
| Database | PostgreSQL + `sqlx` |
| Auth | JWT HS256 |
| Password hashing | Argon2id |
| Validation | `validator` |
| Serialization | `serde` + `serde_json` |
| Errors | `thiserror` + structured API responses |
| Logging | `tracing` + JSON output |
| Cache | Redis |
| Object storage | MinIO / S3-compatible |

---

## 2. Architecture Rules

### 2.1 Topology
- The backend serves tenant APIs under `/api/v1/*`
- The backend serves control-plane APIs under `/control/v1/*`
- Control-plane data lives in the control database
- Tenant business data lives only in the tenant database selected for the request
- Redis is used for cache, session support, blocklists, and rate-limiting support
- MinIO stores photos, signatures, documents, exports, and other object assets

### 2.2 Tenant Isolation
- Tenant identity comes from authenticated context, not request body trust
- JWT claims must include `tenant_id`
- Each request resolves the correct tenant database from control-plane data
- Cross-tenant reads and writes are never permitted
- Tenant isolation bugs are severity-one defects

### 2.3 Schema Strategy
- Tenant databases are created from a full schema script
- Do not introduce a migration runner for tenant databases
- Schema version must be recorded in tenant settings
- Use UTC timestamps in storage
- Tenant-local display and reporting timezone is configurable per tenant

### 2.4 Data Modeling Rules
- Primary keys: UUID v7 generated in Rust
- Money: `NUMERIC(10,2)`
- Quantities: `NUMERIC(10,3)`
- Timestamps: `TIMESTAMPTZ`
- Root-entity soft delete pattern: `is_active`
- Append-only audit/history tables stay immutable

---

## 3. Code Organization

### 3.1 Source Layout
Backend code should follow this structure:

```text
api/src/
├── main.rs
├── config.rs
├── errors.rs
├── db/
├── middleware/
├── modules/
├── background/
├── notifications/
├── storage/
├── pdf/
└── search/
```

### 3.2 Module Boundaries
- Organize features by business module under `src/modules/`
- Keep control-plane modules separate from tenant modules
- Prefer one module per bounded domain: `auth`, `customers`, `vehicles`, `jobs`, `inventory`, `purchases`, `billing`, `tenant`, `control`
- Jobs-related subdomains may split further by lifecycle area when complexity demands it

### 3.3 Internal Layering
Prefer this shape inside a backend module:

```text
module/
├── mod.rs
├── routes.rs
├── types.rs
├── service.rs
└── repo.rs
```

Rules:
- `routes` owns HTTP extraction, response mapping, and route registration
- `service` owns business rules and workflow enforcement
- `repo` owns SQL and persistence details
- `types` owns request, response, DTO, and domain helper types
- Keep SQL out of unrelated modules

If a module is still small, a combined `routes.rs` file is acceptable, but new work should trend toward explicit layering rather than growing god-files.

---

## 4. HTTP and API Contracts

### 4.1 Route Conventions
- Tenant routes live under `/api/v1`
- Control routes live under `/control/v1`
- Use nouns for resources and verb-style suffixes only for workflow actions
- Keep route naming aligned with the master plan route map

Examples:
- `POST /api/v1/auth/login`
- `GET /api/v1/customers`
- `PUT /api/v1/jobs/:id/assign-bay`
- `POST /api/v1/jobs/:id/transition`
- `POST /control/v1/tenants`

### 4.2 Request Handling
- Validate all input at the boundary
- Reject malformed or incomplete requests with clear 4xx responses
- Never trust frontend gating for authorization or workflow sequencing
- Use server-side checks for role access, feature flags, tenant ownership, and state transitions

### 4.3 Response Shape
Errors must be structured JSON in this form:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Email is required"
  }
}
```

Rules:
- Use stable machine-readable error codes
- Avoid leaking internals in production-facing messages
- Log internals with `tracing`, return safe client messages

### 4.4 Status Code Rules
- `200` / `201` for successful reads and creates
- `400` for validation failures
- `401` for missing or invalid auth
- `403` for authenticated but unauthorized requests
- `404` for missing resources
- `409` for domain conflicts and concurrent state collisions
- `503` when core dependencies are unavailable

---

## 5. Security, Auth, and Access Control

### 5.1 Authentication
- Use JWT access and refresh tokens
- Access tokens must include user id, tenant id, and role
- Reject malformed `Authorization: Bearer ...` headers
- Token validation belongs in middleware/extractors, not duplicated in handlers

### 5.2 Authorization
- Enforce RBAC in the API for every protected route
- Use explicit allowed-role checks
- Super-admin control-plane access must remain isolated from tenant APIs
- Mechanic shortcuts and restricted exceptions must be intentional and audit-logged

### 5.3 Feature Flags
- Feature flags have global defaults with tenant overrides
- Disabled features must be functionally inaccessible, not just hidden in UI
- Feature-flag lookups may be cached, but cache invalidation must be supported

### 5.4 Secrets and Sensitive Data
- Load secrets from config/environment, not hardcoded values
- Encrypt tenant-stored credentials where required
- Never log passwords, raw secrets, or full sensitive tokens

---

## 6. Business Logic Rules

### 6.1 Workflow Enforcement
- Job lifecycle rules are backend-owned
- Every transition must validate prerequisites on the server
- Invalid transitions must fail deterministically
- Cancellation rules require reason and appropriate role checks

### 6.2 Auditability
- Status changes, assignments, approvals, and notable actions must be audit-logged
- Append-only activity tables must not be rewritten in normal flows
- Mechanic-created customer or vehicle shortcuts must be explicitly auditable

### 6.3 Concurrency
- Protect contested resources with transactions and locking where needed
- Use database-level guarantees for occupancy and uniqueness conflicts
- Return `409` on race conditions that the client can recover from

### 6.4 Notifications and Documents
- Notification delivery logic belongs in backend services
- PDF generation and object storage flows must remain backend-controlled
- Retention cleanup jobs must preserve metadata even when files are deleted

---

## 7. Database and SQLx Standards

### 7.1 Query Rules
- Prefer `sqlx` for all database access
- Keep queries explicit and readable
- Bind values; never build unsafe SQL from unchecked input
- Use compile-time-checked queries where practical

### 7.2 Transaction Rules
- Use transactions for multi-step writes that must succeed atomically
- Use stricter isolation where business invariants depend on it
- Keep transaction scopes tight

### 7.3 Control DB vs Tenant DB
- Control DB stores tenant registry, feature flags, super-admin data, and control audit logs
- Tenant DB stores workshop business data only
- Do not mix control-plane and tenant writes in a way that obscures failure handling

---

## 8. Config, Observability, and Background Work

### 8.1 Configuration
- Centralize runtime configuration in `config.rs`
- Support environment-based configuration with clear naming
- Fail fast on missing required production configuration

### 8.2 Logging and Diagnostics
- Use structured `tracing` logs
- Log request flow, dependency failures, and authorization failures at appropriate levels
- Prefer consistent event names and fields over ad-hoc log messages

### 8.3 Health Checks
- Liveness checks process availability
- Readiness verifies required dependencies, especially database access
- Keep health endpoints lightweight and stable

### 8.4 Background Tasks
- Background jobs must be explicit modules, not hidden side effects in request handlers
- Periodic cleanup, retention, or async notification work should use Tokio-based background infrastructure

---

## 9. Testing Standards

### 9.1 Required Test Coverage
Every backend change should add or update tests close to the code it changes.

Minimum expectations:
- Unit tests for request/response types and validation
- Unit tests for auth, RBAC, and feature-flag logic
- Tests for error-to-status-code mapping
- Tests for workflow edge cases and conflict handling

### 9.2 Integration Expectations
- Prefer database-backed tests for repository and transaction-heavy logic
- Use transaction-scoped tests where possible
- Test both happy path and business-rule failures

### 9.3 Regression Focus
High-priority regression areas:
- Tenant isolation
- Role enforcement
- Feature-flag enforcement
- Job status transitions
- Bay assignment collisions
- Auth token parsing and expiry handling

---

## 10. Development Workflow

### 10.1 Commands
Use Docker-first commands from the repository root:

```bash
docker-compose -f docker-compose.dev.yml up -d
docker-compose -f docker-compose.dev.yml build api
docker-compose -f docker-compose.dev.yml exec api cargo test
docker-compose -f docker-compose.dev.yml logs -f api
```

### 10.2 Change Discipline
- Keep changes scoped to the module being worked on
- Prefer small, reviewable backend slices
- Update docs when architecture or conventions change
- Do not add alternate patterns when a project standard already exists

### 10.3 Definition of Done
Backend work is not done until:
- Routes, services, and persistence boundaries are clear
- Validation, auth, RBAC, and feature-flag checks are enforced
- Errors are structured correctly
- Tests cover the new behavior or rule
- The change still fits the multi-tenant model

---

## 11. Backend Review Checklist

- Does this change preserve tenant isolation?
- Are workflow rules enforced in the API instead of the client?
- Are control-plane and tenant concerns kept separate?
- Is the database access pattern explicit and safe?
- Are conflicts, missing data, and dependency failures mapped to correct status codes?
- Are logs useful without leaking sensitive information?
- Are tests covering the real regression risk?

---

## 12. Source of Truth

When documents conflict, use this order:
1. `docs/MASTER-PLAN.md`
2. `api/BACKEND-CONSTITUTION.md`
3. Current backend implementation

If implementation diverges from this document, either:
- update the implementation to match the constitution, or
- intentionally revise this constitution and the master plan together
