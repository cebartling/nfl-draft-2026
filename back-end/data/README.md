# NFL Draft Player Data

## Overview

This directory contains manually curated NFL Draft prospect data used to seed the database.

## Files

- `players_2026.json` - Top 150 prospects for the 2026 NFL Draft class

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

### After NFL Combine (Late February/March)

1. Update `players_2026.json` with official measurements
2. Clear existing data: `cargo run -p seed-data clear --year 2026`
3. Reload: `cargo run -p seed-data load`

### Adding More Prospects

Edit `players_2026.json` to add entries and update the `meta.total_players` count. Run validation before loading:

```bash
cargo run -p seed-data validate
```
