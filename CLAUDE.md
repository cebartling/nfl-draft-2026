# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NFL Draft Simulator 2026 - A full-stack application for simulating NFL drafts with real-time updates, AI-driven team decision-making, and comprehensive scouting systems.

This is a full-stack project: Rust backend + SvelteKit frontend + PostgreSQL, all orchestrated via Docker Compose. Backend tests: `cargo test --workspace`. Frontend tests: check package.json for test commands. Always run both test suites after cross-cutting changes.

**Tech Stack:**
- Backend: Rust + Axum + PostgreSQL 18
- Frontend: SvelteKit + TypeScript + Tailwind CSS
- Real-time: WebSocket (tokio-tungstenite)

## Important Rules

Never fabricate or hallucinate real-world data (NFL stats, draft orders, player records, season data). If real data is needed, scrape it from a credible source or ask the user to provide it. Always use the correct season year.

## Docker Workflow

When making code changes to a Dockerized application, always remind the user to rebuild containers (`docker compose up --build`) before testing. Never assume file changes will be reflected in running containers automatically.

When the user says services are already running (e.g., Docker Compose is up), do not try to start dev servers or services again. Verify running state with `docker compose ps` if unsure.

## Git & PR Conventions

This repo uses squash merges only. Never attempt regular merges or fast-forward merges on PRs. Use `gh pr merge --squash` for all PR merges.

## Rust Backend

After any schema or query changes in Rust, regenerate the SQLx offline cache with `cargo sqlx prepare --workspace` before building. SQLx offline mode will fail with stale cache files.

## Frontend (SvelteKit)

When editing Svelte files, prefer using the Write tool to rewrite entire files rather than the Edit tool with partial matching, as tab-character matching frequently fails in Svelte/frontend files.

## PR Review Workflow

When replying to GitHub PR review comments via the API, use `gh api` with the `in_reply_to` parameter on the pulls/comments endpoint. Do not use the generic issues/comments endpoint.

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

> **Tip for AI agents:** When iterating on frontend UI, use the Vite dev server (`npm run dev` from `front-end/`) on port 5173 instead of rebuilding the Docker frontend container. Vite provides hot module replacement (HMR) so changes appear instantly. The dev server proxies API requests to the backend on port 8000. Use the Playwright MCP browser against `http://localhost:5173` for visual testing during development.

**Setup:**
```bash
cd front-end
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

Acceptance tests (`back-end/crates/api/tests/`) provide end-to-end HTTP testing organized by feature:

**Test Files:**
- `health.rs` - Health endpoint validation
- `teams.rs` - Team CRUD operations with database validation
- `players.rs` - Player CRUD operations with database validation
- `drafts.rs` - Complete draft lifecycle with database state verification at each step
- `list.rs` - List endpoints with database count validation
- `errors.rs` - Error handling (404, 400, 409) with database verification
- `common/mod.rs` - Shared test utilities (spawn_app returns pool, create_client, cleanup_database)

**How They Work:**
1. Each test spawns the API server on an ephemeral port (OS-assigned)
2. Uses `tokio::sync::oneshot` channel to notify when server is ready
3. Creates a configured `reqwest::Client` with timeouts (30s overall, 5s connect, 5s per-request)
4. Makes actual HTTP requests and validates responses
5. **Validates data directly in the database** to ensure persistence
6. Compares HTTP responses with database state for consistency
7. Cleans up database after each test

**What They Validate:**
- **HTTP Layer**: Correct status codes (200, 201, 404, 400, 409) and JSON responses
- **Database Layer**: Data is correctly persisted in PostgreSQL
- **Consistency**: HTTP responses match database state
- **State Transitions**: Draft status changes are reflected in the database
- **Data Integrity**: Foreign keys, constraints, and counts are correct

**Important Notes:**
- Must run with `--test-threads=1` (tests share the same test database)
- Each test spawns its own server instance with a shared database pool
- Tests verify both HTTP responses AND database persistence
- Uses ephemeral ports to avoid port conflicts
- Organized by feature for maintainability and scalability
- True end-to-end testing: HTTP → API → Service → Repository → PostgreSQL

**Example Usage:**
```bash
# Run all acceptance tests
cargo test -p api --tests -- --test-threads=1

# Run specific test file
cargo test -p api --test drafts -- --test-threads=1

# Run specific test
cargo test -p api --test drafts test_draft_flow -- --test-threads=1

# Run with verbose output
cargo test -p api --tests -- --test-threads=1 --nocapture
```

### Frontend
- **Unit tests**: Pure functions, utilities, formatters
- **Component tests**: Vitest browser mode for real browser environment
- **E2E tests**: Playwright for complete user flows

Always clean up test data after tests. Use `TEST_DATABASE_URL` for backend tests to avoid polluting development database.

## Feature Demos

Feature demos use Playwright MCP to visually walk through new or changed UI features and capture screenshots. Screenshots are saved to `documentation/demos/` using date-stamped folders:

```
documentation/demos/
└── YYYY-MM-DD-feature-name/
    ├── demo-01-description.png
    ├── demo-02-description.png
    └── ...
```

**Naming conventions:**
- Folder: `YYYY-MM-DD-feature-name` (e.g., `2026-02-15-prospect-rankings`)
- Files: `demo-NN-description.png` with sequential numbering and a short description of what the screenshot shows

**When saving screenshots**, always use the `filename` parameter on `browser_take_screenshot` to save directly into the date-stamped folder. Do not save screenshots to the repo root.

## Reference Documentation

Detailed implementation plan with database schema, API endpoints, and phase-by-phase development guide: `documentation/plans/nfl-draft-simulator-2026.md`
