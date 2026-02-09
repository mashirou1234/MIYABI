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
cmake -S . -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release

# 2. Build the project
cmake --build "$BUILD_DIR"

# 3. Create SDK directory structure
echo "Creating SDK directory..."
mkdir -p "$SDK_DIR"/lib
mkdir -p "$SDK_DIR"/include/miyabi

# 4. Copy libraries
echo "Copying libraries..."
cp "$BUILD_DIR"/core/libsample_game.dylib "$SDK_DIR"/lib/
cp "$BUILD_DIR"/core/libmiyabi_logic.a "$SDK_DIR"/lib/
cp "$BUILD_DIR"/core/libmiyabi_cxxbridge.a "$SDK_DIR"/lib/

# 5. Copy headers
echo "Copying headers..."
cp -R core/include/miyabi/* "$SDK_DIR"/include/miyabi/
cp -R "$BUILD_DIR"/core/corrosion_generated/cxxbridge/miyabi_cxxbridge/include/* "$SDK_DIR"/include/

# 6. Copy template CMake file
echo "Copying template CMakeLists.txt..."
cp sdk_template_CMakeLists.txt "$SDK_DIR"/template_CMakeLists.txt

# 7. Create Zip archive
echo "Creating SDK archive..."
zip -r "$ZIP_NAME" "$SDK_DIR"

echo "SDK Build complete. Packaged into $ZIP_NAME"
