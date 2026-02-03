# ADR 0002: Rust Backend with Axum Web Framework

## Status

Accepted

## Context

We needed to choose a technology stack for the backend API server that would handle:

- RESTful API endpoints for CRUD operations
- WebSocket connections for real-time draft updates
- Database interactions with PostgreSQL
- High concurrency for multiple draft sessions
- Type safety and performance

Options considered:

1. **Node.js + Express/Fastify**: Familiar, large ecosystem, JavaScript/TypeScript
2. **Go + Gin/Echo**: Fast, simple concurrency, compiled
3. **Rust + Axum**: Type-safe, high performance, async/await, compiled
4. **Python + FastAPI**: Rapid development, type hints, large ecosystem

## Decision

We will use **Rust with the Axum web framework** for the backend API server.

Key aspects:

- **Language**: Rust 1.75+ (2021 edition)
- **Web Framework**: Axum (built on Hyper and Tower)
- **Async Runtime**: Tokio
- **Database**: SQLx (compile-time verified queries)
- **WebSocket**: tokio-tungstenite

## Consequences

### Positive

- **Type Safety**: Rust's type system catches errors at compile time, including database query validation with SQLx
- **Performance**: Rust provides native performance without garbage collection, ideal for real-time updates
- **Concurrency**: Tokio's async runtime handles thousands of concurrent WebSocket connections efficiently
- **Memory Safety**: No null pointer exceptions, no data races, guaranteed by the compiler
- **Axum Benefits**:
  - Built on Tower middleware (mature ecosystem)
  - Excellent integration with Tokio
  - Type-safe extractors for request handling
  - Composable handlers and middleware
- **Developer Experience**: Cargo provides excellent dependency management and tooling

### Negative

- **Learning Curve**: Rust has a steeper learning curve than other languages (ownership, lifetimes, borrow checker)
- **Compile Times**: Rust compilation is slower than interpreted languages
- **Smaller Ecosystem**: Fewer libraries compared to Node.js/Python, though quality is often higher
- **Async Complexity**: Rust's async/await can be complex, especially with lifetimes
- **Hiring**: Fewer developers are experienced with Rust compared to Node.js/Python

### Neutral

- **Explicit Error Handling**: Result types force explicit error handling (can be verbose but prevents bugs)
- **No Runtime**: Compiled binary has no runtime dependencies beyond libc
- **Cross-compilation**: Can build for different platforms, though setup can be complex

## Implementation Notes

### Cargo Workspace Structure

Backend is organized as a Cargo workspace with separate crates:

- `api`: Web server, routes, handlers, middleware
- `domain`: Business logic, services, domain models
- `db`: Database layer, SQLx repositories
- `websocket`: WebSocket connection management

This separation enables:

- Clear boundaries between layers
- Independent testing of each crate
- Reusable components

### Database Access with SQLx

SQLx provides:

- Compile-time query verification against PostgreSQL schema
- Type-safe row mapping
- Support for async/await
- Connection pooling

Example:

```rust
let team = sqlx::query_as!(
    TeamDb,
    "SELECT * FROM teams WHERE id = $1",
    team_id
)
.fetch_one(&pool)
.await?;
```

The query is verified at compile time, preventing SQL typos and type mismatches.

## Alternatives Considered

### Node.js + Express

**Pros**: Large ecosystem, familiar to most developers, JavaScript/TypeScript
**Cons**: Single-threaded event loop, slower performance, runtime type errors
**Rejected**: Performance concerns for real-time WebSocket with many concurrent users

### Go + Gin

**Pros**: Simple concurrency with goroutines, fast compile times, easier than Rust
**Cons**: Lacks Rust's compile-time guarantees, less expressive type system
**Rejected**: Type safety was prioritized over ease of learning

### Python + FastAPI

**Pros**: Rapid development, type hints, large ML/AI ecosystem for future features
**Cons**: Significantly slower performance, GIL limits concurrency, runtime type checking
**Rejected**: Performance requirements for real-time updates were critical

## References

- [Axum Documentation](https://docs.rs/axum/)
- [Tokio Documentation](https://tokio.rs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
