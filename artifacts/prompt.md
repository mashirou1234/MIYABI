# Codex Orchestration Task

You are running in non-interactive mode for the repository currently checked out.

## Context
- SCM: github
- Issue: #1
- Title: [Bench-80][01] sample_game/core責務境界を文書化する
- URL: https://github.com/mashirou1234/MIYABI/issues/1

## Issue Body
## 背景
C1到達の直近不足として、`sample_game` と `core` の責務再分離が未完了です。
境界ルールが散在しており、実装判断がスレッド依存になっています。

## 目的
`sample_game` と `core` の責務境界を1ページで明文化し、参照先を固定する。

## 作業範囲 In
- 現在の依存方向（`core` / `logic` / `sample_game`）を整理する
- 境界ルール（許可依存・禁止依存）を文書化する
- 既存ドキュメント（`docs/CORE_DEVELOPMENT_TRACK.md` など）から参照リンクを追加する

## 作業範囲 Out
- 実コードの大規模リファクタリング
- GUI仕様の追加・変更

## 受け入れ基準
- 境界ルールを記したドキュメントが `docs/` 配下に追加されている
- 許可/禁止依存が箇条書きで明示されている
- 既存のトラック文書からリンク可能になっている
- 読むだけで「次に分離すべき箇所」が判断できる

## テスト
1. 追加ドキュメントが `docs/` 配下に存在することを確認する。
2. `rg "sample_game|core|責務" docs` で関連文書から参照できることを確認する。

## リスク
文書だけ先行して実装が追従しないと乖離が発生する。

## Hard Constraints
- Do not run `git commit` or `git push`.
- Do not edit `.git`, `.codex`, or secrets.
- Avoid network-dependent commands unless already prepared by setup.
- Keep changes minimal and focused on this issue.

## Expected Output (must be in final message)
1. Summary of changes made
2. Tests/verification commands executed and results
3. Remaining risks / follow-up checks for reviewer
