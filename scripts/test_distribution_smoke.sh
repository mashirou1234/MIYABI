#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_ROOT="$ROOT_DIR/dist"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-dist-smoke.XXXXXX")"
RUN_LOG="$WORK_DIR/run.log"

cleanup() {
  if [ -n "${APP_PID:-}" ] && kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"

"$ROOT_DIR/scripts/package_macos_game.sh"

ARCHIVE_PATH="$(ls -1t "$DIST_ROOT"/MIYABI_GAME_macOS_*.zip | head -n1)"
EXTRACT_ROOT="$WORK_DIR/extracted"
PACKAGE_DIR="$EXTRACT_ROOT/miyabi_game_macos"
mkdir -p "$EXTRACT_ROOT"
ditto -x -k "$ARCHIVE_PATH" "$EXTRACT_ROOT"

for path in \
  bin/miyabi \
  assets/player.png \
  assets/test.png \
  assets/test_sound.wav \
  shaders/text.vert \
  shaders/text.frag \
  shaders/textured.vert \
  shaders/textured.frag \
  shaders/lit_textured.vert \
  shaders/lit_textured.frag \
  run_miyabi.sh \
  SHA256SUMS.txt \
  docs/README.txt
do
  if [ ! -e "$PACKAGE_DIR/$path" ]; then
    echo "[g3] missing required bundled file: $path"
    exit 1
  fi
done

ORIGINAL_PWD="$PWD"
cd "$PACKAGE_DIR"
./run_miyabi.sh >"$RUN_LOG" 2>&1 &
APP_PID=$!
sleep 5
if ! kill -0 "$APP_PID" 2>/dev/null; then
  echo "[g3] packaged app exited before 5-second smoke window"
  cat "$RUN_LOG"
  exit 1
fi
kill "$APP_PID"
wait "$APP_PID" 2>/dev/null || true
cd "$ORIGINAL_PWD"

echo "[g3] archive: $ARCHIVE_PATH"
echo "[g3] extracted package: $PACKAGE_DIR"
echo "[g3] smoke launch: pass (process stayed alive for 5 seconds)"
