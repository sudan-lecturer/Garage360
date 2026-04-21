# Garage360 — Agent Guidance

## Project Overview

Multi-tenant workshop management SaaS built with Rust (Axum) backend and React 19 frontend.

## Directory Structure

```
/Users/manees/dev/Garage360/
├── api/                    # Rust Axum backend
│   ├── src/
│   │   ├── main.rs        # Entry point, app state, routes
│   │   ├── config.rs     # Configuration loading
│   │   ├── errors.rs    # Error types
│   │   ├── db/          # Database modules
│   │   ├── middleware/  # Auth, tenant, RBAC middleware
│   │   └── modules/    # Auth, tenant, control routes
│   └── Cargo.toml
├── web/                   # React frontend
│   ├── src/
│   │   ├── api/       # API client
│   │   ├── components/
│   │   ├── layouts/
│   │   ├── modules/
│   │   ├── store/    # Zustand
│   │   └── i18n/
│   └── package.json
└── docs/
    └── MASTER-PLAN.md
```

## Tool Usage Patterns

### Searching Rust Files
```javascript
glob pattern="src/**/*.rs" path="/Users/manees/dev/Garage360/api"
```

### Searching React Files
```javascript
glob pattern="src/**/*.tsx" path="/Users/manees/dev/Garage360/web"
```

### Searching Tests
```javascript
glob pattern="**/*.test.ts" path="/Users/manees/dev/Garage360/web"
glob pattern="**/*test*.rs" path="/Users/manees/dev/Garage360/api"
```

## Build Commands

**Always use Docker** - all building and running should use docker-compose.

### Using Docker Compose

```bash
# Start all services (development)
cd /Users/manees/dev/Garage360
docker-compose -f docker-compose.dev.yml up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f api
docker-compose -f docker-compose.dev.yml logs -f web

# Stop all services
docker-compose -f docker-compose.dev.yml down

# Rebuild API (after code changes)
docker-compose -f docker-compose.dev.yml build api

# Rebuild web (after code changes)
docker-compose -f docker-compose.dev.yml build web

# Run tests inside container
docker-compose -f docker-compose.dev.yml exec api cargo test
docker-compose -f docker-compose.dev.yml exec web npm test
```

## Key Technologies

| Component | Tech |
|-----------|------|
| Backend | Rust, Axum, SQLx, PostgreSQL, Redis |
| Frontend | React 19, Vite, Tailwind CSS, Zustand |
| Auth | JWT (HS256), Argon2 |
| API Docs | OpenAPI (utoipa) |

## Important Notes

- Multi-tenant: each tenant has isolated PostgreSQL database
- Feature flags control module visibility per tenant
- Routes: `/api/v1/auth/*`, `/api/v1/customers`, `/api/v1/vehicles`, `/api/v1/jobs`, `/api/v1/inventory`, `/api/v1/bays`, `/api/v1/purchases`, `/api/v1/billing`, `/control/v1/*`
- Health: `/health/liveness`, `/health/readiness`

## Frontend Development

See [`web/FRONTEND-CONSTITUTION.md`](./web/FRONTEND-CONSTITUTION.md) for complete guidelines.

### Quick Start (Frontend)
```bash
cd /Users/manees/dev/Garage360/web
docker-compose -f ../docker-compose.dev.yml up -d web
docker-compose -f ../docker-compose.dev.yml logs -f web
```

## API Routes Summary

| Route | Description |
|-------|------------|
| `/api/v1/auth/login` | Login, refresh, logout, me |
| `/api/v1/customers` | CRUD + FTS search |
| `/api/v1/vehicles` | CRUD + reg search |
| `/api/v1/jobs` | Job cards, status machine, items |
| `/api/v1/inventory` | Items, stock adjustment, alerts |
| `/api/v1/bays` | Service bays CRUD |
| `/api/v1/purchases/suppliers` | Suppliers CRUD |
| `/api/v1/purchases/purchase-orders` | PO lifecycle, GRN, QA |
| `/api/v1/billing/invoices` | Invoices, payments |