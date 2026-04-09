#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Optionally regenerate The Beast 2026 JSON so the seed container can load it.
# Requires Bun + pdftotext locally and THE_BEAST_PASSWORD set in the environment
# (or .env). Skipped silently when prerequisites are missing so users without
# the PDF can still bring up a stack.
BEAST_PDF="documentation/content/the-beast-2026/the-beast-2026.pdf"
BEAST_JSON="back-end/data/the_beast_2026.json"
if [ -f "$BEAST_PDF" ] && command -v bun >/dev/null 2>&1 && command -v pdftotext >/dev/null 2>&1; then
  if [ -z "${THE_BEAST_PASSWORD:-}" ] && [ -f "back-end/.env" ]; then
    # shellcheck disable=SC1091
    THE_BEAST_PASSWORD="$(grep -E '^THE_BEAST_PASSWORD=' back-end/.env | head -n1 | cut -d= -f2- | tr -d '"' || true)"
  fi
  if [ -n "${THE_BEAST_PASSWORD:-}" ]; then
    echo -e "${CYAN}${BOLD}Generating The Beast 2026 JSON...${NC}"
    (cd scrapers && bun run scrape the-beast \
      --pdf "../$BEAST_PDF" \
      --password "$THE_BEAST_PASSWORD" \
      --output "../$BEAST_JSON")
  else
    echo -e "${YELLOW}THE_BEAST_PASSWORD not set; skipping Beast scrape.${NC}"
    echo -e "${YELLOW}  Set it in back-end/.env or in your shell to load Beast profiles.${NC}"
  fi
else
  echo -e "${YELLOW}Skipping Beast scrape (need PDF + bun + pdftotext).${NC}"
fi

echo -e "${RED}${BOLD}Stopping containers and removing volumes...${NC}"
docker compose --profile seed down -v --remove-orphans --rmi local

echo -e "${YELLOW}${BOLD}Rebuilding all images from scratch...${NC}"
docker compose --profile seed build --no-cache

echo -e "${YELLOW}${BOLD}Starting containers...${NC}"
docker compose up -d

echo -e "${CYAN}${BOLD}Seeding the database...${NC}"
docker compose --profile seed run --rm seed

echo ""
echo -e "${BOLD}Service status:${NC}"
docker compose ps

echo ""
echo -e "${GREEN}${BOLD}Done. Services available at:${NC}"
echo -e "  Frontend: ${CYAN}http://localhost:${FRONTEND_PORT:-3000}${NC}"
echo -e "  API:      ${CYAN}http://localhost:${API_PORT:-8000}${NC}"
echo -e "  Postgres: ${CYAN}localhost:${POSTGRES_PORT:-5432}${NC}"
