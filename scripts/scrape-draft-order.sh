#!/usr/bin/env bash
#
# scrape-draft-order.sh — Build and run the draft-order-scraper.
#
# Features:
#   - Staleness check: skips if last scrape was < STALENESS_HOURS ago (default 24)
#   - --force: bypass staleness check
#   - --commit: git add + commit if the draft order file changed (does NOT push)
#
# Environment variables:
#   YEAR              Draft year (default: 2026)
#   STALENESS_HOURS   Hours before data is considered stale (default: 24)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$REPO_ROOT/back-end"
YEAR="${YEAR:-2026}"
STALENESS_HOURS="${STALENESS_HOURS:-24}"
OUTPUT_FILE="$BACKEND_DIR/data/draft_order_${YEAR}.json"
TIMESTAMP_FILE="$BACKEND_DIR/data/.draft_order_last_scraped"

FORCE=false
COMMIT=false

for arg in "$@"; do
    case "$arg" in
        --force) FORCE=true ;;
        --commit) COMMIT=true ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 [--force] [--commit]"
            exit 1
            ;;
    esac
done

# --- Staleness check ---

# Parse an ISO 8601 / RFC 3339 timestamp to epoch seconds (portable macOS + Linux)
parse_timestamp() {
    local ts="$1"
    if date -j -f "%Y-%m-%dT%H:%M:%S" "${ts%%.*}" "+%s" 2>/dev/null; then
        return
    fi
    # Linux fallback
    date -d "$ts" "+%s" 2>/dev/null || echo 0
}

if [ "$FORCE" = false ] && [ -f "$TIMESTAMP_FILE" ]; then
    last_scraped=$(cat "$TIMESTAMP_FILE")
    last_epoch=$(parse_timestamp "$last_scraped")
    now_epoch=$(date "+%s")
    age_hours=$(( (now_epoch - last_epoch) / 3600 ))

    if [ "$age_hours" -lt "$STALENESS_HOURS" ]; then
        echo "Draft order was scraped ${age_hours}h ago (threshold: ${STALENESS_HOURS}h). Skipping."
        echo "Use --force to override."
        exit 0
    fi
    echo "Draft order is ${age_hours}h old (threshold: ${STALENESS_HOURS}h). Re-scraping."
else
    if [ "$FORCE" = true ]; then
        echo "Force flag set — bypassing staleness check."
    else
        echo "No timestamp file found — running scraper."
    fi
fi

# --- Build and run ---

echo "Building draft-order-scraper (release)..."
cargo build --release -p draft-order-scraper --manifest-path "$BACKEND_DIR/Cargo.toml"

echo "Running scraper for year $YEAR..."
"$BACKEND_DIR/target/release/draft-order-scraper" \
    --year "$YEAR" \
    --output "$OUTPUT_FILE"

# --- Optional commit ---

if [ "$COMMIT" = true ]; then
    cd "$REPO_ROOT"
    if git diff --quiet "$OUTPUT_FILE" 2>/dev/null; then
        echo "No changes to draft order — nothing to commit."
    else
        git add "$OUTPUT_FILE"
        git commit -m "Update draft order data for $YEAR ($(date +%Y-%m-%d))"
        echo "Committed updated draft order."
    fi
fi
