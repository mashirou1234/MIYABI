# MIYABI

MIYABI は C++ ベースのゲーム/SDK 開発リポジトリです。  
この README は入口案内に限定し、詳細仕様は各正本ドキュメントを参照します。

## まず読む

- [PLAN.md](PLAN.md): 全体計画と優先度
- [docs/DEVELOPMENT_TRACK.md](docs/DEVELOPMENT_TRACK.md): 開発トラック運用
- [docs/CODEX_MIGRATION_STATUS.md](docs/CODEX_MIGRATION_STATUS.md): Codex 移行状況

## 主要ドキュメント逆引き

- 進め方を把握したい: [PLAN.md](PLAN.md), [docs/DEVELOPMENT_TRACK.md](docs/DEVELOPMENT_TRACK.md)
- 完成までの優先順を把握したい: [docs/COMPLETION_ROADMAP.md](docs/COMPLETION_ROADMAP.md)
- Issue設計と自動化向けテンプレートを確認したい: [docs/ISSUE_DESIGN.md](docs/ISSUE_DESIGN.md)
- 設計判断を確認したい: [DESIGN.md](DESIGN.md), [docs/DESIGN_Renderer.md](docs/DESIGN_Renderer.md), [docs/DESIGN_ECS.md](docs/DESIGN_ECS.md), [docs/DESIGN_FFI.md](docs/DESIGN_FFI.md)
- 進捗や移行状況を追いたい: [docs/CODEX_MIGRATION_STATUS.md](docs/CODEX_MIGRATION_STATUS.md), [docs/CORE_DEVELOPMENT_TRACK.md](docs/CORE_DEVELOPMENT_TRACK.md), [docs/GAME_DEVELOPMENT_TRACK.md](docs/GAME_DEVELOPMENT_TRACK.md)
- SDK/配布まわりを確認したい: [docs/SDK_DEFINITION.md](docs/SDK_DEFINITION.md), [docs/DISTRIBUTION_1OS.md](docs/DISTRIBUTION_1OS.md)
- 品質・性能検証を確認したい: [PERFORMANCE_TEST.md](PERFORMANCE_TEST.md)（3.2 命名規則）, [docs/perf/PERF_BASELINE.md](docs/perf/PERF_BASELINE.md)

## リポジトリ構成（概要）

- `core/`: コアエンジン実装
- `logic/`: ゲームロジック
- `sample_game/`: サンプル実行コード
- `docs/`: 開発・運用ドキュメント
- `scripts/`: 補助スクリプト

## 最短ビルド例

```bash
cmake -S . -B build
cmake --build build -j
```

## CMake Presets 最小ビルド手順

```bash
cmake --list-presets
cmake --preset dev
cmake --build --preset dev -j
```

SDK 生成スクリプトの前提依存を確認する場合は `./build_sdk.sh --help` を参照してください。

## 主要 CMake オプション（抜粋）

| オプション | 既定値 | 目的 | 使う場面 |
| --- | --- | --- | --- |
| `MIYABI_PROFILE` | `OFF` | プロファイリング計測用の計装を有効化する | 処理内訳を計測したいときだけ `ON` |
| `MIYABI_PERFORMANCE_TEST` | `OFF` | 性能計測専用のベンチマーク/テスト経路を有効化する | `perf_baseline` 実行や CI 性能計測時だけ `ON` |

補足: `MIYABI_PERFORMANCE_TEST` は通常開発・通常リリースでは `OFF` のまま運用し、手順詳細は [PERFORMANCE_TEST.md](PERFORMANCE_TEST.md) の 4.9 を参照してください。

### `build_sdk.sh` validate-only 確認（15〜30分目安）

既存 `sdk/` 配下の必須同梱物だけを検証し、再ビルドせずに不足検出したい場合の最短手順です。

```bash
MIYABI_SDK_VALIDATE_ONLY=1 ./build_sdk.sh
```

確認ポイント:
- 成功時は `Required SDK artifacts check passed.` が出力されること
- 失敗時は `ERROR: Required SDK artifacts are missing:` の下に不足ファイル一覧が出ること
- 検証対象の必須同梱物は `build_sdk.sh` の `REQUIRED_ARTIFACTS`（`docs/SDK_DEFINITION.md` を含む）と一致していること

### SDK 外部サンプル再利用スモーク（15〜30分目安）

SDK を temp project へコピーし、`find_package(MIYABI CONFIG REQUIRED)` の外部利用導線が実際に通るかを確認する最短手順です。

```bash
./build_sdk.sh
./scripts/test_sdk_external_sample_reuse.sh ./sdk
```

確認ポイント:
- temp project が `sdk/template_CMakeLists.txt` と `sdk/examples/main.cpp` だけで configure/build/run できること
- `sample_game` ではなく SDK 同梱サンプルが `MIYABI::SDK` を経由して起動できること
- 終了時に `[sdk-reuse] PASS:` が出力されること

### C2 2D/3D 共存回帰ハーネス（15〜30分目安）

2D title UI/text、3D arena overlay/render log、`G2/G3` スモークを 1 コマンドで再実行する最短手順です。

```bash
./scripts/test_core_2d_3d_coexistence.sh
```

確認ポイント:
- 2D title UI/text の証跡行 (`[c2-04][2d-title]`) が出力されること
- 3D arena overlay/render の証跡行 (`[c2-04][3d-arena]`) が出力されること
- `artifacts/c2_04_2d_3d_coexistence_latest.log` に最新の summary が残ること

### G4 3D vertical slice スモーク（15〜30分目安）

3D arena の Pause / GAME OVER / CLEAR / Retry、障害物 fail/clear、設定保持を 1 コマンドで headless 再実行する最短手順です。

```bash
./scripts/test_game_track_g4.sh
```

確認ポイント:
- Pause / Back To Title の証跡行 (`[g4-02][pause-back]`) が出力されること
- GAME OVER の証跡行 (`[g4-02][game-over]`) が出力されること
- CLEAR / Retry の証跡行 (`[g4-02][clear-retry]`) が出力されること
- 障害物 spawn / fail / clear の証跡行 (`[g4-03][spawn]`, `[g4-03][fail]`, `[g4-03][clear]`) が出力されること
- 設定保持の証跡行 (`[g4][settings]`) が出力されること
- `artifacts/g4_vertical_slice_latest.log` に最新の summary が残ること

### C3 3D perf baseline スモーク（15〜30分目安）

代表 3D シーンの性能 baseline 採取と既存 baseline との差分判定を 1 コマンドで再実行する最短手順です。

```bash
./scripts/test_core_c3_3d_perf_baseline.sh
```

確認ポイント:
- `build/perf/current_baseline.json` が更新されること
- `build/perf/regression_report.md` に `arena3d_renderable_build` 行が出力されること
- `artifacts/c3_3d_perf_baseline_latest.log` に最新の summary が残ること

## 新規Contributor向け 最短ビルド確認（15分目安）

1. 依存確認: `cmake --version` と `c++ --version` が実行できることを確認
2. 設定生成: `cmake -S . -B build`
3. ビルド実行: `cmake --build build -j`

確認できれば、次は以下を参照:
- [docs/CORE_DEVELOPMENT_TRACK.md](docs/CORE_DEVELOPMENT_TRACK.md): 現在ステージと次ゲート
- [PLAN.md](PLAN.md): 着手順と優先タスク

## 配布前 preflight 確認（15分目安）

`scripts/package_macos_game.sh` を使った配布前チェックの最短手順です。

```bash
./scripts/package_macos_game.sh --preflight-only
```

確認ポイント:
- 出力に `[preflight] ok` が含まれること
- `cmake` / `zip` / `shasum` の必須コマンド不足エラーが出ないこと
- `assets/player.png` `assets/test.png` `assets/test_sound.wav` の欠落エラーが出ないこと

詳細な配布手順は [docs/DISTRIBUTION_1OS.md](docs/DISTRIBUTION_1OS.md) を参照してください。

## FFI 入力境界テスト（最小手順）

`sample_game` 側の FFI 入力ポインタ境界を確認する最小テストです。

```bash
cargo test --manifest-path sample_game/Cargo.toml --test ffi_input_boundary
```

## セーブ互換性チェック（最小手順）

1. スキーマ版数を確認する: `logic/src/save.rs` の `SAVE_SCHEMA_VERSION` を参照する。
2. 既存セーブを `save/save_data.json` に配置し、`save_version` が `SAVE_SCHEMA_VERSION` と一致することを確認する。
3. 破損データ時の挙動を確認する: `save/save_data.json` を意図的に壊して起動し、`*.bak` へ退避されることを確認する。
4. 最低限の回帰テストを実行する: `cargo test --manifest-path logic/Cargo.toml save::tests::load_version_mismatch_returns_error save::tests::load_corrupt_file_uses_next_backup_when_bak_exists`。
5. 詳細仕様が必要な場合は `docs/CORE_SAVE_SUBSYSTEM.md` と `docs/SPEC_SAMPLE_GAME_2D_VERTICAL_SLICE.md` を参照する。

## AGENTS 運用メモ

- 作業開始時に `artifacts/` を確認する
- 自動実行は `1 run 1 issue` を守る
- 手動で整形した開発 Issue は `codex:order`、自動レーン投入可能なものだけ `codex:queue` を付ける
