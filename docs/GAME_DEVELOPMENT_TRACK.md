# MIYABI ゲーム開発トラック（ユーザー開発）

最終更新: 2026-03-07

## 1. 目的

サンプルゲーム（ドッグフーディング）を通じて、実際に「遊べるゲーム」を完成させる。  
エンジンの不足を実プレイ導線で発見し、コア開発へ還元する。

## 2. ステージ定義

### G0: 2D縦切り仕様固定

- ゲームループ/勝敗条件/シーン遷移を固定
- 最小セーブ/ロード仕様を固定
- 正本: `docs/SPEC_SAMPLE_GAME_2D_VERTICAL_SLICE.md`

### G1: 2Dプレイアブルループ成立

- プレイヤー操作、障害物、勝敗遷移が動作
- タイトル/ゲーム/リザルトを通しでプレイ可能

### G2: 2D Vertical Slice 完成

- 30分連続プレイで進行不能なし
- 既知の致命的不具合なし

### G3: 2D配布候補（1OS）

- 配布ビルド手順が固定され再現可能
- 初回起動から終了までを第三者が再現できる

### G4: 3D Vertical Slice（将来）

- 3Dプレイアブルデモを1本完成
- 3D向けUI/遷移/基本ループを成立

## 3. 現在地

- 現在ステージ: **G3**
- 次ゲート: **G4**
- 直近不足:
  - 3D run の勝敗導線 (`G4-02`)
  - 3D 障害物 1 系統 (`G4-03`)
  - directional light / 2D-3D 共存回帰 (`C2-03` / `C2-04`)

## 4. 管理ドキュメント

- 2D 仕様: `docs/SPEC_SAMPLE_GAME_2D_VERTICAL_SLICE.md`
- G4 仕様: `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md`
- 作業タスク: `PLAN.md`
- 変更履歴: `docs/CODEX_MIGRATION_STATUS.md`

## 5. 検証環境前提

同一手順で再現できることを優先し、G1-G2 の検証は次の前提環境を満たす。

- OS: macOS 14 系（Apple Silicon）
- ビルドツール: `cmake` と `c++` が PATH から実行可能
- 実行前提: `README.md` の最短ビルド確認手順（`cmake -S . -B build` / `cmake --build build -j`）が通ること
- 検証ログ: 実施時は使用OS、コンパイラ版数、実行コマンドを PR または Issue コメントに記録する

環境差がある場合は、先に差分（OS/コンパイラ/依存）を明記してから結果を比較する。

## 5.1 G2 30分連続プレイ検証手順

G2 証跡は、状態遷移確認と 30 分相当の長時間安定性確認を同じ run で残す。

- 最小再現コマンド:
  - `./scripts/test_game_track_g2.sh`
- 内訳:
  - `title_pause_result_and_exit_flow_is_reachable`: Title -> InGame -> Pause -> Result -> Title -> Exit 要求までの導線を headless に確認する
  - `headless_g2_stability_run_reaches_clear_with_safe_corridor`: 安全通路を維持した headless ハーネスで 30 分相当の更新を流し、進行不能やクラッシュなく `CLEAR` へ到達することを確認する
- 任意の追加確認:
  - `cmake -S . -B build && cmake --build build -j`
  - `build/core/miyabi` を起動し、5 分程度の目視操作で Title / InGame / Pause / Result の見え方を確認する

Issue コメントまたは PR に残すテンプレート:

```md
### G2 検証ログ YYYY-MM-DD

- Environment:
  - OS: <例: macOS 14.x (Apple Silicon)>
  - Compiler: <`c++ --version` の要点>
- Commands:
  - `./scripts/test_game_track_g2.sh`
  - 任意: `cmake -S . -B build && cmake --build build -j`
- Result:
  - `title_pause_result_and_exit_flow_is_reachable`: PASS / FAIL
  - `headless_g2_stability_run_reaches_clear_with_safe_corridor`: PASS / FAIL
  - 手動 5 分導線確認: PASS / FAIL / 未実施
- Findings:
  - <なければ `なし`>
- Evidence URL:
  - <Issue comment or PR URL>
```

## 5.2 G3 配布再現スモーク手順

G3 証跡は、配布 ZIP の生成、展開、最低同梱物確認、起動スモークまでを 1 コマンドで再実行できることを優先する。

- 最小再現コマンド:
  - `./scripts/test_distribution_smoke.sh`
- 期待結果:
  - 最新の `dist/MIYABI_GAME_macOS_<timestamp>.zip` が生成される
  - 展開先に最低同梱物が揃う
  - `./run_miyabi.sh` が 5 秒以上生存し、即時クラッシュしない

Issue コメントまたは PR に残すテンプレート:

```md
### G3 配布再現ログ YYYY-MM-DD

- Environment:
  - OS: <例: macOS 14.x (Apple Silicon)>
- Commands:
  - `./scripts/test_distribution_smoke.sh`
- Result:
  - ZIP 生成: PASS / FAIL
  - 同梱物確認: PASS / FAIL
  - 5 秒起動スモーク: PASS / FAIL
- Evidence URL:
  - <Issue comment or PR URL>
```

## 5.3 G4 着手前の固定事項

- 正本: `docs/SPEC_SAMPLE_GAME_3D_VERTICAL_SLICE.md`
- 最初の実装 Task:
  - `G4-01`: 3D arena の最小起動（2026-03-08 実装済み）
  - `G4-02`: 3D run の勝敗導線
  - `G4-03`: 3D 障害物 1 系統

## 6. マイルストーン証跡リンク

各ステージの到達判定を更新するときは、同時に「証跡リンク」を 1 件以上記録する。
リンク先は GitHub 上で第三者が閲覧できるもの（Issue コメント、PR コメント、PR 本文、手順書更新コミット）に限定する。

| ステージ | 到達判定の参照先 | 証跡リンク | 更新条件 |
| --- | --- | --- | --- |
| G0 | 本書「2. ステージ定義 / G0」 | 未設定 | 仕様固定を宣言した Issue/PR を記録 |
| G1 | 本書「2. ステージ定義 / G1」 | https://github.com/mashirou1234/MIYABI/issues/341#issuecomment-4016476986 | 通しプレイ成立を示す Issue/PR を記録 |
| G2 | 本書「2. ステージ定義 / G2」 | https://github.com/mashirou1234/MIYABI/issues/340#issuecomment-4016476977 | 30分連続プレイ結果を示す Issue/PR を記録 |
| G3 | 本書「2. ステージ定義 / G3」 | https://github.com/mashirou1234/MIYABI/issues/342#issuecomment-4016476990 | 配布手順の再現結果を示す Issue/PR を記録 |
| G4 | 本書「2. ステージ定義 / G4」 | 未設定 | 3D縦切り到達を示す Issue/PR を記録 |

記入手順:
1. ステージ更新時に、該当行の「証跡リンク」を `https://github.com/mashirou1234/MIYABI/...` 形式で更新する。
2. 証跡リンクがない状態では「3. 現在地」のステージ値を進めない。

## 7. handoff 時の最小記録項目

ゲーム開発トラックの引き継ぎでは、1 着手単位ごとに次の 5 項目を残す。

1. 対象ステージと目的（例: `G1 -> G2`、今回の到達目標）
2. 実施結果（完了 / 未完了と、その理由を 1 行）
3. 実行コマンド（再現に使った最小コマンドのみ）
4. 生成物への導線（PR、Issue コメント、ログの URL）
5. 次担当の最初の 1 アクション（15-45 分で着手できる粒度）

記録先は PR 本文または Issue コメントとし、未記録のまま handoff しない。
