#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

backend_port="$(service_port backend)"
frontend_port="$(service_port frontend)"

require cargo
( cd "$ROOT/backend" && cargo build ) || fail "backend build failed"
start_bg backend "$ROOT/backend" env PORT="$backend_port" "$ROOT/backend/target/debug/metronome"
wait_port_up "$backend_port" 60 || fail "backend did not open port $backend_port, see $LOGS/backend.log"
log "backend up on $(service_url backend)"

require bun
start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$frontend_port" BACKEND_PORT="$backend_port" bun run dev
wait_port_up "$frontend_port" 60 || fail "frontend did not open port $frontend_port, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
for name in $(service_names); do
  printf "%-14s %s\n" "$name" "$(service_url "$name")"
done
