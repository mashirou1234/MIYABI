# MIYABI

MIYABI は C++ ベースのゲーム/SDK 開発リポジトリです。  
この README は入口案内に限定し、詳細仕様は各正本ドキュメントを参照します。

## まず読む

- [PLAN.md](PLAN.md): 全体計画と優先度
- [docs/DEVELOPMENT_TRACK.md](docs/DEVELOPMENT_TRACK.md): 開発トラック運用
- [docs/CODEX_MIGRATION_STATUS.md](docs/CODEX_MIGRATION_STATUS.md): Codex 移行状況

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

## AGENTS 運用メモ

- 作業開始時に `artifacts/` を確認する
- 自動実行は `1 run 1 issue` を守る
