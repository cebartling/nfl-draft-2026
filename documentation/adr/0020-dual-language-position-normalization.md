# ADR 0020: Dual-Language Position Normalization

## Status

Accepted

## Context

NFL position abbreviations are inconsistent across data sources. The same position may appear as "EDGE", "DE", "OLB", or "EDGE/LB" depending on the source. The application needs to map these variants to a canonical set of positions defined in the domain model's `Position` enum.

This mapping is needed in two places:

1. **TypeScript scrapers** (`scrapers/src/shared/position-normalizer.ts`) — normalize positions when scraping external sources, before writing JSON data files
2. **Rust seed-data loader** (`back-end/crates/seed-data/src/position_mapper.rs`) — normalize positions when loading JSON data into the database

Both implementations must agree on the canonical mappings, but they operate in different languages on different sides of the pipeline.

### Forces

- **Consistency**: A position normalized to "DE" in the scraper must map to `Position::DE` in the Rust loader
- **Language boundary**: The scraper is TypeScript/Bun; the loader is Rust. There is no shared runtime
- **Maintainability**: Adding a new position mapping must be done in two places
- **Pragmatism**: The mapping table is small (~25 entries) and changes infrequently (NFL positions are stable)

## Decision

We maintain **two independent but aligned implementations** of position normalization — one in TypeScript and one in Rust — with no shared source of truth beyond the implicit contract that they produce the same canonical output.

### TypeScript (scrapers)

```typescript
// scrapers/src/shared/position-normalizer.ts
export function normalizePosition(pos: string): string {
  const upper = pos.trim().toUpperCase();
  const map: Record<string, string> = {
    EDGE: "DE", OLB: "LB", ILB: "LB", NT: "DT",
    HB: "RB", FB: "RB", T: "OT", G: "OG", /* ... */
  };
  return map[upper] ?? upper;
}
```

### Rust (seed-data)

```rust
// back-end/crates/seed-data/src/position_mapper.rs
pub fn map_position(source: &str) -> Result<Position> {
    match source.trim().to_uppercase().as_str() {
        "EDGE" => Ok(Position::DE),
        "OLB" | "ILB" | "MLB" => Ok(Position::LB),
        "NT" | "DL" => Ok(Position::DT),
        "HB" | "FB" => Ok(Position::RB),
        /* ... */
        _ => Err(anyhow!("Invalid position: '{}'", source)),
    }
}
```

### Alignment strategy

- Both implementations handle the same set of alternate abbreviations
- The Rust mapper is stricter: it returns an error for unknown positions, while the TypeScript normalizer passes through unrecognized values
- Test coverage in both languages verifies the common mappings
- Ambiguous positions (e.g., "DL" → DT, "EDGE/LB" → DE) have documented rationale in the Rust mapper's comments

## Consequences

### Positive

- **No cross-language build dependency**: Each side compiles and tests independently. No code generation step or shared schema file
- **Language-idiomatic**: The Rust mapper returns `Result<Position>` (fail on unknown positions). The TypeScript normalizer passes through unknown values (fail later during validation). Each follows its language's conventions
- **Small surface area**: ~25 mappings that change very infrequently. The duplication cost is low in practice
- **Independent testing**: Each implementation has its own test suite verifying the mappings

### Negative

- **Violation of DRY**: The same mapping table exists in two places. If a new position abbreviation appears in a data source, both files must be updated
- **Silent divergence risk**: If one implementation adds a mapping and the other doesn't, data may flow through the pipeline with inconsistent positions. There is no automated check for alignment
- **Ambiguous position decisions are duplicated**: The rationale for "DL" → DT (not DE) or "EDGE/LB" → DE (not LB) is documented in the Rust code but not in the TypeScript code

### Neutral

- **Strictness asymmetry**: The TypeScript normalizer is lenient (pass-through) while the Rust mapper is strict (error). This is intentional — the scraper captures what it finds, and the loader validates at the boundary. If a new abbreviation appears, it flows into the JSON file and is caught by the Rust mapper during seeding

## Alternatives Considered

### Shared JSON mapping file consumed by both languages

**Pros**: Single source of truth. Both implementations read from the same file

**Cons**: Adds a build step or runtime file read to both sides. The Rust side would need to load and parse the JSON at startup or compile time (`include_str!`). The TypeScript side would need to import a JSON file outside its source tree. Both add complexity for a ~25-entry lookup table

**Rejected**: The coordination cost exceeds the duplication cost for a table this small and stable

### Code generation from a schema (e.g., protobuf or JSON Schema)

**Pros**: Generates both TypeScript and Rust code from a single definition

**Cons**: Introduces a code generation step to the build pipeline. Requires a schema language that can express "map these N strings to this canonical string." Overkill for a simple lookup table

**Rejected**: The infrastructure cost is disproportionate to the problem

### Normalize only in Rust (remove TypeScript normalizer)

**Pros**: Single implementation. Scrapers store raw position strings; Rust handles all normalization

**Cons**: The JSON data files would contain inconsistent position abbreviations ("EDGE", "DE", "OLB" all appearing). This makes the data files harder to read and validate manually. It also pushes more work to the Rust loader

**Rejected**: Normalizing at the scraper level produces cleaner intermediate data files that are useful for manual inspection

## References

- `scrapers/src/shared/position-normalizer.ts` — TypeScript implementation
- `back-end/crates/seed-data/src/position_mapper.rs` — Rust implementation with documented rationale
- `back-end/crates/domain/src/models/player.rs` — canonical `Position` enum definition
