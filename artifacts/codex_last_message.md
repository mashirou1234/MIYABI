**変更内容**
- `docs/SAMPLE_GAME_CORE_BOUNDARY.md:5`, `docs/SAMPLE_GAME_CORE_BOUNDARY.md:23`, `docs/SAMPLE_GAME_CORE_BOUNDARY.md:33`, `docs/SAMPLE_GAME_CORE_BOUNDARY.md:45` に `core/logic/sample_game` のレイヤーモデル、現状の依存関係、許可/禁止依存ルール、次に分離すべき箇所を集約し、1ページで境界判断ができる指針を用意しました。
- `docs/CORE_DEVELOPMENT_TRACK.md:41` と `docs/CODEX_MIGRATION_STATUS.md:166` から新境界ドキュメントへの参照を追加し、トラック文書から直接リンクできるよう整備しました。

**テスト**
- `rg "sample_game|core|責務" docs`：`docs/SAMPLE_GAME_CORE_BOUNDARY.md` および既存文書からの参照が検出されることを確認。

**リスク・フォローアップ**
- 実装はまだ境界に沿っていない（`logic/src/lib.rs` にサンプルゲーム実装が残る等）ため、ドキュメントに沿った段階的なコード分離が必要。