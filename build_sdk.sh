#!/bin/bash
set -e

echo "Building MIYABI SDK..."

SDK_DIR="sdk"
BUILD_DIR="build"
ZIP_NAME="MIYABI_SDK.zip"

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

# 7. Copy runtime assets and template CMake file
echo "Copying runtime assets..."
cp -R assets "$SDK_DIR"/

echo "Copying template CMakeLists.txt..."
cp sdk_template_CMakeLists.txt "$SDK_DIR"/template_CMakeLists.txt

echo "Copying template source and SDK docs..."
cp sdk_template_main.cpp "$SDK_DIR"/examples/main.cpp
cp docs/SDK_DEFINITION.md "$SDK_DIR"/docs/SDK_DEFINITION.md

# 8. Create Zip archive
echo "Creating SDK archive..."
zip -r "$ZIP_NAME" "$SDK_DIR"

echo "SDK Build complete. Packaged into $ZIP_NAME"
