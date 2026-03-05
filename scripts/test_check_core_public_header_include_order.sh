#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/miyabi-public-header-test.XXXXXX")"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

echo "[test] regression: current public headers pass standalone include check"
bash ./scripts/check_core_public_header_include_order.sh

echo "[test] minimal reproduction: header missing cstdint should fail"
mkdir -p "${TMP_DIR}/include/miyabi"
cat > "${TMP_DIR}/include/miyabi/good.h" <<'EOF'
#pragma once
#include <cstdint>
struct GoodHeader {
    uint32_t value;
};
EOF
cat > "${TMP_DIR}/include/miyabi/bad.h" <<'EOF'
#pragma once
struct BadHeader {
    uint32_t value;
};
EOF

set +e
MIYABI_PUBLIC_INCLUDE_ROOT="${TMP_DIR}/include" \
MIYABI_PUBLIC_HEADER_DIR="${TMP_DIR}/include/miyabi" \
bash ./scripts/check_core_public_header_include_order.sh > "${TMP_DIR}/bad.log" 2>&1
STATUS=$?
set -e

if [ "${STATUS}" -eq 0 ]; then
  echo "Expected check_core_public_header_include_order.sh to fail for bad.h" >&2
  exit 1
fi

grep -Fq "bad.h" "${TMP_DIR}/bad.log"
echo "[test] PASS"
