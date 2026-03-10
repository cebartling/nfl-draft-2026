#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/back-end"

# Pure unit tests (no DB) — run with full parallelism
cargo test -p domain -p seed-data -p websocket --lib -- "$@"

# API unit tests (no DB) — run with full parallelism
cargo test -p api --lib -- "$@"
