# サンプルゲーム 3D Vertical Slice 最小仕様

最終更新: 2026-03-08

## 1. 目的

G4 では、2D の `Box Survival` を 3D 空間へ拡張した最小デモを 1 本成立させる。
「3D を表示できる」ではなく、「開始から終了まで遊べる 3D 体験が 1 本ある」ことを基準にする。

## 2. コア体験

- ジャンル: 3D arena survival
- プレイヤー目標: 3D アリーナ内で 180 秒生存する
- 敗北条件: HP が 0 になる
- 勝利条件: 180 秒経過時点で HP が 1 以上残っている
- プレイ時間目安: 1 run 3 分前後

## 3. 必須シーン

### 3.1 Title

- `Start Game`
- `Start 3D Arena`
- `Exit`
- 音量 / fullscreen / 移動プリセット設定

### 3.2 InGame (3D Arena)

- プレイヤー 1 体
- 障害物または敵 1 系統以上
- 床 / 壁など、3D 空間が認識できる最小地形
- 画面上の HUD (`HP`, `Time`, `Score`)

### 3.3 Pause

- `Resume`
- `Back To Title`
- 設定 UI

### 3.4 Result

- `CLEAR` または `GAME OVER`
- `Retry`
- `Back To Title`
- 最低限のスコア表示

## 4. 最小操作系

- 移動: `Arrow Keys` または `WASD`
- Pause: `ESC`
- クリック UI: マウス左クリック
- カメラ: 初手は自動追従でよい。自由視点やマウス look は G4 完了条件に含めない。

## 5. 2D から流用する要素 / 新規要素

### 5.1 2D から流用する要素

- Title / Pause / Result の画面遷移
- 設定保存 (`master_volume`, `bgm_volume`, `se_volume`, `fullscreen`, `movement_preset`)
- BGM / SE の接続
- `Start/Resume/Retry/BackToTitle/Exit` の UI アクション契約

### 5.2 G4 で新規に必要な要素

- 透視投影カメラ
- 3D プレイヤー表現と障害物表現
- 3D アリーナ（床 / 壁 / 奥行き）
- 深度付き描画
- 3D 空間でも読める HUD の重ね描き

## 6. 受け入れ基準

- [ ] Title -> InGame -> Pause -> Result -> Title の導線が 3D サンプルでも成立する
- [ ] 3D アリーナ内で「奥行き」と「前後関係」が認識できる
- [ ] 180 秒 survive で `CLEAR`、HP 0 で `GAME OVER` になる
- [ ] 設定変更が 3D run の前後で保持される
- [ ] G4 の最初の実装 Task をそのまま切り出せる

## 7. 最初に切る実装 Task

1. `G4-01`: 3D arena の最小起動
   - Title から 3D シーンへ遷移し、床 / 壁 / プレイヤーが表示される
   - 実装済み（2026-03-08）
2. `G4-02`: 3D run の勝敗導線
   - HP / Time / Score と Pause / Result 遷移を接続する
3. `G4-03`: 3D 障害物 1 系統
   - 落下物または巡回敵 1 種で clear / fail 条件を成立させる

補足:
- 2026-03-08 時点の最小 3D 起動は `Title -> Start 3D Arena -> InGame` まで実装済み。
- `Pause -> Result` を含む勝敗導線と障害物導入は `G4-02` / `G4-03` で継続する。

## 8. 明確に対象外とするもの

- 複雑なカットシーン
- 複数ステージ
- マウスによる高自由度カメラ操作
- 本格的な 3D アニメーション
- マルチプレイヤー

G4 の評価は「3D でも遊びの骨格が成立したか」であり、3D 作品としての完成度は C3/C4 の基盤整備後に引き上げる。
