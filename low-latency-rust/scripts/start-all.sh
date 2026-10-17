#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

log "starting"

BIN="$ROOT/backend/target/release/low-latency"
[ -x "$BIN" ] || fail "backend binary missing, run ./scripts/setup.sh first"
[ -d "$ROOT/frontend/node_modules" ] || fail "frontend dependencies missing, run ./scripts/setup.sh first"

start_bg backend "$ROOT/backend" env PORT="$(service_port backend)" AUDIO_BACKEND="${AUDIO_BACKEND:-auto}" "$BIN"
wait_port_up "$(service_port backend)" 60 || fail "backend did not open port $(service_port backend), see $LOGS/backend.log"
log "backend up on $(service_url backend)/api"

start_bg frontend "$ROOT/frontend" bun run dev
wait_port_up "$(service_port frontend)" 60 || fail "frontend did not open port $(service_port frontend), see $LOGS/frontend.log"
log "frontend up on $(service_url frontend)"

"$SCRIPTS/status.sh"

log "links"
printf "%-14s %s\n" backend "$(service_url backend)/api"
printf "%-14s %s\n" frontend "$(service_url frontend)"
