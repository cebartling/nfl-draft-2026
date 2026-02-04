# ADR 0008: API-Based Admin Seeding Endpoint

## Status

Accepted

## Context

The NFL Draft Simulator requires seed data (player prospects) to be loaded into the database before the application is useful. The initial implementation used a standalone CLI binary (`seed-players`) run via Docker Compose's `seed` service profile. This approach works well in local development but presents challenges in hosted environments:

- **Platform limitations**: Some hosted platforms (e.g., DigitalOcean App Platform) don't support ephemeral container execution or one-shot jobs, making it impossible to run the seed binary as a separate container.
- **Filesystem assumptions**: The CLI binary reads player JSON from the filesystem, which may not be available or consistent across deployment environments.
- **Operational complexity**: SSH access for running CLI commands is often unavailable or restricted in managed hosting environments.
- **Code duplication risk**: The parsing, validation, and loading logic in the `seed-data` crate needed to be reusable by other crates without duplicating code.

We needed a portable seeding mechanism that works in any deployment environment while maintaining security and reusing existing code.

## Decision

We will add an **HTTP admin endpoint** for player data seeding with a multi-layered approach covering security, code reuse, and deployment portability.

### 1. Admin Seeding Endpoint

A `POST /api/v1/admin/seed-players` endpoint in the API server that triggers the same seeding logic as the CLI tool.

### 2. Env-Var Gated Endpoint with API Key Authentication

A two-layer security model:

- **Layer 1 — Endpoint hiding**: When `SEED_API_KEY` is not configured (or empty), the endpoint returns `404 Not Found`. The endpoint does not exist unless explicitly enabled.
- **Layer 2 — API key authentication**: When `SEED_API_KEY` is configured, requests must include a matching `X-Seed-Api-Key` header. Mismatched keys return `401 Unauthorized`.

```rust
// If SEED_API_KEY is not set, the endpoint is hidden entirely
if state.seed_api_key.is_none() {
    return Err(ApiError::NotFound("Not found".to_string()));
}

// If set, require matching header
let expected_key = state.seed_api_key.as_ref().unwrap();
match headers.get("X-Seed-Api-Key") {
    Some(provided_key) if provided_key == expected_key => { /* proceed */ }
    _ => return Err(ApiError::Unauthorized("Invalid API key".to_string())),
}
```

### 3. Library + Binary Dual-Target Crate

The `seed-data` crate was converted from binary-only to a dual-target crate (library + binary) so the API crate can depend on its modules directly:

```toml
# back-end/crates/seed-data/Cargo.toml
[lib]
name = "seed_data"
path = "src/lib.rs"

[[bin]]
name = "seed-players"
path = "src/main.rs"
```

The library exports three public modules:
- `seed_data::loader` — JSON parsing and database loading
- `seed_data::validator` — Data validation with warnings and errors
- `seed_data::position_mapper` — Position string normalization

### 4. Embedded Data via `include_str!`

Player JSON data is compiled into the API binary using `include_str!`, removing any filesystem dependency at runtime:

```rust
const PLAYERS_2026_JSON: &str = include_str!("../../../../data/players_2026.json");
```

The API handler calls into the library's parsing and loading functions with this embedded string, using the same code paths as the CLI tool.

## Consequences

### Positive

- **Deployment flexibility**: Seeding works on any platform that can run the API server, regardless of container orchestration support
- **Code reuse**: Single implementation of parsing, validation, and loading logic shared between CLI and API
- **Zero filesystem dependencies**: Embedded data means no runtime file path concerns
- **Defense in depth**: Two-layer security (endpoint hiding + API key) reduces attack surface
- **Operational simplicity**: Seeding can be triggered with a single HTTP request (e.g., `curl`) after deployment
- **Validation parity**: Both CLI and API paths use the same validation logic, ensuring data integrity

### Negative

- **Attack surface**: An authenticated admin endpoint is an additional surface to secure; a leaked API key could allow data manipulation
- **Embedded data staleness**: Player data is baked into the binary at compile time; updating data requires recompilation and redeployment
- **Binary size increase**: Embedding the JSON data (~100KB) increases the compiled binary size
- **Compile-time coupling**: Changes to the JSON data file trigger recompilation of the API crate

### Neutral

- **Additional endpoint to maintain**: The seed handler adds code to the API crate, but the core logic lives in the `seed-data` library
- **CLI tool remains functional**: The original binary still works for local development and Docker Compose workflows
- **Environment variable dependency**: `SEED_API_KEY` must be configured in production for the endpoint to be available

## Alternatives Considered

### Container-Only Seeding (Docker Compose Service)

**Pros**: Already implemented, clean separation, runs once and exits
**Cons**: Not supported by all hosted platforms, requires container orchestration
**Rejected**: Does not work on platforms without ephemeral container support

### CLI Tool with SSH Access

**Pros**: Simple, uses existing binary, no API changes needed
**Cons**: Requires SSH access to production, not available on many managed platforms, manual process
**Rejected**: Not portable across hosting environments

### Database Dump and Restore

**Pros**: Fast, exact replica, no application code involved
**Cons**: Requires direct database access, version-dependent, bypasses validation, not idempotent
**Rejected**: Skips application-level validation and doesn't integrate with the application lifecycle

### Migration-Based Seeding

**Pros**: Runs automatically with migrations, guaranteed to execute
**Cons**: Mixes schema changes with data changes, hard to update seed data independently, runs on every deployment
**Rejected**: Seed data changes more frequently than schema and should be controllable independently

## References

- `back-end/crates/api/src/handlers/seed.rs` — Admin seed endpoint handler
- `back-end/crates/api/src/routes/mod.rs` — Route registration
- `back-end/crates/api/src/config.rs` — `SEED_API_KEY` environment variable loading
- `back-end/crates/api/src/state.rs` — AppState with optional seed API key
- `back-end/crates/seed-data/src/lib.rs` — Library exports (loader, validator, position_mapper)
- `back-end/crates/seed-data/src/loader.rs` — JSON parsing and player loading logic
- `back-end/crates/seed-data/src/validator.rs` — Data validation with warnings and errors
- `back-end/crates/seed-data/Cargo.toml` — Dual-target crate configuration
- `back-end/data/players_2026.json` — Embedded player prospect data
