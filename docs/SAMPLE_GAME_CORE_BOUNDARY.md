# `sample_game` / `core` 責務境界

最終更新: 2026-03-07

## 1. レイヤーモデルと役割

- `core`（C++ / `core/`）  
  - プラットフォーム抽象、レンダリング、入力、音声、物理、アセットI/Oなどのランタイムを保持する。  
  - Rust側とは `core/include/miyabi/bridge.h` と `logic` が生成する `miyabi_logic_cxx` を介してのみ通信する。
- `logic`（Rust / `logic/`）  
  - C++ランタイム用の FFI、ゲームループ、ECS/シリアライズ/API契約を提供する SDK コア。  
  - 現状は `sample_game` の状態機械・UI・アセットレジストリも暫定的に内包している。
- `sample_game`（Rust / `sample_game/`）  
  - SDK利用者（ユーザーゲーム）のモデルケース。  
  - 将来的には `logic` が公開する API だけに依存し、`core` と直接リンクしない。

```
┌────────┐   FFI/ABI   ┌─────────┐   Game API   ┌────────────┐
│  core   │ ─────────▶ │  logic  │ ◀────────── │ sample_game │
└────────┘             └─────────┘             └────────────┘
```

## 2. 現状の依存方向の整理

- `core` → `logic`: `core/CMakeLists.txt` で `miyabi_logic` (`staticlib`) をリンクし、`miyabi_bridge.cpp` から VTable を呼び出している。
- `logic` → `core`: `logic/src/lib.rs` の `cxx::bridge` で `play_sound` や `create_dynamic_box_body` などランタイム呼び出しを宣言し、`core/include/miyabi/bridge.h` に実装を置いている。
- `sample_game` → `logic`: `sample_game/Cargo.toml` で `miyabi_logic` を依存登録しつつ、`sample_game/src/lib.rs` に `SampleGameState` / `SampleGameButtonAction` / `SampleGameLoop` を持つ最小状態機械契約を追加済み。
- `core` ↛ `sample_game`: 実行ファイルは `sample_game` をリンクしておらず、ユーザーのゲームコードを静的に組み込まない想定。
- 例外的に `core` は `../logic/src/performance.cpp` を直接ビルドへ含めており、ここが境界の整理対象。

## 3. 境界ルール

### 許可される依存
- `core` → `logic`（FFI境界）: `get_miyabi_vtable()` を取得してゲームインスタンスを生成し、レンダリング/入力結果を橋渡しする。
- `logic` → `core`（サービス呼び出し）: `bridge.h` に列挙されたランタイム機能（オーディオ、物理、テクスチャロードなど）だけを利用する。
- `sample_game` → `logic`: 公開された ECS 型 (`Component`, `RenderableObject` 等)、保存/ロード API (`logic/src/save.rs`)、UI 部品 (`logic/src/ui.rs`) を介してゲームプレイを実装する。
- `logic` 内部モジュール間: FFI、安全なユーティリティ、アセット同期処理同士は自由に参照してよい。

### 禁止または抑止する依存
- `core` → `sample_game`: C++ 側が特定ゲームのリソース/状態へ直接アクセスすることを禁止する。サンプル固有の定数は Rust 側で完結させる。
- `sample_game` → `core`: SDK 利用者が `core/include/*` を参照したり `glfw` などを直接リンクするのは不可。ランタイム機能を使いたい場合は `logic` が提供する API を経由する。
- `logic` → `core` 実装詳細: `bridge.h` 以外の C++ ヘッダ/ソースを include しない。GL/GLFW/Freetype などプラットフォームライブラリへの直接依存を作らない。
- `core` → `logic` 内部ファイル: Rust クレートの `.rs` を C++ から include する、`logic` が持つゲーム状態 (`Game`, `World`, UI) を C++ で改変する、といった境界破りを禁止する。

#### 禁止依存の具体例（NG/代替）
- NG: `core/src/*` から `sample_game/src/*` のゲーム状態を直接参照する。  
  代替: `core` は `miyabi_bridge.cpp` 経由で `logic` の公開 API のみを呼ぶ。
- NG: `sample_game` で `core/include/miyabi/bridge.h` を `include` して C++ 関数を直接呼ぶ。  
  代替: `sample_game` は `miyabi_logic` の公開型・公開関数のみを利用する。
- NG: `logic` から `core/src/renderer/*` など実装ディレクトリへ依存を張る。  
  代替: `logic` からのランタイム呼び出しは `core/include/miyabi/bridge.h` の契約に限定する。
- NG: `core` の CMake に `sample_game` ターゲットを追加し、`target_link_libraries` で直接リンクする。  
  代替: 実行時の組み合わせは `core` + `logic` の ABI 境界で接続し、ゲーム固有処理は Rust 側で完結させる。

## 4. 2026-03-07 時点の移行状況

- `sample_game/src/lib.rs` に `SampleGameState` / `SampleGameButtonAction` / `SampleGameLoop` を追加し、Title / InGame / Pause / Result の最小遷移契約を `sample_game` 側へ移した。
- `logic/src/ui.rs` は `Button` の hit-test と描画に限定し、戻り値として `action_id` を返す汎用 UI 部品へ縮小した。ゲーム固有の action enum は `logic` から除去した。
- `logic/src/lib.rs` には `apply_sample_action_id()` の互換 shim を残し、現行の `get_miyabi_vtable()` 起動経路を壊さずに sample 側契約を段階移行できる状態にした。
- `sample_game/tests/flow_contract.rs` で action id の round-trip と `Title -> InGame -> Pause -> Result -> Title -> Exit` の契約を固定した。

## 5. 次に分離すべき箇所の指針

- `logic/src/lib.rs`: まだ HUD レンダリング、障害物生成、勝敗判定、保存反映などサンプル固有処理が残る。次段では `sample_game::SampleGameLoop` が返す command/effect を直接実行する構成へ寄せる。
- `logic` と `sample_game` で共有している `sample.*` action id 文字列は暫定契約であり、最終的には共通契約層または `sample_game` 起点の登録 API へ一本化する。
- `core/CMakeLists.txt` 内の `../logic/src/performance.cpp` 取り込みは、C++ から Rust ディレクトリへアクセスしている唯一の箇所。ビルド成果物へ組み込むなら `logic` 側で `extern "C"` API を提供して `core` はそれを呼ぶ形に合わせる。
- `sample_game` の状態機械はまだテスト契約中心で、実行時 boot path は `logic` staticlib のままである。`core` が `sample_game` を知らずに差し替えられる起動方式を次段で固める必要がある。

このルールを満たしたとき、`core` はプラットフォームとランタイムに専念し、`logic` は SDK/API 群、`sample_game` はユーザーコードのサンプルという役割が明確になる。C1 判定では上記 4 点の分離具合を指標にする。
