# PR 自動承認 + CI 成功時自動マージ運用

最終更新: 2026-03-03

## 1. 目的

プルリクエストを自動承認し、CI（Woodpecker）の必須チェックがすべて成功した場合のみ自動マージする。

## 2. 実装内容

- ワークフロー: `.github/workflows/pr-auto-approve-merge.yml`
- トリガ: `pull_request_target`（`opened/reopened/synchronize/ready_for_review/labeled/unlabeled`）
- 処理:
  - PR を自動承認
  - PR の auto-merge（`squash`）を有効化
- 実行条件:
  - Draft ではない
  - ベースブランチが `master` または `main`
  - PR が同一リポジトリ内ブランチ由来
  - 作成者が `OWNER` / `MEMBER` / `COLLABORATOR`
  - `automerge:off` ラベルが付いていない

## 3. GitHub 側の必須設定

1. `Settings > General > Pull Requests > Allow auto-merge` を ON
2. `Settings > Actions > General > Workflow permissions` を `Read and write permissions`
3. `Settings > Actions > General > Allow GitHub Actions to create and approve pull requests` を ON
4. `Settings > Branches` で `master`（または `main`）の Branch protection rule を設定し、Woodpecker の必須チェックを Required にする

※ Required checks には、実際に GitHub 上へ報告されるチェック名を指定すること。

## 4. 運用フロー

1. PR 作成/更新
2. 本ワークフローが PR を自動承認し、auto-merge を有効化
3. Woodpecker が CI を実行
4. Required checks がすべて成功した時点で GitHub が自動マージ

## 5. 制御と注意点

- 自動マージを止めたい場合は `automerge:off` ラベルを付与する
- 外部 fork 由来 PR は対象外
- 署名付きレビューや Code Owners など追加ルールがある場合は、その条件を満たさない限りマージされない
