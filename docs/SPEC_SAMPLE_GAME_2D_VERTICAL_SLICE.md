# MIYABI 2D縦切り仕様書 (Phase 9.1)

最終更新: 2026-02-23
対象: Stage 1 (2D Vertical Slice)

## 1. 目的

本仕様は、Phase 9.1 の「縦切り仕様の固定」を満たすための正本とする。  
本仕様に基づき、Phase 9.2 以降の実装・検証を行う。

## 2. 縦切りタイトル定義

- タイトル（仮）: `MIYABI Box Survival`
- ジャンル: 2D サバイバルアクション
- プレイ単位: 1ラン（最大 30 分）
- 目標: 30分間生存してクリアする

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

## 7. 非対象（Phase 9.1時点）

- ネットワーク要素
- 複数難易度モード
- リプレイ保存
- マップエディタ
- マルチプラットフォーム配布

## 8. 受け入れ基準（Phase 9.2実装完了時に検証）

1. タイトルからゲーム開始し、勝敗いずれかで必ずリザルトへ遷移する
2. リザルトからリトライ/タイトル戻りが機能する
3. 30分連続プレイで進行不能が発生しない
4. セーブデータが再起動後に反映される
5. セーブ破損時にクラッシュせずデフォルト復旧する

## 9. 関連ドキュメント

- `docs/DEVELOPMENT_TRACK.md`
- `docs/GAME_DEVELOPMENT_TRACK.md`
- `docs/CORE_DEVELOPMENT_TRACK.md`
- `PLAN.md`
- `docs/CODEX_MIGRATION_STATUS.md`
