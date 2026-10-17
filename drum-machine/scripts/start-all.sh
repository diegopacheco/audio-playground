#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

BACKEND_PORT="$(service_port backend)"
FRONTEND_PORT="$(service_port frontend)"

require cargo
( cd "$ROOT/backend" && cargo build -q ) || fail "backend build failed"
start_bg backend "$ROOT/backend" env PORT="$BACKEND_PORT" "$ROOT/backend/target/debug/drum-machine"
wait_port_up "$BACKEND_PORT" 60 || fail "backend did not open port $BACKEND_PORT, see $LOGS/backend.log"
log "backend up on $(service_url backend)/api"

require bun
start_bg frontend "$ROOT/frontend" env FRONTEND_PORT="$FRONTEND_PORT" BACKEND_PORT="$BACKEND_PORT" bun run dev
wait_port_up "$FRONTEND_PORT" 60 || fail "frontend did not open port $FRONTEND_PORT, see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/api/patterns"
printf "%-14s %s\n" frontend "$(service_url frontend)"
