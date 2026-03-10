# ADR 0015: Cooperative Cancellation for Auto-Pick Run

## Status

Accepted (amends ADR-0009)

## Context

ADR-0009 established per-session mutexes with `try_lock()` semantics for concurrency control. While effective at preventing concurrent mutations, this design created a critical interaction problem between `pause_session` and `auto_pick_run`:

**The problem**: `auto_pick_run` holds the session mutex for the entire duration of its pick loop (up to ~45 seconds for a 224-pick draft at 200ms per pick). During this time, `pause_session` calls `try_lock()`, fails immediately, and returns an error. The frontend's error-swallowing pattern then shows "Draft session paused" even though the pause request was rejected.

This means:

1. **Users cannot pause a running auto-draft.** The pause button appears responsive but does nothing — the auto-pick loop continues uninterrupted.
2. **Resume after a "phantom pause" fails.** The session was never actually paused on the backend, so the resume attempt either gets "already in progress" or triggers a second auto-pick-run that fails due to lock contention.
3. **Error swallowing hides the failure.** The `DraftState.startSession()` and `pauseSession()` methods caught errors internally without re-throwing, so callers always took the success path regardless of outcome.

This is a fundamental user-experience issue: pausing is a core interaction during a draft, and it must work reliably even when AI picks are being executed.

### Forces

- **User control**: Users must be able to pause a running draft at any time
- **Data consistency**: Picks already committed to the database must not be rolled back
- **Responsiveness**: Pause should complete in under 1-2 seconds, not 45 seconds
- **Simplicity**: The cancellation mechanism should be easy to understand and maintain

## Decision

We introduce **cooperative cancellation** using per-session `AtomicBool` flags, combined with changing `pause_session` from `try_lock()` to a timeout-bounded `lock().await`.

### Implementation

#### 1. Cancellation flags in AppState

```rust
pub struct AppState {
    pub session_locks: Arc<DashMap<Uuid, Arc<Mutex<()>>>>,
    pub auto_pick_cancel: Arc<DashMap<Uuid, Arc<AtomicBool>>>,
}
```

#### 2. Auto-pick-run registers and checks the flag

```rust
pub async fn auto_pick_run(...) {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    state.auto_pick_cancel.insert(id, Arc::clone(&cancel_flag));

    // ... acquire session lock ...

    loop {
        // Check for cancellation before each pick
        if cancel_flag.load(Ordering::SeqCst) {
            tracing::info!("Auto-pick run cancelled (pause requested)");
            break;
        }

        // ... execute pick ...

        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Clean up flag on exit
    state.auto_pick_cancel.remove(&id);
}
```

#### 3. Pause sets the flag then waits for the lock

```rust
pub async fn pause_session(...) {
    // Signal any running auto-pick-run to stop
    if let Some(cancel_flag) = state.auto_pick_cancel.get(&id) {
        cancel_flag.store(true, Ordering::SeqCst);
    }

    // Wait for the lock with a timeout (auto-pick will release within ~200ms)
    let _guard = tokio::time::timeout(
        Duration::from_secs(10),
        lock.lock()
    ).await?;

    // Now safe to update session status
    session.pause()?;
}
```

#### 4. Frontend error propagation

`startSession()` and `pauseSession()` in `DraftState` now re-throw errors after logging and storing them, so callers can distinguish success from failure and show appropriate toasts.

### Cancellation timing

The auto-pick loop checks the cancellation flag at the **top of each iteration**, before selecting the next pick. With the 200ms sleep between picks, the worst-case cancellation latency is ~200ms (one sleep cycle) plus the time for the current pick's DB operations (~50ms). The pause handler's 10-second timeout is generous enough to handle any reasonable delay while still failing cleanly if something goes wrong.

## Consequences

### Positive

- **Pause works during auto-pick**: The cancellation flag stops the loop cooperatively, then pause acquires the lock and transitions the session
- **No data loss**: Picks made before cancellation are already committed to the database; the session's `current_pick_number` is persisted on loop exit
- **Low latency**: Pause completes within ~200-300ms in the typical case
- **Clean separation**: The `AtomicBool` is a fire-and-forget signal; the mutex still provides mutual exclusion for the actual state transition
- **Frontend accuracy**: Errors now propagate to callers, so toast messages accurately reflect what happened

### Negative

- **Additional shared state**: `auto_pick_cancel` DashMap adds another piece of shared mutable state to manage. However, it follows the same DashMap pattern as `session_locks` and `ConnectionManager`
- **Cleanup responsibility**: The auto-pick-run handler must clean up its cancel flag on all exit paths (normal completion, cancellation, error). A `defer`-style cleanup would be more robust, but Rust's `Drop` trait doesn't work for async cleanup in DashMap
- **Pause uses blocking wait**: `lock().await` with a timeout means the pause HTTP request may block for up to 200-300ms waiting for auto-pick to release the lock. This is acceptable for a user-initiated action

### Neutral

- **`try_lock()` retained for auto-pick-run**: The auto-pick-run endpoint still uses `try_lock()` to prevent concurrent runs. Only `pause_session` was changed to use `lock().await` with a timeout
- **`start_session` unchanged**: Start still uses `try_lock()` because there is no long-running operation to cancel — it should fail fast if something else is modifying the session

## Alternatives Considered

### Tokio CancellationToken

**Pros**: Richer API (can await cancellation), integrates with `tokio::select!`

**Cons**: Additional dependency (tokio-util), more complex than needed for a simple boolean signal

**Rejected**: `AtomicBool` is sufficient for a "should I stop?" check at the top of a loop. We don't need the async-await capabilities of `CancellationToken`.

### Re-reading session status from DB each iteration

**Pros**: No additional shared state; the database is the source of truth

**Cons**: Adds a DB query per pick (224 extra queries for a full draft), and creates a race condition where pause updates the DB while auto-pick is mid-loop with stale state

**Rejected**: The in-memory flag is simpler, faster, and avoids the race condition.

### Background task with message channel

**Pros**: Auto-pick-run returns immediately with 202 Accepted; pause sends a message to the background task

**Cons**: Requires significant refactoring of the auto-pick-run handler, response format changes, and frontend polling or SSE for completion notification

**Rejected**: This is the architecturally "correct" long-term solution but is disproportionate to the current single-user deployment. May revisit if we need multi-user or horizontal scaling.

## References

- [ADR-0009: Per-Session Mutex for Concurrency Control](./0009-per-session-mutex-concurrency-control.md) — original design this ADR amends
- [std::sync::atomic::AtomicBool](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicBool.html) — cancellation flag primitive
- [Cooperative cancellation pattern](https://tokio.rs/tokio/topics/shutdown) — Tokio shutdown patterns
