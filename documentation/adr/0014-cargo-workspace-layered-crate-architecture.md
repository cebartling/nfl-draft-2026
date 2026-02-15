# ADR 0014: Cargo Workspace with Layered Crate Architecture

## Status

Accepted

## Context

The NFL Draft Simulator backend is a Rust application with multiple distinct concerns: HTTP API serving, domain business logic, database persistence, WebSocket real-time communication, data seeding, and web scraping tools. As the codebase grew, we needed an organizational strategy that would:

- Enforce layer boundaries at compile time, not just by convention
- Allow independent compilation and testing of each concern
- Prevent accidental coupling (e.g., database types leaking into domain logic)
- Support both tightly-coupled application crates and standalone utility binaries
- Leverage Rust's module and crate system as an architectural enforcement mechanism

ADR-0002 selected Rust with Axum and mentioned the workspace structure as an implementation note. ADR-0003 established the repository pattern with trait-based abstraction across layers. However, neither documents the foundational decision to organize the backend as a **multi-crate Cargo workspace using the `crates/` directory convention** — the structural choice that makes the layered architecture enforceable rather than aspirational.

## Decision

We will organize the backend as a **Cargo workspace** with member crates under a `crates/` directory, following the idiomatic Rust convention used by major projects (Tokio, Bevy, Cargo itself).

### Workspace Structure

```
back-end/
├── Cargo.toml              # Workspace root (members, shared dependencies)
├── crates/
│   ├── api/                # HTTP server, routes, handlers, middleware
│   ├── domain/             # Business logic, services, models, repository traits
│   ├── db/                 # SQLx repository implementations, database models
│   ├── websocket/          # WebSocket connection management
│   ├── seed-data/          # Data loading pipeline (lib + binary)
│   ├── draft-order-scraper/        # Standalone scraping tool
│   └── prospect-rankings-scraper/  # Standalone scraping tool
└── migrations/             # SQLx database migrations
```

### Crate Categories

**Core application crates** form the layered architecture:

| Crate       | Role                        | Local Dependencies          |
|-------------|-----------------------------|-----------------------------|
| `domain`    | Foundation layer            | None                        |
| `db`        | Persistence layer           | `domain`                    |
| `websocket` | Real-time communication     | `domain`                    |
| `api`       | Coordination/entry point    | `domain`, `db`, `websocket`, `seed-data` |

**Data pipeline crates** bridge domain and persistence:

| Crate       | Role                        | Local Dependencies          |
|-------------|-----------------------------|-----------------------------|
| `seed-data` | JSON loading and validation | `domain`, `db`              |

**Standalone tool crates** have zero local dependencies:

| Crate                       | Role                                  | Local Dependencies |
|-----------------------------|---------------------------------------|--------------------|
| `draft-order-scraper`       | Scrapes draft order data from the web | None               |
| `prospect-rankings-scraper` | Scrapes prospect rankings from the web| None               |

### Dependency Graph

```
                    ┌─────────────────────┐
                    │        api          │
                    │  (coordination)     │
                    └──┬────┬────┬────┬───┘
                       │    │    │    │
            ┌──────────┘    │    │    └──────────┐
            ▼               ▼    ▼               ▼
     ┌────────────┐  ┌─────────┐  ┌───────────┐
     │  seed-data │  │   db    │  │ websocket  │
     │ (pipeline) │  │ (SQLx)  │  │ (realtime) │
     └──┬────┬────┘  └────┬────┘  └─────┬──────┘
        │    │             │             │
        │    └─────┐       │       ┌─────┘
        │          ▼       ▼       ▼
        │       ┌─────────────────────┐
        └──────►│      domain         │
                │   (foundation)      │
                └─────────────────────┘

  ┌──────────────────────┐  ┌─────────────────────────────┐
  │ draft-order-scraper  │  │ prospect-rankings-scraper   │
  │    (standalone)      │  │        (standalone)          │
  └──────────────────────┘  └─────────────────────────────┘
```

The `domain` crate sits at the bottom with zero local dependencies. It defines models, service logic, and repository traits. The `db` and `websocket` crates depend only on `domain`. The `api` crate coordinates everything. Standalone scrapers are fully independent.

### Workspace-Level Dependency Management

Shared third-party dependencies are declared once in the workspace root `Cargo.toml` under `[workspace.dependencies]` and referenced by member crates with `workspace = true`. This ensures version consistency and deduplication across all crates.

## Consequences

### Positive

- **Compile-time boundary enforcement**: A crate cannot import from another crate unless it declares the dependency in its `Cargo.toml`. The `domain` crate physically cannot depend on `db` or `sqlx` — the compiler prevents it, not just convention.
- **Independent testing**: Each crate can be tested in isolation (`cargo test -p domain`), making it easy to verify that domain logic works without a database.
- **Incremental compilation**: Changing code in `api` does not trigger recompilation of `domain` or `db`, significantly reducing rebuild times during development.
- **Dependency deduplication**: Workspace-level `[workspace.dependencies]` ensures all crates share the same versions of third-party dependencies, producing a single copy of each library in the final binary.
- **Clear ownership**: Each crate has a focused responsibility, making it obvious where new code belongs.
- **Standalone tools**: Scraper crates compile independently and can be built/run without the full application stack.
- **Idiomatic convention**: The `crates/` directory pattern is widely recognized in the Rust ecosystem, reducing onboarding friction.

### Negative

- **Initial setup boilerplate**: Each crate requires its own `Cargo.toml`, `src/lib.rs` or `src/main.rs`, and module declarations. Adding a new crate involves creating several files.
- **First-build compilation time**: The initial `cargo build --workspace` compiles all crates and their dependencies, which can be slow. Subsequent incremental builds are fast.
- **Cross-crate refactoring friction**: Moving a type from one crate to another requires updating `Cargo.toml` dependencies, import paths, and potentially re-export declarations across multiple crates.
- **Circular dependency impossibility**: Cargo prohibits circular dependencies between crates, which occasionally requires architectural workarounds (e.g., defining traits in `domain` even when they feel like they belong closer to the implementation).

### Neutral

- **Visibility control**: Rust's `pub` visibility works at the crate level — items must be explicitly exported to be used by dependent crates, providing natural encapsulation.
- **Feature flags**: Workspace crates can define feature flags for conditional compilation, though this project has not yet needed them.

## Alternatives Considered

### Single Crate with Modules

**Approach**: One `back-end` crate with `mod api`, `mod domain`, `mod db`, etc.

**Pros**: Simpler setup, no cross-crate boilerplate, easier refactoring within the crate.

**Cons**: Layer boundaries enforced only by convention — nothing prevents `domain/` from importing `sqlx` types. All code recompiles on any change. Cannot build or test layers independently.

**Rejected**: Convention-only boundaries erode over time, especially as the team grows. Compile-time enforcement was deemed essential.

### Flat Workspace (No `crates/` Directory)

**Approach**: Workspace members at the repository root level (`back-end/api/`, `back-end/domain/`, etc.) without a `crates/` parent directory.

**Pros**: Slightly shorter paths.

**Cons**: Mixes workspace metadata files with crate directories, making the root cluttered. Less idiomatic — the `crates/` convention immediately signals "Cargo workspace member" to experienced Rust developers.

**Rejected**: The `crates/` directory provides better organization and follows ecosystem convention.

### Microservices

**Approach**: Separate deployable services for API, WebSocket, etc., communicating over the network.

**Pros**: Independent deployment, language flexibility per service.

**Cons**: Massive operational complexity for a project of this scale. Network serialization overhead for what are currently in-process function calls. Shared domain model becomes a versioned API contract problem.

**Rejected**: The application does not yet need independent scaling of components. A monolithic workspace provides the same code separation benefits without operational overhead.

## Related Decisions

- **ADR-0002** (Rust Backend with Axum): Selects the language and framework; mentions the workspace as an implementation note.
- **ADR-0003** (Repository Pattern with Traits): Defines the trait-based abstraction that makes the `domain` → `db` dependency direction work. The workspace structure is what enforces that `db` depends on `domain` and not vice versa.

## References

- [Cargo Workspaces — The Rust Programming Language](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Tokio workspace structure](https://github.com/tokio-rs/tokio) — foundational async runtime using `crates/` convention
- [Bevy workspace structure](https://github.com/bevyengine/bevy) — game engine with extensive `crates/` organization
- [Cargo's own workspace](https://github.com/rust-lang/cargo) — Rust's package manager, itself a multi-crate workspace
