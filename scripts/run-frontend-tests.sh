#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/front-end"

# Lint (ESLint) — cheapest check first
npm run lint

# Type check (svelte-check)
npm run check

# Unit tests (Vitest)
npm run test
