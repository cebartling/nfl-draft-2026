# ADR 0010: Full-Stack Containerized Acceptance Testing

## Status

Accepted

## Context

The NFL Draft 2026 project has two existing testing layers:

1. **Backend acceptance tests** (Rust, in `back-end/crates/api/tests/`) — HTTP API tests with direct database verification, running against ephemeral servers on the test database.
2. **Frontend E2E tests** (Playwright, in `front-end/tests/`) — Browser tests running against the Vite dev server.

Neither layer exercises the **full production-like deployment**: browser requests flowing through nginx, proxied to the Rust API, persisted in PostgreSQL with seeded data. This gap means we do not validate:

- nginx reverse proxy behavior (API routing via `/api/`, WebSocket upgrade at `/ws`, SPA fallback)
- Docker networking between containers
- Seeded data integrity as seen through the complete stack
- Static asset serving from the production SvelteKit build

We needed a testing layer that exercises the entire containerized stack end-to-end while still providing direct database verification for data integrity assertions.

## Decision

We will create a standalone `acceptance-tests/` directory at the repository root containing Playwright tests that run against the Docker Compose services (postgres, api, frontend). Tests interact with the system through three channels:

1. **Browser** — Playwright drives Chromium against `http://localhost:3000` (nginx frontend container)
2. **REST API** — Direct `fetch` calls to `http://localhost:8000` for fast test setup/teardown
3. **Database** — `pg.Pool` connection to PostgreSQL for data integrity assertions

Key design decisions:

- **No `webServer` block** — Docker containers are managed externally by a shell script, not by Playwright
- **Serial execution** (`workers: 1`) — Tests share the seeded database and must not interfere
- **Seeded data as baseline** — Browse/read tests rely on the `seed` Docker profile; write tests clean up after themselves
- **Chromium only** — Single browser project; cross-browser testing deferred to CI matrix if needed

## Consequences

### Positive

- **Production-fidelity** — Tests exercise the same nginx + API + DB stack that runs in production
- **Data integrity verification** — Direct SQL assertions confirm that UI operations persist correctly
- **Fast setup for write tests** — API-based test setup avoids slow browser interactions for draft/session creation
- **Independent of dev tooling** — Tests do not depend on Vite dev server, hot reload, or local Rust builds
- **Debuggable** — Containers stay running after tests for manual inspection; `--headed` mode for visual debugging

### Negative

- **Container build time** — Tests require building Docker images, which adds latency (~2-5 minutes for first build)
- **Resource consumption** — Running 4 containers (postgres, api, frontend, seed) requires Docker Desktop with adequate resources
- **Serial execution is slower** — `workers: 1` prevents parallel test execution, trading speed for reliability
- **Maintenance of separate test project** — Another `package.json`, `node_modules`, and Playwright version to maintain

### Neutral

- **Coexists with existing tests** — Does not replace or modify existing backend acceptance tests or frontend E2E tests
- **Cleanup responsibility** — Tests that create data must clean up; seeded reference data is preserved

## Alternatives Considered

### Extend frontend Playwright tests

Add database verification to the existing `front-end/tests/` Playwright setup.

**Rejected because**: Couples the frontend test project to `pg` (a Node.js PostgreSQL driver), mixes Vite dev server testing with containerized testing, and conflates two different testing purposes (component-level vs. full-stack).

### Use Cypress instead of Playwright

Cypress has built-in support for database seeding via `cy.task()`.

**Rejected because**: Playwright is already used in the frontend project, adding Cypress introduces a second browser testing framework with different APIs and mental models.

### Test only in CI

Run full-stack tests only in GitHub Actions, not locally.

**Rejected because**: Local feedback during development is essential for debugging; CI-only tests become "someone else's problem" and rot faster.

## References

- [Playwright Test documentation](https://playwright.dev/docs/test-configuration)
- [Docker Compose healthcheck documentation](https://docs.docker.com/compose/how-tos/startup-order/)
- ADR-0006: Comprehensive Testing Strategy (updated to reference this layer)
