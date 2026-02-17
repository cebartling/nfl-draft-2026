#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

TEST_DB_URL="postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"

# ── Ensure PostgreSQL is running ─────────────────────────────────

STARTED_POSTGRES=false

echo "Checking PostgreSQL container..."
if ! docker compose ps --status running --format '{{.Service}}' 2>/dev/null | grep -q '^postgres$'; then
    echo "PostgreSQL container is not running. Starting it..."
    docker compose up -d postgres
    STARTED_POSTGRES=true
    echo "Waiting for PostgreSQL to be ready..."
    until docker compose exec postgres pg_isready -U nfl_draft_user -q 2>/dev/null; do
        sleep 1
    done
    echo "PostgreSQL is ready."
else
    echo "PostgreSQL container is already running."
fi

# ── Recreate test database ───────────────────────────────────────

echo "Recreating test database..."
cd "$REPO_ROOT/back-end"
sqlx database drop --database-url "$TEST_DB_URL" -y 2>/dev/null || true
sqlx database create --database-url "$TEST_DB_URL"
sqlx migrate run --database-url "$TEST_DB_URL"
echo "Test database ready."

# ── Run API acceptance tests ─────────────────────────────────────

echo "Running API acceptance tests..."
TEST_EXIT=0
cargo test -p api --tests -- --test-threads=1 "$@" || TEST_EXIT=$?

# ── Stop PostgreSQL if we started it ─────────────────────────────

if [ "$STARTED_POSTGRES" = true ]; then
    echo "Stopping PostgreSQL container (started by this script)..."
    docker compose stop postgres
fi

exit $TEST_EXIT
