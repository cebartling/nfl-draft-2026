#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

TEST_DB_URL="postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"

BACKEND_UNIT=0
BACKEND_ACCEPTANCE=0
FRONTEND_CHECK=0
FRONTEND_UNIT=0
E2E_ACCEPTANCE=0

BACKEND_UNIT_RESULT=""
BACKEND_ACCEPTANCE_RESULT=""
FRONTEND_CHECK_RESULT=""
FRONTEND_UNIT_RESULT=""
E2E_ACCEPTANCE_RESULT=""

# ── Ensure PostgreSQL is running ───────────────────────────────────

STARTED_POSTGRES=false

echo -e "${YELLOW}${BOLD}Checking PostgreSQL container...${NC}"
if ! docker compose ps --status running --format '{{.Service}}' 2>/dev/null | grep -q '^postgres$'; then
    echo -e "${YELLOW}PostgreSQL container is not running. Starting it...${NC}"
    docker compose up -d postgres
    STARTED_POSTGRES=true
    echo -e "${YELLOW}Waiting for PostgreSQL to be ready...${NC}"
    until docker compose exec postgres pg_isready -U nfl_draft_user -q 2>/dev/null; do
        sleep 1
    done
    echo -e "${GREEN}PostgreSQL is ready.${NC}"
else
    echo -e "${GREEN}PostgreSQL container is already running.${NC}"
fi
echo ""

# ── Recreate test database ──────────────────────────────────────────

echo -e "${YELLOW}${BOLD}Recreating test database...${NC}"
cd "$REPO_ROOT/back-end"
sqlx database drop --database-url "$TEST_DB_URL" -y 2>/dev/null || true
sqlx database create --database-url "$TEST_DB_URL"
sqlx migrate run --database-url "$TEST_DB_URL"
echo -e "${GREEN}Test database ready.${NC}"
echo ""

# ── Backend unit tests ──────────────────────────────────────────────

echo -e "${BLUE}${BOLD}Running backend unit tests...${NC}"
if cargo test --workspace --lib -- --test-threads=1 2>&1; then
    BACKEND_UNIT=1
    BACKEND_UNIT_RESULT="${GREEN}PASS${NC}"
else
    BACKEND_UNIT_RESULT="${RED}FAIL${NC}"
fi
echo ""

# ── Backend acceptance tests ────────────────────────────────────────

echo -e "${BLUE}${BOLD}Running backend acceptance tests...${NC}"
if cargo test --workspace --test '*' -- --test-threads=1 2>&1; then
    BACKEND_ACCEPTANCE=1
    BACKEND_ACCEPTANCE_RESULT="${GREEN}PASS${NC}"
else
    BACKEND_ACCEPTANCE_RESULT="${RED}FAIL${NC}"
fi
cd "$REPO_ROOT"
echo ""

# ── Install frontend dependencies ─────────────────────────────────

echo -e "${CYAN}${BOLD}Installing frontend dependencies...${NC}"
cd "$REPO_ROOT/front-end"
npm ci
echo -e "${GREEN}Frontend dependencies installed.${NC}"
echo ""

# ── Frontend type checks ───────────────────────────────────────────

echo -e "${CYAN}${BOLD}Running frontend type checks...${NC}"
if npm run check 2>&1; then
    FRONTEND_CHECK=1
    FRONTEND_CHECK_RESULT="${GREEN}PASS${NC}"
else
    FRONTEND_CHECK_RESULT="${RED}FAIL${NC}"
fi
echo ""

# ── Frontend unit tests ────────────────────────────────────────────

echo -e "${CYAN}${BOLD}Running frontend unit tests...${NC}"
if npm run test 2>&1; then
    FRONTEND_UNIT=1
    FRONTEND_UNIT_RESULT="${GREEN}PASS${NC}"
else
    FRONTEND_UNIT_RESULT="${RED}FAIL${NC}"
fi
cd "$REPO_ROOT"
echo ""

# ── E2E acceptance tests (containerized) ──────────────────────────

echo -e "${MAGENTA}${BOLD}Running E2E acceptance tests...${NC}"
# Let the E2E runner manage its own container cleanup via its EXIT trap
if KEEP_CONTAINERS=false "$REPO_ROOT/acceptance-tests/run-tests.sh" 2>&1; then
    E2E_ACCEPTANCE=1
    E2E_ACCEPTANCE_RESULT="${GREEN}PASS${NC}"
else
    E2E_ACCEPTANCE_RESULT="${RED}FAIL${NC}"
fi
echo ""

# ── Summary ─────────────────────────────────────────────────────────

TOTAL=$((BACKEND_UNIT + BACKEND_ACCEPTANCE + FRONTEND_CHECK + FRONTEND_UNIT + E2E_ACCEPTANCE))

echo -e "${BOLD}════════════════════════════════════════${NC}"
echo -e "${BOLD}           TEST SUMMARY${NC}"
echo -e "${BOLD}════════════════════════════════════════${NC}"
echo -e "  Backend unit tests:        ${BACKEND_UNIT_RESULT}"
echo -e "  Backend acceptance tests:  ${BACKEND_ACCEPTANCE_RESULT}"
echo -e "  Frontend type checks:      ${FRONTEND_CHECK_RESULT}"
echo -e "  Frontend unit tests:       ${FRONTEND_UNIT_RESULT}"
echo -e "  E2E acceptance tests:      ${E2E_ACCEPTANCE_RESULT}"
echo -e "${BOLD}════════════════════════════════════════${NC}"

if [ "$TOTAL" -eq 5 ]; then
    echo -e "  ${GREEN}${BOLD}All 5 test suites passed.${NC}"
else
    FAILED=$((5 - TOTAL))
    echo -e "  ${RED}${BOLD}${FAILED} of 5 test suites failed.${NC}"
fi

echo -e "${BOLD}════════════════════════════════════════${NC}"

# ── Shut down PostgreSQL if we started it ─────────────────────────

if [ "$STARTED_POSTGRES" = true ]; then
    echo ""
    echo -e "${YELLOW}${BOLD}Stopping PostgreSQL container (started by this script)...${NC}"
    docker compose stop postgres
    echo -e "${GREEN}PostgreSQL container stopped.${NC}"
fi

[ "$TOTAL" -eq 5 ]
