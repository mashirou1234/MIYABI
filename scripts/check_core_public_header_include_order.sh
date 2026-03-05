#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

PUBLIC_INCLUDE_ROOT="${MIYABI_PUBLIC_INCLUDE_ROOT:-core/include}"
PUBLIC_HEADER_DIR="${MIYABI_PUBLIC_HEADER_DIR:-${PUBLIC_INCLUDE_ROOT}/miyabi}"

if ! command -v c++ >/dev/null 2>&1; then
  echo "[ERROR] c++ compiler が見つかりません。" >&2
  exit 2
fi

if [ ! -d "${PUBLIC_HEADER_DIR}" ]; then
  echo "[ERROR] public header directory が見つかりません: ${PUBLIC_HEADER_DIR}" >&2
  exit 2
fi

mapfile -t HEADERS < <(find "${PUBLIC_HEADER_DIR}" -maxdepth 1 -type f -name "*.h" | LC_ALL=C sort)
if [ "${#HEADERS[@]}" -eq 0 ]; then
  echo "[ERROR] 検査対象ヘッダがありません: ${PUBLIC_HEADER_DIR}" >&2
  exit 2
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-public-header-check.XXXXXX")"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

STUB_ROOT="${TMP_DIR}/stubs"
mkdir -p "${STUB_ROOT}/rust" "${STUB_ROOT}/miyabi_logic_cxx"

cat > "${STUB_ROOT}/rust/cxx.h" <<'EOF'
#pragma once
#include <cstddef>
namespace rust {
struct Str {};
template <typename T>
struct Slice {
    const T* ptr;
    std::size_t len;
};
} // namespace rust
EOF

cat > "${STUB_ROOT}/miyabi_logic_cxx/lib.h" <<'EOF'
#pragma once
struct RenderableObject {};
struct AssetCommand {};
struct InputState {};
struct TextCommand {};
EOF

FAILED=0
for header_path in "${HEADERS[@]}"; do
  rel="${header_path#${PUBLIC_INCLUDE_ROOT}/}"
  tu="${TMP_DIR}/check.cpp"
  cat > "${tu}" <<EOF
#include "${rel}"
int main() { return 0; }
EOF

  if ! c++ -std=c++17 -fsyntax-only -I"${STUB_ROOT}" -I"${PUBLIC_INCLUDE_ROOT}" "${tu}" >/dev/null 2>"${TMP_DIR}/compile.log"; then
    FAILED=1
    echo "[NG] include順依存または自己完結性欠如を検知: ${rel}"
    sed 's/^/  /' "${TMP_DIR}/compile.log"
  else
    echo "[OK] ${rel}"
  fi
done

if [ "${FAILED}" -ne 0 ]; then
  echo "[HINT] public header が単体先頭 include で通るように不足 include を追加してください。"
  exit 1
fi

echo "[OK] core/public ヘッダの include順依存は検知されませんでした。"
