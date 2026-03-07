#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-sdk-check.XXXXXX")"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

echo "[test] minimal reproduction: validate-only fails when required artifact is missing"
FAKE_SDK="${TMP_DIR}/sdk"
mkdir -p "${FAKE_SDK}/bin" "${FAKE_SDK}/lib" "${FAKE_SDK}/include/miyabi" "${FAKE_SDK}/cmake" "${FAKE_SDK}/examples" "${FAKE_SDK}/docs"
touch "${FAKE_SDK}/bin/miyabi"
touch "${FAKE_SDK}/lib/libmiyabi_logic.a"
touch "${FAKE_SDK}/lib/libmiyabi_logic_cxx.a"
touch "${FAKE_SDK}/lib/libmiyabi_runtime.a"
touch "${FAKE_SDK}/lib/libbox2d.a"
touch "${FAKE_SDK}/include/miyabi/miyabi.h"
touch "${FAKE_SDK}/include/miyabi/bridge.h"
touch "${FAKE_SDK}/cmake/MIYABIConfig.cmake"
touch "${FAKE_SDK}/cmake/MIYABIConfigVersion.cmake"
touch "${FAKE_SDK}/examples/main.cpp"
touch "${FAKE_SDK}/template_CMakeLists.txt"
# docs/SDK_DEFINITION.md intentionally missing

echo "[test] dry-run check reports missing artifact as non-fatal"
./scripts/check_sdk_artifacts.sh --dry-run "${FAKE_SDK}" > "${TMP_DIR}/dry_run.log" 2>&1
grep -q "\\[sdk-check\\]\\[dry-run\\] MISSING: docs/SDK_DEFINITION.md" "${TMP_DIR}/dry_run.log"
grep -q "\\[sdk-check\\]\\[dry-run\\] DONE:" "${TMP_DIR}/dry_run.log"

set +e
SDK_DIR="${FAKE_SDK}" MIYABI_SDK_VALIDATE_ONLY=1 bash ./build_sdk.sh > "${TMP_DIR}/validate_missing.log" 2>&1
STATUS=$?
set -e
if [ "${STATUS}" -eq 0 ]; then
  echo "Expected validate-only mode to fail when SDK_DEFINITION.md is missing." >&2
  exit 1
fi
grep -q "docs/SDK_DEFINITION.md" "${TMP_DIR}/validate_missing.log"

echo "[test] regression: build_sdk.sh completes and keeps tracked paths.rs unchanged"
bash ./build_sdk.sh > "${TMP_DIR}/build_sdk.log" 2>&1
test -f ./MIYABI_SDK.zip
git diff --exit-code -- logic/src/paths.rs

echo "[test] external sample reuse smoke passes with generated sdk/"
./scripts/test_sdk_external_sample_reuse.sh ./sdk

echo "[test] PASS"
