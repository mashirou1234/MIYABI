import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "tools" / "check_perf_regression.py"


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
                {"name": "sprite_renderable_build", "baseline_avg_ms": 100, "max_regression_pct": 10}
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
                {"name": "sprite_renderable_build", "baseline_avg_ms": 100, "max_regression_pct": 10}
            ]
        }
        current = {"scenarios": [{"name": "sprite_renderable_build", "avg_ms": 105}]}

        proc = self.run_cli(baseline, current)

        self.assertEqual(proc.returncode, 0)
        self.assertNotIn("## Next Actions", proc.stdout)


if __name__ == "__main__":
    unittest.main()
