from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "refresh.py"
SPEC = importlib.util.spec_from_file_location("standing_refresh", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
REFRESH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REFRESH)


class RefreshStandingTest(unittest.TestCase):
    def test_compose_restart_is_fixed_to_app_services_without_volume_flags(
        self,
    ) -> None:
        runtime_home = Path("/srv/wikijump-standing")
        command = REFRESH.compose_command(
            runtime_home,
            "up",
            "--detach",
            "--no-deps",
            *REFRESH.SERVICES,
            override_file=Path("/src/refresh.compose.yaml"),
        )
        self.assertEqual(
            command[-6:],
            ["up", "--detach", "--no-deps", "deepwell", "framerail", "wws"],
        )
        self.assertNotIn("down", command)
        self.assertNotIn("-v", command)
        self.assertNotIn("--volumes", command)
        self.assertNotIn("--remove-volumes", command)

    def test_cli_rejects_every_volume_removal_spelling(self) -> None:
        for forbidden in ("-v", "--volumes", "--remove-volumes"):
            with self.subTest(forbidden=forbidden):
                result = subprocess.run(
                    (sys.executable, str(SCRIPT), forbidden),
                    text=True,
                    capture_output=True,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("unrecognized arguments", result.stderr)

    def test_builds_only_reviewed_local_dockerfiles(self) -> None:
        source_root = Path("/src/wikijump")
        identity = {"wikijump_sha": "a" * 40, "ftml_sha": "b" * 40}
        for service in REFRESH.SERVICES:
            with self.subTest(service=service):
                command = REFRESH.build_command(
                    source_root,
                    service,
                    f"local/{service}:latest",
                    identity,
                    "2026-08-22T00:00:00+00:00",
                )
                self.assertIn(
                    str(source_root / "install" / "local" / service / "Dockerfile"),
                    command,
                )
                self.assertEqual(command[-1], str(source_root))
                self.assertEqual(
                    "FRAMERAIL_ENV=local" in command, service == "framerail"
                )

    def test_environment_rewrite_is_atomic_and_preserves_unrelated_values(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            path = Path(temporary_dir) / ".env"
            path.write_text("KEEP=value\nSTANDING_WIKIJUMP_SHA=old\n", encoding="utf-8")
            values = REFRESH.read_environment(path)
            values["STANDING_WIKIJUMP_SHA"] = "new"
            REFRESH.write_environment(path, values)
            self.assertEqual(
                path.read_text(encoding="utf-8"),
                "KEEP=value\nSTANDING_WIKIJUMP_SHA=new\n",
            )
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)


if __name__ == "__main__":
    unittest.main()
