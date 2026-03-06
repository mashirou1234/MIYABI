# MIYABI Core Save Subsystem 定義

最終更新: 2026-03-06

## 1. 目的

ゲーム固有仕様と分離した、再利用可能な保存/復元サブシステムを提供する。  
本ドキュメントはコア機能としての保存契約を定義する。

## 2. 実装スコープ（Phase: 定義）

実装モジュール:

- `logic/src/save.rs`

提供API:

- `save::save_to_path(path, data)`
  - JSON保存
  - 原子的書き込み（tmp -> rename）
  - 保存先ディレクトリ自動作成
- `save::load_or_default(path)`
  - 保存ファイルがない場合: `Default` 値を返す
  - JSON破損時: `*.bak` へ退避し `Default` 値を返す
  - バージョン不一致時: `VersionMismatch` エラーを返す
- `save::SAVE_SCHEMA_VERSION`
  - 保存スキーマのバージョン定数

## 3. データ契約

保存データは必ず以下の envelope 形式で扱う。

```json
{
  "save_version": 1,
  "payload": { "...": "..." }
}
```

- `save_version`: スキーマ互換性判定用
- `payload`: ゲーム固有データ

## 4. エラーハンドリング契約

- ファイル未存在: 失敗扱いにせず `Default` 値で開始
- JSON破損: ファイルを `*.bak` へ退避後、`Default` 値で開始
  - 既存の `*.bak` が存在する場合は上書きせず、`*.bak.1`, `*.bak.2` ... の空き番号へ退避する
  - 退避成功後、元の破損ファイルは元パスに残らない
- バージョン不一致: `VersionMismatch` として呼び出し側へ返却
  - 自動移行はこの段階では行わない

## 5. テスト

`logic/src/save.rs` にユニットテストを実装済み。

- round trip（save->load）
- missing file -> default
- corrupt file -> backup + default
- corrupt file + 既存 `*.bak` -> 連番バックアップへ退避

## 6. ゲーム接続状況（2026-02-23）

実装済み:

1. `SaveData`（progress/settings）構造体を実装
2. 起動時 `load_or_default` を実行し反映
3. リザルト遷移時とアプリ終了時に `save_to_path` を実行
4. 失敗時ログ出力（`eprintln!`）を追加
5. 設定UI（Title / Pause）を実装し、設定変更時に `save_to_path` を実行
6. 設定値のランタイム適用を接続
   - 音量設定: `master/bgm/se` をC++オーディオへ反映
   - fullscreen: 設定変更で実ウィンドウモードを切替

未実装:

1. 設定変更失敗時のUI通知（現状はログ出力のみ）

## 7. セーブ失敗時の復旧手順（運用）

本手順は `save::save_to_path` または `save::load_or_default` が失敗/異常を返した場合の最小復旧フローとする。

### 7.1 判定条件

- `save::save_to_path` が `Err` を返す
- `save::load_or_default` が `VersionMismatch` を返す
- 起動後に `save/save_data.json` が読み込まれず、`Default` 値にフォールバックしたことをログで確認した

### 7.2 復旧フロー

1. 失敗種別を特定する（保存失敗 / JSON破損 / バージョン不一致）。
2. `save/` 配下を確認し、`save_data.json` と `*.bak` 系ファイルの有無を記録する。
3. JSON破損で `Default` 起動になった場合は、最新の `*.bak` を退避コピーして解析用に保存する。
4. バージョン不一致の場合は自動移行せず、`SAVE_SCHEMA_VERSION` と対象ファイルの `save_version` を比較して互換性判断を記録する。
5. 復旧後に最小回帰として以下を実行し、再発しないことを確認する。
   - `cargo test --manifest-path logic/Cargo.toml save::tests::load_version_mismatch_returns_error save::tests::load_corrupt_file_uses_next_backup_when_bak_exists`

### 7.3 記録ルール

- 復旧対応では「失敗種別」「退避ファイル名」「実施した回帰コマンド」「結果」を 1 セットで記録する。
- 仕様判断で迷う場合は `README.md` の「セーブ互換性チェック（最小手順）」と `docs/CORE_DEVELOPMENT_TRACK.md` の DoD 記録要件を優先する。
