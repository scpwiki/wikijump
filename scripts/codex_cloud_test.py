import os
import pathlib
import re
import tomllib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SETUP = ROOT / "scripts" / "codex-cloud-setup.sh"
MAINTENANCE = ROOT / "scripts" / "codex-cloud-maintenance.sh"
DOCUMENTATION = ROOT / "docs" / "CodexCloudEnvironment.md"


def read(path):
    return path.read_text(encoding="utf-8")


def assignment(script, name):
    match = re.search(rf"^{re.escape(name)}=([^\n]+)$", read(script), re.MULTILINE)
    if match is None:
        raise AssertionError(f"{name} is missing from {script}")
    return match.group(1)


class CodexCloudScriptTests(unittest.TestCase):
    def test_scripts_are_executable_and_share_revision(self):
        self.assertTrue(os.access(SETUP, os.X_OK))
        self.assertTrue(os.access(MAINTENANCE, os.X_OK))
        self.assertEqual(assignment(SETUP, "script_revision"), assignment(MAINTENANCE, "script_revision"))
        self.assertIn("Every script run begins with a revision banner", read(DOCUMENTATION))

    def test_runtime_and_tool_pins_match_repository_ci(self):
        with (ROOT / "rust-toolchain.toml").open("rb") as file:
            rust_version = tomllib.load(file)["toolchain"]["channel"]
        for script in (SETUP, MAINTENANCE):
            self.assertEqual(assignment(script, "required_node_major"), "24")
            self.assertEqual(assignment(script, "required_rust_version"), rust_version)
            self.assertEqual(assignment(script, "pnpm_version"), "11.12.0")
        setup = read(SETUP)
        self.assertIn("cargo-machete --version 0.9.1 --locked", setup)
        self.assertIn("sqlx-cli --version 0.8.6 --locked", setup)
        self.assertIn("actionlint/cmd/actionlint@v1.7.12", setup)
        for manifest in ("deepwell/Cargo.toml", "wws/Cargo.toml", "locales/validator/Cargo.toml"):
            with (ROOT / manifest).open("rb") as file:
                self.assertEqual(tomllib.load(file)["package"]["rust-version"], rust_version)

    def test_maintenance_does_not_execute_task_branch_programs(self):
        maintenance = read(MAINTENANCE)
        self.assertIn("cd /", maintenance)
        self.assertEqual(maintenance.count("--ignore-pnpmfile --ignore-scripts --frozen-lockfile"), 3)
        self.assertEqual(maintenance.count("CARGO_NET_GIT_FETCH_WITH_CLI=false"), 3)
        self.assertEqual(maintenance.count('GIT_CONFIG_GLOBAL=/dev/null'), 3)
        for prohibited in ("pnpm install", "cargo build", "cargo test", "sqlx migrate", "python3 -m", "seed"):
            self.assertNotIn(prohibited, maintenance)

    def test_node_activation_removes_the_legacy_symlink(self):
        for script in (SETUP, MAINTENANCE):
            content = read(script)
            self.assertIn('sudo rm -f -- "$legacy_node24_link"', content)
            self.assertNotRegex(content, r"ln\s+-[^\n]*\$legacy_node24_link")
            self.assertIn("node24.sh", content)

    def test_documentation_points_to_canonical_scripts(self):
        documentation = read(DOCUMENTATION)
        self.assertIn("scripts/codex-cloud-setup.sh", documentation)
        self.assertIn("scripts/codex-cloud-maintenance.sh", documentation)
        self.assertIn("Agent internet access | Off", documentation)
        self.assertIn("DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/wikijump_codex", documentation)


if __name__ == "__main__":
    unittest.main()
