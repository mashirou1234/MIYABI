# Codex移行ステータス

最終更新: 2026-03-07

## 更新ルール（最小記載）

- 更新日: `YYYY-MM-DD` を必ず記載する
- 変更種別: `実装` / `ビルド` / `CI` / `ドキュメント` などを明記する
- 関連ファイル: 変更に関係するファイルを2件以上、相対パスで列挙する
- 変更概要: 次回担当者が差分意図を追えるよう、1〜3行で要点を記載する
- 1 run 1記録: 1回のrunで履歴エントリは1件のみ追加する
- 1記録1テーマ: 1件の履歴エントリには単一テーマのみを記載し、別テーマは別runへ分離する

### 更新粒度ルール（運用）

- 新規runの追記は「0.2 移行記録テンプレ（標準）」を使用する
- 同一日に複数runがある場合は見出しを `YYYY-MM-DD run: issue-<id> <短いタイトル>` で分離する
- 1件の履歴エントリに混在してよいのは、同一テーマに直接必要な差分のみとする
- 既存の旧形式履歴は互換性のため保持し、以降の追記のみ本粒度ルールを適用する

## 0. 開発動線との紐付け

- コア到達判定の正: `docs/CORE_DEVELOPMENT_TRACK.md`
- ゲーム到達判定の正: `docs/GAME_DEVELOPMENT_TRACK.md`
- 作業タスク管理の正: `PLAN.md`
- SDK更新時の同期チェック: `docs/SDK_DEFINITION.md` の「4.1 SDK更新時チェックリスト」
- 現在ステージ:
  - コア: C0（Core Runtime）
  - ゲーム: G1（2Dプレイアブルループ）
- 次ゲート:
  - コア: C1（2D Engine Baseline）
  - ゲーム: G2（2D Vertical Slice 完成）
- 本ドキュメントの役割: 「今スレッドで何を変更したか」を管理する

`PLAN.md` は実装順、`docs/CORE_DEVELOPMENT_TRACK.md` はステージ到達判定、本ドキュメントはスレッド単位の変更履歴を管理する。

## 0.1 スレッド完了時チェックリスト（移行ステータス更新）

スレッドを閉じる前に、以下を上から順に確認する。

- [ ] このスレッドでの変更内容を `docs/CODEX_MIGRATION_STATUS.md` の「2. この移行で反映した内容」に追記した
- [ ] 実装タスクの進捗を `PLAN.md` に反映し、未着手/完了の状態を更新した
- [ ] 到達判定やDoDに影響がある場合、`docs/CORE_DEVELOPMENT_TRACK.md` と `docs/GAME_DEVELOPMENT_TRACK.md` を更新した
- [ ] 仕様や運用手順を変更した場合、該当設計書（例: `docs/SDK_DEFINITION.md`, `PERFORMANCE_TEST.md`）を更新した
- [ ] 実行した最小検証（ビルド/テスト/動作確認）の内容を、Issue または PR 説明に記録した

## 0.2 移行記録テンプレ（標準）

新規 run の記録は、以下テンプレをそのまま「2. この移行で反映した内容」に追記する。

### 最小必須項目

- 背景: なぜこの変更が必要か（1〜2行）
- 変更: 何を変えたか（要点 + 関連ファイル2件以上）
- 検証: 実行コマンドまたは確認観点（最低1件）
- 未解決: 次runへの持ち越し/制約（なければ「なし」）

### 記載テンプレ（転記用）

```md
### 2026-03-04 run: issue-XX <短いタイトル>

- 背景:
  - <背景1>
- 変更:
  - <変更要点1>
  - 関連ファイル:
    - `<path/to/file1>`
    - `<path/to/file2>`
- 検証:
  - `<実行コマンド or 確認内容>`
- 未解決:
  - <未解決事項。なければ「なし」>
```

### 記載例（ダミー）

```md
### 2026-03-04 run: issue-00 dummy-example

- 背景:
  - 変更履歴の粒度を統一し、引き継ぎコストを下げるため。
- 変更:
  - 記録テンプレの章を追加。
  - 関連ファイル:
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `PLAN.md`
- 検証:
  - テンプレ4項目（背景/変更/検証/未解決）が欠落なく記入できることを確認。
- 未解決:
  - なし
```

## 1. 現在の到達点

- `cmake -S . -B build -DMIYABI_PERFORMANCE_TEST=ON`
- `cmake --build build -j4`

上記コマンドで `build/core/miyabi` まで生成できる状態に復旧済み。

## 2. この移行で反映した内容

※ `0.2 移行記録テンプレ（標準）` の形式で追記する。既存履歴は互換性のため維持する。
### 2026-03-07 run: manual Issue設計ガイド整備

- 背景:
  - 完成に直結する Task を Issue 化しやすくするため、実装Issueの粒度と記載項目を正本化する必要があった。
- 変更:
  - `docs/ISSUE_DESIGN.md` を追加し、Issue種別、`codex:queue` 条件、分割ルール、必須項目、優先 Issue 群を整理した。
  - GitHub 用の開発Issueテンプレートを `.github/ISSUE_TEMPLATE/development_task.md` として追加した。
  - 案内導線として `README.md` と `docs/DEVELOPMENT_TRACK.md` から参照できるよう更新した。
  - 関連ファイル:
    - `docs/ISSUE_DESIGN.md`
    - `.github/ISSUE_TEMPLATE/development_task.md`
    - `docs/DEVELOPMENT_TRACK.md`
    - `README.md`
- 検証:
  - `git diff --check`
  - `rg -n "ISSUE_DESIGN|実装Task|Development task|codex:queue" README.md docs/DEVELOPMENT_TRACK.md docs/ISSUE_DESIGN.md .github/ISSUE_TEMPLATE/development_task.md`
- 未解決:
  - 既存の open Issue を新テンプレート基準で棚卸しし、`codex:queue` の再分類を行う作業は未実施。

### 2026-03-06 run: issue-175 更新粒度ルール明確化

- 背景:
  - runごとの記録単位が曖昧だと、履歴追跡と引き継ぎ時の差分把握に時間がかかるため。
- 変更:
  - `更新ルール（最小記載）` に `1 run 1記録` と `1記録1テーマ` を追加した。
  - `更新粒度ルール（運用）` を新設し、同日複数run時の見出し分離と旧形式履歴の扱いを明文化した。
  - 関連ファイル:
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `AGENTS.md`
- 検証:
  - `rg "migration|更新" docs/CODEX_MIGRATION_STATUS.md`
  - `cmake -S . -B build -DMIYABI_PERFORMANCE_TEST=ON`
- 未解決:
  - なし

### 2026-03-06 run: issue-240 SDK破壊的変更時のバージョニング規則明文化

- 背景:
  - SDK ABI の破壊的変更時に、major更新の判断基準と移行情報公開範囲が曖昧だったため。
- 変更:
  - `docs/SDK_DEFINITION.md` に major 更新の判断条件、バージョン更新手順、移行情報公開要件を追記した。
  - 関連ファイル:
    - `docs/SDK_DEFINITION.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `rg "ABI|version|破壊的変更|major" docs/SDK_DEFINITION.md`
- 未解決:
  - なし

- 2026-03-04 | 変更種別: ドキュメント | 関連ファイル: `logic/src/lib.rs`, `docs/LOGIC_PUBLIC_API_INVENTORY.md`, `docs/CODEX_MIGRATION_STATUS.md` | 変更概要: `rg "^pub "` の実測に合わせて公開API棚卸し（行番号・`camera` モジュール）を同期。
- 2026-03-04 | 変更種別: ドキュメント | 関連ファイル: `docs/DESIGN_Build.md`, `docs/ASSET_IMPORT_REIMPORT.md` | 変更概要: ビルド設計書の冒頭にアセット障害時の診断ログ採取・復旧手順への導線を追加。
- 更新日: 2026-03-04
  - 変更種別: ドキュメント
  - 関連ファイル:
    - `docs/LOGIC_PUBLIC_API_INVENTORY.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `PLAN.md`
  - 変更概要: 公開API棚卸しの更新チェック手順を 5 ステップで明文化し、差分抽出・SDK契約確認・計画同期・最小検証・PR記録までの再現可能な運用導線を追加した。
- 更新日: 2026-03-03
  - 変更種別: ビルド
  - 関連ファイル:
    - `core/include/miyabi/miyabi.h`
    - `core/src/miyabi_bridge.cpp`
    - `core/src/physics/PhysicsManager.cpp`
  - 変更概要: cxx生成ヘッダ参照を `miyabi_logic_cxx/lib.h` に統一し、参照揺れを解消した。
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
  - `.woodpecker.yml`（`miyabi-build` ステップで Configure/Build/Smoke を実装）
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
  - `.woodpecker.yml`（`miyabi-build`）
    - `Perf baseline (headless benchmark)`
    - `Perf regression check`
    - `Upload perf artifacts`
- クラッシュ/不具合報告テンプレートを追加
  - `.github/ISSUE_TEMPLATE/bug_report.md`
  - `PLAN.md`（タスク10.3へ反映）
- PR 自動承認 + CI 成功時 auto-merge 導線を追加
  - `.woodpecker.yml` + `scripts/woodpecker_pr_automerge.sh`
    - `pull_request` イベントで PR 自動承認
    - GitHub auto-merge（squash）を有効化
    - 同一リポジトリ由来かつ信頼済み権限（OWNER/MEMBER/COLLABORATOR）の PR のみ対象
    - `automerge:off` ラベル時はスキップ
  - `docs/CI_AUTOMERGE.md`
    - GitHub 必須設定（auto-merge・Branch protection Required checks・Woodpecker secret）を明文化
  - `PLAN.md`（タスク14.2へ反映）
- macOS向け 1OS 配布手順を固定し、再現ビルド導線を追加
  - `scripts/package_macos_game.sh`（クリーンビルド→配布ZIP生成）
  - `docs/DISTRIBUTION_1OS.md`（運用手順と確認項目）
  - `PLAN.md`（タスク9.3完了へ反映）
  - `docs/GAME_DEVELOPMENT_TRACK.md`（直近不足を更新）
- SDK を `find_package` で利用可能に拡張
  - `cmake/sdk-package/MIYABIConfig.cmake`
  - `cmake/sdk-package/MIYABIConfigVersion.cmake`
  - `build_sdk.sh`（`sdk/cmake` への同梱）
  - `sdk_template_CMakeLists.txt` / `sdk/template_CMakeLists.txt`（`find_package(MIYABI CONFIG REQUIRED)` へ移行）
  - `docs/SDK_DEFINITION.md`（配布物とリンク契約を更新）
  - `docs/CORE_DEVELOPMENT_TRACK.md`（直近不足を更新）
- ABIバージョン定数と実行時互換判定を導入
  - `core/include/miyabi/miyabi.h`（`MIYABI_ABI_VERSION_*` / `MiyabiVTable::abi_version`）
  - `logic/src/lib.rs`（`get_miyabi_vtable()` へ `abi_version` を設定）
  - `core/src/main.cpp`（起動時ABI整合チェック）
  - `sdk_template_main.cpp`（SDK利用側の最小ABIチェック）
  - `docs/SDK_DEFINITION.md`（ABI契約を追記）
- リンカ警告を整理
  - `CMakeLists.txt`（`CMAKE_OSX_DEPLOYMENT_TARGET` をSDKバージョンへ同期）
  - `core/CMakeLists.txt`（Darwin向け duplicate libraries warning を抑制）
  - `sdk_template_CMakeLists.txt`（SDK利用側も deployment target を同期）
  - `cmake/sdk-package/MIYABIConfig.cmake`（Darwin向け link option を付与）
- ABI更新ポリシーを明文化
  - `docs/SDK_DEFINITION.md`（major/minor/patch の互換性ルールと運用手順）
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
   - ABI変更時の移行ガイド（利用側コード差分例）のテンプレート化
2. CI拡張
   - 現在は macOS 1ジョブの `configure/build/smoke` + `perf baseline/regression` を実行。
   - 今後は multi-OS とキャッシュ最適化、失敗時アーティファクト収集を追加する。
3. ゲーム開発 G1/G3 に向けた未完了
   - 30分連続プレイの安定性検証（G2判定）
4. コア開発 C1 に向けた未完了
   - `sample_game` と `core` の責務再分離（ゲーム層の分離再整備）  
     - 境界と許可/禁止依存: `docs/SAMPLE_GAME_CORE_BOUNDARY.md`

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
  2. `sample_game` と `core` の責務再分離方針を明文化し、C1到達条件を確定する。
  3. ABI変更時の移行ガイド（利用側コード差分例）をテンプレート化する。
- 合意済みの運用方針:
  - コア開発（システム）とゲーム開発（ユーザー）をドキュメント上で分離して管理する。
  - `sample_game` はユーザー開発導線として扱うが、必要に応じてコア側改修を伴う方針で進める。
