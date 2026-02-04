# Known Issues

Identified during PR #12 review (2026-02-04).

## HIGH Severity (Pre-existing)

### 1. No authentication on trade endpoints

**File:** `crates/api/src/handlers/trades.rs`

`propose_trade`, `accept_trade`, and `reject_trade` do not verify the caller
controls the team making the action. Any client can submit arbitrary `team_id`
values. Needs JWT middleware to extract and validate team ownership.

### 2. N+1 query in `find_pending_for_team`

**File:** `crates/db/src/repositories/trade_repo.rs`

Fetches pending trades, then loops calling `find_trade_with_details()` per
trade. Should join with `pick_trade_details` in a single query or use a batch
fetch.

### 3. Race condition on concurrent trade acceptance

**File:** `crates/domain/src/services/trade_engine.rs`

Window between pick re-validation and transfer allows two concurrent
acceptances to both succeed on overlapping picks. Needs `SELECT FOR UPDATE`
locks in the database transaction.

## MEDIUM Severity

### 4. No transaction around bulk player insert

**File:** `crates/seed-data/src/loader.rs:109-136`

Each player is inserted individually. If insertion fails partway through, the
database is left with partial data. Should wrap in a transaction for atomicity.

### 5. Sequential inserts — O(n) round trips

**File:** `crates/seed-data/src/loader.rs:112-133`

300 individual INSERT statements. Works at this scale but won't scale well.
Consider batch/bulk insert for larger datasets.

### 6. `expect()` on DATABASE_URL panics ungracefully

**File:** `crates/seed-data/src/main.rs:109-110, 126-127`

Should use `anyhow::Context` for a better CLI experience instead of panicking.

### 7. Raw SQL in `clear` command bypasses compile-time verification

**File:** `crates/seed-data/src/main.rs:143`

Uses `sqlx::query()` instead of `sqlx::query!` macro. Parameterized so no
injection risk, but inconsistent with the project pattern of compile-time
checked queries.

### 8. No year validation on `clear` command

**File:** `crates/seed-data/src/main.rs:123`

Accepts any `i32` for `--year` with no bounds check. Should validate the year
is within a reasonable range.

## LOW Severity

### 9. ~~Lossy position mapping~~ (Resolved)

**File:** `crates/seed-data/src/position_mapper.rs`

**Fix:** Added `tracing::debug!` logging when alternate abbreviations (e.g.,
OLB, EDGE, HB) are mapped to canonical positions. This provides observability
into lossy mappings without requiring schema changes.

### 10. ~~Duplicate player names are only warnings~~ (Resolved)

**File:** `crates/seed-data/src/validator.rs`

**Fix:** Promoted duplicate player name detection from warning to error.
Validation now fails when duplicate names are found.

### 11. ~~Height/weight validation ranges duplicated~~ (Resolved)

**Files:** `crates/domain/src/models/player.rs`, `crates/seed-data/src/validator.rs`

**Fix:** Added public constants `MIN_HEIGHT_INCHES`, `MAX_HEIGHT_INCHES`,
`MIN_WEIGHT_POUNDS`, `MAX_WEIGHT_POUNDS` to the domain `Player` model. Both
the domain validation and seed-data validator now reference these constants.

### 12. ~~No early abort on systematic load failures~~ (Resolved)

**File:** `crates/seed-data/src/loader.rs`

**Fix:** Added consecutive-failure counter to both `load_players` and
`load_players_dry_run`. If 5 or more consecutive failures occur, loading
aborts early with a message suggesting a systematic problem.
