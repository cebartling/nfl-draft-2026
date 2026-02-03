# ADR 0006: Comprehensive Testing Strategy

## Status

Accepted

## Context

We needed to establish a testing strategy that ensures code quality and correctness across both backend (Rust) and frontend (SvelteKit) while maintaining developer productivity.

Requirements:

- Fast feedback during development
- Confidence in refactoring and changes
- Catch bugs before production
- Maintain test maintainability
- Support continuous integration
- Test at appropriate levels (unit, integration, E2E)

## Decision

We will implement a multi-layered testing strategy with different tools for different test levels.

### Backend Testing (Rust)

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test repository implementations against test database
3. **Acceptance Tests**: End-to-end HTTP API tests with ephemeral servers

**Tools**:

- Built-in Rust test framework
- `mockall` for mocking repository traits
- SQLx with test database isolation
- `tokio::test` for async tests

### Frontend Testing (TypeScript/Svelte)

1. **Unit Tests**: Test utilities, stores, API clients
2. **Integration Tests**: Test components in browser environment
3. **E2E Tests**: Test complete user flows

**Tools**:

- Vitest for unit/integration tests
- Playwright for E2E tests
- Testing Library for component testing
- jsdom for browser environment simulation

## Consequences

### Positive

- **Comprehensive Coverage**: Multiple test levels catch different types of bugs
- **Fast Feedback**: Unit tests run in milliseconds, providing quick feedback
- **Confidence in Refactoring**: Tests enable safe refactoring of both backend and frontend
- **Documentation**: Tests serve as living documentation of system behavior
- **CI/CD Ready**: All tests can run in GitHub Actions or similar CI systems
- **Test Isolation**: Backend tests use separate database, frontend tests are isolated

### Negative

- **Maintenance Overhead**: More test code to maintain
- **Learning Curve**: Developers need to learn multiple testing frameworks
- **Setup Complexity**: Test database setup required for backend
- **Build Time**: Running all tests takes time (though parallelizable)
- **False Positives**: E2E tests can be flaky if not written carefully

### Neutral

- **Coverage Metrics**: We track coverage but don't enforce strict thresholds
- **Test Organization**: Tests co-located with code for easier maintenance

## Backend Testing Strategy

### Unit Tests

Test business logic in isolation using mocked dependencies.

**Location**: In the same file as the code being tested (inline tests) or in `tests/` subdirectory

**Example**:

```rust
// domain/src/services/draft_service.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        TeamRepo {}

        #[async_trait]
        impl TeamRepository for TeamRepo {
            async fn get_by_id(&self, id: Uuid) -> Result<Option<Team>>;
        }
    }

    #[tokio::test]
    async fn test_validate_pick_team_exists() {
        let mut mock_repo = MockTeamRepo::new();
        mock_repo
            .expect_get_by_id()
            .returning(|_| Ok(Some(team_fixture())));

        let service = DraftService::new(Arc::new(mock_repo));
        let result = service.validate_pick(team_id, player_id).await;

        assert!(result.is_ok());
    }
}
```

### Integration Tests

Test repository implementations against a real test database.

**Location**: `db/src/repositories/` alongside implementations

**Database**: Separate `nfl_draft_test` database

**Example**:

```rust
// db/src/repositories/team_repository.rs
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> PgPool {
        let pool = PgPool::connect(&std::env::var("TEST_DATABASE_URL").unwrap())
            .await
            .unwrap();

        // Clean up from previous tests
        sqlx::query("DELETE FROM teams").execute(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_team() {
        let pool = setup_test_db().await;
        let repo = TeamRepositoryImpl::new(pool);

        let new_team = NewTeam {
            city: "Kansas City".to_string(),
            name: "Chiefs".to_string(),
        };

        let team = repo.create(new_team).await.unwrap();

        assert_eq!(team.city, "Kansas City");
        assert_eq!(team.name, "Chiefs");
    }
}
```

### Acceptance Tests

Test complete HTTP API flows with ephemeral servers.

**Location**: `api/tests/` directory

**Key Features**:

- Spawn API server on ephemeral port (OS-assigned)
- Verify HTTP responses AND database state
- Use `tokio::sync::oneshot` for server readiness signaling
- Configured `reqwest::Client` with timeouts

**Example**:

```rust
// api/tests/drafts.rs
#[tokio::test]
async fn test_draft_flow() {
    let (pool, base_url) = spawn_app().await;
    let client = create_client();

    // 1. Create draft
    let response = client
        .post(&format!("{}/api/drafts", base_url))
        .json(&json!({ "year": 2026, "rounds": 7 }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let draft: Draft = response.json().await.unwrap();

    // 2. Verify in database
    let db_draft = sqlx::query_as!(DraftDb, "SELECT * FROM drafts WHERE id = $1", draft.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(db_draft.year, 2026);
    assert_eq!(db_draft.status, "NotStarted");

    // 3. Start draft
    let response = client
        .post(&format!("{}/api/drafts/{}/start", base_url, draft.id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Verify status changed in database
    let db_draft = sqlx::query_as!(DraftDb, "SELECT * FROM drafts WHERE id = $1", draft.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(db_draft.status, "InProgress");

    cleanup_database(&pool).await;
}
```

**Why Acceptance Tests Validate Database**:

- Ensures data persistence (not just HTTP responses)
- Verifies database constraints and triggers
- Catches discrepancies between HTTP response and stored data
- Provides confidence in complete data flow: HTTP → Service → Repository → Database

### Running Backend Tests

```bash
# All tests (requires test database)
cargo test --workspace -- --test-threads=1

# Unit tests only (no database)
cargo test --workspace --lib

# Integration tests (specific crate)
cargo test -p db

# Acceptance tests only
cargo test -p api --test acceptance -- --test-threads=1
```

**Note**: `--test-threads=1` is required because tests share the test database.

## Frontend Testing Strategy

### Unit Tests (Vitest)

Test pure functions, stores, utilities, and API clients.

**Location**: `*.test.ts` files alongside code

**Example**:

```typescript
// lib/stores/toast.test.ts
import { describe, it, expect } from "vitest";
import { ToastState, ToastType } from "./toast.svelte";

describe("ToastState", () => {
  it("should add toast", () => {
    const state = new ToastState();

    state.success("Test message");

    expect(state.toasts).toHaveLength(1);
    expect(state.toasts[0].type).toBe(ToastType.Success);
    expect(state.toasts[0].message).toBe("Test message");
  });

  it("should auto-remove toast after duration", async () => {
    const state = new ToastState();

    state.info("Test", 100); // 100ms duration

    expect(state.toasts).toHaveLength(1);

    await new Promise((resolve) => setTimeout(resolve, 150));

    expect(state.toasts).toHaveLength(0);
  });
});
```

### Component Integration Tests (Vitest + Testing Library)

Test Svelte components in a browser-like environment.

**Note**: Currently skipped due to Svelte 5 limitations, will be enabled when Testing Library fully supports Svelte 5 runes.

**Example (when supported)**:

```typescript
// lib/components/ui/Button.test.ts
import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, vi } from "vitest";
import Button from "./Button.svelte";

describe("Button", () => {
  it("should call onclick handler", async () => {
    const handleClick = vi.fn();

    render(Button, {
      props: {
        onclick: handleClick,
        children: "Click me",
      },
    });

    const button = screen.getByRole("button", { name: "Click me" });
    await fireEvent.click(button);

    expect(handleClick).toHaveBeenCalledOnce();
  });
});
```

### E2E Tests (Playwright)

Test complete user flows in a real browser.

**Location**: `tests/*.spec.ts`

**Features**:

- Tests run against dev server (started automatically)
- Multiple browser support (Chromium, Firefox, WebKit)
- Screenshots and videos on failure
- Trace recording for debugging

**Example**:

```typescript
// tests/draft-room.spec.ts
import { test, expect } from "@playwright/test";

test.describe("Draft Room", () => {
  test("should display current pick", async ({ page }) => {
    await page.goto("/sessions/123");

    await expect(page.locator(".current-pick")).toContainText("Pick #1");
    await expect(page.locator(".team-name")).toContainText(
      "Kansas City Chiefs",
    );
  });

  test("should make a pick and update draft board", async ({ page }) => {
    await page.goto("/sessions/123");

    // Select a player
    await page.locator(".player-card").first().click();
    await page.locator('button:has-text("Draft Player")').click();

    // Verify pick appears on draft board
    await expect(page.locator(".draft-board .pick")).toHaveCount(1);
    await expect(page.locator(".current-pick")).toContainText("Pick #2");
  });
});
```

### Running Frontend Tests

```bash
cd front-end

# Unit/integration tests
npm test

# Watch mode
npm run test:watch

# E2E tests (requires dev server running)
npm run test:e2e

# Run specific test file
npm test -- path/to/test.test.ts
```

## Test Isolation

### Backend Test Database

- **Database**: `nfl_draft_test` (separate from `nfl_draft` development DB)
- **Setup**: Created once, schema managed by migrations
- **Cleanup**: Each test cleans up its data after execution
- **Isolation**: Tests run serially (`--test-threads=1`) to avoid conflicts

```bash
# Create test database (one-time setup)
sqlx database create --database-url "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"

# Run migrations
sqlx migrate run --database-url "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"
```

### Frontend Test Isolation

- **Unit Tests**: Isolated by default (no shared state)
- **E2E Tests**: Each test starts with fresh state (backend provides clean data)
- **Mock Data**: Fixtures defined in `tests/fixtures.ts`

## Configuration

### Backend Test Configuration

```rust
// .env.test (for test database)
TEST_DATABASE_URL=postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test
```

### Frontend Test Configuration

```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    exclude: [
      "**/node_modules/**",
      "**/tests/**", // Exclude Playwright E2E tests
      "**/*.spec.ts",
    ],
  },
});
```

```typescript
// playwright.config.ts
export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  workers: 1, // Single worker for database stability
  use: {
    baseURL: "http://localhost:5173",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:5173",
    reuseExistingServer: !process.env.CI,
  },
});
```

## Coverage Goals

We track code coverage but don't enforce strict thresholds:

- **Backend**: Aim for 70%+ coverage on domain/business logic
- **Frontend**: Aim for 60%+ coverage on utilities and stores
- **Focus**: Coverage on critical paths (draft logic, trade validation, etc.)

**Why not 100% coverage?**:

- Diminishing returns on trivial code (getters, simple mappers)
- E2E tests provide coverage without unit test duplication
- Developer productivity matters - test what's valuable

## Continuous Integration

All tests run in CI (GitHub Actions):

```yaml
# .github/workflows/test.yml
jobs:
  backend-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Start PostgreSQL
        run: docker compose up -d postgres
      - name: Run tests
        run: cargo test --workspace -- --test-threads=1

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Install dependencies
        run: npm install
      - name: Run unit tests
        run: npm test
      - name: Run E2E tests
        run: npm run test:e2e
```

## References

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest](https://vitest.dev/)
- [Playwright](https://playwright.dev/)
- [Testing Library](https://testing-library.com/)
- [Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html)
