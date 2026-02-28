# `sample_game` / `core` 責務境界

最終更新: 2026-02-26

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
- `sample_game` → `logic`: `sample_game/Cargo.toml` で `miyabi_logic` を依存登録済み（`sample_game/src/lib.rs` では re-export のみ）。
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

## 4. 「次に分離すべき箇所」の指針

- `logic/src/lib.rs`: `GameState`（Title/InGame/Pause/Result）、HUD レンダリング、障害物生成、設定 UI、アセット再読込などサンプルゲーム固有の処理が集中している。`sample_game` クレートへ移すことで `logic` を SDK コアへ純化できる。
- `logic/src/ui.rs`: ボタン定義と `ButtonAction` がサンプル固有の遷移（Start/Resume/Retry/BackToTitle）を前提にしており、UI 部品そのものは残しつつハンドラは `sample_game` へ委譲する必要がある。
- `core/CMakeLists.txt` 内の `../logic/src/performance.cpp` 取り込みは、C++ から Rust ディレクトリへアクセスしている唯一の箇所。ビルド成果物へ組み込むなら `logic` 側で `extern "C"` API を提供して `core` はそれを呼ぶ形に合わせる。
- `sample_game/src/lib.rs`: 現状は `use miyabi_logic::*;` のみで空。ここをゲームエントリーポイント（`create_game` など）に差し替え、`logic` 側から分離した状態機械を登録するのが当面の移行シナリオとなる。

このルールを満たしたとき、`core` はプラットフォームとランタイムに専念し、`logic` は SDK/API 群、`sample_game` はユーザーコードのサンプルという役割が明確になる。C1 判定では上記 4 点の分離具合を指標にする。
