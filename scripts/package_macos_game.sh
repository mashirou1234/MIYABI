#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$ROOT_DIR/build_release_game"
DIST_ROOT="$ROOT_DIR/dist"
PACKAGE_DIR="$DIST_ROOT/miyabi_game_macos"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
ARCHIVE_NAME="MIYABI_GAME_macOS_${TIMESTAMP}.zip"
ARCHIVE_PATH="$DIST_ROOT/$ARCHIVE_NAME"

echo "[package] root: $ROOT_DIR"
echo "[package] build dir: $BUILD_DIR"
echo "[package] output: $ARCHIVE_PATH"

rm -rf "$BUILD_DIR" "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR/bin" "$PACKAGE_DIR/docs"

echo "[package] configure release build"
cmake -S "$ROOT_DIR" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release -DMIYABI_PERFORMANCE_TEST=OFF

echo "[package] build"
cmake --build "$BUILD_DIR" -j4

if [ ! -x "$BUILD_DIR/core/miyabi" ]; then
  echo "[package] ERROR: $BUILD_DIR/core/miyabi not found"
  exit 1
fi

echo "[package] copy runtime"
cp "$BUILD_DIR/core/miyabi" "$PACKAGE_DIR/bin/miyabi"
cp -R "$ROOT_DIR/assets" "$PACKAGE_DIR/assets"

cat > "$PACKAGE_DIR/run_miyabi.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
exec ./bin/miyabi
EOF
chmod +x "$PACKAGE_DIR/run_miyabi.sh"

cat > "$PACKAGE_DIR/docs/README.txt" <<'EOF'
MIYABI Game Distribution (macOS)

1. このフォルダを任意の場所へ展開してください。
2. ターミナルで展開先へ移動します。
3. 次のコマンドで起動します:
   ./run_miyabi.sh

補足:
- 実行時に assets ディレクトリを相対参照します。必ずフォルダ構成を保持してください。
EOF

echo "[package] create checksum"
(
  cd "$PACKAGE_DIR"
  shasum -a 256 bin/miyabi assets/player.png assets/test.png assets/test_sound.wav > SHA256SUMS.txt
)

echo "[package] archive"
mkdir -p "$DIST_ROOT"
(
  cd "$DIST_ROOT"
  zip -r "$ARCHIVE_NAME" "$(basename "$PACKAGE_DIR")"
)

echo "[package] complete: $ARCHIVE_PATH"
