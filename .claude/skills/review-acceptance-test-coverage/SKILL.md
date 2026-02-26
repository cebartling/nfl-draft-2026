---
name: review-acceptance-test-coverage
description: Review API and front-end acceptance tests for coverage gaps, add missing tests, rebuild Docker containers, run all acceptance tests, commit and push. Summarize findings at the end.
---

# Review Acceptance Test Coverage

Audit both the backend API acceptance tests (`back-end/crates/api/tests/`) and the front-end Playwright E2E tests (`acceptance-tests/`) for coverage gaps. Fill gaps with new tests, rebuild the full stack, run all acceptance tests, commit, push, and summarize.

## When to Use

- The user says "review acceptance test coverage", "check acceptance tests", "audit test coverage", or "find missing acceptance tests"
- Before a release or merge when the user wants confidence that acceptance tests are comprehensive
- After adding new API endpoints or frontend pages and wants to verify test coverage exists

## Prerequisites

- Must be on a feature branch (not main)
- Docker Compose available for rebuilding containers
- PostgreSQL, API, and frontend services can be started via Docker Compose
- `gh` CLI authenticated for pushing

## Workflow

### Step 0: Prompt for Model

Before starting the review, ask the user which model they'd like to use. Present options using AskUserQuestion:

- **Opus** (Recommended) — Deepest analysis, best for thorough reviews
- **Sonnet** — Good balance of speed and depth

Do NOT proceed until the user has selected a model.

### Step 1: Inventory API Endpoints and Handlers

Build a complete map of the backend API surface:

```bash
# List all route definitions
grep -rn "\.route\|\.get\|\.post\|\.put\|\.patch\|\.delete" back-end/crates/api/src/routes/

# List all handler functions
grep -rn "pub async fn" back-end/crates/api/src/handlers/
```

For each endpoint, record:
- HTTP method and path
- Handler function name
- Expected request/response types
- Key behaviors (CRUD, status transitions, validation, error cases)

### Step 2: Audit Backend Acceptance Test Coverage

Read all existing acceptance test files in `back-end/crates/api/tests/`:

```bash
ls back-end/crates/api/tests/
```

For each API endpoint found in Step 1, determine if acceptance tests exist covering:

1. **Happy path** — successful request with valid data, correct status code and response body
2. **Not found (404)** — request for nonexistent resource
3. **Bad request (400)** — invalid input, missing required fields
4. **Conflict (409)** — duplicate creation, invalid state transitions
5. **Database verification** — data persisted correctly (query DB after API call)
6. **List endpoints** — correct counts, filtering, empty results

Present gaps as a numbered list:

```
Backend Acceptance Test Gaps:
1. [POST /api/drafts] — No test for creating a draft with missing required fields (400)
2. [GET /api/players/:id] — No test for nonexistent player (404)
3. [PUT /api/drafts/:id/start] — No test for starting an already-started draft (409)
```

### Step 3: Inventory Frontend Pages and Features

Build a map of the frontend UI surface:

```bash
# List all route pages
find front-end/src/routes -name "+page.svelte" -o -name "+page.ts"

# List key components
find front-end/src/lib/components -name "*.svelte"
```

For each page, record:
- Route path
- Key user interactions (forms, buttons, navigation, filters, search)
- Data displayed (lists, details, status indicators)
- Error states (loading failures, empty states, not found)

### Step 4: Audit Frontend E2E Test Coverage

Read all existing Playwright test files in `acceptance-tests/tests/`:

```bash
ls acceptance-tests/tests/
```

For each frontend page and feature found in Step 3, determine if E2E tests exist covering:

1. **Page renders** — navigates to the page, key elements visible
2. **User interactions** — clicks, form submissions, filters, search
3. **Data display** — correct data shown, counts match database
4. **Navigation** — links between pages work, breadcrumbs, back navigation
5. **Error states** — nonexistent resources, failed API calls
6. **Responsive behavior** — if applicable (mobile menu, responsive layouts)

Present gaps as a numbered list:

```
Frontend E2E Test Gaps:
1. [/players/:id] — No test for individual player detail page
2. [/drafts] — No test for filtering or sorting drafts list
3. [/teams/:id] — No test for team need priorities display
```

### Step 5: Add Missing Backend Acceptance Tests

For each gap identified in Step 2:

1. **Read the existing test file** for that feature area (or create a new file if needed)
2. **Follow existing patterns** — use the same test utilities from `common/mod.rs`
3. **Write the test** — match the style of existing tests (spawn_app, create_client, cleanup)
4. **Run the test** to verify it passes:

```bash
cd back-end && cargo test -p api --test <test_file> -- --test-threads=1 --nocapture
```

5. **Commit with a descriptive message** — one commit per test file or logical group:

```bash
git add back-end/crates/api/tests/<file>.rs
git commit -m "test(api): add acceptance tests for <what is now tested>

Covers gap found during acceptance test coverage review."
```

**Important:**
- One commit per logical group of tests
- Run tests after each addition to confirm they pass
- Follow existing patterns in `common/mod.rs` for test setup and cleanup
- Verify database state after API calls, not just HTTP responses

### Step 6: Add Missing Frontend E2E Tests

For each gap identified in Step 4:

1. **Read existing test files** to understand the Screenplay pattern conventions
2. **Check available Tasks and Questions** in `acceptance-tests/src/screenplay/`
3. **Create new Tasks/Questions** if needed for new interactions or verifications
4. **Write the test** following the Screenplay pattern:
   - Use the Actor fixture
   - Use `Navigate.to()` for page navigation
   - Use existing or new Tasks for interactions
   - Use existing or new Questions for assertions
5. **Run the test** to verify it passes:

```bash
cd acceptance-tests && npx playwright test tests/<test_file>.spec.ts --reporter=list
```

6. **Commit with a descriptive message** — one commit per test file or logical group:

```bash
git add acceptance-tests/tests/<file>.spec.ts acceptance-tests/src/screenplay/**
git commit -m "test(e2e): add E2E tests for <what is now tested>

Covers gap found during acceptance test coverage review."
```

**Important:**
- Follow the Screenplay pattern — do not use raw Playwright API in test files
- Prefix test draft names with `"E2E "` so global-teardown cleans them up
- Create new Tasks/Questions in the appropriate directories if needed
- One commit per logical group of tests

### Step 7: Rebuild and Restart Docker Containers

Rebuild all containers to ensure the latest code is included:

```bash
# From repository root — stop, rebuild, and restart all services
docker compose down
docker compose up --build -d

# Wait for services to be healthy
sleep 10

# Verify all services are running
docker compose ps
```

Check that all required services are healthy:
- PostgreSQL on port 5432
- API on port 8000 (`curl -s http://localhost:8000/health`)
- Frontend on port 3000 (`curl -s http://localhost:3000`)

If any service fails to start, check logs:

```bash
docker compose logs <service-name>
```

### Step 8: Run All Acceptance Tests

Run both backend and frontend acceptance test suites:

**Backend acceptance tests:**
```bash
cd back-end && cargo test -p api --tests -- --test-threads=1
```

**Frontend E2E tests:**
```bash
cd acceptance-tests && npx playwright test --reporter=list
```

If any tests fail:
1. Read the failure output carefully
2. Determine if the failure is in a new test (fix the test) or an existing test (investigate regression)
3. Fix and re-run until all tests pass
4. Commit fixes with descriptive messages

### Step 9: Push All Commits

Push all new commits to the remote:

```bash
git push
```

If the push is rejected (remote has new commits), rebase first:

```bash
git pull --rebase && git push
```

### Step 10: Summary

Provide a structured summary:

```
## Acceptance Test Coverage Review Summary

**Branch:** feature/foo-bar
**Date:** YYYY-MM-DD

### API Endpoints Audited
- Total endpoints: N
- Fully covered: N
- Gaps found: N

### Backend Acceptance Tests
- Existing tests: N
- Tests added: N
- Gaps remaining: N (with justification)

### Frontend Pages/Features Audited
- Total pages: N
- Fully covered: N
- Gaps found: N

### Frontend E2E Tests
- Existing tests: N
- Tests added: N
- Gaps remaining: N (with justification)

### Test Results
- Backend acceptance tests: N passed, N failed
- Frontend E2E tests: N passed, N failed
- Duration: Ns backend, Ns frontend

### Commits Pushed
1. test(api): description
2. test(e2e): description
3. ...

### Remaining Gaps (if any)
- [endpoint/page] — Reason it was not covered (e.g., requires infrastructure not available, blocked by known bug)
```

## Edge Cases

**No gaps found:** If both backend and frontend acceptance tests are comprehensive, state this explicitly. Skip Steps 5-6 and proceed directly to Step 7 (rebuild and run all tests as a verification pass).

**New endpoints without any tests:** Prioritize these over adding edge-case tests to already-covered endpoints. A missing happy-path test is more critical than a missing 409 test.

**Screenplay components missing:** If new frontend features require Tasks or Questions that don't exist yet, create them following the existing patterns in `acceptance-tests/src/screenplay/`. Document what was created.

**Database-dependent tests:** Backend acceptance tests require the test database. If `nfl_draft_test` is not available, note which tests were skipped and why.

**Flaky tests:** If a test passes intermittently, add appropriate waits or retries rather than skipping it. Flag it in the summary as potentially flaky.

**Docker build failures:** If container rebuild fails, check for compilation errors in Rust or build errors in SvelteKit. Fix build issues before proceeding with test runs. Commit build fixes separately from test additions.

**Large number of gaps (10+):** Prioritize by impact — focus on untested happy paths first, then error cases, then edge cases. Ask the user if they want to limit scope to the most critical gaps.
