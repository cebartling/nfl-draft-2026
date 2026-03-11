#!/usr/bin/env bash
#
# Convenience script for scraping real NFL Combine results locally.
#
# Usage:
#   ./scripts/scrape-combine-results.sh              # Merge PFR + Mockdraftable
#   ./scripts/scrape-combine-results.sh --source pfr  # PFR only
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SCRAPERS_DIR="$PROJECT_ROOT/scrapers"

cd "$SCRAPERS_DIR"

# Default: merge mode
if [ $# -eq 0 ]; then
    echo "Scraping combine data (merge mode)..."
    bun run src/cli.ts combine --merge --year 2026 --output ../back-end/data/combine_2026.json
else
    echo "Scraping combine data..."
    bun run src/cli.ts combine "$@"
fi

echo "Done."
