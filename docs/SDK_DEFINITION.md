# MIYABI SDK定義 (v0.1)

最終更新: 2026-02-23

## 1. 目的

MIYABI SDK v0.1 は、外部 C++ アプリケーションから MIYABI ロジックを静的リンクで利用するための配布物を定義する。

本SDKは「外部ゲーム開発向け」の最小成立ラインとして、以下を提供する。

- ロジックAPI (`MiyabiVTable`) の公開
- ロジックが必要とするランタイムサービス（音声/物理）の提供
- CMakeテンプレートと最小サンプル

## 2. 想定利用形態

- 利用者は C++17 のホストアプリを実装する
- ホストは SDK 同梱の静的ライブラリ群をリンクする
- ロジック呼び出しは `get_miyabi_vtable()` を起点に行う
- `dlopen/dlsym` による動的ホットリロードは v0.1 のスコープ外

## 3. 公開エントリポイント

### 3.1 ロジックAPI起点

- 宣言: `extern "C" MiyabiVTable get_miyabi_vtable();`
- 定義元: `libmiyabi_logic.a`
- 役割: ゲームロジック更新/描画データ取得/アセット要求などを関数テーブルで提供

### 3.2 ランタイムサービス起点

- 宣言: `void init_engine_systems();`, `void step_engine_systems();`, `void shutdown_engine_systems();`
- 定義元: `libmiyabi_runtime.a`
- 役割: 音声・物理など、ロジック側が要求する C++ サービスを初期化/更新

## 4. 配布物定義

SDK ZIP (`MIYABI_SDK.zip`) には最低限、以下を含める。

- `include/miyabi/miyabi.h`
- `include/miyabi/bridge.h`
- `include/miyabi_logic_cxx/lib.h`
- `include/rust/cxx.h`
- `lib/libmiyabi_logic.a`
- `lib/libmiyabi_logic_cxx.a`
- `lib/libmiyabi_runtime.a`
- `lib/libbox2d.a`
- `template_CMakeLists.txt`
- `examples/main.cpp`
- `docs/SDK_DEFINITION.md`

## 5. リンク契約

静的リンク時は依存解決のためにリンク順序を守る。

1. `miyabi_logic`
2. `miyabi_logic_cxx`
3. `miyabi_runtime`
4. `box2d`

推奨テンプレートは `sdk/template_CMakeLists.txt` を正とする。

## 6. 実行契約

最小実行フロー:

1. `init_engine_systems()`
2. `get_miyabi_vtable()`
3. `create_game()`
4. 毎フレーム `step_engine_systems()` → `update_game()`
5. 終了時 `destroy_game()`
6. 終了時 `shutdown_engine_systems()`

所有権/メモリルールは `docs/DESIGN_FFI.md` と `core/include/miyabi/miyabi.h` を正とする。

## 7. 非スコープ (v0.1)

- ABI互換性の長期保証（同一ZIP内整合のみ保証）
- Windows/Linux の動作保証
- 動的ロジック差し替え (`dlopen/dlsym`) の公式サポート

## 8. 今後の拡張方針

- ABIバージョン定数の導入
- `find_package` 可能な CMake package config の提供
- プラットフォーム別の公式配布とCIによる検証
