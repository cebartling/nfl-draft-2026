#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

TEST_DB_URL="postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test"

BACKEND_UNIT=0
BACKEND_ACCEPTANCE=0
FRONTEND_CHECK=0
FRONTEND_UNIT=0

BACKEND_UNIT_RESULT=""
BACKEND_ACCEPTANCE_RESULT=""
FRONTEND_CHECK_RESULT=""
FRONTEND_UNIT_RESULT=""

# ── Recreate test database ──────────────────────────────────────────

echo -e "${YELLOW}${BOLD}Recreating test database...${NC}"
cd back-end
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
cd "$SCRIPT_DIR"
echo ""

# ── Frontend type checks ───────────────────────────────────────────

echo -e "${CYAN}${BOLD}Running frontend type checks...${NC}"
cd front-end
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
cd "$SCRIPT_DIR"
echo ""

# ── Summary ─────────────────────────────────────────────────────────

TOTAL=$((BACKEND_UNIT + BACKEND_ACCEPTANCE + FRONTEND_CHECK + FRONTEND_UNIT))

echo -e "${BOLD}════════════════════════════════════════${NC}"
echo -e "${BOLD}           TEST SUMMARY${NC}"
echo -e "${BOLD}════════════════════════════════════════${NC}"
echo -e "  Backend unit tests:        ${BACKEND_UNIT_RESULT}"
echo -e "  Backend acceptance tests:  ${BACKEND_ACCEPTANCE_RESULT}"
echo -e "  Frontend type checks:      ${FRONTEND_CHECK_RESULT}"
echo -e "  Frontend unit tests:       ${FRONTEND_UNIT_RESULT}"
echo -e "${BOLD}════════════════════════════════════════${NC}"

if [ "$TOTAL" -eq 4 ]; then
    echo -e "  ${GREEN}${BOLD}All 4 test suites passed.${NC}"
else
    FAILED=$((4 - TOTAL))
    echo -e "  ${RED}${BOLD}${FAILED} of 4 test suites failed.${NC}"
fi

echo -e "${BOLD}════════════════════════════════════════${NC}"

[ "$TOTAL" -eq 4 ]
