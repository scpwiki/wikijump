from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "render.py"
FTML_SHA = "f" * 40


class RenderStandingConfigTest(unittest.TestCase):
    def make_source(self, root: Path) -> tuple[Path, str]:
        source = root / "source"
        (source / "install/prod/deepwell").mkdir(parents=True)
        (source / "deepwell").mkdir(parents=True)
        (source / "locales").mkdir(parents=True)
        (source / "install/prod/deepwell/config.toml").write_text('[database]\nrun-seeder = false\n\n[domain]\nmain = "wikijump.com"\nfiles = "wjfiles.com"\n', encoding="utf-8")
        (source / "deepwell/Cargo.lock").write_text(f'source = "git+https://github.com/Rokurolize/ftml#{FTML_SHA}"\n', encoding="utf-8")
        (source / "locales/en.ftl").write_text("fixture = Fixture\n", encoding="utf-8")
        for command in (("git", "init"), ("git", "config", "user.email", "test@example.invalid"), ("git", "config", "user.name", "Standing test"), ("git", "add", "."), ("git", "commit", "-m", "fixture")):
            subprocess.run(command, cwd=source, check=True, stdout=subprocess.DEVNULL)
        sha = subprocess.check_output(("git", "rev-parse", "HEAD"), cwd=source, text=True).strip()
        return source, sha

    def command(self, source: Path, output: Path, sha: str) -> list[str]:
        return [
            sys.executable,
            str(SCRIPT),
            "--source-root", str(source),
            "--output-dir", str(output),
            "--wikijump-sha", sha,
            "--ftml-sha", FTML_SHA,
            "--database-image", "example/database@sha256:" + "a" * 64,
            "--files-image", "example/files@sha256:" + "b" * 64,
            "--cache-image", "example/cache@sha256:" + "c" * 64,
            "--deepwell-image", "example/deepwell@sha256:" + "d" * 64,
            "--framerail-image", "example/framerail@sha256:" + "e" * 64,
            "--wws-image", "example/wws@sha256:" + "1" * 64,
            "--caddy-image", "example/caddy@sha256:" + "2" * 64,
        ]

    def test_materializes_identity_bound_compose_home(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            source, sha = self.make_source(Path(temporary_dir))
            output = Path(temporary_dir) / "host/standing"
            result = subprocess.run(self.command(source, output, sha), check=True, text=True, capture_output=True)
            rendered = json.loads(result.stdout)
            identity = json.loads((output / "identity.json").read_text(encoding="utf-8"))
            self.assertEqual(rendered["output_dir"], str(output))
            self.assertEqual(identity["wikijump_sha"], sha)
            self.assertEqual(identity["ftml_sha"], FTML_SHA)
            self.assertEqual(identity["project_name"], "wikijump-standing")
            self.assertEqual((output / "deepwell/config.toml").read_text(encoding="utf-8"), '[database]\nrun-seeder = false\n\n[domain]\nmain = "wikijump.localhost"\nfiles = "wjfiles.localhost"\n')
            self.assertEqual(identity["deepwell_domain_override"], {"main": "wikijump.localhost", "files": "wjfiles.localhost"})
            self.assertFalse((output / "deepwell/seeder").exists())
            request = json.loads((output / "caddy/request.json").read_text(encoding="utf-8"))
            self.assertTrue(request["params"]["local"])
            self.assertNotIn("wildcard_cert", request["params"])
            compose = (output / "compose.yaml").read_text(encoding="utf-8")
            self.assertIn("runtime50x-postgres-data", compose)
            self.assertNotIn("./deepwell/seeder", compose)
            self.assertIn("curl --insecure", compose)
            self.assertIn("STANDING_CADDY_IMAGE=example/caddy", (output / ".env").read_text(encoding="utf-8"))

    def test_rendered_deepwell_has_required_runtime_mounts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            source, sha = self.make_source(Path(temporary_dir))
            output = Path(temporary_dir) / "host/standing"
            subprocess.run(self.command(source, output, sha), check=True, text=True, capture_output=True)
            compose = (output / "compose.yaml").read_text(encoding="utf-8")
            environment = (output / ".env").read_text(encoding="utf-8")
            self.assertIn("target: /etc/deepwell.toml", compose)
            self.assertIn("target: /opt/locales", compose)
            self.assertIn(f"STANDING_LOCALES_SOURCE={source / 'locales'}", environment)

    def test_rejects_dirty_source_before_writing_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            source, sha = self.make_source(Path(temporary_dir))
            (source / "dirty.txt").write_text("not committed\n", encoding="utf-8")
            output = Path(temporary_dir) / "host/standing"
            result = subprocess.run(self.command(source, output, sha), text=True, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("source checkout must be clean", result.stderr)
            self.assertFalse(output.exists())

    def test_rejects_unrecognized_production_domain_block(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            source, sha = self.make_source(Path(temporary_dir))
            config = source / "install/prod/deepwell/config.toml"
            config.write_text(config.read_text(encoding="utf-8").replace("wikijump.com", "example.com"), encoding="utf-8")
            subprocess.run(("git", "add", "."), cwd=source, check=True)
            subprocess.run(("git", "commit", "-m", "change domain"), cwd=source, check=True, stdout=subprocess.DEVNULL)
            sha = subprocess.check_output(("git", "rev-parse", "HEAD"), cwd=source, text=True).strip()
            output = Path(temporary_dir) / "host/standing"
            result = subprocess.run(self.command(source, output, sha), text=True, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("must contain exactly one expected domain block", result.stderr)
            self.assertFalse(output.exists())
