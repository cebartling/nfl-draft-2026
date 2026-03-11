# Data Pipeline: Scraping, Seeding, and Loading

This document describes how external data flows from web sources into the NFL Draft Simulator database.

## Overview

```
Web Sources (PFR, Mockdraftable, Tankathon, etc.)
        │
        ▼
┌─────────────────────┐
│  Scraper Crates      │   cargo run -p combine-data-scraper -- scrape ...
│  (Rust + Playwright) │   cargo run -p prospect-rankings-scraper ...
└────────┬────────────┘
         │  JSON files
         ▼
┌─────────────────────┐
│  back-end/data/      │   combine_2026.json, rankings_2026.json, etc.
│  (JSON on disk)      │
└────────┬────────────┘
         │  include_str!() at compile time
         ▼
┌─────────────────────┐
│  seed-data crate     │   combine_loader.rs, rankings_loader.rs, etc.
│  (embedded in API)   │
└────────┬────────────┘
         │  POST /api/v1/admin/seed-*
         ▼
┌─────────────────────┐
│  PostgreSQL          │   combine_results, players, scouting_reports, etc.
└─────────────────────┘
```

## Key Concept: Compile-Time Embedding

The API server uses `include_str!()` to embed JSON data files at compile time (see `api/src/handlers/seed.rs`). This means:

1. **Scrape** produces JSON files in `back-end/data/`
2. **Rebuilding** the API binary (`cargo build -p api`) embeds the updated files
3. **Seeding** via admin endpoints loads embedded data into PostgreSQL

**After scraping new data, you must rebuild the API server** (or rebuild the Docker container) before seeding will reflect the new data.

## Combine Data Pipeline

### Step 1: Scrape

The `combine-data-scraper` crate scrapes real NFL Combine results from Pro Football Reference (PFR) and Mockdraftable.

```bash
cd back-end

# Scrape from a single source
cargo run -p combine-data-scraper -- scrape --source pfr --year 2026 --output data/combine_2026.json

# Scrape both sources and merge (recommended)
cargo run -p combine-data-scraper -- scrape --merge --year 2026 --output data/combine_2026.json

# If direct HTTP is blocked (403), use Playwright browser fallback
cargo run -p combine-data-scraper -- scrape --source pfr --year 2026 --browser
```

Or use the convenience script from the repository root:

```bash
./scripts/scrape-combine-results.sh          # Merge PFR + Mockdraftable
./scripts/scrape-combine-results.sh --source pfr  # PFR only
```

**Sources:**

| Source | Method | URL Pattern |
|--------|--------|-------------|
| Pro Football Reference | HTML table parsing (`table#combine`) | `pro-football-reference.com/draft/{year}-combine.htm` |
| Mockdraftable | `window.INITIAL_STATE` JSON extraction | `mockdraftable.com/search?year={year}` |

**Merge behavior:** PFR is the primary source. For players found in both sources, PFR values are kept and Mockdraftable backfills any `None` fields. Players unique to either source are included.

### Step 2: Output Format

The scraper writes JSON matching the `CombineFileData` schema expected by `combine_loader.rs`:

```json
{
  "meta": {
    "source": "merged",
    "year": 2026,
    "description": "2026 NFL Combine results (merged from multiple sources)",
    "generated_at": "2026-03-10T...",
    "player_count": 320,
    "entry_count": 320
  },
  "combine_results": [
    {
      "first_name": "Cam",
      "last_name": "Ward",
      "position": "QB",
      "source": "combine",
      "year": 2026,
      "forty_yard_dash": 4.72,
      "bench_press": 18,
      "vertical_jump": 32.0,
      "broad_jump": 108,
      "three_cone_drill": 7.05,
      "twenty_yard_shuttle": 4.30,
      "arm_length": 32.5,
      "hand_size": 9.75,
      "wingspan": 77.5,
      "ten_yard_split": 1.65,
      "twenty_yard_split": 2.72
    }
  ]
}
```

All 11 measurable fields are optional (`null` when not recorded). The `meta` fields beyond `source` and `year` are informational — `combine_loader.rs` ignores them.

### Step 3: Load into Database

Currently, combine data is loaded via the admin seeding endpoint:

```bash
# Rebuild the API to embed updated data files
cargo build -p api

# Start the API server
cargo run -p api

# Trigger seeding (requires ADMIN_SEED_ENABLED=true in environment)
curl -X POST http://localhost:8000/api/v1/admin/seed-combine-data
```

The loader:
1. Reads the embedded JSON (compiled from `data/combine_2026_mock.json`)
2. Matches each entry to a player by **case-insensitive first + last name**
3. Skips entries where combine data already exists for that player/year/source
4. Returns counts: loaded, skipped, player_not_found

**Important:** The seed handler currently references `combine_2026_mock.json`. To use real scraped data, either:
- Replace `combine_2026_mock.json` with the scraper output, or
- Update the `include_str!` path in `seed.rs` to point to `combine_2026.json`

### Step 4: Automated Scraping (GitHub Actions)

The workflow `.github/workflows/scrape-combine-results.yml` runs on **manual dispatch only** (the combine is a one-time annual event):

1. Builds the scraper in release mode
2. Runs `--merge` to scrape PFR + Mockdraftable and merge
3. Validates JSON output
4. Creates a PR on branch `auto/combine-results-update`
5. Auto-merges via squash

Trigger it:
```bash
gh workflow run scrape-combine-results.yml
```

## Prospect Rankings Pipeline

The `prospect-rankings-scraper` crate follows the same pattern for prospect big board rankings:

```bash
cd back-end

# Scrape Tankathon
cargo run -p prospect-rankings-scraper -- --source tankathon --year 2026 --output data/rankings/tankathon_2026.json

# Scrape and merge multiple sources
cargo run -p prospect-rankings-scraper -- --merge --primary data/rankings/tankathon_2026.json --secondary data/rankings/walterfootball_2026.json --output data/rankings/rankings_2026.json
```

Automated via `.github/workflows/scrape-prospect-rankings.yml` (daily at 07:00 UTC).

## Position Normalization

All scrapers normalize source-specific position abbreviations to canonical values matching the database `Position` enum. The authoritative mapping is in `seed-data/src/position_mapper.rs`:

| Source Abbreviation | Canonical Value |
|---------------------|-----------------|
| DE, EDGE, EDGE/LB | DE |
| OLB, ILB, MLB | LB |
| DL, NT | DT |
| OG, G, IOL, OL | OG |
| T | OT |
| C | C |
| FS, SS, DB, SAF | S |
| HB, FB | RB |

This mapping is duplicated in three places that must stay in sync:
1. `seed-data/src/position_mapper.rs` — authoritative source
2. `combine-data-scraper/src/models.rs` — `normalize_position()`
3. `back-end/scripts/scrape-combine.mjs` — JavaScript `positionMap`

## File Inventory

### Scraper Output Files

| File | Producer | Consumer |
|------|----------|----------|
| `data/combine_2026.json` | combine-data-scraper (merged) | seed.rs via `include_str!` |
| `data/combine_2026_pfr.json` | combine-data-scraper (PFR only) | merge input |
| `data/combine_2026_mockdraftable.json` | combine-data-scraper (Mockdraftable only) | merge input |
| `data/combine_2026_mock.json` | combine-data-scraper `mock-data` subcommand | seed.rs via `include_str!` (current default) |
| `data/combine_percentiles.json` | combine-data-scraper `template` subcommand | seed.rs via `include_str!` |
| `data/rankings/rankings_2026.json` | prospect-rankings-scraper (merged) | seed.rs via `include_str!` |
| `data/rankings/tankathon_2026.json` | prospect-rankings-scraper | merge input |

### Scraper Crates

| Crate | Binary | Purpose |
|-------|--------|---------|
| `combine-data-scraper` | `combine-data-scraper` | Combine results (PFR + Mockdraftable) |
| `prospect-rankings-scraper` | `prospect-rankings-scraper` | Big board rankings (Tankathon + WalterFootball) |

### Loader Modules (in seed-data crate)

| Module | Data Type | Match Strategy |
|--------|-----------|----------------|
| `combine_loader.rs` | Combine results | Case-insensitive name |
| `rankings_loader.rs` | Prospect rankings | Case-insensitive name |
| `percentile_loader.rs` | Combine percentiles | Position + measurement type |
| `team_loader.rs` | NFL teams | Team abbreviation |
| `draft_order_loader.rs` | Draft pick order | Team + round + pick |

## Typical Workflow After the NFL Combine

```bash
# 1. Scrape real combine data
./scripts/scrape-combine-results.sh

# 2. Verify output
cat back-end/data/combine_2026.json | python3 -m json.tool | head -30

# 3. Replace mock data with real data for the seed handler
cp back-end/data/combine_2026.json back-end/data/combine_2026_mock.json

# 4. Rebuild and restart
docker compose up --build -d

# 5. Re-seed the database
curl -X POST http://localhost:8000/api/v1/admin/seed-combine-data
```
