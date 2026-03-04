# Architecture Checks

この文書は、依存境界チェックを段階的に増やすための運用テンプレートです。
新しいチェックを追加するときは、下記テンプレートをコピーして項目を埋めてください。

## 追加テンプレート

### <チェック名>

- 目的:
- 実行コマンド:
- 対象:
- 除外:
- 失敗時対応:

## 登録済みチェック

### core -> sample_game 依存禁止チェック

- 目的: `core/` から `sample_game` への依存逆流を早期検知する。
- 実行コマンド: `./scripts/check_core_no_sample_game_dependency.sh`
- 対象: `core/` 配下のテキストファイル（`rg` で検索可能なファイル）
- 除外:
  - `core/` 外のファイルは対象外
  - バイナリファイルは `rg` の既定動作で対象外
- 失敗時対応:
  - 出力された該当行を削除・置換し、依存方向を `core -> sample_game` から切り離す
  - 修正後に再実行し、`[OK]` になることを確認する
  - 仕様上必要な参照だった場合は、依存境界の見直しを issue 化して設計判断を残す

実行例（違反あり）:

```text
[NG] core/ から sample_game への参照を検知しました。
[NG] 該当箇所 (path:line):
- core/render/example.cpp:42 | #include "sample_game/foo.h"
[HINT] core/ から sample_game 参照を除去後、同コマンドを再実行してください。
```

注意: 文字列一致ベースのため、文脈に依存する誤検知/見逃しの可能性がある。

## scripts 参照導線

- 既存チェック実装: `scripts/check_core_no_sample_game_dependency.sh`
- 新規チェックを増やす場合も、`scripts/` 配下に追加して本ドキュメントへ対応項目を追記する
