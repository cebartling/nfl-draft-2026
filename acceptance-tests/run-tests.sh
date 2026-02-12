#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BOLD}════════════════════════════════════════${NC}"
echo -e "${BOLD}  E2E Acceptance Tests (Containerized)${NC}"
echo -e "${BOLD}════════════════════════════════════════${NC}"
echo ""

# ── Step 1: Start containers ─────────────────────────────────────

echo -e "${YELLOW}${BOLD}Step 1: Starting Docker containers...${NC}"
cd "$REPO_ROOT"
docker compose up -d --build postgres api frontend

# ── Step 2: Wait for services to be healthy ──────────────────────

echo -e "${YELLOW}${BOLD}Step 2: Waiting for services to be healthy...${NC}"

wait_for_healthy() {
    local service="$1"
    local max_wait="$2"
    local elapsed=0

    while [ $elapsed -lt "$max_wait" ]; do
        local health
        health=$(docker compose ps --format '{{.Service}} {{.Health}}' 2>/dev/null | grep "^${service} " | awk '{print $2}' || echo "unknown")
        if [ "$health" = "healthy" ]; then
            echo -e "  ${GREEN}${service}: healthy${NC}"
            return 0
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done

    echo -e "  ${RED}${service}: not healthy after ${max_wait}s${NC}"
    return 1
}

wait_for_healthy postgres 60
wait_for_healthy api 120
wait_for_healthy frontend 60

echo ""

# ── Step 3: Seed the database ────────────────────────────────────

echo -e "${YELLOW}${BOLD}Step 3: Seeding the database...${NC}"
docker compose --profile seed up seed --build --exit-code-from seed 2>&1 || {
    echo -e "${YELLOW}  Seed may have already run (data exists). Continuing...${NC}"
}
echo ""

# ── Step 4: Install test dependencies ────────────────────────────

echo -e "${YELLOW}${BOLD}Step 4: Installing test dependencies...${NC}"
cd "$SCRIPT_DIR"

# Use nvm if available
if [ -s "$HOME/.nvm/nvm.sh" ]; then
    # shellcheck source=/dev/null
    source "$HOME/.nvm/nvm.sh"
    nvm use 2>/dev/null || nvm install 2>/dev/null || true
fi

npm ci
npx playwright install chromium
echo ""

# ── Step 5: Run tests ────────────────────────────────────────────

echo -e "${YELLOW}${BOLD}Step 5: Running Playwright tests...${NC}"
TEST_EXIT=0
npx playwright test || TEST_EXIT=$?

echo ""

# ── Step 6: Report results ───────────────────────────────────────

if [ $TEST_EXIT -eq 0 ]; then
    echo -e "${GREEN}${BOLD}E2E acceptance tests: PASSED${NC}"
else
    echo -e "${RED}${BOLD}E2E acceptance tests: FAILED${NC}"
    echo -e "${YELLOW}Run 'npx playwright show-report' in acceptance-tests/ to view the report.${NC}"
fi

exit $TEST_EXIT
