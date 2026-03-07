# PERF_BASELINE 運用ノート

本書は `docs/perf/baseline_macos14.json` を更新する前に、測定環境を固定できているかを確認するための補助チェックリストです。  
共通ルールと判定手順の正本は `PERFORMANCE_TEST.md` を参照してください。

## 測定環境固定チェックリスト（Bench-80 #19）

以下をすべて満たす場合のみ、baseline 更新の再計測に進む。

- [ ] OS が `macos-14` である（`sw_vers` で確認）。
- [ ] 同一マシンで計測している（CPU/GPU/メモリ構成を変更していない）。
- [ ] 電源状態を固定している（AC 給電、低電力モード無効）。
- [ ] 計測前に不要アプリ/重負荷プロセスを停止し、バックグラウンド負荷を最小化した。
- [ ] 同一コミット・同一 `--release` ビルドで計測している。
- [ ] 計測コマンドは `logic/src/bin/perf_baseline.rs` を使い、出力先を `build/perf/current_baseline.json` に統一した。
- [ ] `python3 tools/check_perf_regression.py` を実行し、`build/perf/regression_report.md` を生成した。
- [ ] `FAIL (missing scenario)` が 0 件である。
- [ ] 同一コミットで最低2回再計測し、更新候補値の再現性を確認した。

最小再現コマンド:

```bash
./scripts/test_core_c3_3d_perf_baseline.sh
```

## 参照

- baseline: `docs/perf/baseline_macos14.json`
- 判定/更新手順: `PERFORMANCE_TEST.md` の 4.5 〜 4.8
