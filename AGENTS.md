# Codex Project Agent Notes (GitHub)

- 日本語で簡潔に報告する
- まず `artifacts/` を確認して前回結果を把握する
- 手動で整形した開発 Issue は `codex:order` を付ける
- `scripts/orch.sh` を使う自動レーンでは必ず 1 run 1 issue を守る
- 自動レーンに投入する Issue だけを `codex:queue` に入れる
- 手動レーン（Power User の直接対応）は一括処理可。ただし `codex:queue` 系ラベル遷移の整合は維持する
- 失敗時は issue コメントと `codex:blocked` を付与する
