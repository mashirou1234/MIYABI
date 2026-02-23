# Codex移行ステータス

最終更新: 2026-02-23

## 0. 開発動線との紐付け

- コア到達判定の正: `docs/CORE_DEVELOPMENT_TRACK.md`
- ゲーム到達判定の正: `docs/GAME_DEVELOPMENT_TRACK.md`
- 現在ステージ:
  - コア: C0（Core Runtime）
  - ゲーム: G1（2Dプレイアブルループ）
- 次ゲート:
  - コア: C1（2D Engine Baseline）
  - ゲーム: G2（2D Vertical Slice 完成）
- 本ドキュメントの役割: 「今スレッドで何を変更したか」を管理する

## 1. 現在の到達点

- `cmake -S . -B build -DMIYABI_PERFORMANCE_TEST=ON`
- `cmake --build build -j4`

上記コマンドで `build/core/miyabi` まで生成できる状態に復旧済み。

## 2. この移行で反映した内容

- cxx生成ヘッダ参照を `miyabi_logic_cxx/lib.h` に統一
  - `core/include/miyabi/miyabi.h`
  - `core/src/miyabi_bridge.cpp`
  - `core/src/physics/PhysicsManager.cpp`
- `core` から `miyabi_logic_cxx` の直接リンクを外し、重複シンボルを回避
  - `core/CMakeLists.txt`
- Rust警告の解消（`Box::from_raw` / `CString::from_raw` の戻り値処理など）
  - `logic/src/lib.rs`
  - `logic/src/paths.rs.in`
- パフォーマンステスト文書の進捗と typo を更新
  - `PERFORMANCE_TEST.md`
- SDK生成スクリプトを現行出力に合わせて修正
  - `build_sdk.sh`
  - `sdk_template_CMakeLists.txt`
- 設計ドキュメントに「現行は静的リンク」注記を追加
  - `docs/DESIGN_Build.md`
  - `docs/DESIGN_FFI.md`
- SDK定義 v0.1 を明文化し、配布物とエントリポイントを固定
  - `docs/SDK_DEFINITION.md`
  - `core/include/miyabi/miyabi.h`（SDKバージョン定数）
- SDKのランタイムブリッジ実装を独立ライブラリ化
  - `core/CMakeLists.txt`（`miyabi_runtime` 追加）
  - `build_sdk.sh`（`libmiyabi_runtime.a` と `libbox2d.a` 同梱）
  - `sdk_template_CMakeLists.txt`
  - `sdk_template_main.cpp`
- `PLAN.md` の Beyond Phase 8 を、フェーズ9〜14の実行計画へ詳細化
  - 2D縦切り → 2Dプロダクション → 3D基盤 → 3D縦切り → 3Dプロダクション → エコシステム強化
- Phase 9.1 の縦切り仕様を固定
  - `docs/SPEC_SAMPLE_GAME_2D_VERTICAL_SLICE.md`
  - `PLAN.md`（タスク9.1完了）
- Phase 9.2 のプレイループを実装
  - `logic/src/lib.rs`（Title/InGame/Pause/Result、障害物サバイバル、HUD、Result遷移）
  - `logic/src/ui.rs`（Start/Resume/Retry/BackToTitle）
  - `core/src/main.cpp`（`ESC` 入力をロジックへ伝搬）
  - `PLAN.md`（タスク9.2完了）
- コア Save サブシステムを定義
  - `logic/src/save.rs`（save/load API、バージョン、破損時backup、原子的保存）
  - `docs/CORE_SAVE_SUBSYSTEM.md`
  - `PLAN.md`（タスク10.2へ反映）
- セーブ/ロード最小実装をプレイ導線へ統合
  - `logic/src/lib.rs`（起動時ロード、リザルト遷移保存、終了時保存、進行データ表示）
  - `PLAN.md`（タスク10.2へ反映）
  - `docs/CORE_DEVELOPMENT_TRACK.md`
  - `docs/GAME_DEVELOPMENT_TRACK.md`
- 設定UIを実装し、設定変更時保存を接続
  - `logic/src/lib.rs`（Title/Pauseに設定表示と設定ボタン配置）
  - `logic/src/ui.rs`（設定用 ButtonAction と更新処理）
  - `PLAN.md`（タスク10.2へ反映）
  - `docs/CORE_SAVE_SUBSYSTEM.md`
- コア開発とゲーム開発のトラックを分離
  - `docs/CORE_DEVELOPMENT_TRACK.md`
  - `docs/GAME_DEVELOPMENT_TRACK.md`
  - `docs/DEVELOPMENT_TRACK.md`（案内ページ化）

## 3. 現在の構成（正）

- Rustロジック: `logic` クレート（`staticlib`）
- cxxブリッジ生成物: `miyabi_logic_cxx` ターゲット
- C++ホスト: `core` の `miyabi` 実行ファイル
- 呼び出し契約: `get_miyabi_vtable()` を静的リンクして利用

## 4. 残課題（次スレッド優先）

1. SDKの次段階整備
   - `find_package` 可能な CMake package config の提供
   - ABIバージョン定数（互換性判定用）の導入
2. CI導入
   - `.github/workflows` が未整備。
   - まずは configure/build の自動実行を追加する。
3. リンカ警告の整理
   - duplicate libraries warning
   - macOS deployment target warning（26.2 vs 26.0）
4. 性能計画の未完了タスク
   - シーン構築/破棄ストレステスト
   - ベースライン記録
5. ゲーム開発 G1/G3 に向けた未完了
   - 30分連続プレイの安定性検証（G2判定）
   - 1OS向け配布手順の固定化
6. コア開発 C1 に向けた未完了
   - 設定値のランタイム適用（音量反映、実ウィンドウの fullscreen 切替）

## 5. 続スレッド再開コマンド

```bash
cd /Users/shiroguchi/Documents/Github/mashirou1234/Game/MIYABI
git status --short --branch
cmake -S . -B build -DMIYABI_PERFORMANCE_TEST=ON
cmake --build build -j4
```
