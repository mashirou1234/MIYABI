# MIYABI Core Save Subsystem 定義

最終更新: 2026-02-23

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
- バージョン不一致: `VersionMismatch` として呼び出し側へ返却
  - 自動移行はこの段階では行わない

## 5. テスト

`logic/src/save.rs` にユニットテストを実装済み。

- round trip（save->load）
- missing file -> default
- corrupt file -> backup + default

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
