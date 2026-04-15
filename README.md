# Garage360

Multi-tenant workshop management SaaS built with Rust (Axum) and React.

## Tech Stack

- **Backend**: Rust, Axum, SQLx, PostgreSQL
- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS
- **Infrastructure**: Docker, Nginx, Redis, MinIO

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Rust 1.75+
- Node.js 20+

### Development

```bash
# Start infrastructure and development servers
docker-compose -f docker-compose.dev.yml up -d

# Or run locally
cd api && cargo run &
cd web && npm run dev
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
- [SRS](./SRS-Garage360-v4.md)

## License

Proprietary - All rights reserved
