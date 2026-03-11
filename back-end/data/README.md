# NFL Draft Data Files

## Overview

This directory contains NFL Draft data used to seed the database. Files are either manually curated or produced by scraper crates. See [documentation/data-pipeline.md](../../documentation/data-pipeline.md) for the full data pipeline documentation.

## Files

| File | Source | Description |
|------|--------|-------------|
| `players_2026.json` | Manual / scraped | Top 150+ prospects for the 2026 NFL Draft class |
| `combine_2026.json` | `bun run scrape combine --merge` | Real scraped combine data (used by seed handler via `include_str!`) |
| `combine_2026_pfr.json` | `combine-data-scraper scrape --source pfr` | PFR-only combine data (merge input) |
| `combine_2026_mockdraftable.json` | `combine-data-scraper scrape --source mockdraftable` | Mockdraftable-only combine data (merge input) |
| `combine_percentiles.json` | `combine-data-scraper template` | Combine percentile baselines from NFL averages |
| `draft_order_2026.json` | Scraped | Draft pick order for 2026 |
| `teams_nfl.json` | Manual | All 32 NFL teams |
| `team_needs_2026.json` | Manual | Team positional needs |
| `team_seasons_2025.json` | Manual | 2025 season records |
| `rankings/` | `prospect-rankings-scraper` | Prospect big board rankings by source |

## Data Sources

Player data was compiled from the following publicly available sources:

- **NFL.com** - Daniel Jeremiah's Top 50 prospect rankings
- **ESPN** - Consensus rankings and Mel Kiper Jr.'s Big Board
- **Tankathon** - Big Board rankings
- **CBS Sports** - Mike Renner's Top 150 Big Board

## Data Format

Each player entry contains:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `first_name` | string | Yes | Player's first name |
| `last_name` | string | Yes | Player's last name |
| `position` | string | Yes | Position abbreviation (may use source abbreviations like EDGE) |
| `college` | string | No | College/university name |
| `height_inches` | integer | No | Height in inches (60-90 range) |
| `weight_pounds` | integer | No | Weight in pounds (150-400 range) |
| `notes` | string | No | Editorial notes (not loaded to database) |

### Position Abbreviations

Source data may use various position abbreviations. The seed tool normalizes them:

- `EDGE` -> `DE` (Defensive End)
- `HB` -> `RB` (Running Back)
- `T` -> `OT` (Offensive Tackle)
- `G` -> `OG` (Offensive Guard)
- `NT` -> `DT` (Defensive Tackle)
- `OLB`, `ILB`, `MLB` -> `LB` (Linebacker)
- `SS`, `FS` -> `S` (Safety)

## Updating Data

### Scraping Real Combine Results

```bash
# From the repository root — scrapes PFR + Mockdraftable and merges
./scripts/scrape-combine-results.sh
```

### After NFL Combine (Late February/March)

1. Scrape real combine data (see above)
2. Update `players_2026.json` with official measurements if needed
3. Rebuild the API: `cargo build -p api` (or `docker compose up --build`)
4. Re-seed: `curl -X POST http://localhost:8000/api/v1/admin/seed-combine-data`

### Adding More Prospects

Edit `players_2026.json` to add entries and update the `meta.total_players` count. Run validation before loading:

```bash
cargo run -p seed-data validate
```
