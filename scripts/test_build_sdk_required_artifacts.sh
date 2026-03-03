#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$ROOT_DIR"

DOC_FILE="docs/SDK_DEFINITION.md"
DOC_BACKUP="$(mktemp)"
LOG_FILE="$(mktemp)"

cleanup() {
  if [ -f "$DOC_BACKUP" ]; then
    cp "$DOC_BACKUP" "$DOC_FILE"
    rm -f "$DOC_BACKUP"
  fi
  rm -f "$LOG_FILE"
}
trap cleanup EXIT

cp "$DOC_FILE" "$DOC_BACKUP"
rm -f "$DOC_FILE"

echo "[test] missing required artifact should fail"
set +e
MIYABI_SDK_VALIDATE_ONLY=1 ./build_sdk.sh >"$LOG_FILE" 2>&1
status=$?
set -e

if [ "$status" -eq 0 ]; then
  echo "Expected failure when required artifact is missing, but build_sdk.sh succeeded."
  cat "$LOG_FILE"
  exit 1
fi

if ! rg -q "docs/SDK_DEFINITION.md" "$LOG_FILE"; then
  echo "Expected missing artifact path not found in output."
  cat "$LOG_FILE"
  exit 1
fi

cp "$DOC_BACKUP" "$DOC_FILE"

echo "[test] regression: normal build should succeed"
./build_sdk.sh

if [ ! -f "MIYABI_SDK.zip" ]; then
  echo "Expected MIYABI_SDK.zip to be generated."
  exit 1
fi

echo "PASS: required artifact validation and regression build"
