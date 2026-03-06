# MIYABI

MIYABI は C++ ベースのゲーム/SDK 開発リポジトリです。  
この README は入口案内に限定し、詳細仕様は各正本ドキュメントを参照します。

## まず読む

- [PLAN.md](PLAN.md): 全体計画と優先度
- [docs/DEVELOPMENT_TRACK.md](docs/DEVELOPMENT_TRACK.md): 開発トラック運用
- [docs/CODEX_MIGRATION_STATUS.md](docs/CODEX_MIGRATION_STATUS.md): Codex 移行状況

## 主要ドキュメント逆引き

- 進め方を把握したい: [PLAN.md](PLAN.md), [docs/DEVELOPMENT_TRACK.md](docs/DEVELOPMENT_TRACK.md)
- 設計判断を確認したい: [DESIGN.md](DESIGN.md), [docs/DESIGN_Renderer.md](docs/DESIGN_Renderer.md), [docs/DESIGN_ECS.md](docs/DESIGN_ECS.md), [docs/DESIGN_FFI.md](docs/DESIGN_FFI.md)
- 進捗や移行状況を追いたい: [docs/CODEX_MIGRATION_STATUS.md](docs/CODEX_MIGRATION_STATUS.md), [docs/CORE_DEVELOPMENT_TRACK.md](docs/CORE_DEVELOPMENT_TRACK.md), [docs/GAME_DEVELOPMENT_TRACK.md](docs/GAME_DEVELOPMENT_TRACK.md)
- SDK/配布まわりを確認したい: [docs/SDK_DEFINITION.md](docs/SDK_DEFINITION.md), [docs/DISTRIBUTION_1OS.md](docs/DISTRIBUTION_1OS.md)
- 品質・性能検証を確認したい: [PERFORMANCE_TEST.md](PERFORMANCE_TEST.md), [docs/perf/PERF_BASELINE.md](docs/perf/PERF_BASELINE.md)

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

SDK 生成スクリプトの前提依存を確認する場合は `./build_sdk.sh --help` を参照してください。

## 新規Contributor向け 最短ビルド確認（15分目安）

1. 依存確認: `cmake --version` と `c++ --version` が実行できることを確認
2. 設定生成: `cmake -S . -B build`
3. ビルド実行: `cmake --build build -j`

確認できれば、次は以下を参照:
- [docs/CORE_DEVELOPMENT_TRACK.md](docs/CORE_DEVELOPMENT_TRACK.md): 現在ステージと次ゲート
- [PLAN.md](PLAN.md): 着手順と優先タスク

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
