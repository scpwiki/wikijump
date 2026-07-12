import pathlib
import re
import tomllib
import unittest


KOMODO_ROOT = pathlib.Path(__file__).resolve().parent.parent
REPOSITORY_ROOT = KOMODO_ROOT.parents[2]
SENSITIVE_VARIABLES = {
    "POSTGRES_PASSWORD",
    "S3_ACCESS_KEY_ID",
    "S3_SECRET_ACCESS_KEY",
    "MAILJET_API_KEY",
    "MAILJET_SECRET_KEY",
    "DIGITALOCEAN_API_TOKEN",
    "DISCORD_WEBHOOK",
}


class KomodoVariablePolicyTests(unittest.TestCase):
    def setUp(self):
        with (KOMODO_ROOT / "variables.toml").open("rb") as handle:
            self.variables = tomllib.load(handle)["variable"]

    def test_variable_toml_uses_the_complete_schema(self):
        expected_keys = {"name", "description", "value", "is_secret"}
        names = []

        for variable in self.variables:
            self.assertEqual(set(variable), expected_keys)
            self.assertIsInstance(variable["name"], str)
            self.assertIsInstance(variable["description"], str)
            self.assertIsInstance(variable["value"], str)
            self.assertIsInstance(variable["is_secret"], bool)
            self.assertRegex(variable["name"], r"^[A-Z][A-Z0-9_]*$")
            self.assertNotEqual(variable["description"].strip(), "")
            names.append(variable["name"])

        self.assertEqual(len(names), len(set(names)))
        self.assertIn("DIGITALOCEAN_API_TOKEN", names)

    def test_tracked_secret_values_are_empty(self):
        variables_by_name = {variable["name"]: variable for variable in self.variables}
        self.assertTrue(SENSITIVE_VARIABLES.issubset(variables_by_name))
        for name in SENSITIVE_VARIABLES:
            self.assertTrue(variables_by_name[name]["is_secret"], name)

        for variable in self.variables:
            if variable["is_secret"]:
                self.assertEqual(
                    variable["value"],
                    "",
                    f"tracked secret {variable['name']} must not contain a token",
                )

    def test_stack_placeholders_reference_declared_variables(self):
        with (KOMODO_ROOT / "stacks.toml").open("rb") as handle:
            environment = tomllib.load(handle)["stack"][0]["config"]["environment"]

        placeholders = set(re.findall(r"\[\[([A-Z][A-Z0-9_]*)\]\]", environment))
        declared = {variable["name"] for variable in self.variables}
        self.assertNotIn("[[", re.sub(r"\[\[[A-Z][A-Z0-9_]*\]\]", "", environment))
        self.assertTrue(placeholders.issubset(declared))
        self.assertIn("DIGITALOCEAN_API_TOKEN", placeholders)

    def test_bootstrap_import_policy_is_one_time_and_scoped(self):
        with (KOMODO_ROOT / "resource-sync.toml").open("rb") as handle:
            resource_sync = tomllib.load(handle)["resource_sync"][0]["config"]
        documentation = (REPOSITORY_ROOT / "docs/deployment/dev.md").read_text()

        self.assertEqual(resource_sync["resource_path"], ["install/dev/komodo/"])
        enable = documentation.index('set "Sync Variables" to true')
        disable = documentation.index('Set "Sync Variables" back to false')
        self.assertLess(enable, disable)


if __name__ == "__main__":
    unittest.main()
