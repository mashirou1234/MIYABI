# MIYABI 開発トラック案内

最終更新: 2026-02-23

本プロジェクトの開発ドキュメントは、誤解防止のため以下の2系統に分離する。

## 1. コア開発（システム開発）

- 正本: `docs/CORE_DEVELOPMENT_TRACK.md`
- 対象:
  - エンジンランタイム（FFI/ECS/レンダラ/物理/音声/入力）
  - SDK/API/ABI
  - CI・性能基盤・ツール・運用

## 2. ゲーム開発（ユーザー開発）

- 正本: `docs/GAME_DEVELOPMENT_TRACK.md`
- 対象:
  - サンプルゲーム/縦切りの仕様・実装・受け入れ
  - プレイ導線、ゲームルール、セーブ/ロード、配布導線

## 3. 関連仕様

- 2D縦切り仕様（サンプルゲーム）:
  - `docs/SPEC_SAMPLE_GAME_2D_VERTICAL_SLICE.md`
- コア Save サブシステム定義:
  - `docs/CORE_SAVE_SUBSYSTEM.md`
- アセット import/reimport 手順:
  - `docs/ASSET_IMPORT_REIMPORT.md`

## 4. 運用ルール

- コア到達判定は `docs/CORE_DEVELOPMENT_TRACK.md` を正とする
- ゲーム到達判定は `docs/GAME_DEVELOPMENT_TRACK.md` を正とする
- 作業完了時は `PLAN.md` と `docs/CODEX_MIGRATION_STATUS.md` を同時更新する
