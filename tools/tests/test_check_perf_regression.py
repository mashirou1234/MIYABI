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


if __name__ == "__main__":
    unittest.main()
