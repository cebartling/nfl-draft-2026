# ADR 0007: Event Sourcing for Draft Events

## Status

Accepted

## Context

The NFL Draft Simulator needs to track all events that occur during a draft session (picks made, trades executed, time elapsed, etc.). This data is used for:

- Real-time updates to all connected clients
- Draft history and replay functionality
- Analytics and reporting
- Audit trail for debugging
- Potential future ML/AI training data

We needed to decide how to model and store this event data.

Options considered:

1. **Event Sourcing**: Store all events in order, derive current state from events
2. **Current State Only**: Store only the current state (e.g., current pick, picks made)
3. **Hybrid**: Store both current state and event history
4. **Change Data Capture**: Use database triggers to capture changes

## Decision

We will use a **Hybrid Event Sourcing** approach:

- Store all draft events in an `draft_events` table with JSONB event data
- Maintain denormalized current state in core tables (`drafts`, `draft_picks`, etc.)
- Events are the source of truth for history and replay
- Current state tables provide efficient querying

## Architecture

### Draft Events Table

```sql
CREATE TABLE draft_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES draft_sessions(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sequence_number INTEGER NOT NULL
);

CREATE INDEX idx_draft_events_session ON draft_events(session_id, sequence_number);
```

### Event Types

```typescript
type DraftEvent =
  | { type: "DraftStarted"; data: { session_id: string; started_at: string } }
  | { type: "PickMade"; data: { pick: DraftPick; timestamp: string } }
  | { type: "TradeExecuted"; data: { trade: Trade; timestamp: string } }
  | { type: "TimeExpired"; data: { pick_number: number; timestamp: string } }
  | { type: "DraftPaused"; data: { timestamp: string } }
  | { type: "DraftResumed"; data: { timestamp: string } }
  | { type: "DraftCompleted"; data: { timestamp: string } };
```

### Current State Tables

Core tables maintain denormalized current state for efficient querying:

- `drafts`: Current draft configuration and status
- `draft_picks`: All picks made (denormalized from PickMade events)
- `trades`: All trades executed (denormalized from TradeExecuted events)
- `draft_sessions`: Current session state (current pick, time remaining, etc.)

## Consequences

### Positive

- **Complete Audit Trail**: Every event is recorded with timestamp and sequence
- **Replay Capability**: Can reconstruct draft state at any point in time
- **Time Travel Debugging**: Can replay events to debug issues
- **Analytics**: Rich event history enables analysis (e.g., average time per pick, trade patterns)
- **Future ML/AI**: Event data can train AI draft engines
- **Immutable History**: Events are never updated, only appended
- **Flexible Schema**: JSONB allows event schema evolution without migrations
- **Efficient Queries**: Current state tables optimized for common queries

### Negative

- **Storage Overhead**: Storing both events and current state uses more disk space
- **Complexity**: Need to keep events and current state in sync
- **Eventual Consistency**: Event processing could lag behind in high-load scenarios
- **JSONB Queries**: JSONB queries are less efficient than relational queries
- **Schema Evolution**: Changing event schema requires handling old and new formats
- **Replay Logic**: Rebuilding state from events requires complex logic

### Neutral

- **Not Pure Event Sourcing**: Current state tables violate pure event sourcing (CQRS pattern)
- **Performance Trade-off**: Faster reads (current state), slower writes (dual storage)

## Implementation Details

### Event Creation

When a draft action occurs:

1. Validate the action
2. Create event record
3. Store event in `draft_events` table
4. Update current state tables in the same transaction
5. Broadcast event to WebSocket clients

```rust
pub async fn make_pick(
    &self,
    session_id: Uuid,
    pick: DraftPick,
) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    // 1. Get next sequence number
    let sequence = self.get_next_sequence(&mut tx, session_id).await?;

    // 2. Create event
    let event = DraftEvent::PickMade {
        pick: pick.clone(),
        timestamp: Utc::now(),
    };

    // 3. Store event
    sqlx::query!(
        "INSERT INTO draft_events (session_id, event_type, event_data, sequence_number)
         VALUES ($1, $2, $3, $4)",
        session_id,
        "PickMade",
        serde_json::to_value(&event)?,
        sequence
    )
    .execute(&mut *tx)
    .await?;

    // 4. Update current state
    sqlx::query!(
        "INSERT INTO draft_picks (session_id, player_id, team_id, pick_number)
         VALUES ($1, $2, $3, $4)",
        session_id,
        pick.player_id,
        pick.team_id,
        pick.pick_number
    )
    .execute(&mut *tx)
    .await?;

    // 5. Update session state
    sqlx::query!(
        "UPDATE draft_sessions SET current_pick = current_pick + 1 WHERE id = $1",
        session_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // 6. Broadcast to WebSocket clients
    self.ws_manager.broadcast(session_id, event).await;

    Ok(())
}
```

### Event Replay

Reconstruct draft state from events:

```rust
pub async fn replay_events(&self, session_id: Uuid) -> Result<DraftState> {
    let events = sqlx::query_as!(
        DraftEventDb,
        "SELECT * FROM draft_events
         WHERE session_id = $1
         ORDER BY sequence_number ASC",
        session_id
    )
    .fetch_all(&self.pool)
    .await?;

    let mut state = DraftState::new(session_id);

    for event_db in events {
        let event: DraftEvent = serde_json::from_value(event_db.event_data)?;
        state.apply_event(event);
    }

    Ok(state)
}
```

### Event Schema Evolution

Handle multiple event versions:

```rust
#[derive(Deserialize)]
#[serde(tag = "version")]
enum PickMadeEvent {
    V1 {
        pick: DraftPickV1,
        timestamp: DateTime<Utc>,
    },
    V2 {
        pick: DraftPickV2,
        timestamp: DateTime<Utc>,
        metadata: PickMetadata,
    },
}
```

## Event Consistency

### Transaction Boundaries

Events and current state updates happen in the same database transaction:

- **Success**: Both event and state are committed atomically
- **Failure**: Both event and state are rolled back
- **Guarantee**: Events and current state never get out of sync

### Sequence Numbers

Each event has a monotonically increasing sequence number per session:

- Ensures event order
- Enables gap detection (missing events)
- Supports idempotent replay

```sql
-- Get next sequence number
SELECT COALESCE(MAX(sequence_number), 0) + 1
FROM draft_events
WHERE session_id = $1
```

## Query Patterns

### Current State (Fast)

For real-time display, query current state tables:

```sql
-- Get current draft status
SELECT * FROM draft_sessions WHERE id = $1;

-- Get all picks made
SELECT * FROM draft_picks WHERE session_id = $1 ORDER BY pick_number;
```

### Event History (Analytics)

For analytics and reporting, query events:

```sql
-- Get all events for a session
SELECT event_type, event_data, created_at
FROM draft_events
WHERE session_id = $1
ORDER BY sequence_number;

-- Get average time between picks
SELECT AVG(
  EXTRACT(EPOCH FROM (next_event.created_at - prev_event.created_at))
)
FROM draft_events prev_event
JOIN draft_events next_event ON next_event.sequence_number = prev_event.sequence_number + 1
WHERE prev_event.session_id = $1
  AND prev_event.event_type = 'PickMade'
  AND next_event.event_type = 'PickMade';
```

## Future Enhancements

### Event Snapshots

For long-running drafts, create periodic snapshots to speed up replay:

```sql
CREATE TABLE draft_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    sequence_number INTEGER NOT NULL,
    state_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Replay process:

1. Load most recent snapshot before target sequence
2. Replay events from snapshot to target sequence
3. Much faster than replaying all events from the beginning

### Event Streaming

Stream events to external systems (analytics, webhooks):

- Kafka/Pulsar for event streaming
- Debezium for change data capture
- External analytics pipeline

### Projections

Create read-optimized views from events:

- Player statistics (times drafted, average pick position)
- Team tendencies (position preferences, trade frequency)
- Draft patterns (runs on positions, value picks)

## Alternatives Considered

### Pure Event Sourcing (CQRS)

**Pros**: Single source of truth (events), guaranteed consistency, full audit trail
**Cons**: All queries must replay events (slow), complex projections, harder to query
**Rejected**: Read performance critical for real-time draft room

### Current State Only

**Pros**: Simple, fast queries, easy to understand
**Cons**: No history, no replay, no audit trail, can't recover from bugs
**Rejected**: History and audit trail are valuable for this application

### Change Data Capture (CDC)

**Pros**: Automatic event capture, no code changes, works with any update
**Cons**: Database-specific, complex setup, hard to guarantee ordering
**Rejected**: Want explicit control over events and event schema

## References

- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
- [CQRS Pattern](https://martinfowler.com/bliki/CQRS.html)
- [PostgreSQL JSONB](https://www.postgresql.org/docs/current/datatype-json.html)
