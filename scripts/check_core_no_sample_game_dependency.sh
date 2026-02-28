#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# core 配下のみを検査対象にし、バイナリは無視して文字列一致で禁止参照を検知する。
if ! command -v rg >/dev/null 2>&1; then
  echo "[ERROR] ripgrep (rg) が見つかりません。" >&2
  exit 2
fi

MATCHES="$(rg --line-number --no-heading --color=never --fixed-strings "sample_game" core || true)"

if [[ -n "$MATCHES" ]]; then
  echo "[NG] core/ から sample_game への参照を検知しました。"
  echo "$MATCHES"
  exit 1
fi

echo "[OK] core/ から sample_game への参照は検知されませんでした。"
