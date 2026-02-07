#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo -e "${YELLOW}${BOLD}Stopping all containers (including tools and seed profiles)...${NC}"
docker compose --profile tools --profile seed down -v --remove-orphans --rmi local

echo ""
echo -e "${GREEN}${BOLD}Shutdown complete.${NC} All containers, volumes, networks, and locally-built images have been removed."
