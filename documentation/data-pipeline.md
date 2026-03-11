# Data Pipeline: Scraping, Seeding, and Loading

This document describes how external data flows from web sources into the NFL Draft Simulator database.

## Overview

```
Web Sources (PFR, Mockdraftable, Tankathon, WalterFootball, DraftTek)
        │
        ▼
┌─────────────────────────┐
│  TypeScript/Bun Scrapers │   cd scrapers && bun run scrape <command> [options]
│  (Cheerio + Playwright)  │
└────────┬────────────────┘
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

## Scraper Technology Stack

All scrapers live in the `scrapers/` directory and share a unified CLI:

- **Runtime**: [Bun](https://bun.sh/) (JavaScript/TypeScript runtime)
- **Language**: TypeScript (strict mode)
- **HTML Parsing**: [Cheerio](https://cheerio.js.org/) (for static HTML)
- **Browser Automation**: Playwright (for JavaScript-heavy sites like Tankathon)
- **Schema Validation**: [Zod](https://zod.dev/) (runtime type checking for all output)
- **Testing**: [Vitest](https://vitest.dev/)
- **Formatting**: Prettier

### CLI Entry Point

```bash
cd scrapers
bun run scrape <command> [options]

# Commands:
#   draft-order    Scrape NFL draft order from Tankathon
#   rankings       Scrape prospect rankings
#   combine        Scrape NFL Combine data

# Common options:
#   --year <year>                Draft year (default: 2026)
#   --output <path>              Output file path
#   --template                   Generate template without scraping
#   --source <name>              Source to scrape
#   --merge                      Merge data from all sources
#   --allow-template-fallback    Fall back to template if scraping fails
```

## Key Concept: Compile-Time Embedding

The API server uses `include_str!()` to embed JSON data files at compile time (see `api/src/handlers/seed.rs`). This means:

1. **Scrape** produces JSON files in `back-end/data/`
2. **Rebuilding** the API binary (`cargo build -p api`) embeds the updated files
3. **Seeding** via admin endpoints loads embedded data into PostgreSQL

**After scraping new data, you must rebuild the API server** (or rebuild the Docker container) before seeding will reflect the new data.

## Draft Order Pipeline

### Step 1: Scrape

The draft-order scraper fetches the full NFL draft order from Tankathon.

```bash
cd scrapers

# Scrape draft order
bun run scrape draft-order --year 2026 --output ../back-end/data/draft_order_2026.json

# Generate template (offline, no scraping)
bun run scrape draft-order --template --output ../back-end/data/draft_order_2026.json
```

Or use the convenience script from the repository root:

```bash
./scripts/scrape-draft-order.sh              # With staleness check
./scripts/scrape-draft-order.sh --force      # Bypass staleness check
./scripts/scrape-draft-order.sh --commit     # Scrape and git commit
```

**Source:**

| Source | Method | Technology |
|--------|--------|------------|
| Tankathon | Full draft order page | Playwright (JavaScript rendering) |

**Data collected:** Round, pick number, overall pick, team, original team (for trades), compensatory flag, notes.

### Step 2: Automated Scraping (GitHub Actions)

The workflow `.github/workflows/scrape-draft-order.yml` runs daily at 06:00 UTC and on manual dispatch:

1. Sets up Bun and installs dependencies
2. Installs Playwright Chromium
3. Runs the draft-order scraper
4. Validates JSON output
5. Creates a PR on branch `auto/draft-order-update`
6. Auto-merges via squash

```bash
gh workflow run scrape-draft-order.yml
```

## Combine Data Pipeline

### Step 1: Scrape

The combine scraper fetches real NFL Combine results from Pro Football Reference (PFR) and Mockdraftable.

```bash
cd scrapers

# Scrape from a single source
bun run scrape combine --source pfr --year 2026 --output ../back-end/data/combine_2026.json

# Scrape from Mockdraftable
bun run scrape combine --source mockdraftable --year 2026 --output ../back-end/data/combine_2026.json

# Scrape both sources and merge (recommended)
bun run scrape combine --merge --year 2026 --output ../back-end/data/combine_2026.json

# Generate template without scraping
bun run scrape combine --template --output ../back-end/data/combine_2026.json
```

Or use the convenience script from the repository root:

```bash
./scripts/scrape-combine-results.sh              # Merge PFR + Mockdraftable
./scripts/scrape-combine-results.sh --source pfr  # PFR only
```

**Sources:**

| Source | Method | URL Pattern |
|--------|--------|-------------|
| Pro Football Reference | HTML table parsing (Cheerio) | `pro-football-reference.com/draft/{year}-combine.htm` |
| Mockdraftable | `window.INITIAL_STATE` JSON extraction (Playwright) | `mockdraftable.com/search?year={year}` |

**Merge behavior:** PFR is the primary source. For players found in both sources, PFR values are kept and Mockdraftable backfills any null fields. Players unique to either source are included.

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
1. Reads the embedded JSON (compiled from `data/combine_2026.json`)
2. Matches each entry to a player by **case-insensitive first + last name**
3. Skips entries where combine data already exists for that player/year/source
4. Returns counts: loaded, skipped, player_not_found

### Step 4: Automated Scraping (GitHub Actions)

The workflow `.github/workflows/scrape-combine-results.yml` runs on **manual dispatch only** (the combine is a one-time annual event):

1. Sets up Bun and installs dependencies
2. Runs `--merge` to scrape PFR + Mockdraftable and merge
3. Validates JSON output
4. Creates a PR on branch `auto/combine-results-update`
5. Auto-merges via squash

Trigger it:
```bash
gh workflow run scrape-combine-results.yml
```

## Prospect Rankings Pipeline

### Step 1: Scrape

The rankings scraper fetches prospect big board rankings from multiple sources.

```bash
cd scrapers

# Scrape from a single source
bun run scrape rankings --source tankathon --year 2026 --output ../back-end/data/rankings/tankathon_2026.json
bun run scrape rankings --source walterfootball --year 2026 --output ../back-end/data/rankings/walterfootball_2026.json
bun run scrape rankings --source drafttek --year 2026 --output ../back-end/data/rankings/drafttek_2026.json

# Merge from all sources
bun run scrape rankings --merge --year 2026 --output ../back-end/data/rankings/rankings_2026.json

# Generate template without scraping
bun run scrape rankings --template --output ../back-end/data/prospect_rankings_2026.json
```

Or use the convenience script from the repository root:

```bash
./scripts/scrape-prospect-rankings.sh              # Scrape all sources + merge
./scripts/scrape-prospect-rankings.sh --force       # Bypass staleness check
./scripts/scrape-prospect-rankings.sh --commit      # Scrape and git commit
```

**Sources:**

| Source | Method | Technology |
|--------|--------|------------|
| Tankathon | Big board page | Playwright (JavaScript rendering) |
| WalterFootball | Big board page | Cheerio (static HTML) |
| DraftTek | Big board page | Cheerio (static HTML) |

**Data collected:** Rank, first name, last name, position, school, height (optional), weight (optional).

**Merge behavior:** Tankathon is the primary source. The merge combines rankings from all available sources to produce a consensus ranking.

### Step 2: Automated Scraping (GitHub Actions)

The workflow `.github/workflows/scrape-prospect-rankings.yml` runs daily at 07:00 UTC and on manual dispatch:

1. Sets up Bun and installs dependencies
2. Installs Playwright Chromium
3. Scrapes Tankathon and WalterFootball (each with `continue-on-error`)
4. Merges results into consensus rankings
5. Validates JSON output
6. Creates a PR on branch `auto/prospect-rankings-update`
7. Auto-merges via squash

```bash
gh workflow run scrape-prospect-rankings.yml
```

## Position Normalization

All scrapers normalize source-specific position abbreviations to canonical values matching the database `Position` enum. The mapping is maintained in the TypeScript scrapers at `scrapers/src/shared/position-normalizer.ts` and must stay in sync with `seed-data/src/position_mapper.rs`:

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

This mapping is maintained in two places that must stay in sync:
1. `scrapers/src/shared/position-normalizer.ts` — TypeScript scrapers
2. `back-end/crates/seed-data/src/position_mapper.rs` — Rust seed-data loader (authoritative for DB loading)

## File Inventory

### Scraper Output Files

| File | Producer | Consumer |
|------|----------|----------|
| `data/draft_order_2026.json` | `bun run scrape draft-order` | seed.rs via `include_str!` |
| `data/combine_2026.json` | `bun run scrape combine --merge` | seed.rs via `include_str!` |
| `data/combine_percentiles.json` | Template generation | seed.rs via `include_str!` |
| `data/rankings/rankings_2026.json` | `bun run scrape rankings --merge` | seed.rs via `include_str!` |
| `data/rankings/tankathon_2026.json` | `bun run scrape rankings --source tankathon` | merge input |
| `data/rankings/walterfootball_2026.json` | `bun run scrape rankings --source walterfootball` | merge input |

### Scraper Project Structure

```
scrapers/
├── src/
│   ├── cli.ts                    # CLI entry point and command routing
│   ├── commands/                  # Command handlers
│   │   ├── draft-order.ts
│   │   ├── rankings.ts
│   │   └── combine.ts
│   ├── scrapers/                  # Scraping logic per data type
│   │   ├── draft-order/           # Tankathon draft order
│   │   ├── rankings/              # Tankathon, DraftTek, WalterFootball rankings
│   │   └── combine/               # PFR, Mockdraftable combine data
│   ├── types/                     # Zod schemas and TypeScript types
│   └── shared/                    # Position normalizer, name normalizer, team abbreviations
├── tests/                         # Vitest test suite
├── package.json
├── tsconfig.json
└── vitest.config.ts
```

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

# 3. Rebuild and restart
docker compose up --build -d

# 5. Re-seed the database
curl -X POST http://localhost:8000/api/v1/admin/seed-combine-data
```
