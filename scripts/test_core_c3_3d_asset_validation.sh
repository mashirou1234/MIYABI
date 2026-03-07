#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$ROOT_DIR/build/asset_validation"
ARTIFACT_DIR="$ROOT_DIR/artifacts"
ARTIFACT_LOG="$ARTIFACT_DIR/c3_3d_asset_validation_latest.log"
LOG_DIR="$BUILD_DIR/logs"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-c3-assets.XXXXXX")"
TIMESTAMP="$(date '+%Y%m%d-%H%M%S')"
GIT_SHA="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)"
PASS_LOG="$LOG_DIR/validate_3d_assets_${TIMESTAMP}_${GIT_SHA}_pass.log"
FAIL_LOG="$LOG_DIR/validate_3d_assets_${TIMESTAMP}_${GIT_SHA}_invalid_fixture.log"
FIXTURE_PATH="$ROOT_DIR/tools/tests/fixtures/invalid_missing_faces.obj.fixture"

cleanup() {
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

mkdir -p "$BUILD_DIR" "$ARTIFACT_DIR" "$LOG_DIR"
cd "$ROOT_DIR"

python3 tools/validate_3d_assets.py | tee "$PASS_LOG"

set +e
python3 tools/validate_3d_assets.py --obj "$FIXTURE_PATH" | tee "$FAIL_LOG"
invalid_status=$?
set -e

if [ "$invalid_status" -eq 0 ]; then
  echo "[c3-assets] expected invalid fixture to fail validation"
  exit 1
fi

invalid_line="$(grep 'invalid_missing_faces.obj.fixture' "$FAIL_LOG" | tail -n1 || true)"
if [[ "$invalid_line" != *"no drawable faces found"* ]]; then
  echo "[c3-assets] missing expected invalid fixture diagnostic"
  exit 1
fi

{
  printf '### C3 3D asset validation log %s\n' "$(date '+%Y-%m-%d %H:%M:%S %z')"
  printf -- '- Command: `%s`\n' './scripts/test_core_c3_3d_asset_validation.sh'
  printf -- '- Pass log: `%s`\n' "$PASS_LOG"
  printf -- '- Invalid fixture log: `%s`\n' "$FAIL_LOG"
  printf -- '- Invalid fixture result: %s\n' "$invalid_line"
  printf -- '- Invalid fixture exit code: %s\n' "$invalid_status"
} > "$ARTIFACT_LOG"

cat "$ARTIFACT_LOG"
