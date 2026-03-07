# `sample_game` / `core` 責務境界

最終更新: 2026-03-08

## 1. レイヤーモデルと役割

- `core`（C++ / `core/`）
  - プラットフォーム抽象、レンダリング、入力、音声、物理、アセット I/O などのランタイムを保持する。
  - Rust 側とは `core/include/miyabi/bridge.h` と `logic` が生成する `miyabi_logic_cxx` を介してのみ通信する。
- `logic`（Rust / `logic/`）
  - C++ ランタイム用の FFI、ゲームループ、ECS/シリアライズ/API 契約を提供する SDK コア。
  - 現状は `sample_game` の gameplay / HUD / アセットレジストリの一部を暫定的に内包している。
- `sample_game_runtime`（Rust / `sample_game_runtime/`）
  - `sample_game` と `logic` が共有する sample 固有の state/action/effect 契約を保持する。
  - `logic -> sample_game` の Cargo 循環依存を作らずに runtime boot path を反転するための純粋契約層とする。
- `sample_game`（Rust / `sample_game/`）
  - SDK 利用者（ユーザーゲーム）のモデルケース。
  - `sample_game_runtime` の契約を再エクスポートしつつ、将来的には `logic` が公開する API だけに依存し、`core` と直接リンクしない。

```
┌────────┐   FFI/ABI   ┌─────────┐   runtime contract   ┌─────────────────────┐
│  core  │ ─────────▶  │  logic  │ ◀──────────────────▶ │ sample_game_runtime │
└────────┘             └─────────┘                      └──────────┬──────────┘
                                                                   │ re-export
                                                              ┌────▼─────┐
                                                              │sample_game│
                                                              └───────────┘
```

## 2. 現状の依存方向の整理

- `core` → `logic`: `core/CMakeLists.txt` で `miyabi_logic` (`staticlib`) をリンクし、`miyabi_bridge.cpp` から VTable を呼び出している。
- `logic` → `core`: `logic/src/lib.rs` の `cxx::bridge` で `play_sound` や `create_dynamic_box_body` などランタイム呼び出しを宣言し、`core/include/miyabi/bridge.h` に実装を置いている。
- `logic` → `sample_game_runtime`: `logic/Cargo.toml` から `SampleGameLoop` / `SampleGameEvent` / `SampleGameEffect` を取り込み、実行時遷移を直接 dispatch している。
- `sample_game` → `logic`, `sample_game_runtime`: `sample_game/Cargo.toml` で `miyabi_logic` と `sample_game_runtime` を依存登録しつつ、`sample_game/src/lib.rs` から runtime 契約を再エクスポートしている。
- `sample_game_runtime` ↛ `core` / `logic` 実装詳細: 共有 crate はランタイム実装や FFI を持たず、純粋な契約だけを保持する。
- `core` ↛ `sample_game`: 実行ファイルは `sample_game` をリンクしておらず、ユーザーのゲームコードを静的に組み込まない想定。
- 例外的に `core` は `../logic/src/performance.cpp` を直接ビルドへ含めており、ここが境界の整理対象。

## 3. 境界ルール

### 許可される依存

- `core` → `logic`（FFI 境界）: `get_miyabi_vtable()` を取得してゲームインスタンスを生成し、レンダリング/入力結果を橋渡しする。
- `logic` → `core`（サービス呼び出し）: `bridge.h` に列挙されたランタイム機能（オーディオ、物理、テクスチャロードなど）だけを利用する。
- `logic` → `sample_game_runtime`: `SampleGameLoop` / `SampleGameEvent` / `SampleGameEffect` を介して sample 固有の遷移契約だけを共有する。
- `sample_game` → `logic`: 公開された ECS 型 (`Component`, `RenderableObject` 等)、保存/ロード API (`logic/src/save.rs`)、UI 部品 (`logic/src/ui.rs`) を介してゲームプレイを実装する。
- `sample_game` → `sample_game_runtime`: sample 固有の state/action/effect 契約を再エクスポートし、利用者が 1 つの crate から取り込めるようにする。
- `logic` 内部モジュール間: FFI、安全なユーティリティ、アセット同期処理同士は自由に参照してよい。

### 禁止または抑止する依存

- `core` → `sample_game`: C++ 側が特定ゲームのリソース/状態へ直接アクセスすることを禁止する。サンプル固有の定数は Rust 側で完結させる。
- `sample_game` → `core`: SDK 利用者が `core/include/*` を参照したり `glfw` などを直接リンクするのは不可。ランタイム機能を使いたい場合は `logic` が提供する API を経由する。
- `logic` → `core` 実装詳細: `bridge.h` 以外の C++ ヘッダ/ソースを include しない。GL/GLFW/Freetype などプラットフォームライブラリへの直接依存を作らない。
- `sample_game_runtime` → `logic` / `core`: 共通契約層からランタイム実装へ依存を張らない。純粋な状態機械契約だけを保持する。
- `core` → `logic` 内部ファイル: Rust クレートの `.rs` を C++ から include する、`logic` が持つゲーム状態 (`Game`, `World`, UI) を C++ で改変する、といった境界破りを禁止する。

#### 禁止依存の具体例（NG/代替）

- NG: `core/src/*` から `sample_game/src/*` のゲーム状態を直接参照する。
  代替: `core` は `miyabi_bridge.cpp` 経由で `logic` の公開 API のみを呼ぶ。
- NG: `sample_game` で `core/include/miyabi/bridge.h` を `include` して C++ 関数を直接呼ぶ。
  代替: `sample_game` は `miyabi_logic` と `sample_game_runtime` の公開型・公開関数のみを利用する。
- NG: `logic` から `core/src/renderer/*` など実装ディレクトリへ依存を張る。
  代替: `logic` からのランタイム呼び出しは `core/include/miyabi/bridge.h` の契約に限定する。
- NG: `sample_game_runtime` に `cxx::bridge` や `core` 向け FFI を置く。
  代替: 共通 crate は state/action/effect 契約のみに限定し、ランタイム I/O は `logic` に閉じ込める。
- NG: `core` の CMake に `sample_game` ターゲットを追加し、`target_link_libraries` で直接リンクする。
  代替: 実行時の組み合わせは `core` + `logic` の ABI 境界で接続し、ゲーム固有処理は Rust 側で完結させる。

## 4. 2026-03-08 時点の移行状況

- `sample_game_runtime/src/lib.rs` に `SampleGameState` / `SampleGameButtonAction` / `SampleGameEffect` / `SampleGameLoop` を分離し、`sample_game/src/lib.rs` から再エクスポートする形へ更新した。
- `logic/src/lib.rs` は `SampleGameLoop` を直接保持し、`SampleGameEvent` / `SampleGameEffect` を runtime boot path で実行するようにした。`apply_sample_action_id()` 互換 shim は不要になった。
- `core/src/main.cpp` と `core/src/renderer/MeshManager.*` は、固定透視カメラ + 3D/2D 描画パス分離 + OBJ メッシュ登録を追加し、`core -> sample_game` の依存逆流なしで 3D 最小起動を受けられる形にした。
- `sample_game/tests/flow_contract.rs` と `logic` のテストで、`Start 3D Arena` を含む action id 契約、3D arena 最小起動、XZ 平面移動を固定した。

## 5. 次に分離すべき箇所の指針

- `logic/src/lib.rs`: まだ HUD レンダリング、障害物生成、勝敗判定、保存反映などサンプル固有処理が残る。次段では外部サンプルへの再利用を通じて、共通 API と sample 固有実装の境界をさらに削る。
- `logic` と `sample_game` で共有している `sample.*` action id 文字列は `sample_game_runtime` に集約したが、最終的には `sample_game` 起点の登録 API へ寄せる余地がある。
- `core/CMakeLists.txt` 内の `../logic/src/performance.cpp` 取り込みは、C++ から Rust ディレクトリへアクセスしている唯一の箇所。ビルド成果物へ組み込むなら `logic` 側で `extern "C"` API を提供して `core` はそれを呼ぶ形に合わせる。
- C1 の最小証跡は `sample_game` 以外の外部 SDK サンプル 1 本で configure/build/run を確認することとし、外部サンプル 2 本以上の成立は `Wave 4` で扱う。3D 側は `G4-02` / `G4-03`、C2 側は `C2-03` / `C2-04` が次段になる。

## 6. レビュー時チェック質問（15〜45 分の着手単位）

`sample_game` / `core` の責務境界に関する変更をレビューするときは、以下 4 問を順に確認する。

1. 依存方向
   - `core -> sample_game` または `sample_game -> core` の新規依存が差分に含まれていないか。
2. 境界窓口
   - `logic` からのランタイム呼び出しが `core/include/miyabi/bridge.h` の契約内に収まっているか。
3. 変更根拠
   - PR 本文または関連 Issue に、なぜこの境界変更が必要かを 1 文以上で説明しているか。
4. 検証ログ
   - 最低 1 回、`./scripts/check_core_no_sample_game_dependency.sh` を実行し、結果（成功/失敗と対応）を残しているか。

判定:
- 4 問のうち 1 つでも `No` の場合はマージ保留にする。
- 2. の境界窓口を拡張する場合は、`docs/architecture_checks.md` の「除外設定レビュー手順」と整合する説明を PR に追記する。

このルールを満たしたとき、`core` はプラットフォームとランタイムに専念し、`logic` は SDK/API 群、`sample_game` はユーザーコードのサンプルという役割が明確になる。C1 判定では上記 4 点の分離具合を指標にする。
