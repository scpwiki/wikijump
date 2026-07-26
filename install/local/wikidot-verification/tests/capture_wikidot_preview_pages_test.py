import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "scripts" / "capture_wikidot_preview_pages.py"
SPEC = importlib.util.spec_from_file_location("capture_wikidot_preview_pages", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class CaptureWikidotPreviewPagesTest(unittest.TestCase):
    def test_plan_loader_splits_only_on_lf_and_checks_source_identity(self):
        with tempfile.TemporaryDirectory() as root:
            plans_path = Path(root) / "pages.jsonl"
            source = "alpha\u2028beta"
            value = {
                "schema": MODULE.PAGE_PLAN_SCHEMA,
                "page_index": 1,
                "title": "one",
                "source": source,
                "source_sha256": MODULE.sha256(source),
                "cases": [{"case_id": "one"}],
            }
            plans_path.write_text(f"{json.dumps(value, ensure_ascii=False)}\n")
            plans = MODULE.load_plans(plans_path)
            self.assertEqual(plans[0]["source"], source)
            value["source_sha256"] = "0" * 64
            with self.assertRaisesRegex(ValueError, "source identity"):
                MODULE.validate_plan(value)

    def test_preview_body_uses_page_preview_module(self):
        body = MODULE.preview_body({"source": "**x**", "title": "one"})
        self.assertEqual(body["moduleName"], "edit/PagePreviewModule")
        self.assertEqual(body["source"], "**x**")


if __name__ == "__main__":
    unittest.main()
