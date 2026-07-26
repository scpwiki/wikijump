import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "scripts" / "capture_wikidot_saved_pages.py"
SPEC = importlib.util.spec_from_file_location("capture_wikidot_saved_pages", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


def page_plan(source: str = "alpha") -> dict:
    source_hash = MODULE.sha256(source)
    return {
        "schema": MODULE.PAGE_PLAN_SCHEMA,
        "page_index": 1,
        "slug": "run-owned:ftml-diff-20260726-001",
        "title": "FTML differential 001",
        "source": source,
        "source_sha256": source_hash,
        "cases": [{
            "case_id": "alpha",
            "source_sha256": source_hash,
            "page_scope": "batch-safe",
            "marker_begin": "WJDIFF_BEGIN_ALPHA",
            "marker_end": "WJDIFF_END_ALPHA",
        }],
    }


class CaptureWikidotSavedPagesTest(unittest.TestCase):
    def test_plan_validation_requires_exact_run_owned_slug_and_hash(self):
        self.assertEqual(MODULE.validate_plan(page_plan())["slug"], "run-owned:ftml-diff-20260726-001")
        invalid = page_plan()
        invalid["slug"] = "debug"
        with self.assertRaisesRegex(ValueError, "run-owned contract"):
            MODULE.validate_plan(invalid)
        invalid = page_plan()
        invalid["source_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "source hash"):
            MODULE.validate_plan(invalid)

    def test_plan_validation_enforces_saved_page_character_limit(self):
        with self.assertRaisesRegex(ValueError, "saved-page limit"):
            MODULE.validate_plan(page_plan("x" * (MODULE.MAX_SOURCE_CHARACTERS + 1)))

    def test_plan_validation_accepts_one_sentinel_free_isolated_case(self):
        plan = page_plan("alpha_")
        plan["cases"] = [{
            "case_id": "alpha",
            "source_sha256": plan["source_sha256"],
            "page_scope": "isolated",
        }]
        self.assertEqual(MODULE.validate_plan(plan)["cases"][0]["case_id"], "alpha")
        MODULE.verify_saved_markers(plan, {
            "saved_source": "alpha_",
            "saved_source_sha256": plan["source_sha256"],
        })
        plan["cases"][0]["marker_end"] = "WJDIFF_END_ALPHA"
        with self.assertRaisesRegex(ValueError, "not sentinel-free"):
            MODULE.validate_plan(plan)

    def test_ledger_appends_durable_json_lines(self):
        with tempfile.TemporaryDirectory() as root:
            ledger = Path(root) / "ledger.jsonl"
            MODULE.append_ledger(ledger, {"event": "create-intent", "slug": "run-owned:ftml-diff-20260726-001"})
            MODULE.append_ledger(ledger, {"event": "removed", "slug": "run-owned:ftml-diff-20260726-001"})
            values = [json.loads(line) for line in ledger.read_text().splitlines()]
            self.assertEqual([value["event"] for value in values], ["create-intent", "removed"])

    def test_saved_marker_validation_accepts_server_source_normalization(self):
        plan = page_plan()
        snapshot = {
            "saved_source": "WJDIFF_BEGIN_ALPHA\nalpha normalized\nWJDIFF_END_ALPHA",
            "saved_source_sha256": MODULE.sha256("WJDIFF_BEGIN_ALPHA\nalpha normalized\nWJDIFF_END_ALPHA"),
        }
        MODULE.verify_saved_markers(plan, snapshot)
        snapshot["saved_source"] = "alpha normalized"
        with self.assertRaisesRegex(RuntimeError, "preserve its case markers"):
            MODULE.verify_saved_markers(plan, snapshot)


if __name__ == "__main__":
    unittest.main()
