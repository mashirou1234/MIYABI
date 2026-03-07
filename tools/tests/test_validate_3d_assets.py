import importlib.util
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "tools" / "validate_3d_assets.py"
SPEC = importlib.util.spec_from_file_location("validate_3d_assets", SCRIPT_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(MODULE)


class Validate3dAssetsUnitTest(unittest.TestCase):
    def test_validate_default_project_assets_pass(self) -> None:
        summary = MODULE.validate_project(REPO_ROOT, [], [], [])

        self.assertTrue(summary.passed())
        self.assertGreater(len(summary.messages), 0)
        self.assertTrue(any(message.category == "mesh" for message in summary.messages))
        self.assertTrue(any(message.category == "contract" for message in summary.messages))

    def test_invalid_missing_faces_fixture_fails(self) -> None:
        fixture_path = (
            REPO_ROOT
            / "tools"
            / "tests"
            / "fixtures"
            / "invalid_missing_faces.obj.fixture"
        )
        messages = MODULE.validate_obj_file(fixture_path, REPO_ROOT)

        self.assertEqual(len(messages), 1)
        self.assertEqual(messages[0].level, "FAIL")
        self.assertIn("no drawable faces found", messages[0].detail)

    def test_contracts_match_current_repo(self) -> None:
        messages = MODULE.validate_contracts(REPO_ROOT)

        self.assertTrue(messages)
        self.assertTrue(all(message.level == "PASS" for message in messages))
        self.assertTrue(any("arena cube mesh id" == message.target for message in messages))
        self.assertTrue(any("lit textured 3D material id" == message.target for message in messages))


class Validate3dAssetsCliTest(unittest.TestCase):
    def test_cli_reports_invalid_fixture_failure(self) -> None:
        fixture_path = (
            REPO_ROOT
            / "tools"
            / "tests"
            / "fixtures"
            / "invalid_missing_faces.obj.fixture"
        )
        proc = subprocess.run(
            ["python3", str(SCRIPT_PATH), "--obj", str(fixture_path)],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(proc.returncode, 1)
        self.assertIn("# MIYABI 3D Asset Validation", proc.stdout)
        self.assertIn("RESULT: FAIL", proc.stdout)
        self.assertIn("invalid_missing_faces.obj.fixture", proc.stdout)
        self.assertIn("no drawable faces found", proc.stdout)


if __name__ == "__main__":
    unittest.main()
