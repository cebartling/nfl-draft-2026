# ADR 0012: Update to Testing Strategy — Cross-Cutting Acceptance Testing Layer

## Status

Accepted (supplements ADR-0006)

## Context

ADR-0006 established a comprehensive testing strategy with backend tests (unit, integration, acceptance) and frontend tests (unit, component, E2E). With the addition of full-stack containerized acceptance tests (ADR-0010) organized using the Screenplay pattern (ADR-0011), the testing strategy now has three tiers that need to be clearly differentiated.

## Decision

We update the testing strategy to define three distinct acceptance/E2E testing tiers:

### Tier 1: Backend Acceptance Tests (Rust)

- **Location**: `back-end/crates/api/tests/`
- **Target**: HTTP API endpoints + database verification
- **Stack**: reqwest client → ephemeral Axum server → test database (`nfl_draft_test`)
- **Purpose**: Verify API contracts, request/response shapes, status codes, and database persistence
- **Characteristics**: Fast (~seconds), no browser, no containers, clean database per test suite
- **When to use**: Testing API behavior, business logic through HTTP, error responses

### Tier 2: Frontend E2E Tests (Playwright in `front-end/`)

- **Location**: `front-end/tests/`
- **Target**: UI components and user flows in the browser
- **Stack**: Playwright → Vite dev server → local/mocked backend
- **Purpose**: Verify UI rendering, client-side routing, component interactions
- **Characteristics**: Fast iteration with HMR, may use mocked API responses
- **When to use**: Testing UI behavior, responsive design, client-side state management

### Tier 3: Full-Stack Acceptance Tests (Playwright in `acceptance-tests/`)

- **Location**: `acceptance-tests/`
- **Target**: Complete containerized stack (browser → nginx → API → PostgreSQL)
- **Stack**: Playwright → Docker nginx → Docker API → Docker PostgreSQL (with seeded data)
- **Purpose**: Verify production-like deployment, data integrity across the full stack
- **Characteristics**: Slower (~minutes for container builds), requires Docker, uses real seeded data
- **When to use**: Validating deployment configuration, nginx routing, seeded data integrity, end-to-end business flows

### How the tiers relate

```
Tier 3: Full-Stack Acceptance (acceptance-tests/)
  Browser → nginx → API → PostgreSQL + DB assertions
  ┌────────────────────────────────────────────────┐
  │                                                │
  │  Tier 2: Frontend E2E (front-end/)             │
  │    Browser → Vite dev server                   │
  │    ┌─────────────────────────────────┐         │
  │    │                                 │         │
  │    │  Tier 1: Backend Acceptance     │         │
  │    │    HTTP → Axum → PostgreSQL     │         │
  │    │                                 │         │
  │    └─────────────────────────────────┘         │
  │                                                │
  └────────────────────────────────────────────────┘
```

Each tier catches different categories of bugs:

| Bug Category | Tier 1 | Tier 2 | Tier 3 |
|---|---|---|---|
| API contract violations | Yes | — | Yes |
| Database persistence errors | Yes | — | Yes |
| UI rendering issues | — | Yes | Yes |
| nginx proxy misconfig | — | — | Yes |
| Docker networking issues | — | — | Yes |
| Seeded data integrity | — | — | Yes |
| Client-side routing | — | Yes | Yes |

## Consequences

### Positive

- **Clear ownership** — Each tier has a defined purpose and location, preventing overlap
- **Right tool for the job** — Developers choose the appropriate tier based on what they're testing
- **Layered confidence** — Bugs caught by lower tiers don't need to be re-tested at higher tiers

### Negative

- **Three test suites to maintain** — More maintenance overhead than two
- **Longer CI pipeline** — Full test run includes all three tiers sequentially

### Neutral

- **Independent evolution** — Each tier can evolve its tooling independently (e.g., Tier 1 could switch to a different HTTP client without affecting Tier 3)

## References

- [ADR-0006: Comprehensive Testing Strategy](./0006-testing-strategy.md)
- [ADR-0010: Full-Stack Containerized Acceptance Testing](./0010-full-stack-containerized-acceptance-testing.md)
- [ADR-0011: Screenplay Pattern for Acceptance Tests](./0011-screenplay-pattern-for-acceptance-tests.md)
- [Test Pyramid — Martin Fowler](https://martinfowler.com/articles/practical-test-pyramid.html)
