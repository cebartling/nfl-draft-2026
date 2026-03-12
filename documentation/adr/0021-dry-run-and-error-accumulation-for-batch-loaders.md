# ADR 0021: Dry-Run and Error Accumulation for Batch Loaders

## Status

Accepted

## Context

The seed-data pipeline loads large datasets (players, teams, rankings, combine results, scouting reports) from JSON files into PostgreSQL. A typical seeding operation processes hundreds of entries, each of which may fail independently due to validation errors, duplicate keys, missing foreign keys, or invalid data.

Two operational problems emerged early:

1. **Blind loading**: Running a loader against a production or development database with bad data could insert partial results, leaving the database in an inconsistent state. There was no way to preview what would happen without actually doing it
2. **Fail-fast behavior**: If the loader aborted on the first error, the developer would fix that entry, re-run, hit the next error, fix it, re-run — a tedious cycle that could repeat dozens of times for a new data file

### Forces

- **Operational safety**: Developers and CI should be able to validate data without modifying the database
- **Efficiency**: All errors in a data file should be surfaced in a single run, not one at a time
- **Observability**: After loading, operators need to know exactly what happened — how many records loaded, skipped, failed, and why
- **Consistency**: The dry-run output should accurately predict the real run's behavior

## Decision

Every batch loader in the seed-data pipeline implements two patterns:

### 1. Dry-run mode

Each loader provides a `load_*_dry_run()` function that performs all validation without database writes. This includes:

- JSON parsing and deserialization
- Field validation (required fields, value ranges, format checks)
- Position mapping and normalization
- Duplicate detection within the dataset
- Source string validation

Dry-run functions return the same stats structure as the real loader, allowing operators to preview the outcome.

### 2. Error accumulation

During both dry-run and real loading, errors are collected into a `Vec<String>` rather than short-circuiting. Processing continues past errors, and a summary is printed at the end:

```rust
pub struct CombineLoadStats {
    pub loaded: usize,
    pub skipped: usize,           // already exists in DB
    pub skipped_no_data: usize,   // no measurements
    pub player_not_found: usize,  // no matching player
    pub players_discovered: usize, // auto-created players
    pub errors: Vec<String>,       // accumulated errors
}

impl CombineLoadStats {
    pub fn print_summary(&self) { /* ... */ }
}
```

Each loader follows this pattern consistently:
- `CombineLoadStats`, `RankingsLoadStats`, `FreaksLoadStats`, etc.
- Stats structs have a `print_summary()` method for formatted output
- Errors include context (player name, field name, original error message)

### Error handling rules

- **Validation errors** (bad position, invalid value): accumulated, entry skipped, processing continues
- **Infrastructure errors** (database connection lost, transaction failure): propagated immediately via `?` — these affect all remaining entries
- **Duplicate entries** (already exists): counted as `skipped`, not treated as errors

## Consequences

### Positive

- **Preview before commit**: `--dry-run` flag lets operators validate a new data file without touching the database. CI can run dry-run validation as a check step
- **All errors at once**: A data file with 5 problems surfaces all 5 in a single run. Developers fix them all, then re-run once
- **Clear accounting**: The stats summary tells operators exactly what happened. No ambiguity about whether loading "worked"
- **Consistent pattern**: All loaders follow the same structure, making it easy to add new loaders by copying the pattern

### Negative

- **Partial loads on real runs**: If 3 of 300 entries fail, the other 297 are loaded. The database contains partial data. This is usually desirable (don't throw away 297 good records because 3 are bad) but can surprise operators expecting all-or-nothing behavior
- **Error messages are strings**: Errors are accumulated as formatted strings, not structured types. This makes programmatic error handling (e.g., auto-fixing common issues) harder
- **Dry-run fidelity gap**: Dry-run cannot detect database-level conflicts (duplicate keys, foreign key violations) since it doesn't touch the database. A dry-run pass followed by a real run may still encounter errors

### Neutral

- **No transaction rollback on partial failure**: Each entry is committed individually. A transactional approach (load all or rollback) would provide stronger consistency but would lose all progress on any single failure. The current approach matches the operational reality of iterating on data quality
- **Stats structures vary per loader**: Each loader has its own stats struct with domain-specific fields (e.g., `players_discovered` for combine, `prospects_discovered` for rankings). This is more expressive than a generic struct but means each loader has its own type

## Alternatives Considered

### All-or-nothing transactional loading

**Pros**: Database is never in a partial state. Either all entries load or none do

**Cons**: A single bad entry prevents loading 299 good entries. With external data sources that have varying quality, this would make seeding extremely frustrating

**Rejected**: The data pipeline iterates on quality. Partial loading with clear error reporting is more practical than transactional guarantees

### Structured error types with auto-fix suggestions

**Pros**: Could automatically fix common issues (e.g., unknown position "EDGE/LB" → suggest "DE")

**Cons**: Significant additional complexity. The current string-based errors are readable and actionable. Auto-fix logic would need its own test coverage and could introduce subtle bugs

**Rejected**: Current error messages are sufficient. Auto-fix can be added later if error patterns become repetitive

### Fail-fast with `--continue-on-error` flag

**Pros**: Default behavior is safe (stop on first error). Opt-in to accumulation

**Cons**: The common case is wanting all errors at once. Making accumulation the opt-in behavior means most runs would need the flag. This inverts the typical usage pattern

**Rejected**: Error accumulation is the right default for a data loading pipeline

## References

- `back-end/crates/seed-data/src/combine_loader.rs` — `load_combine_data()` and `load_combine_data_dry_run()`
- `back-end/crates/seed-data/src/rankings_loader.rs` — `load_rankings()` and `load_rankings_dry_run()`
- `back-end/crates/seed-data/src/loader.rs` — player loader with same pattern
- `back-end/crates/seed-data/src/main.rs` — CLI integration with `--dry-run` flag
