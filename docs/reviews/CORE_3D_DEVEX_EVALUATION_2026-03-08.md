# C4 3D 開発体験評価 2026-03-08

## 1. メタデータ

- 評価日: 2026-03-08
- 評価対象: `#367` 完了直後の 3D 基盤 / 制作導線
- 対象 commit / PR: `6c64f5af0cadb474c39b50d7cb62d2910427df8e` / PR `#378`
- 記入者: Codex
- 比較対象（任意）: なし（まず MIYABI 単体の現在地評価を固定）
- 対象ドキュメント:
  - `README.md`
  - `docs/CORE_3D_PRODUCTION_BASELINE.md`
  - `docs/CORE_DEVELOPMENT_TRACK.md`
  - `docs/SDK_DEFINITION.md`

## 2. 評価表

| 評価軸 | score (0-3) | 現状の評価 | 証跡リンク | 主な不足 / リスク | 次の Issue 候補 |
| --- | ---: | --- | --- | --- | --- |
| 初期導入速度 | 2 | README と SDK 外部サンプル再利用スモークで最短導線は辿れる。3D 向けの「最初の 1 本」を作る入口も見つけやすい。 | `README.md`, `scripts/test_sdk_external_sample_reuse.sh`, `docs/SDK_DEFINITION.md` | 3D 専用の onboarding 例が 1 本しかなく、最初に何を改変すべきかはまだ暗黙知が残る。 | 3D tutorial / sample 群の体系化 |
| 反復速度 | 1 | 3D perf baseline / asset validation / G4 smoke の 1 コマンド再実行はできる。 | `scripts/test_core_c3_3d_perf_baseline.sh`, `scripts/test_core_c3_3d_asset_validation.sh`, `scripts/test_game_track_g4.sh` | scene reload や asset hot-reload はなく、変更 1 回ごとの build / run コストはまだ高い。 | シーンロード / アンロード、再読込導線 |
| デバッグ容易性 | 2 | perf baseline、asset validator、2D/3D coexistence harness で切り分け起点は揃ってきた。 | `tools/validate_3d_assets.py`, `scripts/test_core_2d_3d_coexistence.sh`, `docs/perf/PERF_BASELINE.md` | 画面内デバッグ可視化や physics/render overlay がなく、目視デバッグは弱い。 | デバッグ可視化オーバーレイ |
| 拡張性 | 2 | `core -> logic -> sample_game` 境界と runtime contract は維持できている。 | `docs/SAMPLE_GAME_CORE_BOUNDARY.md`, `sample_game_runtime/src/lib.rs`, `docs/CORE_DEVELOPMENT_TRACK.md` | plugin / module 方針がなく、拡張ポイントの整理はまだ文書化不足。 | プラグイン / モジュール方針 |
| 運用性 | 2 | CI、auto-merge、配布スモーク、perf baseline、asset validation が揃い、3D でも継続運用の骨格はある。 | `docs/CI_AUTOMERGE.md`, `scripts/test_distribution_smoke.sh`, `scripts/test_core_c3_3d_perf_baseline.sh`, `scripts/test_core_c3_3d_asset_validation.sh` | macOS 1 系統中心で、multi-platform の検証導線はまだない。 | マルチプラットフォーム検証導線 |
| ドキュメント成熟度 | 2 | C2/C3/C4 正本、track、roadmap、README の導線は揃っている。 | `docs/CORE_3D_FOUNDATION_CONTRACT.md`, `docs/CORE_3D_PRODUCTION_BASELINE.md`, `docs/COMPLETION_ROADMAP.md`, `README.md` | tutorial / API / 運用の横断導線は今後さらに整理が必要。 | チュートリアル / サンプル群の体系化 |

## 3. サマリー

- 強い軸:
  - デバッグ容易性: headless harness と validator により、壊れ方の分類はしやすくなった。
  - 運用性: CI / auto-merge / perf / asset validation が一通り接続されている。
- 弱い軸:
  - 反復速度: asset 差し替えや scene 更新の開発サイクルはまだ重い。
- 直近 1-3 件の推奨 follow-up:
  1. デバッグ可視化オーバーレイ
  2. マルチプラットフォーム検証導線
  3. 3D tutorial / sample 群の体系化

## 4. 判定メモ

- この記録だけで next issue を切れるか:
  - はい。`反復速度` / `デバッグ容易性` / `運用性` の不足はそれぞれ独立 issue に分解できる。
- 追加で必要な証跡:
  - 実計測付きの build / reload 所要時間、multi-platform smoke の結果、tutorial completion 所要時間。
- 今回あえて評価対象外にしたもの:
  - 外部商用エンジンとの定量比較、DCC ツール連携、事業判断。
