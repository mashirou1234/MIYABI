#!/usr/bin/env bash
set -euo pipefail

SDK_DIR="${1:-sdk}"

REQUIRED_FILES=(
  "include/miyabi/miyabi.h"
  "include/miyabi/bridge.h"
  "include/miyabi_logic_cxx/lib.h"
  "include/rust/cxx.h"
  "lib/libmiyabi_logic.a"
  "lib/libmiyabi_logic_cxx.a"
  "lib/libmiyabi_runtime.a"
  "lib/libbox2d.a"
  "cmake/MIYABIConfig.cmake"
  "cmake/MIYABIConfigVersion.cmake"
  "template_CMakeLists.txt"
  "examples/main.cpp"
  "docs/SDK_DEFINITION.md"
)

if [[ ! -d "$SDK_DIR" ]]; then
  echo "[sdk-check] ERROR: SDK directory not found: $SDK_DIR" >&2
  exit 1
fi

missing=0
for rel in "${REQUIRED_FILES[@]}"; do
  if [[ ! -f "$SDK_DIR/$rel" ]]; then
    echo "[sdk-check] MISSING: $rel" >&2
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo "[sdk-check] FAIL: missing required SDK artifacts" >&2
  exit 1
fi

echo "[sdk-check] PASS: all required SDK artifacts exist in $SDK_DIR"
