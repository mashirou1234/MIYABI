# MIYABI 2D縦切り仕様書 (Phase 9.1)

最終更新: 2026-03-04
対象: Stage 1 (2D Vertical Slice)

## 1. 目的

本仕様は、Phase 9.1 の「縦切り仕様の固定」を満たすための正本とする。  
本仕様に基づき、Phase 9.2 以降の実装・検証を行う。

## 2. 縦切りタイトル定義

- タイトル（仮）: `MIYABI Box Survival`
- ジャンル: 2D サバイバルアクション
- プレイ単位: 1ラン（最大 30 分）
- 目標: 30分間生存してクリアする

<a id="spec-game-loop"></a>
## 3. ゲームループ（確定）

1. タイトル画面で `ゲーム開始`
2. ゲーム本編を開始
3. プレイヤーは移動しながら障害を回避し、スコアを獲得
4. `HP <= 0` でゲームオーバー
5. `生存時間 >= 1800秒` でゲームクリア
6. リザルト画面で `リトライ` または `タイトルへ戻る`

### 3.1 勝敗条件

- 勝利条件:
  - `survival_time_sec >= 1800`
  - かつ `HP > 0`
- 敗北条件:
  - `HP <= 0`

### 3.2 スコア定義

- 基本スコア:
  - `score = survival_time_sec * 10 + avoid_count * 100`
- `avoid_count` は障害物を回避した回数
- 小数は切り捨て、整数で管理する

### 3.3 難易度推移

- 60秒ごとに難易度レベルを1段階上げる
- 難易度上昇で以下を増加:
  - 障害物の生成頻度
  - 障害物の移動速度

<a id="spec-scenes-ui"></a>
## 4. シーン構成と遷移（確定）

### 4.1 シーン一覧

1. `Title`
2. `InGame`
3. `Pause`
4. `Result`

### 4.2 遷移条件

- `Title -> InGame`
  - `ゲーム開始` ボタン押下
- `InGame -> Pause`
  - `ESC` 押下
- `Pause -> InGame`
  - `再開` ボタン押下
- `Pause -> Title`
  - `タイトルへ戻る` ボタン押下
- `InGame -> Result`
  - 勝利条件または敗北条件を満たす
- `Result -> InGame`
  - `リトライ` ボタン押下
- `Result -> Title`
  - `タイトルへ戻る` ボタン押下

### 4.3 UI要素（最小）

- `Title`
  - ゲーム開始
  - 終了
- `InGame HUD`
  - HP
  - 生存時間（秒）
  - スコア
  - 難易度レベル
- `Pause`
  - 再開
  - タイトルへ戻る
- `Result`
  - 結果（Clear / Game Over）
  - 最終スコア
  - 生存時間
  - ハイスコア表示
  - リトライ
  - タイトルへ戻る

## 5. 操作仕様（キーボード）

- 移動: `↑ ↓ ← →`
- 決定/UI操作: `Enter` / マウス左クリック
- ポーズ: `ESC`

<a id="spec-input-latency"></a>
## 5.1 入力遅延の計測観点（Phase 9.2 追加）

### 5.1.1 目的

- 操作体感の劣化を早期検知するため、`入力受付 -> 画面反映` を定量観測する
- 計測手順は `PERFORMANCE_TEST.md` の運用方針と矛盾しない形で記録する

### 5.1.2 計測対象イベント

- `←/→/↑/↓` 押下時のプレイヤー移動開始
- `ESC` 押下時の `InGame -> Pause` 遷移開始
- `Enter` 押下時のメニュー決定反映（Title/Pause/Result）

### 5.1.3 指標定義

- `input_to_simulation_ms`:
  - 入力を受理した時刻から、対応するゲーム状態更新が確定するまでの時間
- `input_to_present_ms`:
  - 入力を受理した時刻から、対応結果が画面に描画されるまでの時間
- `input_delay_frames`:
  - 入力受理フレームから画面反映フレームまでの差分フレーム数

### 5.1.4 合格ライン（暫定）

- 計測条件: 60FPS 相当、通常負荷（デバッグ表示のみ有効）
- `input_delay_frames <= 2` を維持する
- 95パーセンタイルの `input_to_present_ms <= 50ms` を維持する
- 単発スパイクは許容するが、連続3回以上の閾値超過は失敗とする

### 5.1.5 計測手順（最小）

1. 計測モードで sample_game を起動する
2. 対象イベントごとに30回入力し、`input_to_simulation_ms` / `input_to_present_ms` を記録する
3. 95パーセンタイルと最大値、`input_delay_frames` の閾値超過回数を集計する
4. 結果を PR または Issue コメントに添付し、必要なら `PERFORMANCE_TEST.md` の関連シナリオに追記する

<a id="spec-save-load"></a>
## 6. セーブ/ロード最小仕様（確定）

### 6.1 保存対象

`progress`
- `best_score`（最高スコア）
- `best_survival_sec`（最長生存時間）
- `total_play_count`（総プレイ回数）
- `total_clear_count`（総クリア回数）

`settings`
- `master_volume`（0.0〜1.0）
- `bgm_volume`（0.0〜1.0）
- `se_volume`（0.0〜1.0）
- `fullscreen`（bool）

### 6.2 保存タイミング

- リザルト遷移時
- 設定変更時
- アプリ終了時

### 6.3 読み込みタイミング

- アプリ起動時に1回

### 6.4 失敗時挙動

- 破損または未存在時はデフォルト値で起動
- 破損ファイルは `*.bak` に退避し、新規作成

### 6.5 ファイル仕様

- 形式: JSON
- 推奨パス: `./save/save_data.json`（実行ディレクトリ基準）
- 文字コード: UTF-8
- スキーマバージョン: `save_version`

サンプル:

```json
{
  "save_version": 1,
  "progress": {
    "best_score": 0,
    "best_survival_sec": 0,
    "total_play_count": 0,
    "total_clear_count": 0
  },
  "settings": {
    "master_volume": 1.0,
    "bgm_volume": 0.8,
    "se_volume": 0.8,
    "fullscreen": false
  }
}
```

<a id="spec-non-target"></a>
## 7. 非対象（Phase 9.1時点）

- ネットワーク要素
- 複数難易度モード
- リプレイ保存
- マップエディタ
- マルチプラットフォーム配布

<a id="spec-acceptance"></a>
## 8. 受け入れ基準（Phase 9.2実装完了時に検証）

1. タイトルからゲーム開始し、勝敗いずれかで必ずリザルトへ遷移する
2. リザルトからリトライ/タイトル戻りが機能する
3. 30分連続プレイで進行不能が発生しない
4. セーブデータが再起動後に反映される
5. セーブ破損時にクラッシュせずデフォルト復旧する

<a id="spec-plan-phase9-10-map"></a>
## 9. PLAN Phase 9/10 と縦切り仕様の対応表

参照元: `PLAN.md` の「Phase 9/10 仕様対応」節

### 9.1 Phase 9 タスクとの対応

| PLANタスク | 仕様項目 | 対応状況 | 備考 |
| --- | --- | --- | --- |
| [タスク9.1](../PLAN.md#plan-task-9-1) | [3. ゲームループ](#spec-game-loop), [4. シーン構成と遷移](#spec-scenes-ui), [6. セーブ/ロード最小仕様](#spec-save-load) | 対応済み | 勝敗条件、UI遷移、最小セーブ仕様を固定 |
| [タスク9.2](../PLAN.md#plan-task-9-2) | [3. ゲームループ](#spec-game-loop), [8. 受け入れ基準](#spec-acceptance) | 対応済み | 実装/検証で到達すべきプレイループ要件を定義 |
| [タスク9.3](../PLAN.md#plan-task-9-3) | [8. 受け入れ基準](#spec-acceptance) | 部分対応 | 配布手順は `docs/DISTRIBUTION_1OS.md` が正本 |

### 9.2 Phase 10 タスクとの対応

| PLANタスク | 仕様項目 | 対応状況 | 備考 |
| --- | --- | --- | --- |
| [タスク10.1](../PLAN.md#plan-task-10-1) | [7. 非対象](#spec-non-target) | 未対応 | アニメーション/タイルマップ/入力マッピングは本仕様の対象外 |
| [タスク10.2](../PLAN.md#plan-task-10-2) | [6. セーブ/ロード最小仕様](#spec-save-load) | 部分対応 | Saveの最小要件のみ対応。アセットパイプライン全体は別資料 |
| [タスク10.3](../PLAN.md#plan-task-10-3) | [8. 受け入れ基準](#spec-acceptance) | 未対応 | CI/性能/不具合テンプレートは `PLAN.md` と運用資料側で管理 |

## 10. 関連ドキュメント

- `docs/DEVELOPMENT_TRACK.md`
- `docs/GAME_DEVELOPMENT_TRACK.md`
- `docs/CORE_DEVELOPMENT_TRACK.md`
- `PERFORMANCE_TEST.md`
- `PLAN.md`（Phase 9/10 対応: `PLAN.md#plan-phase9-10-spec-map`）
- `docs/CODEX_MIGRATION_STATUS.md`
