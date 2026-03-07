#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="$ROOT_DIR/artifacts"
ARTIFACT_LOG="$ARTIFACT_DIR/g4_vertical_slice_latest.log"
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

run_logic_test \
  "verify 3D obstacle renderables" \
  "start_3d_arena_spawns_falling_obstacle_renderables" \
  "$WORK_DIR/obstacle_spawn.log"
grep -q "\[g4-03\]\[spawn\]" "$WORK_DIR/obstacle_spawn.log"

run_logic_test \
  "verify 3D obstacle game over flow" \
  "start_3d_arena_obstacle_hits_can_reach_game_over" \
  "$WORK_DIR/obstacle_fail.log"
grep -q "\[g4-03\]\[fail\]" "$WORK_DIR/obstacle_fail.log"

run_logic_test \
  "verify 3D obstacle clear flow" \
  "start_3d_arena_obstacle_avoidance_can_reach_clear" \
  "$WORK_DIR/obstacle_clear.log"
grep -q "\[g4-03\]\[clear\]" "$WORK_DIR/obstacle_clear.log"

run_logic_test \
  "verify 3D settings persistence" \
  "start_3d_arena_preserves_settings_across_result_and_retry" \
  "$WORK_DIR/settings.log"
grep -q "\[g4\]\[settings\]" "$WORK_DIR/settings.log"

{
  printf '### G4 vertical slice log %s\n' "$(date '+%Y-%m-%d %H:%M:%S %z')"
  printf -- '- Command: `%s`\n' './scripts/test_game_track_g4.sh'
  printf -- '- Pause/back evidence: %s\n' "$(grep '\[g4-02\]\[pause-back\]' "$WORK_DIR/pause_back.log" | tail -n1)"
  printf -- '- Game over evidence: %s\n' "$(grep '\[g4-02\]\[game-over\]' "$WORK_DIR/game_over.log" | tail -n1)"
  printf -- '- Clear/retry evidence: %s\n' "$(grep '\[g4-02\]\[clear-retry\]' "$WORK_DIR/clear_retry.log" | tail -n1)"
  printf -- '- Obstacle spawn evidence: %s\n' "$(grep '\[g4-03\]\[spawn\]' "$WORK_DIR/obstacle_spawn.log" | tail -n1)"
  printf -- '- Obstacle fail evidence: %s\n' "$(grep '\[g4-03\]\[fail\]' "$WORK_DIR/obstacle_fail.log" | tail -n1)"
  printf -- '- Obstacle clear evidence: %s\n' "$(grep '\[g4-03\]\[clear\]' "$WORK_DIR/obstacle_clear.log" | tail -n1)"
  printf -- '- Settings evidence: %s\n' "$(grep '\[g4\]\[settings\]' "$WORK_DIR/settings.log" | tail -n1)"
} > "$ARTIFACT_LOG"

cat "$ARTIFACT_LOG"
echo "[g4-02] wrote artifact log: $ARTIFACT_LOG"
