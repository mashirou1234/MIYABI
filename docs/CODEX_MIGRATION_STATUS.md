# Codex移行ステータス

最終更新: 2026-03-08

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
  - コア: C3（3D Engine Baseline）
  - ゲーム: G4（3D Vertical Slice）
- 次ゲート:
  - コア: C4（Ecosystem Competitiveness）
  - ゲーム: Wave 4（2D再利用性の成立）
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
### 2026-03-08 run: issue-304 週次Git棚卸し

- 背景:
  - `codex:queue` で投入された週次棚卸し Issue #304 の結果を、repo 正本の移行ログへ反映する必要があった。
- 変更:
  - 2026-03-06 時点の棚卸し結果（変更件数 0件 / 新規項目 0件 / 保留なし）を記録し、次回も同条件で差分監視を継続する方針を明文化した。
  - 関連ファイル:
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `README.md`
- 検証:
  - `rg -n "2026-03-08 run: issue-304|最終更新: 2026-03-08" docs/CODEX_MIGRATION_STATUS.md`
- 未解決:
  - なし

### 2026-03-08 run: issue-305 週次Git棚卸し 2026-03-06

- 背景:
  - `codex:queue` で投入された週次棚卸し Issue #305 の結果を、移行ログへ追記して次回比較の基準を維持する必要があった。
- 変更:
  - 2026-03-06 実行分の棚卸し結果（コミット推奨 0 / 保留 0 / `.gitignore` 候補 0）を反映した。
  - 定常ノイズ再発頻度（`artifacts/` 未追跡: 2/4 repo、`api/.hypothesis/` 未追跡: 1/4 repo）を記録し、監視継続方針を明文化した。
  - 関連ファイル:
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `rg -n "issue-305|artifacts/ 未追跡: 2/4 repo|api/.hypothesis/ 未追跡: 1/4 repo" docs/CODEX_MIGRATION_STATUS.md`
- 未解決:
  - なし

### 2026-03-08 run: issue-306 週次Git棚卸し 2026-03-06

- 背景:
  - `codex:queue` で投入された週次棚卸し Issue #306 の結果を、移行ログへ反映して監視の継続条件を明文化する必要があった。
- 変更:
  - 2026-03-06 実行分の棚卸し結果（コミット推奨 0 / 保留 0 / `.gitignore` 候補 0）を追記した。
  - 根拠コマンド `git status --porcelain=v1 --untracked-files=all` が空であることを記録し、次週も同一条件で差分確認する方針を維持した。
  - 関連ファイル:
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `rg -n "issue-306|git status --porcelain=v1 --untracked-files=all" docs/CODEX_MIGRATION_STATUS.md`
- 未解決:
  - なし

### 2026-03-08 run: manual issue-361 SDK外部サンプル再利用証跡を追加

- 背景:
  - C1 の残件として、`sample_game` 以外でも MIYABI SDK を使って最小サンプルを起動できる証跡が必要だった。
- 変更:
  - `scripts/test_sdk_external_sample_reuse.sh` を追加し、`sdk/` を temp project にコピーして `find_package(MIYABI CONFIG REQUIRED)` から configure/build/run できる再利用スモークを自動化した。
  - `scripts/test_build_sdk_required_artifacts.sh` から上記スモークを呼び出すようにし、SDK 配布物確認と外部サンプル再利用確認を一連の手順へまとめた。
  - `sdk_template_main.cpp` と `sdk/examples/main.cpp` に `shutdown_engine_systems()` を追加し、SDK の最小実行契約を明示的に閉じるようにした。
  - `README.md` / `docs/SDK_DEFINITION.md` / `docs/CORE_DEVELOPMENT_TRACK.md` / `docs/COMPLETION_ROADMAP.md` / `docs/SAMPLE_GAME_CORE_BOUNDARY.md` / `PLAN.md` を更新し、C1 到達と Wave 4 の再利用条件を分離して記録した。
  - 関連ファイル:
    - `scripts/test_sdk_external_sample_reuse.sh`
    - `scripts/test_build_sdk_required_artifacts.sh`
    - `sdk_template_main.cpp`
    - `sdk/examples/main.cpp`
    - `README.md`
    - `docs/SDK_DEFINITION.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `docs/SAMPLE_GAME_CORE_BOUNDARY.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `PLAN.md`
- 検証:
  - `./build_sdk.sh`
  - `./scripts/test_sdk_external_sample_reuse.sh ./sdk`
  - `./scripts/test_build_sdk_required_artifacts.sh`
  - `cargo test --manifest-path logic/Cargo.toml --lib -- --nocapture`
  - `./scripts/check_core_no_sample_game_dependency.sh`
- 未解決:
  - 外部サンプル 2 本以上での再利用成立は Wave 4 として別途継続する。
  - 次のコア着手は `C2-03` / `C2-04`。

### 2026-03-08 run: manual issue-362 directional light 1 灯と最小陰影を追加

- 背景:
  - `C2-03` として、3D arena の床 / 壁 / プレイヤーに陰影差を付け、G4 最小シーンの立体感を補う必要があった。
- 変更:
  - `core/src/renderer/MeshManager.cpp` で OBJ の法線読み込み / 面法線補完を追加し、3D メッシュを `position + uv + normal` でアップロードするようにした。
  - `core/src/shaders/lit_textured.vert` / `core/src/shaders/lit_textured.frag` を追加し、Lambert 相当の directional light 1 灯を使う 3D 用 shader を導入した。
  - `core/src/main.cpp` で 2D `textured` shader と 3D `lit_textured` shader を分離し、3D material / directional light uniform / instancing attribute を 3D cube mesh にも接続した。
  - `logic/src/lib.rs` で 3D arena の renderable に `MATERIAL_ID_LIT_TEXTURED_3D` を割り当て、2D/3D の shader 契約を material_id で分離した。
  - `scripts/package_macos_game.sh` / `scripts/test_distribution_smoke.sh` と C2/Core/Game track 文書を更新し、新規 shader 配布物と `C2-03` 完了状態を同期した。
  - 関連ファイル:
    - `core/src/renderer/MeshManager.cpp`
    - `core/src/main.cpp`
    - `core/src/shaders/lit_textured.vert`
    - `core/src/shaders/lit_textured.frag`
    - `core/src/shaders/textured.vert`
    - `logic/src/lib.rs`
    - `scripts/package_macos_game.sh`
    - `scripts/test_distribution_smoke.sh`
    - `docs/CORE_3D_FOUNDATION_CONTRACT.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/GAME_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `PLAN.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `cargo test --manifest-path logic/Cargo.toml --lib -- --nocapture`
  - `cmake --build build -j`
  - `./scripts/test_game_track_g2.sh`
  - `./scripts/test_distribution_smoke.sh`
- 未解決:
  - `C2-04` の 2D/3D 共存回帰ハーネスは別 issue で継続する。

### 2026-03-08 run: manual issue-363 2D/3D 共存回帰ハーネスを追加

- 背景:
  - `C2-04` として、2D title UI/text と 3D arena overlay/render の両方を再現可能に確認し、`G2/G3` スモークまでまとめて再実行できる手順が必要だった。
- 変更:
  - `logic/src/lib.rs` に `title_screen_exposes_2d_ui_and_text_commands` / `start_3d_arena_preserves_2d_text_overlay_and_3d_renderables` / `start_3d_arena_pause_and_back_to_title_flow_is_reachable` を追加し、2D UI/text、3D overlay/render、2D/3D 状態遷移を headless で固定した。
  - `scripts/test_core_2d_3d_coexistence.sh` を追加し、上記 targeted test と `./scripts/test_game_track_g2.sh` / `./scripts/test_distribution_smoke.sh` を束ねる 1 コマンド entrypoint を作成した。
  - ハーネス実行時に `artifacts/c2_04_2d_3d_coexistence_latest.log` へ summary を書き出すようにし、3D 最小描画証跡を再取得しやすくした。
  - `README.md` / `docs/CORE_3D_FOUNDATION_CONTRACT.md` / `docs/CORE_DEVELOPMENT_TRACK.md` / `docs/GAME_DEVELOPMENT_TRACK.md` / `docs/COMPLETION_ROADMAP.md` / `PLAN.md` を更新し、`C2-04` 完了と Core 現在地 `C2` / 次ゲート `C3` を同期した。
  - 関連ファイル:
    - `logic/src/lib.rs`
    - `scripts/test_core_2d_3d_coexistence.sh`
    - `README.md`
    - `docs/CORE_3D_FOUNDATION_CONTRACT.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/GAME_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `PLAN.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `cargo test --manifest-path logic/Cargo.toml --lib -- --nocapture`
  - `./scripts/test_game_track_g2.sh`
  - `./scripts/test_distribution_smoke.sh`
  - `./scripts/test_core_2d_3d_coexistence.sh`
- 未解決:
  - Core の次段は `#366` / `#367`（C3）で継続する。

### 2026-03-08 run: manual issue-364 3D run の勝敗導線を接続

- 背景:
  - `G4-02` として、3D arena を `Pause / GAME OVER / CLEAR / Retry` まで繋ぎ、2D run に戻らず 3D run を再開できる最小 loop を固定する必要があった。
- 変更:
  - `sample_game_runtime/src/lib.rs` に `SampleGameRunMode` と `SampleGameLoop::from_state_and_mode()` / `run_mode()` を追加し、`RetryGame` が直前の 2D/3D run mode に応じて `StartNewRun` / `StartNew3dRun` を返すようにした。
  - `sample_game/src/lib.rs` と `sample_game/tests/flow_contract.rs` を更新し、`Start 3D Arena -> Result -> RetryGame` が 3D run へ戻る契約を固定した。
  - `logic/src/lib.rs` で `RunMode` と `SampleGameRunMode` の変換を追加し、Title / InGame / Pause / Result / deserialize 復元時に run mode を `SampleGameLoop` へ保持するようにした。
  - `logic/src/lib.rs` の 3D run 更新処理に、HP 0 の `GAME OVER`、`180.0 sec` 到達時の `CLEAR`、difficulty / score 更新、`3D Arena Result` 表示を追加した。
  - `logic` の targeted test と `scripts/test_game_track_g4.sh` を追加し、Pause / GAME OVER / CLEAR / Retry の headless スモークを 1 コマンドで再実行できるようにした。
  - `README.md` / `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md` / `docs/GAME_DEVELOPMENT_TRACK.md` / `docs/COMPLETION_ROADMAP.md` / `docs/SAMPLE_GAME_CORE_BOUNDARY.md` / `PLAN.md` を更新し、`G4-02` 完了と残件 `G4-03` を同期した。
  - 関連ファイル:
    - `sample_game_runtime/src/lib.rs`
    - `sample_game/src/lib.rs`
    - `sample_game/tests/flow_contract.rs`
    - `logic/src/lib.rs`
    - `scripts/test_game_track_g4.sh`
    - `README.md`
    - `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md`
    - `docs/GAME_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `docs/SAMPLE_GAME_CORE_BOUNDARY.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `PLAN.md`
- 検証:
  - `cargo test --manifest-path sample_game/Cargo.toml -- --nocapture`
  - `cargo test --manifest-path logic/Cargo.toml --lib -- --nocapture`
  - `cmake --build build -j`
  - `./scripts/test_game_track_g4.sh`
- 未解決:
  - `G4-03` の障害物導入までは、3D `GAME OVER` は headless test で `hp=0` を与える最小導線に留まる。

### 2026-03-08 run: manual issue-365 3D 障害物 1 系統を実装

- 背景:
  - `G4-03` として、3D arena に実際の fail 要因を入れ、`GAME OVER` / `CLEAR` を headless state injection ではなく実障害物つき loop として成立させる必要があった。
- 変更:
  - `logic/src/lib.rs` に 3D falling obstacle の spawn / update / collision を追加し、`Start 3D Arena` 直後から cube obstacle が落下して HP を削り、回避時は `avoid_count` と score に反映するようにした。
  - `logic/src/lib.rs` に `player_bounds_3d()` と 3D obstacle targeted test を追加し、spawn、実衝突による `GAME OVER`、回避による `CLEAR`、3D run 前後での settings 保持を headless で固定した。
  - `scripts/test_game_track_g4.sh` を G4 全体 smoke へ拡張し、`Pause / GAME OVER / CLEAR / Retry` に加えて obstacle spawn/fail/clear と settings evidence を `artifacts/g4_vertical_slice_latest.log` へ残すようにした。
  - `README.md` / `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md` / `docs/GAME_DEVELOPMENT_TRACK.md` / `docs/COMPLETION_ROADMAP.md` / `docs/SAMPLE_GAME_CORE_BOUNDARY.md` / `PLAN.md` を更新し、`G4-03` 完了とゲーム現在地 `G4` を同期した。
  - 関連ファイル:
    - `logic/src/lib.rs`
    - `scripts/test_game_track_g4.sh`
    - `README.md`
    - `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md`
    - `docs/GAME_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `docs/SAMPLE_GAME_CORE_BOUNDARY.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `PLAN.md`
- 検証:
  - `cargo test --manifest-path logic/Cargo.toml --lib start_3d_arena_spawns_falling_obstacle_renderables -- --nocapture`
  - `cargo test --manifest-path logic/Cargo.toml --lib start_3d_arena_obstacle_hits_can_reach_game_over -- --nocapture`
  - `cargo test --manifest-path logic/Cargo.toml --lib start_3d_arena_obstacle_avoidance_can_reach_clear -- --nocapture`
  - `cargo test --manifest-path logic/Cargo.toml --lib start_3d_arena_preserves_settings_across_result_and_retry -- --nocapture`
  - `./scripts/test_game_track_g4.sh`
- 未解決:
  - ゲーム側の 3D 縦切り残件は解消した。次段は `#366` / `#367` と `Wave 4` の継続運用。

### 2026-03-08 run: manual issue-366 3D シーン性能ベースライン採取を自動化

- 背景:
  - `C3` の初手として、代表 3D シーンの perf 値を手順ではなく 1 コマンドで再取得し、baseline compare と記録先まで固定する必要があった。
- 変更:
  - `logic/src/perf.rs` に `arena3d_renderable_build` scenario を追加し、`Arena3d` の floor / walls / player / 3D cube 群を使う代表 3D scene renderable build を baseline 対象へ加えた。
  - `logic/src/bin/perf_baseline.rs` に `--arena3d-obstacles` を追加し、3D scene の規模を CLI から調整できるようにした。
  - `scripts/test_core_c3_3d_perf_baseline.sh` を追加し、`perf_baseline` 実行、`tools/check_perf_regression.py` 比較、`build/perf/logs/` への compare log 保存、`artifacts/c3_3d_perf_baseline_latest.log` への summary 出力を 1 run で行うようにした。
  - `docs/perf/baseline_macos14.json` / `PERFORMANCE_TEST.md` / `docs/perf/PERF_BASELINE.md` / `README.md` / `docs/CORE_DEVELOPMENT_TRACK.md` / `docs/CORE_3D_PRODUCTION_BASELINE.md` / `docs/COMPLETION_ROADMAP.md` / `PLAN.md` を更新し、`#366` 完了と `#367` のみが C3 残件である状態へ同期した。
  - 関連ファイル:
    - `logic/src/perf.rs`
    - `logic/src/bin/perf_baseline.rs`
    - `scripts/test_core_c3_3d_perf_baseline.sh`
    - `docs/perf/baseline_macos14.json`
    - `PERFORMANCE_TEST.md`
    - `docs/perf/PERF_BASELINE.md`
    - `README.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/CORE_3D_PRODUCTION_BASELINE.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `PLAN.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `cargo test --manifest-path logic/Cargo.toml perf -- --nocapture`
  - `./scripts/test_core_c3_3d_perf_baseline.sh`
  - `cmake --build build -j`
- 未解決:
  - `C3` の残件は `#367` の 3D アセット検証コマンド追加。

### 2026-03-08 run: manual issue-367 3D アセット検証コマンドを追加

- 背景:
  - `C3` の残件として、3D mesh / texture / shader / material 契約の不整合を事前に検知し、壊れた asset を人間が読める形で診断できるコマンドが必要だった。
- 変更:
  - `tools/validate_3d_assets.py` を追加し、既定の 3D OBJ / texture / shader 存在確認、OBJ 構文検証、`logic` と `core` の mesh/material ID 契約一致確認を 1 コマンドで実行できるようにした。
  - `tools/tests/test_validate_3d_assets.py` と `tools/tests/fixtures/invalid_missing_faces.obj.fixture` を追加し、正常系と `no drawable faces found` を返す異常系の両方を固定した。
  - `scripts/test_core_c3_3d_asset_validation.sh` を追加し、正規 asset の PASS と invalid fixture の FAIL をまとめて再実行し、`artifacts/c3_3d_asset_validation_latest.log` へ summary を保存するようにした。
  - `README.md` / `docs/CORE_3D_PRODUCTION_BASELINE.md` / `docs/CORE_DEVELOPMENT_TRACK.md` / `docs/COMPLETION_ROADMAP.md` / `PLAN.md` を更新し、`#367` 完了と Core 現在地 `C3` / 次ゲート `C4` を同期した。
  - 関連ファイル:
    - `tools/validate_3d_assets.py`
    - `tools/tests/test_validate_3d_assets.py`
    - `tools/tests/fixtures/invalid_missing_faces.obj.fixture`
    - `scripts/test_core_c3_3d_asset_validation.sh`
    - `README.md`
    - `docs/CORE_3D_PRODUCTION_BASELINE.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `PLAN.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `python3 tools/validate_3d_assets.py`
  - `python3 -m unittest tools.tests.test_validate_3d_assets`
  - `./scripts/test_core_c3_3d_asset_validation.sh`
- 未解決:
  - `C4` の次段は `#368` の開発体験評価テンプレート整備で継続する。

### 2026-03-08 run: manual issue-368 3D 開発体験評価テンプレートを整備

- 背景:
  - `C4` の最初の task として、初期導入速度 / 反復速度 / デバッグ容易性 / 拡張性 / 運用性 / ドキュメント成熟度を同じ形式で評価し、次の issue 候補まで切り出せる文書が必要だった。
- 変更:
  - `docs/templates/CORE_3D_DEVEX_EVALUATION_TEMPLATE.md` を追加し、C4 の 6 評価軸、0-3 スコア定義、証跡リンク必須、`score <= 1` で next issue 候補を残す運用ルールを固定した。
  - `docs/reviews/CORE_3D_DEVEX_EVALUATION_2026-03-08.md` を追加し、`#367` 完了直後の current state を試験記入して、次の C4 follow-up 候補（デバッグ可視化 / マルチプラットフォーム検証 / tutorial 体系化）を具体化した。
  - `README.md` / `docs/CORE_3D_PRODUCTION_BASELINE.md` / `docs/CORE_DEVELOPMENT_TRACK.md` / `docs/COMPLETION_ROADMAP.md` / `PLAN.md` を更新し、テンプレートと試験記入への導線、および C4 残件の次段整理方針を同期した。
  - 関連ファイル:
    - `docs/templates/CORE_3D_DEVEX_EVALUATION_TEMPLATE.md`
    - `docs/reviews/CORE_3D_DEVEX_EVALUATION_2026-03-08.md`
    - `README.md`
    - `docs/CORE_3D_PRODUCTION_BASELINE.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `PLAN.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `rg -n "CORE_3D_DEVEX_EVALUATION_TEMPLATE|CORE_3D_DEVEX_EVALUATION_2026-03-08|#368" README.md docs PLAN.md`
  - `git diff --check`
- 未解決:
  - C4 は完了ではなく、試験記入で抽出した follow-up を次 issue として再分解して継続する。

### 2026-03-08 run: manual issue-356-359 runtime boot反転と 3D 最小起動

- 背景:
  - C1 の残件だった `sample_game` runtime boot path の最終反転と、C2/G4 の初手実装 (`C2-01`, `C2-02`, `G4-01`) が未完だった。
- 変更:
  - `sample_game_runtime/` crate を追加し、`SampleGameState` / `SampleGameButtonAction` / `SampleGameEffect` / `SampleGameLoop` を `logic` と `sample_game` の共有契約層へ移した。
  - `logic/src/lib.rs` に `RunMode` / `RenderMesh` / `SampleGameLoop` 実行導線を追加し、`Start 3D Arena` action から最小 3D arena（床 / 壁 / プレイヤー）を起動できるようにした。
  - `core/src/main.cpp` で 3D 透視パスと 2D オーバーレイパスを分離し、`core/src/renderer/MeshManager.*` に OBJ mesh registry / loader を追加した。`assets/meshes/arena_cube.obj` を配布物へ同梱するようにした。
  - トラック文書、境界文書、公開 API 棚卸し、`PLAN.md` を今回の実装状況へ同期した。
  - 関連ファイル:
    - `sample_game_runtime/Cargo.toml`
    - `sample_game_runtime/src/lib.rs`
    - `sample_game/src/lib.rs`
    - `sample_game/tests/flow_contract.rs`
    - `sample_game/tests/ffi_input_boundary.rs`
    - `sample_game/Cargo.toml`
    - `sample_game/Cargo.lock`
    - `logic/Cargo.toml`
    - `logic/Cargo.lock`
    - `logic/src/lib.rs`
    - `logic/src/perf.rs`
    - `core/src/main.cpp`
    - `core/src/renderer/MeshManager.hpp`
    - `core/src/renderer/MeshManager.cpp`
    - `assets/meshes/arena_cube.obj`
    - `docs/COMPLETION_ROADMAP.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/GAME_DEVELOPMENT_TRACK.md`
    - `docs/SAMPLE_GAME_CORE_BOUNDARY.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
    - `docs/LOGIC_PUBLIC_API_INVENTORY.md`
    - `docs/CORE_3D_FOUNDATION_CONTRACT.md`
    - `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md`
    - `docs/CORE_3D_PRODUCTION_BASELINE.md`
    - `PLAN.md`
- 検証:
  - `cargo test --manifest-path logic/Cargo.toml --lib -- --nocapture`
  - `cargo test --manifest-path sample_game/Cargo.toml -- --nocapture`
  - `cmake --build build -j`
  - `./scripts/test_game_track_g2.sh`
  - `./scripts/test_distribution_smoke.sh`
- 未解決:
  - C1 完了には外部サンプルでの再利用確認と証跡追加が残る。
  - 3D 系の次段は `C2-03` / `C2-04` / `G4-02` / `G4-03`。

### 2026-03-07 run: manual issue-343-350 境界最小分離と 3D 正本化

- 背景:
  - C1 の残件だった `sample_game` / `logic` 境界の最小分離と、Wave 5-7 の 3D 着手条件がまだ正本化されていなかった。
- 変更:
  - `sample_game/src/lib.rs` に `SampleGameState` / `SampleGameButtonAction` / `SampleGameLoop` を追加し、`sample_game/tests/flow_contract.rs` で scene/action 契約を固定した。
  - `logic/src/ui.rs` を action id ベースの汎用 UI 部品へ縮小し、`logic/src/lib.rs` には現行 `get_miyabi_vtable()` 起動経路を維持する互換 shim を追加した。
  - `docs/CORE_3D_FOUNDATION_CONTRACT.md` / `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md` / `docs/CORE_3D_PRODUCTION_BASELINE.md` を追加し、トラック文書・ロードマップ・`PLAN.md` を同期した。
  - 関連ファイル:
    - `sample_game/src/lib.rs`
    - `sample_game/tests/flow_contract.rs`
    - `sample_game/Cargo.toml`
    - `logic/src/ui.rs`
    - `logic/src/lib.rs`
    - `docs/SAMPLE_GAME_CORE_BOUNDARY.md`
    - `docs/CORE_3D_FOUNDATION_CONTRACT.md`
    - `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md`
    - `docs/CORE_3D_PRODUCTION_BASELINE.md`
    - `docs/CORE_DEVELOPMENT_TRACK.md`
    - `docs/GAME_DEVELOPMENT_TRACK.md`
    - `docs/COMPLETION_ROADMAP.md`
    - `docs/LOGIC_PUBLIC_API_INVENTORY.md`
    - `PLAN.md`
- 検証:
  - `cargo test --manifest-path logic/Cargo.toml --lib -- --nocapture`
  - `cargo test --manifest-path sample_game/Cargo.toml -- --nocapture`
  - `cmake --build build -j`
  - `./scripts/test_game_track_g2.sh`
  - `./scripts/test_distribution_smoke.sh`
- 未解決:
  - `core` は依然として `logic` staticlib の `get_miyabi_vtable()` を起点に起動しており、`sample_game` 直接 boot への最終反転は未実施。
### 2026-03-07 run: manual 配布生成物 dist を gitignore へ追加

- 背景:
  - `scripts/package_macos_game.sh` による配布生成物 `dist/` が未追跡差分として残り、通常のレビュー差分と混ざっていた。
- 変更:
  - `.gitignore` に `/dist/` を追加し、配布 ZIP と展開済み配布物を Git 管理対象から外した。
  - 関連ファイル:
    - `.gitignore`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `git status --short --branch`
  - `git diff --check`
- 未解決:
  - 既に手元に生成済みの `dist/` 自体はローカルに残るが、以後は未追跡差分として表示されない。

### 2026-03-07 run: manual 完成ロードマップを追加

- 背景:
  - `codex:order` で完成直結の開発 Issue を積むために、C0/G1 から G2/G3/C1 へ向かう優先順を 1 枚で固定する必要があった。
- 変更:
  - `docs/COMPLETION_ROADMAP.md` を追加し、Wave 0 から Wave 7 までの到達順、完了条件、直近 90 日の優先順を定義した。
  - `README.md` に完成ロードマップへの導線を追加した。
  - 関連ファイル:
    - `docs/COMPLETION_ROADMAP.md`
    - `README.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `rg -n "COMPLETION_ROADMAP|Wave 0|Wave 1|codex:order" docs/COMPLETION_ROADMAP.md README.md`
- 未解決:
  - GitHub 上の open Issue へ本ロードマップ番号を反映する作業は未実施。

### 2026-03-07 run: manual 開発Issueラベル運用を codex:order 起点へ整理

- 背景:
  - 手動レーンで repo 文脈に合わせて開発 Issue を整形し、自動レーン投入前の状態と `codex:queue` を分離して扱いたくなった。
- 変更:
  - `docs/ISSUE_DESIGN.md` に `codex:order` / `codex:queue` / `codex:claimed` / `codex:blocked` の役割と推奨ライフサイクルを追加した。
  - 開発 Issue テンプレートと運用メモに `codex:order` 起点の運用を追記した。
  - 関連ファイル:
    - `docs/ISSUE_DESIGN.md`
    - `.github/ISSUE_TEMPLATE/development_task.md`
    - `README.md`
    - `AGENTS.md`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `rg -n "codex:order|codex:queue|codex:claimed|codex:blocked" docs/ISSUE_DESIGN.md .github/ISSUE_TEMPLATE/development_task.md README.md AGENTS.md`
- 未解決:
  - GitHub 側の実ラベル作成と既存 open Issue への付け替えは未実施。

### 2026-03-07 run: manual ローカル生成物の gitignore 追加

- 背景:
  - `artifacts/` や `build_*`、Python の `__pycache__` が未追跡差分として残り、レビュー対象との差分判読を妨げていた。
- 変更:
  - `.gitignore` にローカル自動化生成物、アドホックビルド出力、Python キャッシュを追加した。
  - 関連ファイル:
    - `.gitignore`
    - `docs/CODEX_MIGRATION_STATUS.md`
- 検証:
  - `git diff --check`
  - `git status --short --branch`
- 未解決:
  - 既存の ignore ルールが他のローカル生成物を十分に網羅しているかの棚卸しは未実施。

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
