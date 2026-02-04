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

### 1. Start Infrastructure Services

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

### 2. Backend Setup & Development

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

### 3. Frontend Setup & Development

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

### Full Stack Development

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
   - Frontend: http://localhost:5173
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
