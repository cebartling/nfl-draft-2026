# Scripts

Shell scripts for application lifecycle, testing, and data scraping. All scripts should be run from anywhere — they resolve the repository root automatically.

## App Lifecycle

### `run-app.sh`

Full clean rebuild and startup of all Docker services with database seeding.

**Usage:**

```bash
./scripts/run-app.sh
```

**What it does:**

1. Stops all containers and removes volumes/local images
2. Rebuilds all images from scratch (`--no-cache`)
3. Starts containers in detached mode
4. Runs the `seed` profile to populate the database
5. Prints service status and URLs

**Environment Variables:**

| Variable | Default | Description |
|---|---|---|
| `FRONTEND_PORT` | `3000` | Port displayed for frontend URL |
| `API_PORT` | `8000` | Port displayed for API URL |
| `POSTGRES_PORT` | `5432` | Port displayed for PostgreSQL |

**Prerequisites:** Docker and Docker Compose.

---

### `shutdown.sh`

Complete Docker teardown — stops all containers (including `tools` and `seed` profiles), removes volumes, networks, and locally-built images.

**Usage:**

```bash
./scripts/shutdown.sh
```

**Prerequisites:** Docker and Docker Compose.

## Testing

### `run-tests.sh`

Orchestrates all 5 test suites and prints a summary table with pass/fail results.

**Usage:**

```bash
./scripts/run-tests.sh
```

**What it does:**

1. Starts PostgreSQL if not already running (and stops it afterward)
2. Drops and recreates the test database with migrations
3. Runs backend unit tests (`cargo test --workspace --lib`)
4. Runs backend acceptance tests (`cargo test --workspace --test '*'`)
5. Installs frontend dependencies (`npm ci`)
6. Runs frontend type checks (`npm run check`)
7. Runs frontend unit tests (`npm run test`)
8. Runs E2E acceptance tests via `acceptance-tests/run-tests.sh`
9. Prints a summary and exits with non-zero status if any suite failed

**Prerequisites:**

- Docker and Docker Compose (for PostgreSQL)
- Rust toolchain with `cargo` and `sqlx-cli`
- Node.js with `npm`
- E2E test infrastructure in `acceptance-tests/`

---

### `validate.sh`

Delegates validation to the Claude CLI, which rebuilds Docker containers, runs backend and frontend tests, runs Clippy, and reports pass/fail status.

**Usage:**

```bash
./scripts/validate.sh
```

**Output:** JSON (via `--output-format json`).

**Prerequisites:** Claude CLI (`claude`) installed and available on PATH.

## Data Scraping

### `scrape-draft-order.sh`

Builds and runs the `draft-order-scraper` crate to scrape NFL draft order data from Tankathon.

**Usage:**

```bash
./scripts/scrape-draft-order.sh [--force] [--commit]
```

**Flags:**

| Flag | Description |
|---|---|
| `--force` | Bypass staleness check and always re-scrape |
| `--commit` | Git add and commit the output file if it changed (does NOT push) |

**Environment Variables:**

| Variable | Default | Description |
|---|---|---|
| `YEAR` | `2026` | Draft year to scrape |
| `STALENESS_HOURS` | `24` | Hours before cached data is considered stale |

**Output:** `back-end/data/draft_order_{YEAR}.json`

**Prerequisites:** Rust toolchain with `cargo`.

---

### `scrape-prospect-rankings.sh`

Builds and runs the `prospect-rankings-scraper` crate against multiple sources (Tankathon, WalterFootball), then merges results into a consensus ranking.

**Usage:**

```bash
./scripts/scrape-prospect-rankings.sh [--force] [--commit]
```

**Flags:**

| Flag | Description |
|---|---|
| `--force` | Bypass staleness check and always re-scrape |
| `--commit` | Git add and commit changed ranking files (does NOT push) |

**Environment Variables:**

| Variable | Default | Description |
|---|---|---|
| `YEAR` | `2026` | Draft year to scrape |
| `STALENESS_HOURS` | `24` | Hours before cached data is considered stale |

**Output:**

| File | Description |
|---|---|
| `back-end/data/rankings/tankathon_{YEAR}.json` | Tankathon source rankings |
| `back-end/data/rankings/walterfootball_{YEAR}.json` | WalterFootball source rankings |
| `back-end/data/rankings/rankings_{YEAR}.json` | Merged consensus rankings |

**Fault tolerance:** Each source scrape runs independently. If one source fails, the script continues with the remaining sources. The merge step runs as long as at least one source succeeded.

**Prerequisites:** Rust toolchain with `cargo`.
