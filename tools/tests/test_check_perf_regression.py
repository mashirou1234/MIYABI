import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "tools" / "check_perf_regression.py"
SPEC = importlib.util.spec_from_file_location("check_perf_regression", SCRIPT_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(MODULE)


class CheckPerfRegressionCliTest(unittest.TestCase):
    def run_cli(self, baseline: dict, current: dict) -> subprocess.CompletedProcess:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmpdir_path = Path(tmpdir)
            baseline_path = tmpdir_path / "baseline.json"
            current_path = tmpdir_path / "current.json"
            baseline_path.write_text(json.dumps(baseline), encoding="utf-8")
            current_path.write_text(json.dumps(current), encoding="utf-8")
            return subprocess.run(
                [
                    "python3",
                    str(SCRIPT_PATH),
                    "--baseline",
                    str(baseline_path),
                    "--current",
                    str(current_path),
                ],
                capture_output=True,
                text=True,
                check=False,
            )

    def test_fail_report_includes_recovery_guidance(self) -> None:
        baseline = {
            "scenarios": [
                {
                    "name": "sprite_renderable_build",
                    "baseline_avg_ms": 100,
                    "max_regression_pct": 10,
                }
            ]
        }
        current = {"scenarios": [{"name": "sprite_renderable_build", "avg_ms": 200}]}

        proc = self.run_cli(baseline, current)

        self.assertEqual(proc.returncode, 1)
        self.assertIn("## Summary", proc.stdout)
        self.assertIn("## Next Actions", proc.stdout)
        self.assertIn("python3 tools/check_perf_regression.py", proc.stdout)
        self.assertIn("baseline を更新しない", proc.stdout)
        self.assertIn("PERFORMANCE_TEST.md", proc.stdout)

    def test_pass_report_does_not_include_recovery_guidance(self) -> None:
        baseline = {
            "scenarios": [
                {
                    "name": "sprite_renderable_build",
                    "baseline_avg_ms": 100,
                    "max_regression_pct": 10,
                }
            ]
        }
        current = {"scenarios": [{"name": "sprite_renderable_build", "avg_ms": 105}]}

        proc = self.run_cli(baseline, current)

        self.assertEqual(proc.returncode, 0)
        self.assertIn("## Summary", proc.stdout)
        self.assertNotIn("## Next Actions", proc.stdout)


class CheckPerfRegressionWarnTest(unittest.TestCase):
    def test_warn_for_current_only_scenario(self) -> None:
        baseline = {
            "scenarios": [
                {
                    "name": "baseline_scenario",
                    "baseline_avg_ms": 10.0,
                    "max_regression_pct": 10,
                }
            ]
        }
        current = {
            "scenarios": [
                {"name": "baseline_scenario", "avg_ms": 10.5},
                {"name": "new_scenario", "avg_ms": 8.0},
            ]
        }

        rows, all_passed = MODULE.compare(baseline, current)

        self.assertTrue(all_passed)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[1]["name"], "new_scenario")
        self.assertEqual(rows[1]["status"], "WARN (not in baseline)")
        self.assertEqual(rows[1]["current_avg_ms"], 8.0)
        self.assertIsNone(rows[1]["baseline_avg_ms"])

    def test_only_additional_scenarios_keeps_exit_success(self) -> None:
        baseline = {"scenarios": []}
        current = {"scenarios": [{"name": "new_only", "avg_ms": 1.23}]}

        rows, all_passed = MODULE.compare(baseline, current)

        self.assertTrue(all_passed)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["status"], "WARN (not in baseline)")


class CheckPerfRegressionSummaryTest(unittest.TestCase):
    def test_summary_counts_match_mixed_rows(self) -> None:
        rows = [
            {
                "name": "ok",
                "baseline_avg_ms": 1.0,
                "current_avg_ms": 1.0,
                "threshold_ms": 1.2,
                "delta_pct": 0.0,
                "status": "PASS",
            },
            {
                "name": "ng",
                "baseline_avg_ms": 1.0,
                "current_avg_ms": 1.5,
                "threshold_ms": 1.2,
                "delta_pct": 50.0,
                "status": "FAIL",
            },
            {
                "name": "missing",
                "baseline_avg_ms": 1.0,
                "current_avg_ms": None,
                "threshold_ms": None,
                "delta_pct": None,
                "status": "FAIL (missing scenario)",
            },
        ]

        summary = MODULE.summarize_rows(rows)
        self.assertEqual(summary, {"total": 3, "pass": 1, "fail": 2})

        markdown = MODULE.render_markdown(rows, "baseline.json", "current.json")
        self.assertIn("- total: 3", markdown)
        self.assertIn("- PASS: 1", markdown)
        self.assertIn("- FAIL: 2", markdown)

    def test_report_generation_succeeds_with_existing_baseline_shape(self) -> None:
        baseline_path = REPO_ROOT / "docs" / "perf" / "baseline_macos14.json"
        baseline = MODULE.load_json(str(baseline_path))

        current = {
            "schema_version": 1,
            "platform": "macos-14",
            "scenarios": [
                {"name": item["name"], "avg_ms": item["baseline_avg_ms"]}
                for item in baseline["scenarios"]
            ],
        }

        with tempfile.TemporaryDirectory() as tmp_dir:
            current_path = Path(tmp_dir) / "current.json"
            current_path.write_text(json.dumps(current), encoding="utf-8")

            rows, all_passed = MODULE.compare(baseline, current)
            report = MODULE.render_markdown(rows, str(baseline_path), str(current_path))

        self.assertTrue(all_passed)
        self.assertIn("# Performance Regression Report", report)
        self.assertIn("## Summary", report)
        self.assertIn("| scenario | baseline_avg_ms | current_avg_ms |", report)


if __name__ == "__main__":
    unittest.main()
