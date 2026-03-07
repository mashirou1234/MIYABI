# MIYABI 開発トラック案内

最終更新: 2026-03-07

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
- logic公開API棚卸し:
  - `docs/LOGIC_PUBLIC_API_INVENTORY.md`
- Issue設計と開発Issueテンプレート:
  - `docs/ISSUE_DESIGN.md`

## 4. フェーズ間依存（`PLAN.md`準拠）

- フェーズ1 完了 → フェーズ2 開始
- フェーズ2 完了 → フェーズ3 開始
- フェーズ3 完了 → フェーズ4 開始
- フェーズ4 完了 → フェーズ5 開始
- フェーズ5 完了 → フェーズ6 開始
- フェーズ6 完了 → フェーズ7 開始
- フェーズ7 完了 → フェーズ8 開始
- フェーズ8 完了 → フェーズ9 開始
- フェーズ9 完了 → フェーズ10 開始
- フェーズ10 完了 → フェーズ11 開始
- フェーズ11 完了 → フェーズ12 開始
- フェーズ12 完了 → フェーズ13 開始
- フェーズ13 完了 → フェーズ14 開始

依存判定の正本は `PLAN.md` とし、到達判定（DoD）は各トラック正本（`docs/CORE_DEVELOPMENT_TRACK.md` / `docs/GAME_DEVELOPMENT_TRACK.md`）を参照する。

## 5. 運用ルール

- コア到達判定は `docs/CORE_DEVELOPMENT_TRACK.md` を正とする
- ゲーム到達判定は `docs/GAME_DEVELOPMENT_TRACK.md` を正とする
- 実装Taskの Issue は `docs/ISSUE_DESIGN.md` のテンプレートと分割ルールに従う
- 作業完了時は `PLAN.md` と `docs/CODEX_MIGRATION_STATUS.md` を同時更新する

## 6. 依存解決ブロッカー記録欄

フェーズ間依存（本書 4 章）で停止が発生した場合は、`PLAN.md` の該当タスク更新と同時に以下を追記する。

- 記録対象:
  - フェーズ開始条件を満たせず、当日中に解消できない依存ブロッカー
  - 外部要因（環境・権限・上流仕様未確定）で着手不能になった事項
- 更新タイミング:
  - ブロッカーを検知した当日の作業終了までに1行追加
  - 解消時は同じ行の `状態` を `resolved` に更新し、`解消日` を記録

| 記録日 | 関連フェーズ/タスク (`PLAN.md`) | ブロッカー内容 | 依存先 | 暫定対応 | 状態 | 解消日 |
|---|---|---|---|---|---|---|
| (例) 2026-03-07 | Phase 10 / タスク10.1 | 入力マッピング仕様の承認待ちで実装開始不可 | 仕様レビュー | 既存入力APIの棚卸しのみ先行 | open | - |

判断に迷う場合は、到達判定は `docs/CORE_DEVELOPMENT_TRACK.md`、作業順は `PLAN.md` を優先して整合を取る。
