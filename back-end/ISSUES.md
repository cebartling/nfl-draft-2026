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

### 9. Lossy position mapping

**File:** `crates/seed-data/src/position_mapper.rs:21`

`OLB/ILB/MLB -> LB` loses sub-position specificity. The original position
string is not preserved.

### 10. Duplicate player names are only warnings

**File:** `crates/seed-data/src/validator.rs:53-57`

Two players with the same full name pass validation. Could cause confusion in
the draft UI.

### 11. Height/weight validation ranges duplicated

**File:** `crates/seed-data/src/validator.rs:83,93`

Validator hardcodes `60-90` (height) and `150-400` (weight), same as the
domain model. Should share constants.

### 12. No early abort on systematic load failures

**File:** `crates/seed-data/src/loader.rs:112-133`

If the database is down, all 300 inserts fail individually. Could detect
systematic errors and abort early.
