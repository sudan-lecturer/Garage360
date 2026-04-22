# Garage360

## Workflow

- Docker-first repo. Prefer the root `Makefile` and `docker-compose.dev.yml` over host-local Rust/Node workflows.
- Run `make up` before any command that uses `docker-compose ... exec` (`make test*`, `make lint`, `make typecheck`, `make fmt`, `make shell-*`).
- Common root commands: `make up`, `make down`, `make restart`, `make test-api`, `make test-web`, `make lint`, `make typecheck`, `make logs-api`, `make logs-web`, `make shell-api`, `make shell-web`.
- There is no checked-in CI workflow; treat `Makefile` + `docker-compose.dev.yml` as the executable source of truth.
- `make build-web` is currently a no-op in dev compose (`docker-compose -f docker-compose.dev.yml build web` prints `No services to build`). Do not treat `make build` or `make build-web` as frontend build verification.

## Focused Commands

- Single API test: `docker-compose -f docker-compose.dev.yml exec api cargo test <test_name>`
- Single web test file: `docker-compose -f docker-compose.dev.yml exec web npm test -- src/.../file.test.tsx`
- Single web test by name: `docker-compose -f docker-compose.dev.yml exec web npm test -- -t "test name"`
- Actual frontend production build check: `docker-compose -f docker-compose.dev.yml exec web npm run build`

## Repo Shape

- `api/` is the only Rust crate (`garage360-api`); there is no root Cargo workspace.
- `web/` is a standalone npm/Vite app; there is no monorepo tool.
- API entrypoint is `api/src/main.rs`; it mounts tenant routes at `/api/v1` and control-plane routes at `/control/v1`.
- Web entrypoints are `web/src/main.tsx` (QueryClient, BrowserRouter, service worker registration) and `web/src/App.tsx` (route table and auth gate).

## Backend

- Control DB bootstrap lives in `api/schema/control-db.sql` and is mounted into Postgres `docker-entrypoint-initdb.d`; changing that file will not update an existing `control-db-data-dev` volume automatically.
- Tenant DBs are provisioned from the full `api/schema/tenant_schema.sql`; there is no `api/migrations/` directory and no `sqlx` migrator in the codebase.
- API config is defined in `api/src/config.rs`: it reads `config`, then `.env`, then environment variables. Keep new env keys aligned with `AppConfig`.
- Verified Rust gotchas already present in code:
  - Query row structs derive `sqlx::FromRow`.
  - Pass `&state.control_db` / `&pool` directly into `sqlx` calls.
  - Request extractors implement `FromRequestParts<AppState>`.
  - Tenant pool cache uses `LruCache::new(NonZeroUsize::new(...).unwrap())`.

## Frontend

- Use the `@` alias for `web/src`.
- Keep app API calls relative (`/api`, `/control`, `/health`); Vite proxies those paths to `localhost:8080`, and the shared axios client is `web/src/api/client.ts`.
- Auth state lives in the persisted Zustand store in `web/src/store/auth.ts` under the `garage360-auth` key.
- UI tokens live in `web/src/styles/globals.css`; the frontend constitution also requires screens to remain usable at `375px` width.
- Vitest runs in `jsdom` with MSW from `web/src/test/setup.ts`; unmocked network requests fail because `onUnhandledRequest` is set to `error`.
- The dev `web` container runs `npm install && npm run dev -- --host` on startup, so dependency changes usually need a container restart rather than a host-side install.

## Verification

- Frontend-only changes: `make lint`, `make typecheck`, `make test-web`, and `docker-compose -f docker-compose.dev.yml exec web npm run build`.
- Backend-only changes: `make test-api`.
- Full-stack changes: `make test` plus the frontend checks above if `web/` changed.

## Git

- Run `make hooks-install` once per clone.
- `.githooks/commit-msg` enforces Conventional Commits: `type(scope): summary`.

## Deeper Rules

- `api/BACKEND-CONSTITUTION.md` covers backend domain and tenancy constraints.
- `web/FRONTEND-CONSTITUTION.md` covers UI, testing, and accessibility constraints.
- `docs/MASTER-PLAN.md` is the product/route intent reference when implementation is incomplete.
