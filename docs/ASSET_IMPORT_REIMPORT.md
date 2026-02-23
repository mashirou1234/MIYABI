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

## 4. 失敗時挙動

- 読み込み失敗時はコンソールにエラーを出力する
- 既存テクスチャがある場合は既存表示を維持する
- ロジック側の要求は応答済みとして消費される（再試行は再度 `U` 押下）

## 5. 現在の制約

- 対象はTextureのみ
- BGM/SEなど音声アセットのreimportは未対応
- 自動ファイル監視（watcher）による再読込は未対応

## 6. 関連実装

- `logic/src/lib.rs`
  - `AssetCommandType::{LoadTexture, ReloadTexture}`
  - `AssetServer::reimport_all_textures()`
  - `U` キーショートカット
- `core/src/renderer/TextureManager.cpp`
  - `load_texture()`
  - `reload_texture()`
- `core/src/main.cpp`
  - `AssetCommandType` の処理分岐
