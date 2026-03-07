#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$ROOT_DIR/build/perf"
ARTIFACT_DIR="$ROOT_DIR/artifacts"
ARTIFACT_LOG="$ARTIFACT_DIR/c3_3d_perf_baseline_latest.log"
CURRENT_JSON="$BUILD_DIR/current_baseline.json"
REPORT_MD="$BUILD_DIR/regression_report.md"
LOG_DIR="$BUILD_DIR/logs"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-c3-perf.XXXXXX")"
TIMESTAMP="$(date '+%Y%m%d-%H%M%S')"
GIT_SHA="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
LOG_PATH="$LOG_DIR/perf_regression_${TIMESTAMP}_macos14_${GIT_SHA}.log"

cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

mkdir -p "$BUILD_DIR" "$ARTIFACT_DIR" "$LOG_DIR"
cd "$ROOT_DIR"

cargo run --release --manifest-path logic/Cargo.toml --bin perf_baseline -- \
  --output "$CURRENT_JSON" | tee "$WORK_DIR/perf_run.log"

set +e
python3 tools/check_perf_regression.py \
  --baseline docs/perf/baseline_macos14.json \
  --current "$CURRENT_JSON" \
  --output "$REPORT_MD" | tee "$LOG_PATH"
compare_status=$?
set -e

scenario_row="$(grep '| arena3d_renderable_build |' "$REPORT_MD" | tail -n1)"

{
  printf '### C3 3D perf baseline log %s\n' "$(date '+%Y-%m-%d %H:%M:%S %z')"
  printf -- '- Command: `%s`\n' './scripts/test_core_c3_3d_perf_baseline.sh'
  printf -- '- Current report: `%s`\n' "$CURRENT_JSON"
  printf -- '- Regression report: `%s`\n' "$REPORT_MD"
  printf -- '- Log file: `%s`\n' "$LOG_PATH"
  printf -- '- 3D scenario row: %s\n' "$scenario_row"
  printf -- '- Regression exit code: %s\n' "$compare_status"
} > "$ARTIFACT_LOG"

cat "$ARTIFACT_LOG"
exit "$compare_status"
