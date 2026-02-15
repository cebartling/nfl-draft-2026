#!/usr/bin/env bash
#
# scrape-prospect-rankings.sh — Build and run the prospect-rankings-scraper
# for multiple sources, then merge results.
#
# Features:
#   - Staleness check: skips if last scrape was < STALENESS_HOURS ago (default 24)
#   - --force: bypass staleness check
#   - --commit: git add + commit if ranking files changed (does NOT push)
#   - Fault-tolerant: each source scrape runs independently; merge runs if
#     at least one source succeeded
#
# Environment variables:
#   YEAR              Draft year (default: 2026)
#   STALENESS_HOURS   Hours before data is considered stale (default: 24)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$REPO_ROOT/back-end"
YEAR="${YEAR:-2026}"
STALENESS_HOURS="${STALENESS_HOURS:-24}"
RANKINGS_DIR="$BACKEND_DIR/data/rankings"
TIMESTAMP_FILE="$RANKINGS_DIR/.rankings_last_scraped"

TANKATHON_FILE="$RANKINGS_DIR/tankathon_${YEAR}.json"
WALTERFOOTBALL_FILE="$RANKINGS_DIR/walterfootball_${YEAR}.json"
MERGED_FILE="$RANKINGS_DIR/rankings_${YEAR}.json"

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
        echo "Rankings were scraped ${age_hours}h ago (threshold: ${STALENESS_HOURS}h). Skipping."
        echo "Use --force to override."
        exit 0
    fi
    echo "Rankings are ${age_hours}h old (threshold: ${STALENESS_HOURS}h). Re-scraping."
else
    if [ "$FORCE" = true ]; then
        echo "Force flag set — bypassing staleness check."
    else
        echo "No timestamp file found — running scraper."
    fi
fi

# --- Build ---

echo "Building prospect-rankings-scraper (release)..."
cargo build --release -p prospect-rankings-scraper --manifest-path "$BACKEND_DIR/Cargo.toml"

SCRAPER="$BACKEND_DIR/target/release/prospect-rankings-scraper"
SOURCES_SUCCEEDED=0

# --- Scrape Tankathon (fault-tolerant) ---

echo ""
echo "=== Scraping Tankathon ==="
if "$SCRAPER" \
    --source tankathon \
    --year "$YEAR" \
    --output "$TANKATHON_FILE"; then
    echo "Tankathon scrape succeeded."
    SOURCES_SUCCEEDED=$((SOURCES_SUCCEEDED + 1))
else
    echo "WARNING: Tankathon scrape failed (exit $?). Continuing..."
fi

# --- Scrape WalterFootball (fault-tolerant) ---

echo ""
echo "=== Scraping WalterFootball ==="
if "$SCRAPER" \
    --source walterfootball \
    --year "$YEAR" \
    --output "$WALTERFOOTBALL_FILE"; then
    echo "WalterFootball scrape succeeded."
    SOURCES_SUCCEEDED=$((SOURCES_SUCCEEDED + 1))
else
    echo "WARNING: WalterFootball scrape failed (exit $?). Continuing..."
fi

# --- Merge (only if at least one source succeeded) ---

if [ "$SOURCES_SUCCEEDED" -eq 0 ]; then
    echo ""
    echo "ERROR: All source scrapes failed. Nothing to merge."
    exit 1
fi

echo ""
echo "=== Merging rankings ($SOURCES_SUCCEEDED source(s) succeeded) ==="

MERGE_ARGS=("--merge" "--output" "$MERGED_FILE")

# Build primary/secondary args based on which files exist and have content
if [ -f "$TANKATHON_FILE" ]; then
    MERGE_ARGS+=("--primary" "$TANKATHON_FILE")
    if [ -f "$WALTERFOOTBALL_FILE" ]; then
        MERGE_ARGS+=("--secondary" "$WALTERFOOTBALL_FILE")
    fi
elif [ -f "$WALTERFOOTBALL_FILE" ]; then
    MERGE_ARGS+=("--primary" "$WALTERFOOTBALL_FILE")
else
    echo "ERROR: No ranking files found to merge."
    exit 1
fi

# Only run merge if we have both primary and secondary
if [[ " ${MERGE_ARGS[*]} " == *" --secondary "* ]]; then
    "$SCRAPER" "${MERGE_ARGS[@]}"
    echo "Merge complete."
else
    echo "Only one source available — copying as merged output."
    cp "${MERGE_ARGS[3]}" "$MERGED_FILE"
    echo "Copied ${MERGE_ARGS[3]} to $MERGED_FILE"
fi

# --- Optional commit ---

if [ "$COMMIT" = true ]; then
    cd "$REPO_ROOT"
    CHANGED_FILES=()
    for f in "$TANKATHON_FILE" "$WALTERFOOTBALL_FILE" "$MERGED_FILE"; do
        if [ -f "$f" ] && ! git diff --quiet "$f" 2>/dev/null; then
            CHANGED_FILES+=("$f")
        fi
        # Also pick up newly created (untracked) files
        if [ -f "$f" ] && git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
            : # already tracked, handled by diff above
        elif [ -f "$f" ]; then
            CHANGED_FILES+=("$f")
        fi
    done

    if [ ${#CHANGED_FILES[@]} -eq 0 ]; then
        echo "No changes to rankings — nothing to commit."
    else
        git add "${CHANGED_FILES[@]}"
        git commit -m "Update prospect rankings data for $YEAR ($(date +%Y-%m-%d))"
        echo "Committed updated prospect rankings."
    fi
fi
