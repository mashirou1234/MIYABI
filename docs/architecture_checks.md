# Architecture Checks

## core -> sample_game 依存禁止チェック

- 目的: `core/` から `sample_game` への依存逆流を早期検知する。
- 実行コマンド: `./scripts/check_core_no_sample_game_dependency.sh`
- 検査対象: `core/` 配下のテキストファイル（`rg` で検索可能なファイル）
- 検査条件: `sample_game` 文字列を含む行を禁止参照として扱う
- 除外条件:
  - `core/` 外のファイルは対象外
  - バイナリファイルは `rg` の既定動作で対象外

注意: 文字列一致ベースのため、文脈に依存する誤検知/見逃しの可能性がある。
