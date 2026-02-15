---
name: playwright-e2e-tests
description: Run Playwright end-to-end acceptance tests from the acceptance-tests/ directory against the full stack (PostgreSQL + Rust API + SvelteKit frontend). Use when the user wants to run e2e tests, verify acceptance criteria, or check that the full stack works together.
---

# Playwright E2E Tests

Run the Playwright acceptance test suite in `acceptance-tests/`. Tests validate the full stack (UI + API + Database) using the Screenplay pattern. Services must be running externally — Playwright does not manage server lifecycle.

## When to Use

- The user says "run e2e tests", "run acceptance tests", "run playwright tests", or "test the full stack"
- A feature has been implemented and needs end-to-end verification
- The user wants to confirm nothing is broken across the full stack before pushing or merging

## Prerequisites

- Tests live in `acceptance-tests/` at the repository root (not `front-end/`)
- Node 24 (see `acceptance-tests/.nvmrc`)
- Three services must be running externally:
  - **PostgreSQL** on port 5432 (with seeded data — 32 teams, players)
  - **API server** on port 8000 (`http://localhost:8000/health`)
  - **Frontend** on port 3000 (`http://localhost:3000`) or port 5173 (Vite dev server)
- No `webServer` block in `playwright.config.ts` — Playwright will not start/stop servers

## Workflow

### Step 1: Verify Services Are Running

Check that required services are up before running tests. The `global-setup.ts` script performs health checks automatically, but verifying early avoids confusing errors.

```bash
# Check Docker containers
docker compose ps

# Or check ports directly
lsof -i :5432 -i :8000 -i :3000 -i :5173 | grep LISTEN
```

If services are not running:

```bash
# From repository root
docker compose up -d postgres

# Start API (from back-end/)
cd back-end && cargo run -p api &

# Start frontend dev server (from front-end/)
cd front-end && npm run dev &
```

Or ask the user — they may already have services running via Docker Compose.

### Step 2: Install Dependencies

```bash
cd acceptance-tests
npm install
npx playwright install chromium
```

Only Chromium is needed — the Playwright config defines a single `chromium` project.

### Step 3: Run Tests

All commands run from the `acceptance-tests/` directory:

```bash
# Run full suite
npx playwright test

# Run with visible browser
npx playwright test --headed

# Run specific test file
npx playwright test tests/smoke.spec.ts
npx playwright test tests/draft-lifecycle.spec.ts

# Run by grep pattern
npx playwright test --grep "creates a draft"

# Run with verbose list output
npx playwright test --reporter=list

# Debug mode (step through tests)
npx playwright test --debug

# Interactive UI mode
npx playwright test --ui
```

Tests run sequentially (`workers: 1`) because they share a database.

### Step 4: Handle Failures

**Check artifacts** — Playwright captures diagnostics on failure:

```bash
# Screenshots (captured on failure)
ls acceptance-tests/test-results/

# Traces (captured on first retry)
ls acceptance-tests/test-results/*/trace.zip

# Video (retained on failure)
ls acceptance-tests/test-results/*.webm
```

**View the HTML report:**

```bash
cd acceptance-tests
npx playwright show-report
```

**Common failure categories:**

| Category | Symptoms | Likely Cause |
|---|---|---|
| Service down | Connection refused, timeout on navigation | API or frontend not running — check `docker compose ps` |
| Stale test data | Expected 32 teams but got 0 | Database not seeded — run seed-data loader |
| UI changed | Locator timeout, element not found | Frontend markup changed — update `data-testid` attributes or Screenplay tasks/questions |
| API changed | Assertion mismatch on response data | Backend API response shape changed — update Screenplay questions |
| Timing | Flaky pass/fail on same test | Increase timeout or add explicit `waitFor` in the task |

**Test data conventions:** All test-created drafts use the `"E2E "` prefix (e.g., `"E2E Lifecycle Draft"`). The `global-teardown.ts` cleans up drafts with this prefix after the suite runs, preserving seeded reference data.

### Step 5: Summarize Results

Present results with actual test file names:

```
## E2E Test Results

**Suite:** 27 tests across 9 files
**Passed:** 25
**Failed:** 2
**Skipped:** 0
**Duration:** 52s

### Failures
1. draft-detail-page.spec.ts > "NotStarted draft shows unified panel with team selector"
   - Timeout waiting for [data-testid="team-selector"]
   - Likely cause: component markup changed — check DraftDetailPage.svelte

2. data-integrity.spec.ts > "players page count matches database"
   - Expected 150 players, got 148
   - Likely cause: seed data changed — verify player seed loader ran completely

### Services
- PostgreSQL (localhost:5432): healthy
- API (localhost:8000): healthy
- Frontend (localhost:3000): healthy
- Test data cleanup: global-teardown ran successfully
```

## Configuration Reference

Values from `acceptance-tests/playwright.config.ts`:

| Setting | Value |
|---|---|
| `baseURL` | `http://localhost:3000` (env: `FRONTEND_URL`) |
| API URL | `http://localhost:8000` (env: `API_URL`) |
| `testDir` | `./tests` |
| `workers` | 1 (sequential — shared database) |
| `fullyParallel` | false |
| `retries` | 1 local, 2 in CI |
| `timeout` | 60s per test |
| `actionTimeout` | 10s |
| `navigationTimeout` | 15s |
| `trace` | on-first-retry |
| `screenshot` | only-on-failure |
| `video` | retain-on-failure |
| `reporter` | HTML + list |
| Browser | Chromium only |
| `globalSetup` | `./global-setup.ts` (health checks) |
| `globalTeardown` | `./global-teardown.ts` (cleanup E2E drafts) |

## Test Suite Overview

| File | Tests | What It Covers |
|---|---|---|
| `smoke.spec.ts` | 3 | Frontend serves pages, API health, DB has 32 teams |
| `navigation.spec.ts` | 2 | Navbar links, mobile menu toggle |
| `draft-lifecycle.spec.ts` | 2 | Create draft via UI, verify in DB, check drafts list |
| `draft-detail-page.spec.ts` | 9 | Team selector, start draft, auto-pick, cancel, progress/board/stats visibility |
| `draft-session.spec.ts` | 1 | Create session via API, navigate, run auto-pick |
| `players-browse.spec.ts` | 3 | Player list, search, position group filter |
| `teams-browse.spec.ts` | 3 | Team list (32 teams), team detail navigation, DB match |
| `data-integrity.spec.ts` | 2 | Teams count matches DB, players count matches DB |
| `error-handling.spec.ts` | 2 | Nonexistent draft/team shows error state |

## Architecture: Screenplay Pattern

Tests use the Screenplay pattern for readability and maintainability. Code lives in `acceptance-tests/src/screenplay/`:

- **Actor** (`actor.ts`) — Test persona that performs tasks and asks questions
- **Abilities** (`abilities/`) — What actors can do:
  - `BrowseTheWeb` — Playwright page interactions
  - `CallApi` — HTTP requests to the API
  - `QueryDatabase` — Direct PostgreSQL queries for verification
- **Tasks** (`tasks/`) — High-level actions: `Navigate.to()`, `CreateDraft.named()`, `SelectTeam.byName()`
- **Questions** (`questions/`) — Queries that return values: `PageHeading.text()`, `DraftStatus.inDatabaseFor()`, `TeamCount.inDatabase()`

Custom test fixture at `src/fixtures/test-fixture.ts` provides each test with an Actor that has all three abilities.

**Example test pattern:**
```typescript
await actor.attemptsTo(Navigate.to('/drafts'));
await actor.attemptsTo(CreateDraft.named('E2E Test Draft'));
const count = await actor.asks(DraftCount.inDatabase());
```

## Test Data & Cleanup

- Test drafts are prefixed with `"E2E "` for scoped cleanup
- `global-teardown.ts` deletes E2E drafts and cascades to sessions, picks, events, and trades
- `src/db/cleanup.ts` contains cleanup utilities (`cleanupTestDrafts()`)
- `src/db/client.ts` manages the PostgreSQL connection pool
- Seeded reference data (32 NFL teams, players) is preserved across test runs

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `FRONTEND_URL` | `http://localhost:3000` | Base URL for UI tests |
| `API_URL` | `http://localhost:8000` | Backend API base URL |
| `DB_HOST` | `localhost` | PostgreSQL host |
| `DB_PORT` | `5432` | PostgreSQL port |
| `DB_NAME` | `nfl_draft` | Database name |
| `DB_USER` | `nfl_draft_user` | Database user |
| `DB_PASSWORD` | `nfl_draft_pass` | Database password |

## Full Docker-Based Runs

The `acceptance-tests/run-tests.sh` script orchestrates everything:

```bash
cd acceptance-tests

# Standard run (starts containers, seeds DB, runs tests, cleans up)
./run-tests.sh

# Rebuild containers first
./run-tests.sh --build

# Keep containers running after tests
KEEP_CONTAINERS=true ./run-tests.sh
```

The script handles: starting Docker containers, waiting for service health, seeding the database, installing npm dependencies, installing Playwright browsers, running the suite, and cleanup.
