#!/usr/bin/env python3
import argparse
import json
import pathlib
import sys
from typing import Dict, List, Tuple


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="MIYABI performance baseline regression checker",
        epilog="Exit codes: 0=all scenarios within threshold, 1=regression or missing scenario detected.",
    )
    parser.add_argument("--baseline", required=True, help="baseline JSON path")
    parser.add_argument("--current", required=True, help="current report JSON path")
    parser.add_argument("--output", help="optional markdown output path")
    return parser.parse_args()


def load_json(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def map_scenarios(items: List[dict], field_name: str) -> Dict[str, dict]:
    mapped: Dict[str, dict] = {}
    for item in items:
        name = item.get(field_name)
        if not name:
            continue
        mapped[name] = item
    return mapped


def compare(baseline: dict, current: dict) -> Tuple[List[dict], bool]:
    baseline_map = map_scenarios(baseline.get("scenarios", []), "name")
    current_map = map_scenarios(current.get("scenarios", []), "name")
    rows: List[dict] = []
    all_passed = True

    for name, baseline_item in baseline_map.items():
        current_item = current_map.get(name)
        if current_item is None:
            all_passed = False
            rows.append(
                {
                    "name": name,
                    "baseline_avg_ms": baseline_item.get("baseline_avg_ms"),
                    "current_avg_ms": None,
                    "threshold_ms": None,
                    "delta_pct": None,
                    "status": "FAIL (missing scenario)",
                }
            )
            continue

        baseline_avg = float(baseline_item.get("baseline_avg_ms", 0.0))
        max_regression_pct = float(baseline_item.get("max_regression_pct", 0.0))
        threshold_ms = baseline_avg * (1.0 + max_regression_pct / 100.0)
        current_avg = float(current_item.get("avg_ms", 0.0))
        delta_pct = (
            ((current_avg - baseline_avg) / baseline_avg) * 100.0
            if baseline_avg > 0.0
            else 0.0
        )

        passed = current_avg <= threshold_ms
        if not passed:
            all_passed = False

        rows.append(
            {
                "name": name,
                "baseline_avg_ms": baseline_avg,
                "current_avg_ms": current_avg,
                "threshold_ms": threshold_ms,
                "delta_pct": delta_pct,
                "status": "PASS" if passed else "FAIL",
            }
        )

    return rows, all_passed


def render_markdown(rows: List[dict], baseline_path: str, current_path: str) -> str:
    lines: List[str] = []
    lines.append("# Performance Regression Report")
    lines.append("")
    lines.append(f"- baseline: `{baseline_path}`")
    lines.append(f"- current: `{current_path}`")
    lines.append("")
    lines.append(
        "| scenario | baseline_avg_ms | current_avg_ms | threshold_ms | delta_pct | status |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | --- |")
    for row in rows:
        baseline_avg = (
            f"{row['baseline_avg_ms']:.3f}"
            if row["baseline_avg_ms"] is not None
            else "-"
        )
        current_avg = (
            f"{row['current_avg_ms']:.3f}" if row["current_avg_ms"] is not None else "-"
        )
        threshold_ms = (
            f"{row['threshold_ms']:.3f}" if row["threshold_ms"] is not None else "-"
        )
        delta_pct = f"{row['delta_pct']:+.1f}%" if row["delta_pct"] is not None else "-"
        lines.append(
            f"| {row['name']} | {baseline_avg} | {current_avg} | {threshold_ms} | {delta_pct} | {row['status']} |"
        )
    lines.append("")

    failing_rows = [row for row in rows if row["status"].startswith("FAIL")]
    if failing_rows:
        has_missing = any("missing scenario" in row["status"] for row in failing_rows)
        has_threshold_regression = any(
            row["status"] == "FAIL" for row in failing_rows
        )
        lines.append("## Next Actions")
        lines.append("")
        lines.append("1. まず再計測で再現確認")
        lines.append("```bash")
        lines.append(
            "cargo run --release --manifest-path logic/Cargo.toml --bin perf_baseline -- \\"
        )
        lines.append(f"  --output {current_path}")
        lines.append("python3 tools/check_perf_regression.py \\")
        lines.append(f"  --baseline {baseline_path} \\")
        lines.append(f"  --current {current_path} \\")
        lines.append("  --output build/perf/regression_report.md")
        lines.append("```")
        lines.append("")
        lines.append("2. 原因に応じて分岐")
        if has_missing:
            lines.append(
                "- `FAIL (missing scenario)` がある場合: baseline更新前に計測入力/シナリオ名の不整合を修正する。"
            )
        if has_threshold_regression:
            lines.append(
                "- 閾値超過 (`FAIL`) の場合: 先に回帰原因を調査し、意図的な変更と合意できる場合のみ baseline 更新を検討する。"
            )
        lines.append("")
        lines.append(
            "> 注意: 根本原因が未確認の段階で baseline を更新しないでください。"
        )
        lines.append("")
        lines.append(
            "詳細手順: `PERFORMANCE_TEST.md` の「4. 運用手順（2026-02-23）」を参照。"
        )
        lines.append("")

    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    baseline = load_json(args.baseline)
    current = load_json(args.current)

    rows, all_passed = compare(baseline, current)
    report_md = render_markdown(rows, args.baseline, args.current)

    print(report_md)
    if args.output:
        output_path = pathlib.Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(report_md, encoding="utf-8")

    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
