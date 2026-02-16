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

echo -e "${RED}${BOLD}Stopping containers and removing volumes...${NC}"
docker compose down -v --remove-orphans --rmi local

echo -e "${YELLOW}${BOLD}Rebuilding all images from scratch...${NC}"
docker compose build --no-cache

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
