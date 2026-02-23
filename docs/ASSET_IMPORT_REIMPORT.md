# MIYABI アセット import/reimport 手順（Texture）

最終更新: 2026-02-23

## 1. 対象

本手順は、2Dサンプルゲームで利用する **Textureアセット** を対象とする。

## 2. import 手順（新規取り込み）

1. 画像ファイルを `assets/` 配下へ配置する
2. ロジック側で `asset_server.load_texture("assets/<file>")` を呼び出す
3. 実行時に `AssetCommandType::LoadTexture` が発行され、C++側でGPUアップロードされる

補足:
- 同一パスの重複 `load_texture` はハンドル再利用になる
- 初回取り込み時に `asset_id` を採番し、`path -> texture_handle -> asset_id` の対応を登録する

## 3. reimport 手順（差し替え反映）

### 3.1 実行時 reimport

1. `assets/` 内の既存テクスチャファイルを上書きする
2. ゲーム実行中に `U` キーを押す
3. ロジック側で `AssetCommandType::ReloadTexture` を発行
4. C++側で同じ `texture_id` に対して再アップロードする

### 3.2 期待される挙動

- マテリアルが保持している既存ハンドルは不変
- 対応するGPUテクスチャ内容のみ更新される
- `texture_id` の再採番は行わない

## 4. アセットID管理と参照整合チェック

- レジストリは以下の相互参照を保持する
  - `texture_handle_map`: `path -> texture_handle`
  - `texture_path_map`: `texture_handle -> path`
  - `texture_asset_id_map`: `texture_handle -> asset_id`
  - `asset_id_path_map`: `asset_id -> path`
- ロジック更新中に 30 フレーム間隔で参照整合チェックを実行する
  - マテリアルが参照する `texture_handle` を収集する
  - レジストリ不整合（相互参照崩れ）を検出する
  - 未登録 `texture_handle` を検出する
  - `texture_map` 未解決かつ pending 要求なしの場合に reimport を自動再キューする

## 5. 失敗時の診断ログ

- レジストリ不整合:
  - `[asset] integrity: registry inconsistency detected`
- 未登録ハンドル参照:
  - `[asset] integrity: missing registry for texture_handle=<handle>`
- 未解決参照の再キュー:
  - `[asset] integrity: unresolved reference handle=<handle> asset_id=<id> path=<path> queued_reimport=<true|false>`

## 6. 復旧手順（運用）

1. ログに `missing registry` が出た場合:
   - 該当マテリアル生成時の `texture_handle` 設定を確認する
   - `load_texture` を経由しない手動ハンドル注入がないか確認する
2. ログに `unresolved reference` が継続する場合:
   - まず `U` キーで全テクスチャ reimport を実行する
   - 対象 `path` の実ファイル存在と読み取り権限を確認する
   - 画像破損の可能性がある場合は元アセットへ差し戻す
3. `registry inconsistency detected` が継続する場合:
   - `AssetServer` のマップ更新処理（load/reimport周辺）を点検する
   - 必要に応じて再起動してレジストリを再構築する

## 7. 失敗時挙動

- 読み込み失敗時はコンソールにエラーを出力する
- 既存テクスチャがある場合は既存表示を維持する
- ロジック側の要求は応答済みとして消費される（再試行は再度 `U` 押下）

## 8. 現在の制約

- 対象はTextureのみ
- BGM/SEなど音声アセットのreimportは未対応
- 自動ファイル監視（watcher）による再読込は未対応

## 9. 関連実装

- `logic/src/lib.rs`
  - `AssetCommandType::{LoadTexture, ReloadTexture}`
  - `AssetServer::reimport_all_textures()`
  - `AssetServer::{path_for_texture_handle, asset_id_for_texture_handle, is_registry_consistent}`
  - `Game::run_asset_integrity_check()`
  - `U` キーショートカット
- `core/src/renderer/TextureManager.cpp`
  - `load_texture()`
  - `reload_texture()`
- `core/src/main.cpp`
  - `AssetCommandType` の処理分岐
