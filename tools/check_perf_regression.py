#!/usr/bin/env python3
import argparse
import json
import pathlib
import sys
from typing import Dict, List, Tuple


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="MIYABI performance baseline regression checker"
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
