#!/bin/bash
claude -p "Rebuild Docker containers with 'docker compose up --build -d', wait for health checks, run 'cargo test --workspace' for backend tests, run frontend tests, run 'cargo clippy --workspace' and report any warnings. Summarize pass/fail status." \
  --allowedTools "Bash,Read,Glob" \
  --output-format json
