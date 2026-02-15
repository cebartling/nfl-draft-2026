# Architectural Decision Records (ADRs)

This directory contains Architectural Decision Records (ADRs) for the NFL Draft Simulator 2026 project. ADRs document important architectural and design decisions made during the development of the project.

## What is an ADR?

An Architectural Decision Record (ADR) is a document that captures an important architectural decision made along with its context and consequences. ADRs help:

- Document why decisions were made
- Provide context for future developers
- Prevent revisiting settled decisions
- Enable informed evolution of the architecture

## ADR Format

Each ADR follows this structure:

- **Title**: Short, descriptive title (e.g., "Use Rust Backend with Axum")
- **Status**: Accepted, Proposed, Deprecated, or Superseded
- **Context**: The situation and forces at play
- **Decision**: The architectural decision made
- **Consequences**: Positive, negative, and neutral impacts

## Index of ADRs

### Infrastructure & Organization

- [ADR-0001: Monorepo Structure](./0001-monorepo-structure.md)
  - Decision to use a monorepo with separate `back-end/` and `front-end/` directories
  - Enables coordinated development and shared documentation

### Backend Decisions

- [ADR-0002: Rust Backend with Axum](./0002-rust-backend-with-axum.md)
  - Choice of Rust + Axum for backend API server
  - Prioritizes type safety, performance, and concurrency

- [ADR-0003: Repository Pattern with Traits](./0003-repository-pattern-with-traits.md)
  - Trait-based abstraction for data access layer
  - Enables testability and separation of concerns

- [ADR-0007: Event Sourcing for Draft Events](./0007-event-sourcing-for-draft-events.md)
  - Hybrid event sourcing with JSONB storage
  - Maintains both event history and current state

- [ADR-0008: API-Based Admin Seeding Endpoint](./0008-api-based-admin-seeding.md)
  - HTTP admin endpoint for player data seeding in hosted environments
  - Env-var gated security, library crate reuse, embedded data via `include_str!`

- [ADR-0009: Per-Session Mutex for Concurrency Control](./0009-per-session-mutex-concurrency-control.md)
  - In-memory per-session mutex with `try_lock()` for immediate rejection
  - Prevents concurrent mutation of draft session state

- [ADR-0014: Cargo Workspace with Layered Crate Architecture](./0014-cargo-workspace-layered-crate-architecture.md)
  - Multi-crate workspace using `crates/` directory convention
  - Compile-time enforcement of layer boundaries between api, domain, db, and websocket

### Frontend Decisions

- [ADR-0004: SvelteKit with Svelte 5 Runes](./0004-sveltekit-with-svelte-5-runes.md)
  - Choice of SvelteKit and Svelte 5 runes for frontend
  - Prioritizes bundle size, performance, and developer experience

- [ADR-0013: Client-Side Consensus Ranking Computation](./0013-client-side-consensus-ranking-computation.md)
  - Compute consensus rankings client-side from per-source rankings
  - Avoids backend aggregation complexity, enables instant re-sorting

### Cross-Cutting Concerns

- [ADR-0005: WebSocket for Real-Time Updates](./0005-websocket-real-time-updates.md)
  - WebSocket for bidirectional real-time communication
  - Architecture for connection management and message protocol

- [ADR-0006: Comprehensive Testing Strategy](./0006-testing-strategy.md)
  - Multi-layered testing approach (unit, integration, E2E)
  - Test database isolation and CI/CD integration

- [ADR-0010: Full-Stack Containerized Acceptance Testing](./0010-full-stack-containerized-acceptance-testing.md)
  - Browser-level acceptance tests against Docker Compose stack
  - Playwright + direct PostgreSQL verification via `pg`

- [ADR-0011: Screenplay Pattern for Acceptance Tests](./0011-screenplay-pattern-for-acceptance-tests.md)
  - Actors, Abilities, Tasks, and Questions for multi-channel testing
  - Cleanly separates browser, API, and database interactions

- [ADR-0012: Update to Testing Strategy — Cross-Cutting](./0012-update-testing-strategy-cross-cutting.md)
  - Three-tier testing model: backend acceptance, frontend E2E, full-stack acceptance
  - Bug category matrix across tiers

## Decision Status

### Accepted

All current ADRs are in **Accepted** status, meaning they represent active architectural decisions in the project.

### Future Statuses

- **Proposed**: Decision under consideration, not yet implemented
- **Deprecated**: Decision no longer applicable, kept for historical context
- **Superseded**: Replaced by a newer decision (with reference to the new ADR)

## Creating New ADRs

When making a significant architectural decision:

1. **Create a new ADR file**: Use the next sequential number (e.g., `0008-title.md`)
2. **Use the standard template**:

   ```markdown
   # ADR XXXX: [Title]

   ## Status

   [Proposed | Accepted | Deprecated | Superseded]

   ## Context

   [Describe the situation and forces at play]

   ## Decision

   [The architectural decision made]

   ## Consequences

   ### Positive

   [Benefits of this decision]

   ### Negative

   [Drawbacks or trade-offs]

   ### Neutral

   [Neither clearly positive nor negative]

   ## Alternatives Considered

   [Other options and why they were rejected]

   ## References

   [Links to relevant documentation]
   ```

3. **Update this index**: Add the new ADR to the appropriate section
4. **Commit with descriptive message**: `docs: add ADR-XXXX for [decision]`

## What Deserves an ADR?

Create an ADR for decisions that:

- **Have long-term impact** on the project architecture
- **Are difficult or expensive to change** later
- **Affect multiple parts** of the system
- **Involve trade-offs** between competing concerns
- **Others will wonder about** in the future ("Why did we choose X over Y?")

Examples:

- ✅ Choice of programming language or framework
- ✅ Database schema design patterns
- ✅ Authentication/authorization approach
- ✅ API design patterns
- ✅ Testing strategy
- ❌ Variable naming conventions
- ❌ Code formatting preferences (use linter config instead)
- ❌ Library version updates

## Additional Resources

- [ADR GitHub Organization](https://adr.github.io/)
- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [Sustainable Architectural Decisions](https://www.thoughtworks.com/insights/blog/architecture/architecture-decision-records)
