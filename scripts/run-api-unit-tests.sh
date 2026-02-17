#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/back-end"

# Pure unit tests (no DB) — run with full parallelism
cargo test -p domain -p seed-data -p websocket --lib -- "$@"

# DB-dependent tests (api + db crates) — run single-threaded
cargo test -p db -p api --lib -- --test-threads=1 "$@"
