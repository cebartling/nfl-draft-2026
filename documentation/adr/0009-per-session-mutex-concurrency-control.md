# ADR 0009: Per-Session Mutex for Concurrency Control

## Status

Accepted

## Context

The draft session system exposes multiple mutation endpoints that modify session state:

- `POST /sessions/:id/start` — transitions session to InProgress
- `POST /sessions/:id/pause` — transitions session to Paused
- `POST /sessions/:id/advance-pick` — increments current_pick_number
- `POST /sessions/:id/auto-pick-run` — loops through AI picks, advancing state with each pick

These endpoints can be called concurrently by multiple clients (e.g., multiple browser tabs, WebSocket-triggered actions, or rapid UI clicks). Without coordination, concurrent requests can corrupt session state:

- Two `advance_pick` calls could skip a pick number
- A `pause` during `auto_pick_run` could leave the session in an inconsistent state (paused but still executing AI picks)
- Two `auto_pick_run` calls could make duplicate picks for the same slot

The `auto_pick_run` endpoint is particularly problematic because it holds session state in memory across multiple database writes within a loop, making it a long-running operation that is inherently unsafe to run concurrently.

Forces at play:

- **Correctness**: Session state transitions must be serialized to prevent corruption
- **Responsiveness**: Concurrent requests to *different* sessions should not block each other
- **Simplicity**: The solution should be easy to reason about and maintain
- **User experience**: Rejected requests should fail fast with a clear error, not hang
- **Memory**: Lock resources should not leak as sessions complete

## Decision

We will use an **in-memory per-session mutex** implemented as `DashMap<Uuid, Arc<Mutex<()>>>` in application state, with `try_lock()` semantics for immediate rejection of concurrent requests.

### Implementation

Each session gets its own `tokio::sync::Mutex`, stored in a `DashMap` keyed by session ID:

```rust
// state.rs
pub struct AppState {
    /// Per-session mutex to prevent concurrent mutation requests
    pub session_locks: Arc<DashMap<Uuid, Arc<Mutex<()>>>>,
}
```

All session mutation handlers acquire the lock before performing any work:

```rust
// handlers/sessions.rs
let lock = state
    .session_locks
    .entry(id)
    .or_insert_with(|| Arc::new(Mutex::new(())))
    .clone();
let _guard = lock.try_lock().map_err(|_| {
    DomainError::InvalidState(
        "Session is being modified by another request".to_string(),
    )
})?;
```

Lock entries are cleaned up when a session completes to prevent unbounded DashMap growth:

```rust
drop(_guard);
if session.status == SessionStatus::Completed {
    state.session_locks.remove(&id);
}
```

## Consequences

### Positive

- **Per-session granularity**: Operations on different sessions never contend with each other
- **Fail-fast behavior**: `try_lock()` returns immediately with a clear error instead of queuing requests, preventing request pile-up behind a long-running auto-pick loop
- **Simple mental model**: At most one mutation per session at any time — easy to reason about correctness
- **No external dependencies**: Uses only Tokio primitives and DashMap, no Redis or database advisory locks needed
- **Automatic cleanup**: DashMap entries removed on session completion

### Negative

- **Single-server only**: In-memory locks do not work across multiple server instances; horizontal scaling would require a different approach (e.g., database advisory locks or Redis distributed locks)
- **Coarse granularity**: All mutations are serialized, even ones that could theoretically run concurrently (e.g., `pause` while `advance_pick` is in-flight). In practice, these are all fast operations except `auto_pick_run`, so this is acceptable
- **Immediate rejection UX**: When `auto_pick_run` is active, other operations return a 400 error rather than queuing. The client must retry or wait. This is acceptable because auto-pick runs are the only long-running operation and the frontend can disable controls during auto-pick
- **Lock entry lifecycle**: DashMap entries are created lazily and only cleaned up on session completion. Abandoned sessions (never completed) will leave orphan entries. This is bounded by the number of sessions created and is negligible in practice

### Neutral

- **DashMap choice**: We already use DashMap for the WebSocket connection manager (ADR-0005), so this is a consistent pattern in the codebase
- **Error mapping**: `try_lock()` failures map to `DomainError::InvalidState`, which produces a 400 HTTP response. This is semantically appropriate — the session is in a transient state that doesn't allow the requested operation

## Alternatives Considered

### Database Advisory Locks (`pg_advisory_lock`)

**Pros**: Works across multiple server instances, leverages existing PostgreSQL infrastructure, automatically released on connection close

**Cons**: Ties lock lifetime to a database connection (complex with connection pooling), blocking variant can cause connection pool exhaustion, non-blocking variant (`pg_try_advisory_lock`) requires careful cleanup

**Rejected**: Adds database coupling to what is fundamentally an application-level concern. Advisory locks are better suited for cross-process coordination, which we don't need in a single-server deployment.

### Optimistic Concurrency (Version Column)

**Pros**: No locks needed, works across multiple servers, standard pattern

**Cons**: Requires retry logic in every handler, doesn't prevent the *execution* of concurrent auto-pick loops (only detects conflicts at write time), version conflicts during a multi-pick auto-pick loop would require complex rollback

**Rejected**: Optimistic concurrency detects conflicts after work is done. For `auto_pick_run`, we need to *prevent* concurrent execution entirely, not just detect it.

### Global Mutex (Single Lock for All Sessions)

**Pros**: Simplest possible implementation

**Cons**: All sessions contend for the same lock. A long auto-pick run on one session blocks all operations on every other session.

**Rejected**: Unacceptable contention in multi-session scenarios.

### Queuing with `.lock().await` (Blocking Variant)

**Pros**: Requests queue up and eventually execute in order, no client-side retry needed

**Cons**: A request behind a long-running `auto_pick_run` (which may take 30+ seconds for a full draft) would hold an HTTP connection open indefinitely, risking timeouts and resource exhaustion. Multiple queued requests could pile up.

**Rejected**: The fail-fast approach with `try_lock()` is safer for HTTP request handling. Clients can show "in progress" UI rather than hanging on a long request.

## References

- [DashMap documentation](https://docs.rs/dashmap/)
- [tokio::sync::Mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html)
- [ADR-0005: WebSocket for Real-Time Updates](./0005-websocket-real-time-updates.md) — DashMap usage precedent
