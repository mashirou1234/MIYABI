#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# core 配下のみを検査対象にし、バイナリは無視して文字列一致で禁止参照を検知する。
if ! command -v rg >/dev/null 2>&1; then
  echo "[ERROR] ripgrep (rg) が見つかりません。" >&2
  exit 2
fi

MATCHES="$(rg --vimgrep --no-heading --color=never --fixed-strings "sample_game" core || true)"

if [[ -n "$MATCHES" ]]; then
  echo "[NG] core/ から sample_game への参照を検知しました。"
  echo "[NG] 該当箇所 (path:line):"
  while IFS= read -r match; do
    path="${match%%:*}"
    rest="${match#*:}"
    line_no="${rest%%:*}"
    text="${match#*:*:*:}"
    printf -- "- %s:%s | %s\n" "$path" "$line_no" "$text"
  done <<< "$MATCHES"
  echo "[HINT] core/ から sample_game 参照を除去後、同コマンドを再実行してください。"
  exit 1
fi

echo "[OK] core/ から sample_game への参照は検知されませんでした。"
