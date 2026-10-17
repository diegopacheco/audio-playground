#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/common.sh"

MODEL_DIR="$ROOT/models"
MODEL_FILE="$MODEL_DIR/ggml-base.en.bin"
MODEL_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"

log "setup started"

require ffmpeg

if ! command -v whisper-cli >/dev/null 2>&1; then
  require brew
  log "installing whisper.cpp"
  brew install whisper-cpp || fail "brew install whisper-cpp failed"
fi
log "whisper-cli at $(command -v whisper-cli)"

mkdir -p "$MODEL_DIR"
if [ ! -s "$MODEL_FILE" ]; then
  require curl
  log "downloading ggml base.en model"
  curl -fL --retry 3 -o "$MODEL_FILE.part" "$MODEL_URL" || fail "model download failed"
  mv "$MODEL_FILE.part" "$MODEL_FILE"
fi
log "model ready"

require cargo
( cd "$ROOT/backend" && cargo build ) || fail "backend build failed"

require bun
( cd "$ROOT/frontend" && bun install ) || fail "frontend install failed"

if [ -z "${ANTHROPIC_API_KEY:-}" ]; then
  log "ANTHROPIC_API_KEY is not set, summaries will return 503 until it is exported"
fi

log "setup done"
