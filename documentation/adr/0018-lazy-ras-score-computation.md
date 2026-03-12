# ADR 0018: Lazy RAS Score Computation

## Status

Accepted

## Context

The Relative Athletic Score (RAS) grades each prospect's combine performance against position-specific historical percentiles. A prospect's RAS is composed of sub-scores (Size, Speed, Strength, Explosion, Agility) computed by grouping individual measurements and looking up their percentile rank within the player's position group.

We needed to decide when and where to compute these scores:

1. **At seed time** — precompute and store RAS scores in the database when combine data is loaded
2. **On demand** — compute RAS scores at query time from raw combine results and percentile baselines

### Forces

- **Data freshness**: Percentile baselines may be updated independently of combine results. Precomputed scores would become stale
- **Query performance**: Computing scores on every request adds latency compared to reading a precomputed value
- **Storage simplicity**: Avoiding a separate RAS scores table reduces schema complexity and migration burden
- **Minimum data threshold**: RAS requires at least 6 measurements to produce a meaningful overall score. Many prospects have fewer, so a precomputed table would have many null entries

## Decision

We compute RAS scores **lazily at query time** rather than materializing them in the database.

### Implementation

The computation lives in `back-end/crates/domain/src/services/ras_scoring.rs`:

1. **Per-request**: The API handler fetches a player's combine results and the position's percentile baselines, then calls the RAS scoring service
2. **Bulk endpoint**: A `GET /api/v1/combine-results/ras` endpoint computes scores for all players with combine data in a single request, fetching all combine results, percentiles, and players in three queries to avoid N+1
3. **Minimum threshold**: If a player has fewer than 6 measurements with matching percentile baselines, the overall score is `null` and an explanation string describes what's missing
4. **Sub-scores**: Each sub-score (Size, Speed, etc.) is computed independently and can be `null` if none of its constituent measurements are available

```rust
// Scoring is a pure function over data — no database writes
pub fn calculate_ras_with_percentiles(
    player: &Player,
    combine_results: &CombineResults,
    percentiles: &[CombinePercentile],
) -> RasScore { ... }
```

Frontend formatting utilities are extracted as pure functions for testability.

## Consequences

### Positive

- **Always fresh**: Scores automatically reflect updated percentile baselines without re-seeding or migration
- **No schema coupling**: Adding new measurements or changing the scoring formula requires no database migration — just a code change
- **Simpler data pipeline**: The combine loader doesn't need to know about RAS scoring. Responsibilities are cleanly separated
- **Graceful degradation**: Players with partial data get partial sub-scores rather than a binary "has score / doesn't have score"

### Negative

- **Query-time cost**: Computing scores on every request adds CPU overhead. Mitigated by the bulk endpoint for list views, but still more expensive than a table lookup
- **No historical tracking**: We can't answer "what was this player's RAS score before we updated the percentile baselines?" since scores are not persisted
- **Repeated computation**: The same player's score may be recomputed many times across requests. No caching layer exists currently

### Neutral

- **Bulk endpoint as optimization**: The bulk endpoint amortizes the database queries but still computes scores in-process. If performance becomes an issue, a caching layer (Redis or in-memory with TTL) could be added without changing the computation logic
- **Frontend handles formatting**: Score display formatting (color coding, grade labels) is a frontend concern, not part of the API response

## Alternatives Considered

### Precompute at seed time and store in a `ras_scores` table

**Pros**: O(1) query-time lookup. Enables historical tracking and database-level aggregations

**Cons**: Scores become stale when percentile baselines are updated. Requires a re-computation step and migration for schema changes. Adds a dependency between the combine loader and the scoring algorithm

**Rejected**: The tight coupling between seeding and scoring would make the data pipeline more fragile. The current dataset size (~300 players) does not justify the optimization

### Cache scores in Redis with TTL

**Pros**: Eliminates repeated computation without schema coupling

**Cons**: Introduces a new infrastructure dependency (Redis). Current query-time cost is negligible for the dataset size

**Rejected**: Premature optimization. The bulk endpoint handles the main performance concern (list views). Can be added later if needed

### Materialized view in PostgreSQL

**Pros**: Database handles caching and refresh. No application-level cache management

**Cons**: Materialized views require explicit refresh. The RAS computation involves application logic (sub-score grouping, minimum thresholds) that is awkward to express in SQL

**Rejected**: The computation is better expressed in Rust than in SQL, and the refresh timing problem is equivalent to the stale-data problem of precomputation

## References

- `back-end/crates/domain/src/services/ras_scoring.rs` — scoring computation
- `back-end/crates/domain/src/models/ras_score.rs` — score model with sub-scores
- `back-end/crates/api/src/handlers/ras.rs` — RAS endpoint handlers (single and bulk)
- `back-end/crates/seed-data/src/percentile_loader.rs` — percentile baseline loading
