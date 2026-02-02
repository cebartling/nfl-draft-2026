# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NFL Draft Simulator 2026 - A full-stack application for simulating NFL drafts with real-time updates, AI-driven team decision-making, and comprehensive scouting systems.

**Tech Stack:**
- Backend: Rust + Axum + PostgreSQL 18
- Frontend: SvelteKit + TypeScript + Tailwind CSS
- Real-time: WebSocket (tokio-tungstenite)

## Architecture

### Monorepo Structure

This is a **monorepo** containing multiple projects:

```
back-end/         # Rust backend (Cargo workspace)
├── crates/
│   ├── api/      # Axum web server (routes, handlers, middleware)
│   ├── domain/   # Business logic, services, domain models
│   ├── db/       # Database layer (SQLx repositories)
│   └── websocket/ # WebSocket connection management
└── migrations/   # SQLx database migrations

frontend/         # SvelteKit application (to be added)

documentation/    # Architecture and planning docs

docker-compose.yml  # Shared infrastructure (PostgreSQL, pgAdmin)
```

**Key Architectural Patterns:**
- **Repository Pattern**: Domain defines traits, DB crate implements with SQLx
- **Dependency Injection**: Services depend on repository traits, not concrete implementations
- **Layer Separation**: API → Domain (services) → DB (repositories) → PostgreSQL
- **Event Sourcing**: Draft events stored in JSONB for complete audit trail

### Database Schema Philosophy

The database is organized into logical domains:
1. **Teams & Organizations** (teams)
2. **Players & Scouting** (players, scouting_reports, combine_results, team_needs)
3. **Drafts & Picks** (drafts, draft_picks, pick_trades, pick_trade_details)
4. **Real-time Sessions** (draft_sessions, draft_events)

## Development Commands

### Docker Environment

**Infrastructure services (PostgreSQL, pgAdmin) are managed from the repository root:**

**Start services:**
```bash
# Start PostgreSQL only
docker compose up -d postgres

# Start PostgreSQL + pgAdmin (database GUI)
docker compose --profile tools up -d

# View logs
docker compose logs -f postgres

# Stop services
docker compose down

# Stop and remove volumes (destructive)
docker compose down -v
```

**Database access:**
```bash
# Connect to PostgreSQL via psql
docker compose exec postgres psql -U nfl_draft_user -d nfl_draft

# Or use pgAdmin at http://localhost:5050
# Credentials: admin@nfldraft.local / admin
```

### Backend (Rust)

**Initial Setup:**
```bash
# Start PostgreSQL (from repository root)
docker compose up -d postgres

# Setup backend
cd back-end

# Copy environment variables
cp .env.example .env

# Install sqlx-cli for migrations (if not already installed)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations for development database
sqlx migrate run

# Create and setup test database
sqlx database create --database-url "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"
sqlx migrate run --database-url "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"
```

**Development:**
```bash
cd back-end

# Build entire workspace
cargo build --workspace

# Run API server
cargo run -p api

# Run tests for all crates
cargo test --workspace

# Run tests for specific crate
cargo test -p domain

# Format and lint
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

**Database Migrations:**
```bash
cd back-end

# Create new migration
sqlx migrate add create_table_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Frontend (SvelteKit)

**Setup:**
```bash
cd frontend
npm install
```

**Development:**
```bash
# Development server (with HMR)
npm run dev

# Type checking
npm run check

# Format
npm run format

# Lint
npm run lint

# Build
npm run build

# Preview production build
npm run preview
```

**Testing:**
```bash
# Unit tests (Vitest)
npm run test

# Component tests (Vitest browser mode)
npm run test:integration

# E2E tests (Playwright)
npm run test:e2e

# Run specific test file
npm run test -- path/to/test.test.ts
```

## Key Implementation Details

### Backend: Adding a New Feature

When adding a new feature that spans the stack:

1. **Domain Model** (`back-end/crates/domain/src/models/`): Define the core entity
2. **Repository Trait** (`back-end/crates/domain/src/repositories/`): Define data access interface
3. **Service** (`back-end/crates/domain/src/services/`): Implement business logic
4. **DB Repository** (`back-end/crates/db/src/repositories/`): Implement trait with SQLx
5. **API Handler** (`back-end/crates/api/src/handlers/`): Create HTTP endpoint
6. **Route** (`back-end/crates/api/src/routes/`): Wire up the handler

This order ensures you're always coding against abstractions, not concrete implementations.

### Backend: Database Queries

**Use SQLx with compile-time verification:**
```rust
// Query macros are verified at compile time
let row = sqlx::query_as!(
    TeamDb,
    "SELECT * FROM teams WHERE id = $1",
    team_id
)
.fetch_one(&pool)
.await?;
```

**Important:** SQLx requires database connection during compilation for query verification. Set `DATABASE_URL` in your environment or use offline mode with `.sqlx/` cache.

### Frontend: State Management

**Use Svelte 5 runes** (not traditional stores):
```typescript
// lib/stores/draft.svelte.ts
export class DraftState {
  currentPick = $state<number>(1);
  picks = $state<DraftPick[]>([]);

  get currentTeam() {
    return this.draftOrder[this.currentPick - 1];
  }

  makePick(pick: DraftPick) {
    this.picks.push(pick);
  }
}
```

This is the modern approach (2026) and provides better TypeScript support than traditional Svelte stores.

### Frontend: API Integration

**Domain-specific API modules** match backend structure:
```typescript
// lib/api/draft.ts
export const draftApi = {
  async getSession(id: string) { ... },
  async makePick(id: string, pick: DraftPick) { ... }
};
```

Types in `lib/types/` should mirror Rust structs from the backend for end-to-end type safety.

### WebSocket Integration

**Backend** (`back-end/crates/websocket/`):
- Connection manager using DashMap for concurrent access
- Broadcasting to all clients in a session
- Reconnection handled client-side

**Frontend** (`lib/api/websocket.ts`):
- Auto-reconnection with exponential backoff
- Type-safe message handlers
- Integrated with Svelte stores for reactive updates

## Testing Philosophy

### Backend

#### Test Types

- **Unit tests**: Domain services with mock repositories (mockall)
- **Integration tests**: Full API endpoints with test database
- **Repository tests**: Against real PostgreSQL test database
- **Acceptance tests**: End-to-end HTTP tests with ephemeral server

#### Test Database Isolation

- All tests use `TEST_DATABASE_URL` environment variable
- Tests run against `nfl_draft_test` database (separate from `nfl_draft` development DB)
- Tests clean up data after execution to maintain isolation
- Never run tests against the production or development database

#### Running Tests

**All Tests:**
```bash
cd back-end

# Ensure TEST_DATABASE_URL is set in .env
# Run all tests (using test database)
cargo test --workspace -- --test-threads=1

# Run specific crate tests
cargo test -p domain
cargo test -p db
cargo test -p api
```

**Unit/Integration Tests Only:**
```bash
# Run all unit/integration tests (faster, no HTTP overhead)
cargo test --workspace --lib -- --test-threads=1
```

**Acceptance Tests Only:**
```bash
# Run end-to-end HTTP tests with ephemeral servers
cargo test -p api --test acceptance -- --test-threads=1

# With output (useful for debugging)
cargo test -p api --test acceptance -- --test-threads=1 --nocapture
```

#### Acceptance Tests

Acceptance tests (`back-end/crates/api/tests/acceptance.rs`) provide end-to-end HTTP testing:

**How They Work:**
1. Each test spawns the API server on an ephemeral port (OS-assigned)
2. Uses `tokio::sync::oneshot` channel to notify when server is ready
3. Creates a configured `reqwest::Client` with timeouts (30s overall, 5s connect, 5s per-request)
4. Makes actual HTTP requests and validates responses
5. Cleans up database after each test

**Test Coverage:**
- `test_health_check` - Health endpoint validation
- `test_create_and_get_team` - Team CRUD operations
- `test_create_and_get_player` - Player CRUD operations
- `test_draft_flow` - Complete draft lifecycle (create → initialize → start → pick → pause → resume → complete)
- `test_list_endpoints` - List all resources (teams, players, drafts)
- `test_error_handling` - Error responses (404, 400, 409)

**Important Notes:**
- Must run with `--test-threads=1` (tests share the same test database)
- Each test spawns its own server instance
- Tests verify actual HTTP status codes and JSON responses
- Uses ephemeral ports to avoid port conflicts

**Example Usage:**
```bash
# Run all acceptance tests
cargo test -p api --test acceptance -- --test-threads=1

# Run specific acceptance test
cargo test -p api --test acceptance test_draft_flow -- --test-threads=1

# Run with verbose output
cargo test -p api --test acceptance -- --test-threads=1 --nocapture
```

### Frontend
- **Unit tests**: Pure functions, utilities, formatters
- **Component tests**: Vitest browser mode for real browser environment
- **E2E tests**: Playwright for complete user flows

Always clean up test data after tests. Use `TEST_DATABASE_URL` for backend tests to avoid polluting development database.

## Reference Documentation

Detailed implementation plan with database schema, API endpoints, and phase-by-phase development guide: `documentation/plans/nfl-draft-simulator-2026.md`
