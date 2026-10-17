#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "tests started"

require cargo
( cd "$ROOT/backend" && cargo test ) || fail "backend tests failed"
( cd "$ROOT/backend" && cargo clippy --all-targets -- -D warnings ) || fail "backend clippy failed"

require bun
( cd "$ROOT/frontend" && bun test src ) || fail "frontend tests failed"
( cd "$ROOT/frontend" && bun run build ) || fail "frontend build failed"

log "tests passed"
