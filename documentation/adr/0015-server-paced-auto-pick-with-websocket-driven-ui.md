# ADR 0015: Server-Paced Auto-Pick with WebSocket-Driven UI Updates

## Status

Accepted

## Context

The `auto_pick_run` endpoint executes AI picks in a loop until it reaches a user-controlled team's turn or the draft completes. Originally, the loop used `tokio::task::yield_now()` between iterations, completing all picks in ~100ms. WebSocket `pick_made` broadcasts fired near-simultaneously, and the frontend suppressed both toast notifications and pick advancement during auto-pick (gated by `isAutoPickRunning`). The HTTP response was treated as the authoritative state update.

This created a poor user experience: the draft jumped from pick N to pick N+30 with no visual feedback about intermediate picks. Users had no sense of the draft progressing — it appeared to freeze and then snap to a future state.

Two competing forces were at play:

- **HTTP response latency**: Faster loop = faster HTTP response, but no real-time feedback
- **Real-time UX**: Users expect to see each pick announced individually, as in a real draft broadcast
- **Dual-authority complexity**: The frontend had two state update paths during auto-pick — suppressed WebSocket messages and the authoritative HTTP response — creating subtle ordering bugs and making the data flow hard to reason about

Additionally, the frontend's `wsPickCounter` effect (which refreshes the available players list) was also suppressed during auto-pick, meaning the player list became stale until the entire auto-pick run completed.

## Decision

We will **pace auto-pick iterations with a 1-second delay** on the backend and **remove all WebSocket suppression guards** on the frontend, making WebSocket messages the single real-time update path regardless of auto-pick state.

### Backend Change

Replace `tokio::task::yield_now()` with `tokio::time::sleep(Duration::from_secs(1))` in the `auto_pick_run` loop. Each pick's WebSocket broadcast now arrives ~1 second after the previous one.

### Frontend Changes

1. Remove `if (!draftState.isAutoPickRunning)` guards around `draftState.advancePick()` and `toastState.info(...)` in the `pick_made` WebSocket handler
2. Remove `!draftState.isAutoPickRunning` from the `wsPickCounter` effect that refreshes available players
3. Retain `isAutoPickRunning` solely as a **concurrency guard** to prevent re-triggering `autoPickRun()` from multiple call sites

### Retained Use of `isAutoPickRunning`

The flag is still set/cleared around the `autoPickRun()` HTTP call in two places:

- `+page.svelte`: After a manual pick triggers AI continuation
- `DraftCommandCenter.svelte`: When the user clicks the auto-pick button

It prevents concurrent auto-pick requests (complementing the server-side per-session mutex from ADR-0009). It no longer suppresses any UI updates.

## Consequences

### Positive

- **Pick-by-pick feedback**: Users see individual toast notifications ~1 second apart, creating a "live draft broadcast" feel
- **Simplified data flow**: WebSocket `pick_made` is the single source of real-time UI updates during both manual and auto-pick modes — no conditional suppression logic
- **Real-time player list**: Available players refresh after each AI pick, not just after the entire batch completes
- **Easier reasoning**: The `pick_made` handler has one unconditional code path instead of branching on `isAutoPickRunning`
- **Better testability**: Tests assert that `advancePick` and `toastState.info` are always called, regardless of auto-pick state — simpler test expectations

### Negative

- **Slower HTTP response**: A 32-pick auto-pick run now takes ~32 seconds to return instead of ~100ms. The frontend must tolerate a long-running HTTP request while WebSocket messages drive the UI. This is acceptable because the `isAutoPickRunning` guard prevents duplicate requests, and the per-session mutex (ADR-0009) rejects concurrent server-side attempts
- **Toast accumulation**: Many picks produce many toasts. The existing toast system auto-dismisses after a timeout, but a 32-pick run will cycle through 32 toasts. This is the desired behavior (users want to see each pick) but could be refined later with toast stacking or a consolidated notification
- **Increased server hold time**: The per-session mutex (ADR-0009) is held for the duration of the auto-pick run, which is now ~N seconds for N picks. This is acceptable because `try_lock()` immediately rejects concurrent requests rather than queuing them

### Neutral

- **HTTP response still updates final state**: The `auto_pick_run` response still returns the final session state and all picks made, which the frontend uses to reconcile state after the run completes. This provides a consistency checkpoint even though the UI was already updated incrementally via WebSocket
- **1-second interval is a UX choice**: The delay is not load-driven but UX-driven. It could be made configurable (e.g., per-session `pick_delay_seconds`) in the future, but a fixed 1-second delay matches the cadence of a draft broadcast

## Alternatives Considered

### Client-Side Throttling of WebSocket Messages

Queue incoming `pick_made` messages on the frontend and process them one per second, keeping the backend loop fast.

**Pros**: Fast HTTP response, pacing logic is client-controlled

**Cons**: Adds complexity to the frontend WebSocket handler (queue management, timers, cleanup on disconnect). The "fast burst then slow replay" pattern feels artificial and introduces a growing delay between server state and displayed state.

**Rejected**: Moving pacing to the server is simpler and keeps the WebSocket message stream inherently paced. The client can trust that messages arrive at a consumable rate.

### Configurable Server Delay via Session Settings

Add a `pick_delay_ms` field to `DraftSession` and expose it in the create-session API, allowing per-session control over pacing.

**Pros**: Flexibility for different UX modes (fast simulation vs. broadcast-style viewing)

**Cons**: Premature configurability. The 1-second interval works well for the current use case and there's no demand for variation.

**Rejected for now**: Can be added later without architectural change — it's just a parameter swap from `Duration::from_secs(1)` to `Duration::from_millis(session.pick_delay_ms)`.

### Keep Suppression Guards, Add Batch Summary Toast

Keep the frontend suppression and instead show a single summary toast after the HTTP response ("AI made 12 picks").

**Pros**: Fast response, no toast spam

**Cons**: Users still see the draft "jump" with no intermediate feedback. The draft board would snap forward. This fails the core UX goal of showing real-time progression.

**Rejected**: The whole point is real-time visibility into each pick.

## References

- [ADR-0005: WebSocket for Real-Time Updates](./0005-websocket-real-time-updates.md) — WebSocket architecture
- [ADR-0009: Per-Session Mutex for Concurrency Control](./0009-per-session-mutex-concurrency-control.md) — `try_lock()` semantics for auto-pick-run
