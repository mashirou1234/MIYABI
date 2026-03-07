# Architecture Checks

この文書は、依存境界チェックを段階的に増やすための運用テンプレートです。
新しいチェックを追加するときは、下記テンプレートをコピーして項目を埋めてください。
ファイル名 `architecture_checks.md` に合わせ、検索確認では `architecture` 文字列を利用できます。

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

### 2.5 ローカル実行失敗（環境要因）の切り分け（2 分）

`exit_code=2` や `command not found` など、チェック以前で停止した場合は次を確認します。

1. 実行ディレクトリ誤りの確認

```bash
pwd
test -x ./scripts/check_core_no_sample_game_dependency.sh && echo ok || echo ng
```

- `ng` の場合: リポジトリルート（`MIYABI/`）へ移動して再実行します。

2. `rg` 未導入/パス不備の確認

```bash
rg --version
command -v rg
```

- 失敗する場合: `rg` を導入し、シェルを再起動して PATH を反映します。

3. 実行権限の確認

```bash
ls -l ./scripts/check_core_no_sample_game_dependency.sh
```

- `x` 権限が無い場合:

```bash
chmod +x ./scripts/check_core_no_sample_game_dependency.sh
```

4. スクリプト単体の終了コード確認

```bash
bash -x ./scripts/check_core_no_sample_game_dependency.sh
echo "exit_code=$?"
```

- `exit_code=0/1`: チェック自体は実行できています（以降は通常の違反切り分けへ）。
- `exit_code=2`: 環境要因のため、上記 1-3 を再確認します。

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

## 除外設定レビュー手順

`architecture_checks` で除外（対象外パス・条件）を追加/変更する場合は、以下 4 点を PR レビューで確認する。

1. 変更理由の明記
   - PR 説明に「どのチェックの除外か」「なぜ既存ルールで扱えないか」を 1 文以上で記載する。
2. 範囲最小化
   - 除外は最小単位（具体パスまたは限定的なパターン）で定義し、ディレクトリ丸ごと除外は原則避ける。
3. 代替検知の有無
   - 除外によって検知できなくなる論点がある場合、代替の確認手順（別チェック/手動確認）を同時に記載する。
4. 運用整合
   - 手動判断が必要なケースは [`docs/CI_AUTOMERGE.md`](./CI_AUTOMERGE.md) の「6. 自動マージ除外ケース（手動レビュー必須）」と矛盾しないことを確認する。

判定基準:
- 上記 1-4 のうち 1 つでも欠ける場合は「除外設定のレビュー未完了」としてマージしない。

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
- 除外ルール（運用）:
  - このチェックに恒久的な許可リスト（特定パス/文字列の常時除外）を追加しない。
  - 誤検知対処で一時除外が必要な場合は、期限と撤去条件を issue/PR 本文へ明記し、次回リリース前に除外を必ず撤去する。

実行例（違反あり）:

```text
[NG] core/ から sample_game への参照を検知しました。
[NG] 該当箇所 (path:line):
- core/render/example.cpp:42 | #include "sample_game/foo.h"
[HINT] core/ から sample_game 参照を除去後、同コマンドを再実行してください。
```

注意: 文字列一致ベースのため、文脈に依存する誤検知/見逃しの可能性がある。

### core/public ヘッダ include順依存チェック

- 目的: `core/include/miyabi/*.h` が他ヘッダの先行 include に依存しないことを検証する。
- 実行コマンド: `./scripts/check_core_public_header_include_order.sh`
- 対象:
  - `core/include/miyabi/*.h`
  - 各ヘッダを「単体先頭 include」で `c++ -fsyntax-only` 検証
- 除外:
  - `core/include/miyabi/` 以外の private ヘッダ
  - 実体が必要な生成物はスタブで代替（`rust/cxx.h`, `miyabi_logic_cxx/lib.h`）
- 失敗時対応:
  - 失敗ヘッダに不足 include を追加し、単体 include で再実行する
  - 必要であれば forward declaration で依存を縮小する
  - 修正後に同コマンドが `[OK]` になることを確認する

実行例（違反あり）:

```text
[NG] include順依存または自己完結性欠如を検知: miyabi/bad.h
  ... error: unknown type name 'uint32_t'
```

## scripts 参照導線

- 既存チェック実装: `scripts/check_core_no_sample_game_dependency.sh`
- 既存チェック実装: `scripts/check_core_public_header_include_order.sh`
- 新規チェックを増やす場合も、`scripts/` 配下に追加して本ドキュメントへ対応項目を追記する
