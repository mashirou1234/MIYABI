#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="$ROOT_DIR/artifacts"
ARTIFACT_LOG="$ARTIFACT_DIR/g4_02_3d_run_flow_latest.log"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-g4.XXXXXX")"

cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

mkdir -p "$ARTIFACT_DIR"
cd "$ROOT_DIR"

run_logic_test() {
  local label="$1"
  local test_name="$2"
  local log_path="$3"

  echo "[g4-02] ${label}"
  cargo test --manifest-path logic/Cargo.toml "$test_name" -- --nocapture | tee "$log_path"
}

run_logic_test \
  "verify 3D arena pause/back flow" \
  "start_3d_arena_pause_and_back_to_title_flow_is_reachable" \
  "$WORK_DIR/pause_back.log"
grep -q "\[g4-02\]\[pause-back\]" "$WORK_DIR/pause_back.log"

run_logic_test \
  "verify 3D arena game over result" \
  "start_3d_arena_game_over_reaches_result_screen" \
  "$WORK_DIR/game_over.log"
grep -q "\[g4-02\]\[game-over\]" "$WORK_DIR/game_over.log"

run_logic_test \
  "verify 3D arena clear and retry flow" \
  "start_3d_arena_clear_and_retry_stays_in_3d" \
  "$WORK_DIR/clear_retry.log"
grep -q "\[g4-02\]\[clear-retry\]" "$WORK_DIR/clear_retry.log"

{
  printf '### G4-02 3D run flow log %s\n' "$(date '+%Y-%m-%d %H:%M:%S %z')"
  printf -- '- Command: `%s`\n' './scripts/test_game_track_g4.sh'
  printf -- '- Pause/back evidence: %s\n' "$(grep '\[g4-02\]\[pause-back\]' "$WORK_DIR/pause_back.log" | tail -n1)"
  printf -- '- Game over evidence: %s\n' "$(grep '\[g4-02\]\[game-over\]' "$WORK_DIR/game_over.log" | tail -n1)"
  printf -- '- Clear/retry evidence: %s\n' "$(grep '\[g4-02\]\[clear-retry\]' "$WORK_DIR/clear_retry.log" | tail -n1)"
} > "$ARTIFACT_LOG"

cat "$ARTIFACT_LOG"
echo "[g4-02] wrote artifact log: $ARTIFACT_LOG"
