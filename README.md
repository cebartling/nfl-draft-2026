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

## Getting Started

You can run the application in two ways:
1. **Docker Compose** (Recommended) - Full stack with one command
2. **Local Development** - Run services individually for development

### Option 1: Docker Compose (Full Stack)

The easiest way to run the complete application:

```bash
# Start all services (PostgreSQL + Backend API + Frontend)
docker compose up -d

# View logs
docker compose logs -f

# Stop all services
docker compose down
```

**Access Points:**
- Frontend: http://localhost:3000
- Backend API: http://localhost:8000
- PostgreSQL: localhost:5432

**Optional: Start with pgAdmin (database GUI):**

```bash
docker compose --profile tools up -d
```

- pgAdmin URL: http://localhost:5050
- Email: `admin@nfldraft.local`
- Password: `admin`

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

### Building Docker Images

**Backend:**
```bash
cd back-end
./build-docker.sh
```

**Frontend:**
```bash
cd front-end
./build-docker.sh
```

### Running with Docker Compose

The recommended way to run the full stack:

```bash
# Build and start all services
docker compose up -d --build

# View logs
docker compose logs -f

# Stop services (keeps data)
docker compose down

# Stop and remove all data (destructive)
docker compose down -v
```

### Individual Container Management

**Frontend only:**
```bash
docker compose up -d frontend
```

**Backend only:**
```bash
docker compose up -d api
```

**Database only:**
```bash
docker compose up -d postgres
```

### Health Checks

All services include health checks:

```bash
# Check service status
docker compose ps

# Test health endpoints
curl http://localhost:3000/health  # Frontend
curl http://localhost:8000/health  # Backend API
```

### Environment Variables

Configure services via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_DB` | `nfl_draft` | PostgreSQL database name |
| `POSTGRES_USER` | `nfl_draft_user` | PostgreSQL username |
| `POSTGRES_PASSWORD` | `nfl_draft_pass` | PostgreSQL password |
| `POSTGRES_PORT` | `5432` | PostgreSQL host port |
| `API_PORT` | `8000` | Backend API host port |
| `FRONTEND_PORT` | `3000` | Frontend host port |
| `PGADMIN_PORT` | `5050` | pgAdmin host port |

Create a `.env` file in the repository root to override defaults:

```bash
# .env
FRONTEND_PORT=8080
API_PORT=9000
```

### Docker Documentation

For detailed Docker setup, configuration, and troubleshooting:
- Backend: `back-end/DOCKER.md`
- Frontend: `front-end/DOCKER.md`

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

### Backend Issues

**"database does not exist" error:**

```bash
cd back-end
sqlx database create
sqlx migrate run
```

**Port 8000 already in use:**

```bash
# Find and kill the process
lsof -ti:8000 | xargs kill -9
```

### Frontend Issues

**"ECONNREFUSED" errors:**

- Ensure the backend API server is running on port 8000

**Module not found errors:**

```bash
npm install
```

### Database Issues

**Can't connect to PostgreSQL:**

```bash
# Check if container is running
docker compose ps

# Restart PostgreSQL
docker compose restart postgres

# Check logs
docker compose logs postgres
```

**Reset database (destructive):**

```bash
# Stop and remove volumes
docker compose down -v

# Restart and re-run migrations
docker compose up -d postgres
cd back-end
sqlx migrate run
```

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
