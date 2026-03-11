#!/usr/bin/env bash
#
# Convenience script for scraping real NFL Combine results locally.
#
# Usage:
#   ./scripts/scrape-combine-results.sh              # Merge PFR + Mockdraftable
#   ./scripts/scrape-combine-results.sh --source pfr  # PFR only
#   ./scripts/scrape-combine-results.sh --browser     # Use Playwright fallback
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/back-end"

cd "$BACKEND_DIR"

# Default: merge mode
if [ $# -eq 0 ]; then
    echo "Building combine-data-scraper..."
    cargo build -p combine-data-scraper

    echo "Scraping combine data (merge mode)..."
    cargo run -p combine-data-scraper -- scrape --merge --year 2026 --output data/combine_2026.json
else
    echo "Building combine-data-scraper..."
    cargo build -p combine-data-scraper

    echo "Scraping combine data..."
    cargo run -p combine-data-scraper -- scrape "$@"
fi

echo "Done."
