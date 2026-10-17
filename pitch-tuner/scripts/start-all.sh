#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

require cargo
require bun

BACKEND_PORT="$(service_port backend)"
FRONTEND_PORT="$(service_port frontend)"
export BACKEND_PORT FRONTEND_PORT

if ! port_up "$BACKEND_PORT"; then
  ( cd "$ROOT/backend" && cargo build ) || fail "backend build failed"
fi
start_bg backend "$ROOT/backend" env PORT="$BACKEND_PORT" "$ROOT/backend/target/debug/pitch-tuner"
wait_port_up "$BACKEND_PORT" 60 || fail "backend did not open port $BACKEND_PORT, see $LOGS/backend.log"
log "backend up on $(service_url backend)"

start_bg frontend "$ROOT/frontend" bun run dev
wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
for name in $(service_names); do
  printf "%-14s %s\n" "$name" "$(service_url "$name")"
done
