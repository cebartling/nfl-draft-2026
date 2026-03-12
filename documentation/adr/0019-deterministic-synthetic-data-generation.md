# ADR 0019: Deterministic Synthetic Data Generation

## Status

Accepted

## Context

The application requires scouting reports for every player-team combination (each team scouts each prospect independently). With ~300 prospects and 32 teams, this means ~9,600 scouting reports. Real scouting data is proprietary and unavailable, so we generate synthetic reports during the seed process.

These synthetic reports need to be **deterministic** — given the same player and team, the generated grades, fit scores, and concern flags must be identical across runs. This ensures:

- Tests that assert on specific scouting data remain stable
- Re-seeding a database produces identical state
- Developers see consistent data across environments

Rust's standard library `DefaultHasher` explicitly does **not** guarantee determinism across compiler versions or platforms. Using it for synthetic data generation would produce different scouting reports depending on the developer's Rust toolchain.

### Forces

- **Reproducibility**: Same inputs must produce same outputs across machines and toolchain versions
- **Realism**: Grades should vary realistically by team (different teams value different players)
- **Simplicity**: The generation logic should be straightforward, not require ML models or external services
- **Performance**: Generating 9,600+ reports during seeding should complete in seconds

## Decision

We use **FNV-1a hashing** as a deterministic pseudo-random number generator for synthetic scouting data, implemented in `back-end/crates/seed-data/src/grade_generator.rs`.

### Implementation

#### Hash function

FNV-1a is a non-cryptographic hash with a published specification that guarantees identical output for identical input on any platform:

```rust
fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000000001b3); // FNV prime
    }
    hash
}
```

#### Grade derivation

- **Base grade**: Derived from prospect rank using a linear formula: `9.5 - (rank - 1) * 0.03`, floored at 3.0. This ensures top prospects consistently grade highest
- **Team-specific offset**: FNV-1a hash of `"{team_abbreviation}-{player_id}"` produces a deterministic offset in the range `[-0.8, +0.8]`, simulating team-specific evaluations
- **Fit grade**: Hash-derived probability: 70% chance of "B" (consensus fit), 15% "A" (bump up), 15% "C" (bump down)
- **Concern flags**: ~5% chance each for injury and character concerns, derived from hash

#### Determinism scope

The hash input is a string combining team abbreviation and player UUID. Player UUIDs are generated via `uuid::Uuid::new_v4()` (system RNG), so they are **not** deterministic across separate seed operations. However, within a single seed run, each player is assigned a UUID once, and all scouting reports for that player use the same UUID — making the FNV-1a output deterministic within that run.

**Important limitation**: Re-seeding the database (clearing and reloading) assigns new UUIDs to players, which produces different scouting report grades. Determinism holds within a seed run but not across database resets.

## Consequences

### Positive

- **Cross-platform reproducibility**: FNV-1a produces identical output on macOS, Linux, and in CI regardless of Rust toolchain version
- **Realistic variation**: Team-specific offsets create believable scouting disagreements (e.g., one team grades a player 8.2 while another grades 7.5)
- **Fast**: FNV-1a is extremely fast — generating 9,600 reports takes milliseconds
- **No external dependencies**: The hash function is implemented in ~10 lines with no crate dependency

### Negative

- **Not truly realistic**: Linear rank-to-grade mapping doesn't capture real scouting dynamics (e.g., positional value differences, scheme fits)
- **Determinism is per-seed-run**: If player UUIDs change (e.g., database is cleared and re-seeded), grades change too. Determinism holds within a single seed operation, not across database resets
- **Fixed distribution**: The 70/15/15 fit grade distribution and 5% concern rates are hardcoded. Changing them changes all generated data

### Neutral

- **FNV-1a is not cryptographic**: This is irrelevant since we use it for distribution, not security
- **Test coupling**: Tests that assert on specific grade values are coupled to the FNV-1a implementation. This is acceptable since the implementation is intentionally frozen

## Alternatives Considered

### Rust's `DefaultHasher`

**Pros**: No custom implementation needed

**Cons**: Rust explicitly states `DefaultHasher` output may change between compiler versions. This would break test stability and cross-environment reproducibility

**Rejected**: The non-determinism guarantee is a dealbreaker

### Seeded PRNG (e.g., `rand` crate with fixed seed)

**Pros**: Well-tested statistical properties. Easy to generate any distribution

**Cons**: Adds a dependency. The `rand` crate's algorithms may change between major versions, reintroducing the reproducibility problem. Overkill for simple grade generation

**Rejected**: FNV-1a is simpler and sufficient for our use case

### External data file with pre-generated reports

**Pros**: Complete control over the data. Can be hand-tuned for realism

**Cons**: 9,600+ entries is impractical to maintain manually. Adding a new player or team requires regenerating the entire file

**Rejected**: Doesn't scale with the number of players and teams

## References

- `back-end/crates/seed-data/src/grade_generator.rs` — FNV-1a implementation and grade derivation
- [FNV-1a specification](http://www.isthe.com/chongo/tech/comp/fnv/) — hash algorithm reference
- [Rust DefaultHasher stability note](https://doc.rust-lang.org/std/collections/hash_map/struct.DefaultHasher.html) — documentation of non-guarantee
