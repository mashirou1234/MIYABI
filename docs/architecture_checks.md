# Architecture Checks

この文書は、依存境界チェックを段階的に増やすための運用テンプレートです。
新しいチェックを追加するときは、下記テンプレートをコピーして項目を埋めてください。

## ローカル再現ガイド（所要 10 分目安）

### 0. 前提確認（1 分）

- 実行場所: リポジトリルート（`MIYABI/`）
- 必須コマンド:
  - `bash`
  - `rg`（ripgrep）

確認コマンド:

```bash
pwd
rg --version
```

### 1. チェック実行（1 分）

```bash
./scripts/check_core_no_sample_game_dependency.sh
echo "exit_code=$?"
```

- `exit_code=0`: パス（境界違反なし）
- `exit_code=1`: 失敗（`core/` 内に `sample_game` 参照あり）
- `exit_code=2`: 実行環境エラー（`rg` 未導入）

### 2. 失敗時の切り分け（3 分）

```bash
rg --line-number --no-heading --color=never --fixed-strings "sample_game" core
```

- 上記コマンドは `scripts/check_core_no_sample_game_dependency.sh` 内部と同一条件です。
- 出力された行を修正対象として扱います。

### 3. 修正後の確認（1 分）

```bash
./scripts/check_core_no_sample_game_dependency.sh
echo "exit_code=$?"
```

`exit_code=0` になるまで修正と再実行を繰り返します。

## 典型的な失敗パターン（既存ルール名: `core -> sample_game 依存禁止チェック`）

### 失敗例1: `core/` の C++ ソースで `sample_game` 名を直接参照

- 例:
  - `core/src/*.cpp` に `"sample_game"` を含むログ/コメント/文字列定数を追加してしまう
- 原因:
  - 境界ルール上、`core` はゲーム層（`sample_game`）を知らない前提のため
- 最小修正方針:
  - `sample_game` という具体名を削除し、`game module` など中立な表現へ置換する
  - 参照自体が必要なら `logic` 側 API を経由する設計へ寄せる

### 失敗例2: `core/` のビルド設定で `sample_game` 依存を導入

- 例:
  - `core/CMakeLists.txt` に `sample_game` 文字列を含むリンク設定やコメントを追加してしまう
- 原因:
  - `core` から `sample_game` への依存逆流（依存方向違反）
- 最小修正方針:
  - `core` 側から `sample_game` 参照を削除する
  - 必要な依存は `sample_game -> logic -> core` の方向で再設計する

## 更新責任範囲

- スクリプト更新時（`scripts/check_core_no_sample_game_dependency.sh`）:
  - 変更 PR の作成者が同時に本ドキュメントを更新する
- ルール変更レビュー時:
  - レビュアは「コマンド一致」と「失敗時対応の妥当性」を確認する
- 目安:
  - CI で当該チェックが失敗した場合、同一または次の PR で本書を追従更新する

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
