#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$ROOT_DIR/build_release_game"
DIST_ROOT="$ROOT_DIR/dist"
PACKAGE_DIR="$DIST_ROOT/miyabi_game_macos"
TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
ARCHIVE_NAME="MIYABI_GAME_macOS_${TIMESTAMP}.zip"
ARCHIVE_PATH="$DIST_ROOT/$ARCHIVE_NAME"
PRECHECK_ONLY=0

if [ "${1:-}" = "--preflight-only" ]; then
  PRECHECK_ONLY=1
elif [ $# -gt 0 ]; then
  echo "[package] ERROR: unknown option: $1"
  echo "usage: $0 [--preflight-only]"
  exit 2
fi

require_command() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "[preflight] ERROR: required command not found: $cmd"
    return 1
  fi
}

preflight_diagnostics() {
  local failed=0
  echo "[preflight] start"

  for cmd in cmake zip shasum; do
    require_command "$cmd" || failed=1
  done

  if [ ! -d "$ROOT_DIR/assets" ]; then
    echo "[preflight] ERROR: assets directory not found: $ROOT_DIR/assets"
    failed=1
  fi

  for asset in player.png test.png test_sound.wav; do
    if [ ! -f "$ROOT_DIR/assets/$asset" ]; then
      echo "[preflight] ERROR: required asset not found: $ROOT_DIR/assets/$asset"
      failed=1
    fi
  done

  if [ ! -w "$ROOT_DIR" ]; then
    echo "[preflight] ERROR: root directory is not writable: $ROOT_DIR"
    failed=1
  fi

  mkdir -p "$DIST_ROOT"
  if [ ! -w "$DIST_ROOT" ]; then
    echo "[preflight] ERROR: dist directory is not writable: $DIST_ROOT"
    failed=1
  fi

  if [ "$failed" -ne 0 ]; then
    echo "[preflight] failed"
    return 1
  fi

  echo "[preflight] ok"
}

echo "[package] root: $ROOT_DIR"
echo "[package] build dir: $BUILD_DIR"
echo "[package] output: $ARCHIVE_PATH"

preflight_diagnostics
if [ "$PRECHECK_ONLY" -eq 1 ]; then
  echo "[package] preflight-only mode: build/packaging skipped"
  exit 0
fi

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
cp -R "$ROOT_DIR/core/src/shaders" "$PACKAGE_DIR/shaders"

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
  shasum -a 256 \
    bin/miyabi \
    assets/player.png \
    assets/test.png \
    assets/test_sound.wav \
    shaders/text.vert \
    shaders/text.frag \
    shaders/textured.vert \
    shaders/textured.frag \
    shaders/lit_textured.vert \
    shaders/lit_textured.frag > SHA256SUMS.txt
)

echo "[package] archive"
mkdir -p "$DIST_ROOT"
(
  cd "$DIST_ROOT"
  zip -r "$ARCHIVE_NAME" "$(basename "$PACKAGE_DIR")"
)

echo "[package] complete: $ARCHIVE_PATH"
