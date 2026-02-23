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
- 設定値のランタイム適用を接続
  - `logic/src/lib.rs`（設定変更時にC++側へ反映要求）
  - `core/src/miyabi_bridge.cpp`（オーディオ設定反映、fullscreen要求キュー）
  - `core/include/miyabi/bridge.h`
  - `core/src/main.cpp`（fullscreen要求を消費して GLFW へ適用）
  - `PLAN.md`（タスク10.2へ反映）
- BGM実再生導線を接続
  - `core/src/miyabi_bridge.cpp`（BGM再生/停止、BGMグループ音量反映）
  - `core/include/miyabi/bridge.h`（`play_bgm/stop_bgm/shutdown_engine_systems`）
  - `core/src/main.cpp`（終了時 `shutdown_engine_systems`）
  - `logic/src/lib.rs`（Title/InGame/Result 遷移でBGM適用）
  - `PLAN.md`（タスク10.2へ反映）
- アセット import/reimport 手順を整備
  - `logic/src/lib.rs`（`ReloadTexture` 要求と `U` キー導線）
  - `core/src/main.cpp`（`LoadTexture/ReloadTexture` の実処理）
  - `core/src/renderer/TextureManager.cpp`（`reload_texture` 実装）
  - `docs/ASSET_IMPORT_REIMPORT.md`
  - `PLAN.md`（タスク10.2へ反映）
- アセットID管理と参照整合チェックを導入し、診断ログ/復旧手順を整備
  - `logic/src/lib.rs`（asset registry、参照整合チェック、未解決参照の自動reimport再キュー）
  - `docs/ASSET_IMPORT_REIMPORT.md`（診断ログと復旧手順を追加）
  - `PLAN.md`（タスク10.2へ反映）
- CIで `configure/build/smoke` を自動実行
  - `.github/workflows/build.yml`（Configure/Build/Smoke を実装）
  - Smoke対象:
    - `logic` の `cargo test`
    - `sample_game` の `cargo test`（ユーザー開発側のコンパイル整合チェック）
    - `build/core/miyabi` 生成確認
  - `PLAN.md`（タスク10.3へ反映）
- 性能ベースライン計測と回帰判定を導入
  - `logic/src/perf.rs`（`sprite/ui/scene_construct_destruct` のヘッドレス計測）
  - `logic/src/bin/perf_baseline.rs`（計測実行とJSON出力）
  - `docs/perf/baseline_macos14.json`（初期ベースライン）
  - `tools/check_perf_regression.py`（閾値比較による回帰判定）
  - `PERFORMANCE_TEST.md`（運用手順・初期値の記録）
  - `PLAN.md`（タスク10.3へ反映）
- CIへ性能計測と回帰判定を統合
  - `.github/workflows/build.yml`
    - `Perf baseline (headless benchmark)`
    - `Perf regression check`
    - `Upload perf artifacts`
- クラッシュ/不具合報告テンプレートを追加
  - `.github/ISSUE_TEMPLATE/bug_report.md`
  - `PLAN.md`（タスク10.3へ反映）
- macOS向け 1OS 配布手順を固定し、再現ビルド導線を追加
  - `scripts/package_macos_game.sh`（クリーンビルド→配布ZIP生成）
  - `docs/DISTRIBUTION_1OS.md`（運用手順と確認項目）
  - `PLAN.md`（タスク9.3完了へ反映）
  - `docs/GAME_DEVELOPMENT_TRACK.md`（直近不足を更新）
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
2. CI拡張
   - 現在は macOS 1ジョブの `configure/build/smoke` + `perf baseline/regression` を実行。
   - 今後は multi-OS とキャッシュ最適化、失敗時アーティファクト収集を追加する。
3. リンカ警告の整理
   - duplicate libraries warning
   - macOS deployment target warning（26.2 vs 26.0）
4. ゲーム開発 G1/G3 に向けた未完了
   - 30分連続プレイの安定性検証（G2判定）
5. コア開発 C1 に向けた未完了
   - `sample_game` と `core` の責務再分離（ゲーム層の分離再整備）

## 5. 続スレッド再開コマンド

```bash
cd /Users/shiroguchi/Documents/Github/mashirou1234/Game/MIYABI
git status --short --branch
cmake -S . -B build -DMIYABI_PERFORMANCE_TEST=ON
cmake --build build -j4
```

## 6. 次スレッド引き継ぎメモ（2026-02-23）

- 現在のローカル状態:
  - ブランチ: `master`
  - `origin/master` に対して `ahead 7`（未pushコミットあり）
- 直近コミット（新しい順）:
  - `0a361a8` `ci: configure build smoke を自動実行`
  - `dc8ff8d` `feat: アセットID整合チェックと復旧導線を追加`
  - `f7b7e46` `feat: texture import/reimport導線を整備`
  - `f455a5e` `feat: BGM実再生導線を追加`
  - `ba2d8f0` `feat: 設定値のランタイム適用を実装`
- 次スレッドの推奨着手順:
  1. G2判定向けに 30分連続プレイの安定性検証（進行不能/クラッシュ有無）を実施する。
  2. SDK次段階として `find_package` 可能な CMake package config 提供に着手する。
  3. duplicate libraries / deployment target 警告を整理し、ビルドログノイズを低減する。
- 合意済みの運用方針:
  - コア開発（システム）とゲーム開発（ユーザー）をドキュメント上で分離して管理する。
  - `sample_game` はユーザー開発導線として扱うが、必要に応じてコア側改修を伴う方針で進める。
