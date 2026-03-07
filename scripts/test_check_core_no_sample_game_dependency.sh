#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

TMP_REL_PATH="core/tmp_sample_game_dependency_regression_test.txt"
cleanup() {
  rm -f "${ROOT_DIR}/${TMP_REL_PATH}"
}
trap cleanup EXIT

echo "[test] regression: current repository passes dependency check"
bash ./scripts/check_core_no_sample_game_dependency.sh

echo "[test] minimal reproduction: file including sample_game should fail"
cat > "${ROOT_DIR}/${TMP_REL_PATH}" <<'EOF'
sample_game dependency marker for regression test
EOF

set +e
bash ./scripts/check_core_no_sample_game_dependency.sh > /tmp/miyabi_core_dep_check.log 2>&1
STATUS=$?
set -e

if [ "${STATUS}" -eq 0 ]; then
  echo "Expected check_core_no_sample_game_dependency.sh to fail" >&2
  cat /tmp/miyabi_core_dep_check.log >&2
  exit 1
fi

if [ "${STATUS}" -ne 1 ]; then
  echo "Expected exit code 1, got ${STATUS}" >&2
  cat /tmp/miyabi_core_dep_check.log >&2
  exit 1
fi

grep -Fq "${TMP_REL_PATH}" /tmp/miyabi_core_dep_check.log
echo "[test] PASS"
