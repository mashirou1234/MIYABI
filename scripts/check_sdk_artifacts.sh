#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/check_sdk_artifacts.sh [--dry-run] [sdk_dir]

Options:
  --dry-run  Missing files are reported, but exit status is always 0.
  -h, --help Show this help.
EOF
}

DRY_RUN=0
SDK_DIR="sdk"
SDK_DIR_SET=0

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [[ "$SDK_DIR_SET" -eq 1 ]]; then
        echo "[sdk-check] ERROR: too many positional arguments" >&2
        usage >&2
        exit 2
      fi
      SDK_DIR="$1"
      SDK_DIR_SET=1
      shift
      ;;
  esac
done

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
    if [[ "$DRY_RUN" -eq 1 ]]; then
      echo "[sdk-check][dry-run] MISSING: $rel" >&2
    else
      echo "[sdk-check] MISSING: $rel" >&2
    fi
    missing=1
  fi
done

if [[ "$missing" -ne 0 && "$DRY_RUN" -ne 1 ]]; then
  echo "[sdk-check] FAIL: missing required SDK artifacts" >&2
  exit 1
fi

if [[ "$missing" -ne 0 && "$DRY_RUN" -eq 1 ]]; then
  echo "[sdk-check][dry-run] DONE: missing artifacts found in $SDK_DIR (non-fatal)"
  exit 0
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[sdk-check][dry-run] PASS: all required SDK artifacts exist in $SDK_DIR"
else
  echo "[sdk-check] PASS: all required SDK artifacts exist in $SDK_DIR"
fi
