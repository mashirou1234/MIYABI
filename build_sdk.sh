#!/bin/bash
set -euo pipefail

print_help() {
    cat <<'EOF'
Usage:
  ./build_sdk.sh [--help|-h]

Environment Variables:
  SDK_DIR                    Output SDK directory (default: sdk)
  BUILD_DIR                  CMake build directory (default: build)
  ZIP_NAME                   SDK archive name (default: MIYABI_SDK.zip)
  MIYABI_SDK_VALIDATE_ONLY   If set to 1, only validates generated SDK artifacts

Prerequisites:
  - cmake
  - zip
  - find
  - standard POSIX utilities (cp, mkdir, rm, mktemp)
EOF
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    print_help
    exit 0
fi

echo "Building MIYABI SDK..."

SDK_DIR="${SDK_DIR:-sdk}"
BUILD_DIR="${BUILD_DIR:-build}"
ZIP_NAME="${ZIP_NAME:-MIYABI_SDK.zip}"
VALIDATE_ONLY="${MIYABI_SDK_VALIDATE_ONLY:-0}"
PATHS_RS_FILE="logic/src/paths.rs"
PATHS_RS_BACKUP=""

REQUIRED_ARTIFACTS=(
    "bin/miyabi"
    "lib/libmiyabi_logic.a"
    "lib/libmiyabi_logic_cxx.a"
    "lib/libmiyabi_runtime.a"
    "lib/libbox2d.a"
    "include/miyabi/miyabi.h"
    "include/miyabi/bridge.h"
    "cmake/MIYABIConfig.cmake"
    "cmake/MIYABIConfigVersion.cmake"
    "examples/main.cpp"
    "docs/SDK_DEFINITION.md"
    "template_CMakeLists.txt"
)

restore_paths_rs() {
    if [ -n "${PATHS_RS_BACKUP}" ] && [ -f "${PATHS_RS_BACKUP}" ]; then
        cp "${PATHS_RS_BACKUP}" "${PATHS_RS_FILE}"
        rm -f "${PATHS_RS_BACKUP}"
    fi
}

validate_required_artifacts() {
    local missing=()
    local rel=""
    for rel in "${REQUIRED_ARTIFACTS[@]}"; do
        if [ ! -e "${SDK_DIR}/${rel}" ]; then
            missing+=("${rel}")
        fi
    done

    if [ "${#missing[@]}" -gt 0 ]; then
        echo "ERROR: Required SDK artifacts are missing:"
        for rel in "${missing[@]}"; do
            echo "  - ${rel}"
        done
        return 1
    fi

    echo "Required SDK artifacts check passed."
}

if [ "${VALIDATE_ONLY}" = "1" ]; then
    validate_required_artifacts
    exit 0
fi

if [ -f "${PATHS_RS_FILE}" ]; then
    PATHS_RS_BACKUP="$(mktemp "${TMPDIR:-/tmp}/miyabi-paths-rs.XXXXXX")"
    cp "${PATHS_RS_FILE}" "${PATHS_RS_BACKUP}"
    trap restore_paths_rs EXIT
fi

# Clean up previous SDK directory
if [ -d "$SDK_DIR" ]; then
    echo "Removing previous sdk directory..."
    rm -rf "$SDK_DIR"
fi
if [ -f "$ZIP_NAME" ]; then
    echo "Removing previous sdk zip..."
    rm -f "$ZIP_NAME"
fi
if [ -d "$BUILD_DIR" ]; then
    echo "Removing previous build directory..."
    rm -rf "$BUILD_DIR"
fi


# 1. Configure CMake for Release build
cmake -S . -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release -DMIYABI_PERFORMANCE_TEST=ON

# 2. Build the project
cmake --build "$BUILD_DIR"

# 3. Create SDK directory structure
echo "Creating SDK directory..."
mkdir -p "$SDK_DIR"/bin
mkdir -p "$SDK_DIR"/lib
mkdir -p "$SDK_DIR"/include/miyabi
mkdir -p "$SDK_DIR"/cmake
mkdir -p "$SDK_DIR"/docs
mkdir -p "$SDK_DIR"/examples

# 4. Copy runtime executable
echo "Copying runtime executable..."
cp "$BUILD_DIR"/core/miyabi "$SDK_DIR"/bin/

# 5. Copy static libraries
echo "Copying static libraries..."
cp "$BUILD_DIR"/logic/libmiyabi_logic.a "$SDK_DIR"/lib/
cp "$BUILD_DIR"/logic/libmiyabi_logic_cxx.a "$SDK_DIR"/lib/
cp "$BUILD_DIR"/core/libmiyabi_runtime.a "$SDK_DIR"/lib/

BOX2D_LIB="$(find "$BUILD_DIR" -name 'libbox2d.a' -print -quit)"
if [ -z "$BOX2D_LIB" ]; then
    echo "ERROR: libbox2d.a not found under $BUILD_DIR"
    exit 1
fi
cp "$BOX2D_LIB" "$SDK_DIR"/lib/

# 6. Copy headers
echo "Copying headers..."
cp -R core/include/miyabi/* "$SDK_DIR"/include/miyabi/
cp -R "$BUILD_DIR"/logic/corrosion_generated/cxxbridge/miyabi_logic_cxx/include/* "$SDK_DIR"/include/

# 7. Copy CMake package config
echo "Copying CMake package config..."
cp cmake/sdk-package/MIYABIConfig.cmake "$SDK_DIR"/cmake/
cp cmake/sdk-package/MIYABIConfigVersion.cmake "$SDK_DIR"/cmake/

# 8. Copy runtime assets and template CMake file
echo "Copying runtime assets..."
cp -R assets "$SDK_DIR"/

echo "Copying template CMakeLists.txt..."
cp sdk_template_CMakeLists.txt "$SDK_DIR"/template_CMakeLists.txt

echo "Copying template source and SDK docs..."
cp sdk_template_main.cpp "$SDK_DIR"/examples/main.cpp
cp docs/SDK_DEFINITION.md "$SDK_DIR"/docs/SDK_DEFINITION.md

validate_required_artifacts

# Verify required SDK artifacts against docs/SDK_DEFINITION.md
echo "Checking required SDK artifacts..."
./scripts/check_sdk_artifacts.sh "$SDK_DIR"

# 10. Create Zip archive
echo "Creating SDK archive..."
# Remove Finder metadata before packaging to keep artifacts reproducible.
find "$SDK_DIR" -type f -name '.DS_Store' -delete
zip -r "$ZIP_NAME" "$SDK_DIR" -x '*/.DS_Store'

echo "SDK Build complete. Packaged into $ZIP_NAME"
