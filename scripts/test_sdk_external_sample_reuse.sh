#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/test_sdk_external_sample_reuse.sh [sdk_dir]

Builds and runs the SDK's bundled sample from a temporary external project.
If sdk_dir is omitted, ./sdk is used.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SDK_DIR="${1:-${ROOT_DIR}/sdk}"
TMP_PARENT="${TMPDIR%/}"
if [[ -z "${TMP_PARENT}" || ! -d "${TMP_PARENT}" ]]; then
  TMP_PARENT="/tmp"
fi
TMP_DIR="$(mktemp -d "${TMP_PARENT}/miyabi-sdk-reuse.XXXXXX")"

cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

if [[ ! -d "${SDK_DIR}" ]]; then
  echo "[sdk-reuse] ERROR: SDK directory not found: ${SDK_DIR}" >&2
  exit 1
fi

if [[ ! -f "${SDK_DIR}/template_CMakeLists.txt" ]]; then
  echo "[sdk-reuse] ERROR: missing template_CMakeLists.txt in ${SDK_DIR}" >&2
  exit 1
fi

if [[ ! -f "${SDK_DIR}/examples/main.cpp" ]]; then
  echo "[sdk-reuse] ERROR: missing examples/main.cpp in ${SDK_DIR}" >&2
  exit 1
fi

echo "[sdk-reuse] root: ${ROOT_DIR}"
echo "[sdk-reuse] sdk: ${SDK_DIR}"
echo "[sdk-reuse] temp project: ${TMP_DIR}"

mkdir -p "${TMP_DIR}/src"
cp -R "${SDK_DIR}" "${TMP_DIR}/miyabi_sdk"
cp "${SDK_DIR}/template_CMakeLists.txt" "${TMP_DIR}/CMakeLists.txt"
cp "${SDK_DIR}/examples/main.cpp" "${TMP_DIR}/src/main.cpp"
if [[ -d "${SDK_DIR}/assets" ]]; then
  cp -R "${SDK_DIR}/assets" "${TMP_DIR}/assets"
fi

echo "[sdk-reuse] configure"
cmake -S "${TMP_DIR}" -B "${TMP_DIR}/build"

echo "[sdk-reuse] build"
cmake --build "${TMP_DIR}/build" -j

echo "[sdk-reuse] run"
(
  cd "${TMP_DIR}"
  ./build/my_game
)

echo "[sdk-reuse] PASS: configured, built, and ran external sample via MIYABI SDK"
