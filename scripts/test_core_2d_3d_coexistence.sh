#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="$ROOT_DIR/artifacts"
ARTIFACT_LOG="$ARTIFACT_DIR/c2_04_2d_3d_coexistence_latest.log"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-c2-04.XXXXXX")"

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

  echo "[c2-04] ${label}"
  cargo test --manifest-path logic/Cargo.toml "$test_name" -- --nocapture | tee "$log_path"
}

run_logic_test \
  "verify 2D title UI/text contract" \
  "title_screen_exposes_2d_ui_and_text_commands" \
  "$WORK_DIR/title_ui.log"
grep -q "\[c2-04\]\[2d-title\]" "$WORK_DIR/title_ui.log"

run_logic_test \
  "verify 3D arena overlay/render contract" \
  "start_3d_arena_preserves_2d_text_overlay_and_3d_renderables" \
  "$WORK_DIR/arena_overlay.log"
grep -q "\[c2-04\]\[3d-arena\]" "$WORK_DIR/arena_overlay.log"

run_logic_test \
  "verify 3D pause/back flow contract" \
  "start_3d_arena_pause_and_back_to_title_flow_is_reachable" \
  "$WORK_DIR/arena_flow.log"
grep -q "\[c2-04\]\[3d-flow\]" "$WORK_DIR/arena_flow.log"

echo "[c2-04] run G2 smoke"
./scripts/test_game_track_g2.sh | tee "$WORK_DIR/g2.log"

echo "[c2-04] run G3 distribution smoke"
./scripts/test_distribution_smoke.sh | tee "$WORK_DIR/g3.log"

{
  printf '### C2-04 2D/3D coexistence log %s\n' "$(date '+%Y-%m-%d %H:%M:%S %z')"
  printf -- '- Command: `%s`\n' './scripts/test_core_2d_3d_coexistence.sh'
  printf -- '- 2D title evidence: %s\n' "$(grep '\[c2-04\]\[2d-title\]' "$WORK_DIR/title_ui.log" | tail -n1)"
  printf -- '- 3D arena evidence: %s\n' "$(grep '\[c2-04\]\[3d-arena\]' "$WORK_DIR/arena_overlay.log" | tail -n1)"
  printf -- '- 3D flow evidence: %s\n' "$(grep '\[c2-04\]\[3d-flow\]' "$WORK_DIR/arena_flow.log" | tail -n1)"
  printf -- '- G2 smoke: PASS\n'
  printf -- '- G3 smoke: PASS\n'
} > "$ARTIFACT_LOG"

cat "$ARTIFACT_LOG"
echo "[c2-04] wrote artifact log: $ARTIFACT_LOG"
