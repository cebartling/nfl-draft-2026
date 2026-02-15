# ADR 0013: Client-Side Consensus Ranking Computation

## Status

Accepted

## Context

The NFL Draft 2026 application stores prospect rankings from multiple big board sources (Tankathon, Walter Football, ESPN, etc.) in the database. Each source provides an independent ranking for a subset of prospects. Users need a unified "consensus ranking" view that aggregates these rankings to show where prospects stand across all sources.

There were two viable approaches for computing consensus rankings:

1. **Server-side computation**: Add a new API endpoint that computes and returns consensus rankings, potentially with a materialized view or computed column in the database.
2. **Client-side computation**: Fetch the raw per-source rankings via existing APIs and compute the consensus on the frontend.

## Decision

We compute consensus rankings entirely on the client side using pure utility functions that operate on the existing `Map<string, RankingBadge[]>` data structure already fetched by the rankings API.

### Consensus Algorithm

- **Average rank**: For each player, compute the arithmetic mean of their rank across all sources that include them.
- **Sort order**: Primary sort by consensus rank ascending (lower = better). Tie-break by source count descending (more sources = higher confidence). Final tie-break by player ID for deterministic ordering.
- **No weighting**: All sources are treated equally. Source weighting can be added later without changing the API contract.

### Implementation

- `computeConsensusRankings(Map<string, RankingBadge[]>)` → `Map<string, ProspectRanking>` — computes average rank and source count per player.
- `sortByConsensusRank(Map<string, ProspectRanking>)` → `string[]` — returns player IDs in consensus rank order.

Both functions are pure, stateless, and independently testable.

## Consequences

### Positive

- **No backend changes required**: The existing `/rankings` and `/ranking-sources` endpoints provide all necessary data. Zero migration, zero new endpoints.
- **Instant iteration**: Changes to the ranking algorithm (weighted averages, source filtering, etc.) require only frontend changes and a redeploy — no database migration or API version bump.
- **Testable in isolation**: Pure functions with simple inputs/outputs make unit testing straightforward. No database fixtures or API mocking needed.
- **Data reuse**: The same `loadAllPlayerRankings()` call already used by the `/players` page is reused by the `/prospects` page, avoiding duplicate API calls when the data is cached.

### Negative

- **Computation on every page load**: The consensus ranking is recalculated each time the user visits the prospects page. With ~200 players and ~5 sources, this is negligible (sub-millisecond), but would need revisiting if the dataset grows to thousands of players.
- **No server-side caching**: If multiple clients need the same consensus ranking, each computes it independently. This is acceptable at the current scale.
- **Algorithm divergence risk**: If the backend ever needs consensus rankings (e.g., for AI draft logic), the algorithm would need to be replicated server-side. This is mitigated by the algorithm's simplicity (arithmetic mean).

## Alternatives Considered

### Server-side computed endpoint

A `/prospects/consensus` endpoint that returns pre-computed rankings. Rejected because:
- Adds API surface area for a purely presentational concern
- Requires deciding the algorithm server-side, reducing frontend flexibility
- The dataset is small enough that client-side computation is instant

### Database materialized view

A `consensus_rankings` view that joins ranking tables and computes averages. Rejected because:
- Adds schema complexity for a read-only UI concern
- Requires cache invalidation when rankings are updated
- Overkill for ~200 players × ~5 sources
