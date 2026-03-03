#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

PATHS_FILE="logic/src/paths.rs"
PATHS_BACKUP="$(mktemp)"
cp "$PATHS_FILE" "$PATHS_BACKUP"

cleanup() {
  cp "$PATHS_BACKUP" "$PATHS_FILE"
  rm -f "$PATHS_BACKUP"
  rm -f assets/.DS_Store
  rm -f "${ZIP_LIST:-}"
}
trap cleanup EXIT

echo "Preparing .DS_Store contamination fixture..."
printf 'finder-metadata-fixture\n' > assets/.DS_Store

echo "Running build_sdk.sh..."
bash ./build_sdk.sh

ZIP_LIST="$(mktemp)"
unzip -l MIYABI_SDK.zip > "$ZIP_LIST"

echo "Checking MIYABI_SDK.zip does not contain .DS_Store..."
if grep -Eq '\.DS_Store' "$ZIP_LIST"; then
  echo "ERROR: .DS_Store found in MIYABI_SDK.zip"
  exit 1
fi

echo "Checking required SDK files exist in archive..."
required_entries=(
  "sdk/bin/miyabi"
  "sdk/cmake/MIYABIConfig.cmake"
  "sdk/cmake/MIYABIConfigVersion.cmake"
  "sdk/docs/SDK_DEFINITION.md"
  "sdk/examples/main.cpp"
  "sdk/template_CMakeLists.txt"
)

for entry in "${required_entries[@]}"; do
  if ! grep -Fq "${entry}" "$ZIP_LIST"; then
    echo "ERROR: missing required entry in MIYABI_SDK.zip: ${entry}"
    exit 1
  fi
done

rm -f "$ZIP_LIST"

echo "PASS: .DS_Store excluded and required SDK entries are present."
