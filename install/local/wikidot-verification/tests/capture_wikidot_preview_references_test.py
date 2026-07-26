import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "scripts" / "capture_wikidot_preview_references.py"
SPEC = importlib.util.spec_from_file_location("capture_wikidot_preview_references", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class CaptureWikidotPreviewReferencesTest(unittest.TestCase):
    def test_reference_record_binds_source_and_html(self):
        case = MODULE.validate_case(
            {
                "schema": MODULE.CASE_SCHEMA,
                "case_id": "bold",
                "source": "**bold**",
                "wikidot_observation_tier": MODULE.PAGE_PREVIEW_TIER,
                "local_execution_tier": "ftml",
            }
        )
        record = MODULE.reference_record(
            case,
            "<strong>bold</strong>",
            site="sandbox-for-codex",
            site_domain="sandbox-for-codex.wikidot.com",
            wikidot_version="4.4.1",
            wikidot_commit="4af7c8eaec00a3e7a29fe502234e0aeeef968233",
            requirements_sha256="frozen-lock",
            captured_at="2026-07-26T00:00:00+00:00",
        )
        self.assertEqual(record["source_sha256"], MODULE.sha256("**bold**"))
        self.assertEqual(record["raw_html_sha256"], MODULE.sha256("<strong>bold</strong>"))
        self.assertFalse(record["provenance"]["authenticated"])
        self.assertFalse(record["provenance"]["mutated"])
        self.assertEqual(record["provenance"]["site_domain"], "sandbox-for-codex.wikidot.com")
        self.assertEqual(
            record["provenance"]["wikidot_py_commit"],
            "4af7c8eaec00a3e7a29fe502234e0aeeef968233",
        )

    def test_non_preview_observation_tier_fails_closed(self):
        with self.assertRaisesRegex(ValueError, "unsupported Wikidot observation tier"):
            MODULE.validate_case(
                {
                    "schema": MODULE.CASE_SCHEMA,
                    "case_id": "include",
                    "source": "[[include component:x]]",
                    "wikidot_observation_tier": "saved-page",
                    "local_execution_tier": "wikijump-runtime",
                }
            )

    def test_live_case_manifest_selects_only_preview_isolated_cases(self):
        preview = MODULE.validate_case(
            {
                "schema": MODULE.LIVE_CASE_SCHEMA,
                "case_id": "isolated",
                "source": "[[code]]x[[/code]]",
                "execution_class": "page-preview-isolated",
            }
        )
        batch = MODULE.validate_case(
            {
                "schema": MODULE.LIVE_CASE_SCHEMA,
                "case_id": "batch",
                "source": "**x**",
                "execution_class": "saved-page-batch",
            }
        )
        self.assertEqual(preview["case_id"], "isolated")
        self.assertEqual(preview["local_execution_tier"], "ftml")
        self.assertIsNone(batch)

    def test_case_loader_splits_only_on_lf(self):
        with tempfile.TemporaryDirectory() as root:
            cases = Path(root) / "cases.jsonl"
            value = {
                "schema": MODULE.LIVE_CASE_SCHEMA,
                "case_id": "unicode-line-separator",
                "source": "alpha\u2028beta",
                "execution_class": "page-preview-isolated",
            }
            cases.write_text(f"{json.dumps(value, ensure_ascii=False)}\n")
            loaded = MODULE.load_cases(cases)
            self.assertEqual(loaded[0]["source"], "alpha\u2028beta")

    def test_case_loader_can_select_batch_cases_for_solo_validation(self):
        with tempfile.TemporaryDirectory() as root:
            cases = Path(root) / "cases.jsonl"
            value = {
                "schema": MODULE.LIVE_CASE_SCHEMA,
                "case_id": "batch",
                "source": "**alpha**",
                "execution_class": "saved-page-batch",
            }
            cases.write_text(f"{json.dumps(value)}\n")
            loaded = MODULE.load_cases(cases, {"saved-page-batch"})
            self.assertEqual([case["case_id"] for case in loaded], ["batch"])

    def test_runtime_live_case_keeps_runtime_local_execution_tier(self):
        runtime = MODULE.validate_case(
            {
                "schema": MODULE.LIVE_CASE_SCHEMA,
                "case_id": "runtime",
                "source": "[[include component:x]]",
                "execution_class": "wikijump-runtime",
            },
            {"wikijump-runtime"},
        )
        self.assertEqual(runtime["local_execution_tier"], "wikijump-runtime")

    def test_frozen_output_cannot_be_replaced(self):
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / "references.jsonl"
            MODULE.write_frozen(output, [{"case_id": "a"}])
            self.assertEqual(json.loads(output.read_text()), {"case_id": "a"})
            with self.assertRaises(FileExistsError):
                MODULE.write_frozen(output, [{"case_id": "b"}])


if __name__ == "__main__":
    unittest.main()
