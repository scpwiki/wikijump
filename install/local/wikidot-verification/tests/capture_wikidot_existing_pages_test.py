import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "scripts" / "capture_wikidot_existing_pages.py"
SPEC = importlib.util.spec_from_file_location("capture_wikidot_existing_pages", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class CaptureWikidotExistingPagesTest(unittest.TestCase):
    def test_load_plans_accepts_read_only_scp_page(self):
        plan = {
            "schema": MODULE.PLAN_SCHEMA,
            "case_id": "scp-9507",
            "site": "scp-wiki",
            "slug": "scp-9507",
            "selector": ".anom-bar-container",
            "expected": {
                "required_class_tokens": ["anom-bar-container"],
                "forbidden_literals": ["[[include"],
            },
        }
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "plans.jsonl"
            path.write_text(f"{json.dumps(plan)}\n")
            self.assertEqual(MODULE.load_plans(path), [plan])

    def test_rejects_unapproved_sites_and_complex_selectors(self):
        plan = {
            "schema": MODULE.PLAN_SCHEMA,
            "case_id": "one",
            "site": "example",
            "slug": "one",
            "selector": ".one",
            "expected": {"required_class_tokens": ["one"], "forbidden_literals": []},
        }
        with self.assertRaisesRegex(ValueError, "allowlist"):
            MODULE.validate_plan(plan)
        plan["site"] = "scp-wiki"
        plan["selector"] = "#page-content .one"
        with self.assertRaisesRegex(ValueError, "class selector"):
            MODULE.validate_plan(plan)


if __name__ == "__main__":
    unittest.main()
