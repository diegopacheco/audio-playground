#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

require cargo
( cd "$ROOT/backend" && cargo build ) || fail "backend build failed"
start_bg backend "$ROOT/backend" env PORT="$(service_port backend)" "$ROOT/backend/target/debug/tone-generator"
wait_port_up "$(service_port backend)" 60 || fail "backend did not open port $(service_port backend), see $LOGS/backend.log"
log "backend up on $(service_url backend)/api"

require bun
start_bg frontend "$ROOT/frontend" bun run dev
wait_port_up "$(service_port frontend)" 60 || fail "frontend did not open port $(service_port frontend), see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/api/waveforms"
printf "%-14s %s\n" frontend "$(service_url frontend)"
