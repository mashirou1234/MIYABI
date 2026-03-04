# logic 公開API棚卸し（Bench-80 #03）

最終更新: 2026-02-28
更新責任: `logic/src/lib.rs` の `pub` 追加・削除時に本書を同時更新する

## 対象と抽出方法

- 対象ファイル: `logic/src/lib.rs`
- 抽出コマンド: `rg "^pub " logic/src/lib.rs`
- 本書は「現状の公開面」を記録し、即時の可視性変更は行わない

## 公開面一覧（`rg "^pub "` 準拠）

### A. SDK契約上の公開維持（ABI/FFIに直結）

- `logic/src/lib.rs:18` `pub struct RenderableObjectSlice`
- `logic/src/lib.rs:24` `pub struct AssetCommandSlice`
- `logic/src/lib.rs:30` `pub struct TextCommandSlice`
- `logic/src/lib.rs:36` `pub struct MiyabiVTable`
- `logic/src/lib.rs:58` `pub extern "C" fn get_miyabi_vtable() -> MiyabiVTable`
- `logic/src/lib.rs:96` `pub mod ffi`
- `logic/src/lib.rs:216` `pub enum GameState`
- `logic/src/lib.rs:864` `pub struct Game`
- `logic/src/lib.rs:2218` `pub extern "C" fn create_game() -> *mut Game`
- `logic/src/lib.rs:2223` `pub extern "C" fn destroy_game(game: *mut Game)`
- `logic/src/lib.rs:2234` `pub extern "C" fn serialize_game(game: *const Game) -> *mut c_char`
- `logic/src/lib.rs:2244` `pub extern "C" fn deserialize_game(json: *const c_char) -> *mut Game`
- `logic/src/lib.rs:2264` `pub extern "C" fn free_serialized_string(s: *mut c_char)`
- `logic/src/lib.rs:2273` `pub extern "C" fn update_game(game: *mut Game) -> GameState`
- `logic/src/lib.rs:2283` `pub extern "C" fn get_renderables(game: *mut Game) -> RenderableObjectSlice`
- `logic/src/lib.rs:2298` `pub extern "C" fn get_asset_commands(game: *mut Game) -> AssetCommandSlice`
- `logic/src/lib.rs:2313` `pub extern "C" fn clear_asset_commands(game: *mut Game)`
- `logic/src/lib.rs:2322` `pub extern "C" fn notify_asset_loaded(game: *mut Game, request_id: u32, asset_id: u32)`
- `logic/src/lib.rs:2335` `pub extern "C" fn update_input_state(game: *mut Game, input: *const ffi::InputState)`
- `logic/src/lib.rs:2345` `pub extern "C" fn get_asset_command_path_cstring(command: *const ffi::AssetCommand) -> *mut c_char`
- `logic/src/lib.rs:2354` `pub extern "C" fn get_text_commands(game: *mut Game) -> TextCommandSlice`
- `logic/src/lib.rs:2369` `pub extern "C" fn get_text_command_text_cstring(command: *const ffi::TextCommand) -> *mut c_char`
- `logic/src/lib.rs:2378` `pub extern "C" fn free_cstring(s: *mut c_char)`

補足:
- SDK利用契約（`docs/SDK_DEFINITION.md`）の起点は `get_miyabi_vtable()`。
- 上記 `extern "C"` 群は VTable 経由で利用されるため、公開維持対象。

### B. 内部化候補（段階的に `pub(crate)` 等へ縮小）

- `logic/src/lib.rs:2` `pub mod perf`
- `logic/src/lib.rs:3` `pub mod save`
- `logic/src/lib.rs:14` `pub mod ui`
- `logic/src/lib.rs:79` `pub trait Component`
- `logic/src/lib.rs:84` `pub enum ComponentType`
- `logic/src/lib.rs:227` `pub struct SaveProgress`
- `logic/src/lib.rs:246` `pub struct SaveSettings`
- `logic/src/lib.rs:265` `pub struct SaveData`
- `logic/src/lib.rs:296` `pub struct AssetServer`
- `logic/src/lib.rs:309` `pub struct PendingAssetRequest`
- `logic/src/lib.rs:424` `pub struct Material`
- `logic/src/lib.rs:439` `pub struct Player`
- `logic/src/lib.rs:445` `pub struct Sprite`
- `logic/src/lib.rs:451` `pub struct Obstacle`
- `logic/src/lib.rs:457` `pub struct PhysicsBody`
- `logic/src/lib.rs:465` `pub struct Entity(pub u64)`
- `logic/src/lib.rs:470` `pub struct Archetype`
- `logic/src/lib.rs:488` `pub struct InternalWorld`
- `logic/src/lib.rs:623` `pub trait ComponentBundle`
- `logic/src/lib.rs:916` `pub type World = Game`

補足:
- これらは現状 `logic` クレート内部実装の都合で公開されている要素が中心。
- `Game` は FFIシグネチャ上の型名として公開維持が必要だが、`Game` の公開フィールドは内部化余地がある。

## 在庫更新時チェックリスト（API追加/削除時）

- `logic/src/lib.rs` の `pub` 宣言差分を `rg "^pub " logic/src/lib.rs` で再抽出し、本書の A/B 一覧と行番号・識別子が一致していること。
- `extern "C"` 関数を追加・削除した場合、`docs/SDK_DEFINITION.md` の「3. 公開エントリポイント」と整合していること。
- `get_miyabi_vtable()` の契約に影響する変更（関数ポインタ追加/削除/型変更）がある場合、`docs/SDK_DEFINITION.md` の「9. ABI更新ポリシー」に沿って ABI バージョン更新要否を確認すること。
- A（公開維持）/B（内部化候補）の分類理由を追記し、変更理由を PR 説明または Issue コメントで追跡できること。
- FFI 文字列管理や所有権規約に関わる変更がある場合、`docs/DESIGN_FFI.md` のメモリルール参照先を確認すること。

## SDK定義書との対応関係

| 公開API棚卸しの観点 | SDK定義書の参照先 | 確認内容 |
| --- | --- | --- |
| `get_miyabi_vtable()` を起点とする公開契約 | `docs/SDK_DEFINITION.md` 3.1 | ロジックAPI起点の説明と棚卸しA群（公開維持）が一致すること |
| ランタイム連携の境界 | `docs/SDK_DEFINITION.md` 3.2 / 6 | `update_game` など実行フロー上の公開関数が最小実行契約から追跡可能であること |
| ABI互換判定と更新判断 | `docs/SDK_DEFINITION.md` 3.3 / 9 | APIシグネチャ変更時に ABI 判定定数と更新ルールの確認漏れがないこと |
| SDK配布物への反映 | `docs/SDK_DEFINITION.md` 4 | 公開ヘッダ/ライブラリ構成に影響する変更が配布物定義と矛盾しないこと |

## 次Issue向け TODO（着手粒度）

- `TODO-1`: `perf/save/ui` の公開要否を評価し、`pub(crate)` 化可能な最小パッチを分割する。
- `TODO-2`: ECS内部型（`Entity`/`Archetype`/`InternalWorld`/`ComponentBundle`）の可視性縮小影響を `cargo test` で検証する。
- `TODO-3`: `Game` の公開フィールドを段階的に非公開化し、FFI経由アクセスのみを許可する設計案を作成する。
- `TODO-4`: `ffi` モジュール内で外部公開が不要な型の範囲を `cxx::bridge` 制約込みで再評価する。

## 確認ログ

- `rg "^pub " logic/src/lib.rs` と本書の宣言一覧が一致することを確認済み（2026-02-28）。
