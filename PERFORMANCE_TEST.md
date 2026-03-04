# MIYABI 性能テスト計画書 (Performance Test Plan)

## 1. 目的

本ドキュメントは、MIYABIエンジンのパフォーマンスを客観的に測定し、継続的に改善していくための計画を定義するものです。アーキテクチャの堅牢性に加え、ゲームエンジンとしての生命線である実行性能を確保し、あらゆる変更が性能に与える影響を追跡可能にすることを目的とします。

## 2. 基本方針

「性能測定に適した構造」から「性能を常に測定できる環境」へと進化させるため、以下の3ステップアプローチを採ります。

1.  **計測器 (Instrumentation) の実装:** エンジンの動作状況を可視化するための「速度計」や「診断機」を組み込みます。
2.  **ベンチマークシナリオ (Benchmark Scenarios) の作成:** 特定の機能に高い負荷をかけ、限界性能やボトルネックを特定するための標準的な「テストコース」を設けます。
3.  **ベースライン (Baseline) の確立:** 各シナリオにおける初期性能を記録し、将来の性能改善・悪化を判断するための「基準タイム」とします。

---

## 3. 実行計画 (Action Plan)

### ステップ1：計測器の実装 (Instrumentation)

エンジンに以下の計測機能を実装し、性能をリアルタイムで可視化します。この計測機能全体は、コンパイル時フラグによって有効/無効を切り替えられるように設計します。

-   [x] **タスク1.0: パフォーマンス計測用コンパイルフラグの導入**
    -   内容: パフォーマンス計測コードを有効化/無効化するための、コンパイル時フラグ（例: `MIYABI_PROFILE`）を導入します。CMakeでこのフラグをON/OFFできるように設定します。
    -   目的: デフォルト（OFF）の状態では、計測コードがバイナリに一切含まれなくなり、通常の開発やリリースビルドに全く影響を与えません。これにより、MIYABIのクリーンな設計思想を維持します。

-   [x] **タスク1.1: 基本的なフレームレート表示機能**
    -   内容: ウィンドウタイトル、または画面の隅に現在のFPS（Frames Per Second）とフレームタイム（1フレームの描画に要した時間、単位: ms）を常時表示する機能を実装します。これは最も基本的かつ重要な性能指標となります。
    -   実装箇所: `core` (C++) のメインループ内。

-   [x] **タスク1.2: 詳細プロファイリング機能の基盤**
    -   内容: コードの特定区間の実行時間を計測するための、シンプルなプロファイラを導入します。RAIIパターン（実行時間をコンストラクタで記録開始、デストラクタで結果を出力する手法）などが考えられます。
    -   実装箇所:
        -   C++側: `Renderer::draw()`, `PhysicsManager`関連処理など、主要な低レベル処理。
        -   Rust側: ECSの各`System`の実行時間、描画コマンドバッファの生成時間など、主要なロジック処理。
    -   目標: 「`physics_system`: 2.1ms, `render_command_generation`: 0.8ms, `renderer::draw`: 5.3ms」のような詳細な内訳を出力できるようにする。

### ステップ2：ベンチマークシナリオの作成

エンジンのサブシステムに意図的に高負荷をかける、複数のテストシーンを実装します。これらのシナリオは、実行時に簡単に切り替えられるようにします。

-   [x] **タスク2.1: 描画負荷テスト (Sprite Stress Test)**
    -   内容: 同一、または複数のテクスチャを持つスプライトを大量（例: 1,000, 10,000, 50,000個）に画面内に配置し、描画します。
    -   目的: エンジンの描画スループット、特にインスタンスレンダリングの効率と限界性能を測定します。

-   [x] **タスク2.2: 物理演算負荷テスト (Physics Stress Test)**
    -   内容: `Collider`コンポーネントを持つ大量のエンティティ（例: 500, 2,000個）を互いに衝突させ、物理演算ループを実行します。
    -   目的: Rust側で実装されている当たり判定システムの処理性能を測定します。

-   [x] **タスク2.3: UI/テキスト描画負荷テスト (UI Stress Test)**
    -   内容: 大量の文字列、または複雑なレイアウトのUI要素を画面上に描画します。
    -   目的: テキストレンダリング（グリフ生成、テクスチャアトラス管理、描画）のパイプライン全体の性能を測定します。

-   [x] **タスク2.4: シーン構築/破棄テスト (Scene Management Stress Test)**
    -   内容: 大量のエンティティを持つシーンの生成と破棄を短時間で繰り返します。
    -   目的: エンティティやコンポーネントの生成・削除に伴うメモリ確保・解放のオーバーヘッドを測定し、ECS実装の効率を評価します。
    -   実装: `logic/src/perf.rs` の `scene_construct_destruct` シナリオで計測します。

### ステップ3：ベースラインの確立

-   [x] **タスク3.1: ベースライン性能の記録**
    -   内容: ステップ1, 2が完了した後、各ベンチマークシナリオを実行し、その結果（平均FPS、フレームタイムの内訳など）をこのドキュメント、または別のファイルに記録します。
    -   目的: この記録が、今後の全ての性能評価の基準となります。機能追加やリファクタリングの前後でベンチマークを再実行し、性能が向上したか、あるいは意図せず悪化（リグレッション）していないかを確認します。
    -   記録先:
        -   ベースライン: `docs/perf/baseline_macos14.json`
        -   計測バイナリ: `logic/src/bin/perf_baseline.rs`
        -   回帰判定: `tools/check_perf_regression.py`
        -   CIアーティファクト: `build/perf/current_baseline.json`, `build/perf/regression_report.md`

### 3.2 性能計測ドキュメント配置ルール（Bench-80-28）

- `PERFORMANCE_TEST.md` に置く情報:
  - 計測の目的、判定基準、実行コマンド、運用手順のような「共通ルール」
  - 将来も参照される固定的な方針（閾値判定の考え方、復旧フロー）
- `docs/perf/` に置く情報:
  - 実測値やベースラインJSONなどの「成果物」
  - 実行環境ごとの差分が出る記録（OS別、マシン別、日付別の結果）
  - 個別Runの補足メモや比較レポート

判断目安:
- 複数Runで共通に使う説明なら `PERFORMANCE_TEST.md`
- 1回または特定環境に依存する結果なら `docs/perf/`

命名規則（最小例）:
- ベースライン: `baseline_<os><major>.json`（例: `baseline_macos14.json`）
- Run別記録: `report_<yyyyMMdd>_<env>.md`（例: `report_20260303_macos14.md`）

---

## 4. 運用手順（2026-02-23）

### 4.1 ローカルで計測

```bash
cargo run --release --manifest-path logic/Cargo.toml --bin perf_baseline -- \
  --output build/perf/current_baseline.json
python3 tools/check_perf_regression.py \
  --baseline docs/perf/baseline_macos14.json \
  --current build/perf/current_baseline.json \
  --output build/perf/regression_report.md
```

### 4.2 CIでの判定

- `.github/workflows/build.yml` で上記2コマンドを自動実行する。
- 判定ロジック: `current_avg_ms <= baseline_avg_ms * (1 + max_regression_pct / 100)`。
- `tools/check_perf_regression.py` の終了コード: `0` は全シナリオが閾値内、`1` は回帰またはシナリオ欠落あり。
- レポートは CI アーティファクト `perf-report-<run_id>` に保存する。

### 4.3 回帰検知時の復旧導線

- `tools/check_perf_regression.py` は FAIL 時に `Next Actions` を出力し、再計測コマンドと調査導線を提示する。
- `FAIL (missing scenario)` は入力不整合の可能性が高いため、baseline 更新ではなく計測入力の整合修正を優先する。
- 閾値超過 `FAIL` は原因調査を先行し、意図的変更であることを合意できる場合のみ baseline 更新を検討する。
- 根本原因の確認前に baseline を更新しない。

### 4.4 初期ベースライン値（macos-14）

| scenario | baseline_avg_ms | max_regression_pct |
| --- | ---: | ---: |
| `sprite_renderable_build` | 0.121 | 200 |
| `ui_text_command_build` | 0.069 | 200 |
| `scene_construct_destruct` | 1.364 | 200 |

### 4.5 ベースライン更新の事前条件（チェックリスト）

ベースライン更新は「性能の悪化を隠さない」ための例外運用とし、次の条件を全て満たした場合のみ実施する。

- [ ] `tools/check_perf_regression.py` の出力を確認し、`missing scenario` が 0 件である（入力不整合を解消済み）。
- [ ] 回帰または変動の原因を特定し、意図的変更である根拠（対応PR/Issue・設計変更）を説明できる。
- [ ] 同一コミットで再計測を最低2回実施し、更新候補値が許容範囲で再現する。
- [ ] 計測条件（OS/ビルド種別/負荷状態）が既存 baseline と同等であることを確認した。
- [ ] `build/perf/regression_report.md` の `Next Actions` に未対応項目がない。

更新時は、PR説明に「更新理由」「再計測結果」「対象シナリオ」を明記し、レビュー時に追跡できるようにする。

### 4.6 baseline 更新フロー（`docs/perf/baseline_macos14.json`）

更新対象は `docs/perf/baseline_macos14.json` のみとし、計測入力と判定結果を `build/perf/` に残してから差し替える。

1. 計測前提を固定する（`macos-14`、`--release`、バックグラウンド負荷が低い状態、`logic/src/bin/perf_baseline.rs` を使用）。
2. 次のコマンドで再計測し、`build/perf/current_baseline.json` を生成する。
   ```bash
   cargo run --release --manifest-path logic/Cargo.toml --bin perf_baseline -- \
     --output build/perf/current_baseline.json
   ```
3. 次のコマンドで既存 baseline と比較し、`build/perf/regression_report.md` を生成して差分理由を確認する。
   ```bash
   python3 tools/check_perf_regression.py \
     --baseline docs/perf/baseline_macos14.json \
     --current build/perf/current_baseline.json \
     --output build/perf/regression_report.md
   ```
4. 意図した変更のみであることを確認後、`build/perf/current_baseline.json` の内容を `docs/perf/baseline_macos14.json` に反映し、`generated_on` を更新する。
5. PR には `PERFORMANCE_TEST.md`、`docs/perf/baseline_macos14.json`、`build/perf/regression_report.md` の確認結果を記載する。

### 4.7 baseline 更新レビューのチェックリスト

- [ ] `PERFORMANCE_TEST.md` の手順どおりに `build/perf/current_baseline.json` と `build/perf/regression_report.md` を再生成している。
- [ ] `docs/perf/baseline_macos14.json` の `platform` が `macos-14`、`baseline_source` が `logic/src/bin/perf_baseline.rs` のままである。
- [ ] `scenarios[].name` の集合が変更されていない（欠落・追加がある場合は理由を PR に明記している）。
- [ ] `baseline_avg_ms` の変化に対する根拠（コード変更または計測条件の差分）を PR に記載している。
