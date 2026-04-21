# Garage360

Multi-tenant workshop management SaaS built with Rust (Axum) and React.

## Tech Stack

- **Backend**: Rust, Axum, SQLx, PostgreSQL
- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS
- **Infrastructure**: Docker, Nginx, Redis, MinIO

## Quick Start

### Prerequisites

- Docker & Docker Compose

### Development

```bash
make up
```

Garage360 is Docker-first for local development. Build, test, lint, and typecheck should run through the Docker development stack instead of mixing host-local Rust and Node workflows.

### Common Local Commands

```bash
make help
make up
make build
make test
make test-api
make test-web
make lint
make typecheck
make logs-api
make logs-web
make down
```

### Git Workflow

Use short-lived branches and Conventional Commits.

Branch naming:
- `feat/<scope>-<short-purpose>`
- `fix/<scope>-<short-purpose>`
- `chore/<scope>-<short-purpose>`
- `docs/<scope>-<short-purpose>`
- `codex-<short-purpose>` for local Codex-driven task branches when slash-prefixed refs are inconvenient in the local environment

Commit format:
```text
type(scope): short summary
```

Examples:
```text
feat(api): add tenant readiness check
fix(web): handle expired access token
docs(repo): document docker workflow
build(devops): standardize local compose commands
```

Install the local commit-message hook once per clone:

```bash
make hooks-install
```

### Production

```bash
# Build and start
docker-compose up -d
```

## Project Structure

```
garage360/
├── api/                    # Rust Axum backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── db/
│   │   ├── middleware/
│   │   ├── modules/
│   │   └── errors.rs
│   └── schema/
├── web/                   # React frontend
│   ├── src/
│   │   ├── components/
│   │   ├── modules/
│   │   ├── layouts/
│   │   └── store/
│   └── vite.config.ts
├── docker/
└── docker-compose.yml
```

## Documentation

- [Master Plan](./docs/MASTER-PLAN.md)
- [Backend Constitution](./api/BACKEND-CONSTITUTION.md)
- [Frontend Constitution](./web/FRONTEND-CONSTITUTION.md)
- [SRS](./SRS-Garage360-v4.md)

## License

Proprietary - All rights reserved
