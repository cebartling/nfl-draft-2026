# NFL Draft Simulator 2026

A full-stack application for simulating NFL drafts with real-time updates, AI-driven team decision-making, and comprehensive scouting systems.

## Tech Stack

- **Backend**: Rust + Axum + PostgreSQL 18
- **Frontend**: SvelteKit + TypeScript + Tailwind CSS
- **Real-time**: WebSocket (tokio-tungstenite)
- **Infrastructure**: Docker + Docker Compose

## Prerequisites

Before working with this project, ensure you have the following installed:

### Required Dependencies

1. **Docker & Docker Compose**
   - Docker Desktop (macOS/Windows) or Docker Engine + Docker Compose (Linux)
   - Version: Docker 20.10+ and Docker Compose 2.0+
   - Download: https://docs.docker.com/get-docker/

2. **Rust** (for backend development)
   - Version: 1.75+ (2021 edition)
   - Install via rustup: https://rustup.rs/

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Node.js & npm** (for frontend development)
   - Version: Node.js 20+ LTS
   - Download: https://nodejs.org/
   - Verify installation:

   ```bash
   node --version  # Should be v20.x.x or higher
   npm --version   # Should be 10.x.x or higher
   ```

4. **SQLx CLI** (for database migrations)
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

### Optional Tools

- **pgAdmin** - Database GUI (included in docker-compose with `--profile tools`)
- **Rust Analyzer** - IDE support for Rust (VS Code extension)
- **Svelte for VS Code** - IDE support for Svelte (VS Code extension)

## Project Structure

```
nfl-draft-2026/
├── back-end/              # Rust backend (Cargo workspace)
│   ├── crates/
│   │   ├── api/          # Axum web server
│   │   ├── domain/       # Business logic & services
│   │   ├── db/           # Database layer (SQLx)
│   │   └── websocket/    # WebSocket connections
│   └── migrations/       # SQLx database migrations
├── front-end/            # SvelteKit application
│   ├── src/
│   │   ├── lib/          # Components, stores, utilities
│   │   └── routes/       # SvelteKit pages
│   └── tests/            # E2E tests (Playwright)
├── documentation/        # Architecture & planning docs
└── docker-compose.yml    # Infrastructure services
```

## Quick Start (Containerized - Recommended)

The fastest way to run the complete application:

```bash
# Start all services (PostgreSQL + Backend API + Frontend)
docker compose up -d

# Wait for services to be healthy (about 30 seconds)
docker compose ps

# Access the application
open http://localhost:3000
```

**Access Points:**
- **Frontend**: http://localhost:3000 (SvelteKit UI)
- **Backend API**: http://localhost:8000 (Rust/Axum)
- **API Docs**: http://localhost:8000/health (Health check)
- **PostgreSQL**: localhost:5432 (Database)

**View logs:**
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f frontend
docker compose logs -f api
docker compose logs -f postgres
```

**Stop services:**
```bash
# Stop (keeps data)
docker compose down

# Stop and remove all data (destructive)
docker compose down -v
```

### What Happens on First Run

When you run `docker compose up -d` for the first time:

1. **Pulls base images** (Node.js, Rust, nginx, PostgreSQL)
2. **Builds custom images**:
   - Backend API (~147MB, takes 2-3 minutes)
   - Frontend (~50MB, takes 1-2 minutes)
3. **Creates PostgreSQL database** with initial schema
4. **Starts all services** with health checks

**Note:** Database migrations must be run manually. See [Running Migrations](#running-migrations) below.

**Total time: ~5 minutes on first run** (subsequent starts: ~10 seconds)

### Optional: Database GUI (pgAdmin)

```bash
# Start with pgAdmin
docker compose --profile tools up -d

# Access pgAdmin
open http://localhost:5050
```

**pgAdmin Credentials:**
- Email: `admin@nfldraft.local`
- Password: `admin`

**Add Database Connection:**
- Host: `postgres` (Docker service name)
- Port: `5432`
- Database: `nfl_draft`
- Username: `nfl_draft_user`
- Password: `nfl_draft_pass`

## Getting Started

You can run the application in two ways:
1. **Docker Compose** (Recommended) - Full stack with one command
2. **Local Development** - Run services individually for active development

### Option 1: Docker Compose (Full Stack)

See **Quick Start** section above for the fastest path.

**Rebuild after code changes:**
```bash
docker compose up -d --build
```

**Check service health:**
```bash
docker compose ps
curl http://localhost:3000/health  # Frontend
curl http://localhost:8000/health  # Backend
```

### Option 2: Local Development

For active development, run services individually:

#### 1a. Start Infrastructure Services

Start PostgreSQL (required for both backend and frontend):

```bash
# Start PostgreSQL only
docker compose up -d postgres

# OR start PostgreSQL + pgAdmin (database GUI)
docker compose --profile tools up -d

# View logs
docker compose logs -f postgres

# Stop services
docker compose down
```

**pgAdmin Access** (if using `--profile tools`):

- URL: http://localhost:5050
- Email: `admin@nfldraft.local`
- Password: `admin`

#### 1b. Backend Setup & Development

Navigate to the backend directory:

```bash
cd back-end
```

#### Initial Setup

1. **Copy environment variables:**

   ```bash
   cp .env.example .env
   ```

2. **Run database migrations:**

   ```bash
   # Development database
   sqlx migrate run

   # Test database (for running tests)
   sqlx database create --database-url "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"
   sqlx migrate run --database-url "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"
   ```

#### Development Commands

```bash
# Build entire workspace
cargo build --workspace

# Run API server (default: http://localhost:8000)
cargo run -p api

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p domain
cargo test -p db
cargo test -p api

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace -- -D warnings
```

#### Create New Migration

```bash
sqlx migrate add create_table_name
# Edit the generated migration file in migrations/
sqlx migrate run
```

#### 1c. Frontend Setup & Development

Navigate to the frontend directory:

```bash
cd front-end
```

#### Initial Setup

1. **Install dependencies:**

   ```bash
   npm install
   ```

2. **Ensure backend is running:**
   - The frontend proxies API requests to `http://localhost:8000`
   - Make sure the backend API server is running first

#### Development Commands

```bash
# Start development server (http://localhost:5173)
npm run dev

# Type checking
npm run check

# Run unit/integration tests
npm test

# Run tests in watch mode
npm run test:watch

# Run E2E tests (requires dev server running)
npm run test:e2e

# Lint code
npm run lint

# Format code
npm run format

# Build for production
npm run build

# Preview production build
npm run preview
```

## Development Workflow

### Full Stack Development (Docker Compose)

The simplest way to run everything:

```bash
# Start all services
docker compose up -d

# View logs for specific service
docker compose logs -f frontend
docker compose logs -f api

# Rebuild after code changes
docker compose up -d --build

# Stop everything
docker compose down
```

**Access Points:**
- Frontend: http://localhost:3000
- Backend API: http://localhost:8000
- WebSocket: ws://localhost:8000/ws

### Local Development (Individual Services)

For active development with hot-reloading:

1. **Start infrastructure** (from repository root):

   ```bash
   docker compose up -d postgres
   ```

2. **Start backend** (in one terminal):

   ```bash
   cd back-end
   cargo run -p api
   ```

3. **Start frontend** (in another terminal):

   ```bash
   cd front-end
   npm run dev
   ```

4. **Access the application:**
   - Frontend: http://localhost:5173 (Vite dev server with HMR)
   - Backend API: http://localhost:8000
   - WebSocket: ws://localhost:8000/ws

### Database Access

**Via psql (command line):**

```bash
docker compose exec postgres psql -U nfl_draft_user -d nfl_draft
```

**Via pgAdmin (GUI):**

1. Start with profile: `docker compose --profile tools up -d`
2. Open http://localhost:5050
3. Login with credentials above
4. Add server connection:
   - Host: `postgres` (Docker service name)
   - Port: `5432`
   - Database: `nfl_draft`
   - Username: `nfl_draft_user`
   - Password: `nfl_draft_pass`

## Docker Deployment

### Container Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Docker Compose Stack                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────┐      ┌──────────────────┐            │
│  │   Frontend        │      │   Backend API     │            │
│  │   (nginx:alpine) │─────▶│   (Rust/Axum)    │            │
│  │   Port: 3000     │      │   Port: 8000      │            │
│  │   Size: 50MB     │      │   Size: 147MB     │            │
│  └──────────────────┘      └──────────────────┘            │
│         │                           │                        │
│         │ /api/* → reverse proxy    │                        │
│         │ /ws → WebSocket proxy     │                        │
│         └───────────────────────────┘                        │
│                                     │                        │
│                                     ▼                        │
│                          ┌──────────────────┐               │
│                          │   PostgreSQL 18   │               │
│                          │   Port: 5432      │               │
│                          │   Volume: Persist │               │
│                          └──────────────────┘               │
│                                                               │
│  Optional (--profile tools):                                 │
│  ┌──────────────────┐                                       │
│  │   pgAdmin         │                                       │
│  │   Port: 5050      │                                       │
│  └──────────────────┘                                       │
└─────────────────────────────────────────────────────────────┘
```

### Building Docker Images

Each service has its own multi-stage Dockerfile for optimized production images.

**Build all images:**
```bash
docker compose build
```

**Build individual images:**

**Backend:**
```bash
cd back-end
./build-docker.sh
# Image: nfl-draft-2026-api (147MB)
```

**Frontend:**
```bash
cd front-end
./build-docker.sh
# Image: nfl-draft-2026-frontend (50MB)
```

**Build with custom options:**
```bash
# ARM64 (M1/M2 Mac)
DOCKER_PLATFORM=linux/arm64 ./build-docker.sh

# Custom tag
DOCKER_IMAGE_TAG=v1.0.0 ./build-docker.sh
```

### Running with Docker Compose

**Start full stack:**
```bash
# Build and start (first time or after code changes)
docker compose up -d --build

# Start (using existing images)
docker compose up -d

# Start with pgAdmin
docker compose --profile tools up -d
```

**View logs:**
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f frontend
docker compose logs -f api
docker compose logs -f postgres

# Last 100 lines
docker compose logs --tail=100
```

**Stop services:**
```bash
# Stop (keeps data and images)
docker compose down

# Stop and remove volumes (deletes database data)
docker compose down -v

# Stop and remove images
docker compose down --rmi all
```

### Individual Container Management

**Start specific services:**

```bash
# Database only
docker compose up -d postgres

# Backend only (requires postgres)
docker compose up -d api

# Frontend only (requires api)
docker compose up -d frontend
```

**Restart a service:**
```bash
docker compose restart frontend
docker compose restart api
```

**Rebuild and restart a service:**
```bash
docker compose up -d --build frontend
```

### Health Checks

All services include automated health checks:

```bash
# Check service status
docker compose ps

# Example output:
# NAME                     STATUS                   PORTS
# nfl-draft-2026-frontend  Up (healthy)             0.0.0.0:3000->8080/tcp
# nfl-draft-2026-api       Up (healthy)             0.0.0.0:8000->8000/tcp
# nfl-draft-2026-postgres  Up (healthy)             0.0.0.0:5432->5432/tcp
```

**Test health endpoints:**
```bash
# Frontend (nginx health check)
curl http://localhost:3000/health
# Response: "healthy"

# Backend API (JSON response)
curl http://localhost:8000/health
# Response: {"service":"nfl-draft-2026-api","status":"healthy","version":"0.1.0"}

# PostgreSQL (via docker exec)
docker compose exec postgres pg_isready -U nfl_draft_user
# Response: /var/run/postgresql:5432 - accepting connections
```

### Environment Variables

Configure services via environment variables. Create a `.env` file in the repository root:

```bash
# .env (optional - defaults shown)

# PostgreSQL Configuration
POSTGRES_DB=nfl_draft
POSTGRES_USER=nfl_draft_user
POSTGRES_PASSWORD=nfl_draft_pass
POSTGRES_PORT=5432

# Backend API Configuration
API_PORT=8000
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=8000

# Frontend Configuration
FRONTEND_PORT=3000

# pgAdmin Configuration (--profile tools)
PGADMIN_PORT=5050
PGADMIN_EMAIL=admin@nfldraft.local
PGADMIN_PASSWORD=admin
```

**Environment Variable Reference:**

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_DB` | `nfl_draft` | PostgreSQL database name |
| `POSTGRES_USER` | `nfl_draft_user` | PostgreSQL username |
| `POSTGRES_PASSWORD` | `nfl_draft_pass` | PostgreSQL password (change in production!) |
| `POSTGRES_PORT` | `5432` | PostgreSQL host port |
| `API_PORT` | `8000` | Backend API host port |
| `FRONTEND_PORT` | `3000` | Frontend host port |
| `RUST_LOG` | `info` | Logging level (trace, debug, info, warn, error) |
| `PGADMIN_PORT` | `5050` | pgAdmin host port |
| `PGADMIN_EMAIL` | `admin@nfldraft.local` | pgAdmin login email |
| `PGADMIN_PASSWORD` | `admin` | pgAdmin login password (change in production!) |

**Example custom configuration:**

```bash
# .env
FRONTEND_PORT=8080
API_PORT=9000
RUST_LOG=debug
POSTGRES_PASSWORD=super_secure_password_change_me
```

### Database Migrations

Migrations must be run manually before starting the services.

#### Running Migrations

**From the host (requires sqlx-cli):**

```bash
# Install sqlx-cli (if not already installed)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
cd back-end
sqlx migrate run
```

**Using Docker:**

```bash
# Start PostgreSQL
docker compose up -d postgres

# Run migrations via docker exec
docker compose exec postgres psql -U nfl_draft_user -d nfl_draft -f /docker-entrypoint-initdb.d/schema.sql

# Or connect and verify schema manually
docker compose exec postgres psql -U nfl_draft_user -d nfl_draft -c "\dt"
```

**Reset database (destructive):**

```bash
docker compose down -v
docker compose up -d postgres
# Run migrations again before starting API
cd back-end && sqlx migrate run
docker compose up -d api
```

### Data Persistence

PostgreSQL data is persisted in a Docker volume:

```bash
# List volumes
docker volume ls | grep postgres

# Inspect volume
docker volume inspect nfl-draft-2026_postgres_data

# Backup database
docker compose exec postgres pg_dump -U nfl_draft_user nfl_draft > backup.sql

# Restore database
docker compose exec -T postgres psql -U nfl_draft_user -d nfl_draft < backup.sql
```

### Docker Documentation

For detailed Docker configuration, troubleshooting, and advanced usage:
- **Backend**: `back-end/DOCKER.md` - Multi-stage Rust build, offline SQLx cache
- **Frontend**: `front-end/DOCKER.md` - nginx reverse proxy, static optimization
- **Docker Compose**: This README section

### Production Deployment

**Security Checklist:**

```bash
# 1. Change default passwords
#    - PostgreSQL: POSTGRES_PASSWORD
#    - pgAdmin: PGADMIN_PASSWORD

# 2. Use proper secrets management
#    - Don't commit .env to git
#    - Use Docker secrets or environment-specific configs

# 3. Enable HTTPS
#    - Add nginx SSL configuration
#    - Use Let's Encrypt or valid certificates

# 4. Configure logging
#    - Set RUST_LOG=warn or RUST_LOG=error
#    - Ship logs to centralized logging system

# 5. Resource limits
#    - Add memory/CPU limits to docker-compose.yml
#    - Configure PostgreSQL connection pooling

# 6. Backup strategy
#    - Automated database backups
#    - Volume snapshots
#    - Disaster recovery plan
```

**Production docker-compose example:**

```yaml
# docker-compose.prod.yml
services:
  api:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '1'
          memory: 512M
    restart: always

  frontend:
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 256M
        reservations:
          cpus: '0.25'
          memory: 128M
    restart: always
```

**Deploy to production:**

```bash
# Use production config
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
```

## Testing

### Backend Tests

```bash
cd back-end

# Run all tests
cargo test --workspace -- --test-threads=1

# Run specific test suite
cargo test -p api --test acceptance -- --test-threads=1

# Run with output
cargo test --workspace -- --test-threads=1 --nocapture
```

**Note:** Backend tests use a separate `nfl_draft_test` database to avoid polluting development data.

### Frontend Tests

```bash
cd front-end

# Unit/integration tests (Vitest)
npm test

# E2E tests (Playwright) - requires dev server running
npm run test:e2e

# Run specific test file
npm test -- path/to/test.test.ts
```

## Troubleshooting

### Docker Issues

**Services won't start:**

```bash
# Check service status
docker compose ps

# View logs for errors
docker compose logs

# Restart all services
docker compose restart

# Full reset (removes containers, not volumes)
docker compose down
docker compose up -d
```

**Port already in use:**

```bash
# Error: "port is already allocated"

# Option 1: Stop conflicting service
lsof -ti:3000 | xargs kill -9  # Frontend port
lsof -ti:8000 | xargs kill -9  # Backend port
lsof -ti:5432 | xargs kill -9  # PostgreSQL port

# Option 2: Change ports in .env
echo "FRONTEND_PORT=8080" >> .env
echo "API_PORT=9000" >> .env
docker compose up -d
```

**Container fails health check:**

```bash
# Check health status
docker compose ps

# View container logs
docker compose logs api
docker compose logs frontend

# Test health endpoints manually
curl http://localhost:3000/health
curl http://localhost:8000/health

# If unhealthy, try restart
docker compose restart api
docker compose restart frontend
```

**Build failures:**

```bash
# Clear Docker cache and rebuild
docker compose build --no-cache

# Remove old images
docker compose down --rmi all
docker compose up -d --build

# Check disk space
docker system df

# Clean up Docker system (removes unused data)
docker system prune -a
```

**Database connection errors:**

```bash
# Error: "connection refused" or "database does not exist"

# Check PostgreSQL is running
docker compose ps postgres

# View PostgreSQL logs
docker compose logs postgres

# Verify database exists
docker compose exec postgres psql -U nfl_draft_user -l

# Recreate database (destructive)
docker compose down -v
docker compose up -d
```

### Backend Issues

**"database does not exist" error (local development):**

```bash
cd back-end
sqlx database create
sqlx migrate run
```

**Port 8000 already in use:**

```bash
# Find and kill the process
lsof -ti:8000 | xargs kill -9

# Or use docker
docker compose down
docker compose up -d api
```

**Rust compilation errors:**

```bash
# Update dependencies
cd back-end
cargo update

# Clean and rebuild
cargo clean
cargo build --workspace

# Check Rust version (need 1.75+)
rustc --version
```

### Frontend Issues

**"ECONNREFUSED" errors:**

```bash
# Ensure backend is running
docker compose ps api
curl http://localhost:8000/health

# Or for local development
cd back-end
cargo run -p api
```

**Module not found errors:**

```bash
cd front-end
npm install

# Clear cache if issues persist
rm -rf node_modules package-lock.json
npm install
```

**Vite port conflicts:**

```bash
# If port 5173 is in use, specify different port
npm run dev -- --port 5174
```

### Database Issues

**Can't connect to PostgreSQL:**

```bash
# Check if container is running
docker compose ps postgres

# Check PostgreSQL logs
docker compose logs postgres

# Restart PostgreSQL
docker compose restart postgres

# Verify connection
docker compose exec postgres pg_isready -U nfl_draft_user
```

**Reset database (destructive):**

```bash
# Stop and remove volumes (deletes all data!)
docker compose down -v

# Restart and run migrations manually
docker compose up -d postgres
cd back-end && sqlx migrate run
docker compose up -d

# For local development
cd back-end
sqlx database drop
sqlx database create
sqlx migrate run
```

**Migration errors:**

```bash
# View migration history
docker compose exec postgres psql -U nfl_draft_user -d nfl_draft \
  -c "SELECT * FROM _sqlx_migrations;"

# Manually run migrations
cd back-end
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Performance Issues

**Slow Docker builds:**

```bash
# Use BuildKit for faster builds
DOCKER_BUILDKIT=1 docker compose build

# Use build cache
docker compose build

# Parallel builds
docker compose build --parallel
```

**High memory usage:**

```bash
# Check container resources
docker stats

# Limit container memory (add to docker-compose.yml)
services:
  api:
    deploy:
      resources:
        limits:
          memory: 1G
```

**Slow database queries:**

```bash
# Connect to database
docker compose exec postgres psql -U nfl_draft_user -d nfl_draft

# Check slow queries
SELECT pid, usename, query, state
FROM pg_stat_activity
WHERE state = 'active';

# Analyze query performance
EXPLAIN ANALYZE SELECT ...;
```

### Common Error Messages

**"network nfl-draft-network not found":**

```bash
# Recreate network
docker compose down
docker compose up -d
```

**"container already exists":**

```bash
# Remove existing containers
docker compose down
docker compose up -d
```

**"permission denied" errors:**

```bash
# Fix file permissions
chmod +x back-end/build-docker.sh
chmod +x front-end/build-docker.sh

# Or use sudo (not recommended)
sudo docker compose up -d
```

**"platform mismatch" warnings:**

```bash
# Build for your platform
cd front-end
DOCKER_PLATFORM=linux/arm64 ./build-docker.sh  # M1/M2 Mac

cd back-end
DOCKER_PLATFORM=linux/arm64 ./build-docker.sh  # M1/M2 Mac
```

### Getting Help

If you're still experiencing issues:

1. **Check logs**: `docker compose logs -f`
2. **Verify health**: `docker compose ps`
3. **Review documentation**:
   - Backend: `back-end/DOCKER.md`
   - Frontend: `front-end/DOCKER.md`
4. **Clean slate**: `docker compose down -v && docker compose up -d --build`
5. **Open an issue**: Include logs and `docker compose ps` output

## Architecture

### Backend Architecture

- **API Layer** (`crates/api`): HTTP routes, handlers, middleware
- **Domain Layer** (`crates/domain`): Business logic, services, models
- **Database Layer** (`crates/db`): SQLx repositories implementing domain traits
- **WebSocket Layer** (`crates/websocket`): Real-time connection management

**Design Patterns:**

- Repository Pattern with trait-based abstraction
- Dependency Injection via constructor injection
- Event Sourcing for draft events (stored in JSONB)

### Frontend Architecture

- **Svelte 5 Runes**: Modern reactive state management
- **Domain-specific API modules**: Match backend structure
- **Type-safe WebSocket client**: Auto-reconnection with exponential backoff
- **Component library**: Reusable UI components in `src/lib/components/`

## Contributing

1. Create a feature branch from `main`
2. Make your changes with tests
3. Ensure all checks pass:

   ```bash
   # Backend
   cd back-end
   cargo test --workspace
   cargo clippy --workspace
   cargo fmt --all --check

   # Frontend
   cd front-end
   npm test
   npm run check
   npm run lint
   npm run format -- --check
   ```

4. Create a pull request

## License

[Add your license here]

## Documentation

For detailed implementation plans and architecture decisions, see the `documentation/` directory.
