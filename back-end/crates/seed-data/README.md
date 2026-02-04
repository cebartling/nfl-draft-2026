# seed-data

CLI tool for seeding NFL Draft player data into the database.

## Prerequisites

- PostgreSQL running (via `docker compose up -d postgres` from repo root)
- Migrations applied (`sqlx migrate run` from `back-end/`)
- `DATABASE_URL` set in `.env` file

## Usage

All commands are run from the `back-end/` directory.

### Validate Data

Check the JSON data file for errors without touching the database:

```bash
cargo run -p seed-data validate
```

Validate a specific file:

```bash
cargo run -p seed-data validate -f path/to/data.json
```

### Load Players

Load players into the database:

```bash
cargo run -p seed-data load
```

Dry run (validate and simulate without writing to database):

```bash
cargo run -p seed-data load --dry-run
```

Load from a specific file:

```bash
cargo run -p seed-data load -f path/to/data.json
```

### Clear Players

Remove all players for a specific draft year:

```bash
cargo run -p seed-data clear --year 2026
```

## Data File

The default data file is `data/players_2026.json` in the `back-end/` directory. See `data/README.md` for data format and source documentation.
