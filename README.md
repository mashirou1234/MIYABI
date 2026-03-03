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

## AGENTS 運用メモ

- 作業開始時に `artifacts/` を確認する
- 自動実行は `1 run 1 issue` を守る
