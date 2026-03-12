# ADR 0016: Multi-Source Data Merge Strategy for Combine Results

## Status

Accepted

## Context

The NFL Combine data pipeline scrapes player measurements from multiple external sources (Pro Football Reference, Mockdraftable). Each source has different coverage — PFR may have official combine results while Mockdraftable has pro day measurements. No single source provides complete coverage for all prospects.

When combining data from multiple sources, we must decide how to handle conflicts — cases where two sources report different values for the same measurement on the same player. We also need to handle the common case where one source has a measurement that another source lacks entirely.

### Forces

- **Data accuracy**: Official combine measurements should take priority over pro day or third-party data
- **Coverage**: Maximizing the number of players with measurements improves the scouting experience
- **Simplicity**: The merge strategy should be easy to understand and debug
- **Determinism**: Given the same inputs, the merge should always produce the same output

## Decision

We use a **backfill-only merge strategy** where the primary source (PFR) is authoritative and secondary sources only fill in measurements that are `null` in the primary data.

### Implementation

The merge operates in `scrapers/src/scrapers/combine/merge.ts`:

1. **Primary source** (PFR) provides the base dataset — all its entries and measurements are preserved as-is
2. **Secondary sources** (Mockdraftable, etc.) are iterated in order
3. For each secondary entry, a **name-based lookup** finds the matching primary entry using normalized names (lowercase, whitespace-collapsed)
4. If a match is found, each measurement field is checked: if the primary value is `null` and the secondary value is not, the secondary value is backfilled
5. If no match is found in the primary dataset, the entire secondary entry is **appended** as a new entry (new player coverage)

```typescript
// Pseudocode for the merge logic
for (const secondary of secondarySources) {
  for (const entry of secondary.combine_results) {
    const match = findByNormalizedName(primary, entry);
    if (match) {
      // Backfill: only fill nulls, never overwrite
      for (const field of measurementFields) {
        if (match[field] === null && entry[field] !== null) {
          match[field] = entry[field];
        }
      }
    } else {
      // New player: append entirely
      primary.combine_results.push(entry);
    }
  }
}
```

## Consequences

### Positive

- **Primary source integrity**: PFR's official measurements are never overwritten by secondary sources, preserving data accuracy
- **Maximized coverage**: Players only measured at pro days (captured by Mockdraftable) still appear in the dataset
- **Simple mental model**: "Primary wins, secondaries fill gaps" is easy to explain and debug
- **Deterministic**: Source ordering is fixed (PFR first, then Mockdraftable), so output is reproducible

### Negative

- **Source ordering matters**: If PFR has an incorrect value, no secondary source can correct it. Manual intervention is required
- **No confidence weighting**: We don't track which source provided each measurement, making it harder to audit data provenance after merge
- **Potential duplicates**: Name normalization may miss matches for players with significantly different name spellings across sources, resulting in duplicate entries

### Neutral

- **Secondary source order**: When multiple secondary sources are added in the future, the first secondary to provide a value "wins" for backfill. This is acceptable since secondaries are lower-priority by definition
- **Meta field**: The merged output uses `source: "merged"` to indicate it came from the merge pipeline, losing per-entry source attribution

## Alternatives Considered

### Averaging or weighted blending

**Pros**: Could produce more accurate values by combining multiple observations

**Cons**: Averaging a 4.42s and 4.48s forty time is misleading — they likely represent different events (combine vs. pro day). Mixing contexts produces meaningless numbers

**Rejected**: Athletic measurements are point-in-time observations, not estimates to be refined

### Last-write-wins

**Pros**: Simplest to implement

**Cons**: Source ordering becomes critical and fragile. A secondary source could silently overwrite official combine data with pro day numbers

**Rejected**: Unacceptable risk of data degradation

### Per-field source tracking with conflict resolution UI

**Pros**: Maximum transparency and control

**Cons**: Significant complexity for a problem that rarely occurs in practice. Most conflicts are combine-vs-pro-day, where the answer is always "use the combine value"

**Rejected**: Over-engineered for current needs

## References

- `scrapers/src/scrapers/combine/merge.ts` — merge implementation
- `scrapers/src/commands/combine.ts` — CLI orchestration of scrape-and-merge
- `scrapers/src/shared/name-normalizer.ts` — name normalization for matching
