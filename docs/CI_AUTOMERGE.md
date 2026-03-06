# PR 自動承認 + CI 成功時自動マージ運用

最終更新: 2026-03-03

## 1. 目的

プルリクエストを自動承認し、CI（Woodpecker）の必須チェックがすべて成功した場合のみ自動マージする。

## 2. 実装内容

- CI 設定: `.woodpecker.yml`
- 自動承認スクリプト: `scripts/woodpecker_pr_automerge.sh`
- トリガ: Woodpecker の `pull_request` イベント
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
2. `Settings > Branches` で `master`（または `main`）の Branch protection rule を設定し、Woodpecker の必須チェックを Required にする
3. Woodpecker の secret `github_token` に PR 承認/更新可能なトークンを設定する

※ Required checks は実際の報告名（例: `ci/woodpecker/pr/woodpecker`）に一致させること。

## 4. 運用フロー

1. PR 作成/更新
2. Woodpecker が PR 自動承認と auto-merge 有効化を実行
3. Woodpecker が CI を実行
4. Required checks がすべて成功した時点で GitHub が自動マージ

## 5. 制御と注意点

- 自動マージを止めたい場合は `automerge:off` ラベルを付与する
- 外部 fork 由来 PR は対象外
- 署名付きレビューや Code Owners など追加ルールがある場合は、その条件を満たさない限りマージされない

## 5.1 自動マージが走らないときの確認ポイント（15〜45分）

`scripts/woodpecker_pr_automerge.sh` のスキップ分岐に沿って、次の順で確認する。

1. トークン設定を確認する
   - `GH_TOKEN` または `GITHUB_TOKEN` が CI secret に設定されているか。
2. PR 実行コンテキストを確認する
   - `CI_REPO`（または `CI_REPO_OWNER` + `CI_REPO_NAME`）と `CI_COMMIT_PULL_REQUEST` が渡っているか。
3. PR 属性を確認する
   - Draft ではないか。
   - base ブランチが `master` / `main` か。
   - head が同一リポジトリ由来か（fork ではないか）。
   - 作成者の `author_association` が `OWNER` / `MEMBER` / `COLLABORATOR` か。
4. ラベル制御を確認する
   - `automerge:off` が付いていないか。
5. GitHub 側設定を確認する
   - リポジトリ設定で `Allow auto-merge` が有効か。
   - Branch protection の Required checks 名が実際の CI 報告名と一致しているか。

補足: #169 は「適用除外条件」の整理が対象。本節は「スキップ時の切り分け手順」を追加し、同じ `automerge:off` を使う運用でも用途を分離している。

## 6. 自動マージ除外ケース（手動レビュー必須）

次の変更は、CI が成功しても `automerge:off` を付与して手動レビューを行う。

1. 保存データ互換性に影響する変更
   - 対象例: `logic/src/save.rs` の `SAVE_SCHEMA_VERSION` 更新、`save_version`/`payload` 契約の変更
   - 根拠: `docs/CORE_SAVE_SUBSYSTEM.md`
2. 性能ベースラインや回帰判定ルールに影響する変更
   - 対象例: `docs/perf/baseline_macos14.json`、`tools/check_perf_regression.py`、`PERFORMANCE_TEST.md` の閾値/判定手順変更
   - 根拠: `PERFORMANCE_TEST.md`
3. CI 必須チェック定義そのものの変更
   - 対象例: Branch protection の Required checks 名変更、`.github/workflows/build.yml` の判定ジョブ変更
4. 自動マージ制御や権限境界に影響する変更
   - 対象例: `scripts/woodpecker_pr_automerge.sh` の承認条件変更、`OWNER` / `MEMBER` / `COLLABORATOR` 判定ロジック変更、`github_token` の権限前提変更
   - 根拠: 本ドキュメント「2. 実装内容」「3. GitHub 側の必須設定」

### 6.1 除外時の運用手順

1. PR 作成時に `automerge:off` を付与する。
2. PR 説明に「除外理由」「影響範囲」「復旧手順」を記載する。
   - どの除外ケース（6章の番号）に該当するかを明記する。
3. 手動レビュー完了後、必要に応じて `automerge:off` を外す。

## 7. 互換性破壊変更時の告知テンプレート

ABI 変更や公開手順の破壊的変更など、既存利用者へ影響する変更を含む PR では、`automerge:off` を付与したうえで以下テンプレートを PR 本文または Issue コメントに記載する。

```md
## 互換性破壊変更の告知
- 対象PR/Issue: #<番号>
- 変更種別: ABI破壊 / 手順破壊 / 設定破壊（該当を残す）
- 影響範囲: <利用者・モジュール・環境>
- 破壊内容: <何が互換でなくなるか>
- 移行手順:
  1. <手順1>
  2. <手順2>
- ロールバック手順: <失敗時の戻し方>
- 適用予定日: YYYY-MM-DD
- 告知先: README / RELEASE_NOTE / Issue / PR コメント
```

告知時は次の3点を最低限確認する。

1. `docs/SDK_DEFINITION.md` の互換性ルールと整合していること
2. `PERFORMANCE_TEST.md` など関連手順書の更新有無を明記すること
3. 適用前に最小再現テストと回帰テスト結果を添えること
