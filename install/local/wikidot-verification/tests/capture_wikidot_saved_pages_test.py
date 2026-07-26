import importlib.util
import json
import signal
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

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

    def test_capture_records_survive_a_later_exception_and_count_for_resume(self):
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / "captures.jsonl"
            with self.assertRaisesRegex(RuntimeError, "AMC not_ok"):
                with output.open("x", encoding="utf-8") as result:
                    MODULE.append_capture_record(result, {"capture_status": "captured", "page_identity": 1})
                    MODULE.append_capture_record(result, {"capture_status": "render-failed", "page_identity": 2})
                    raise RuntimeError("AMC not_ok")

            records = [json.loads(line) for line in output.read_text().splitlines()]
            self.assertEqual([record["page_identity"] for record in records], [1, 2])
            self.assertEqual(len(records), 2)
            with self.assertRaises(FileExistsError):
                output.open("x", encoding="utf-8")

    def test_capture_record_is_flushed_and_fsynced_immediately(self):
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / "captures.jsonl"
            with output.open("x", encoding="utf-8") as result:
                with mock.patch.object(MODULE.os, "fsync") as fsync:
                    MODULE.append_capture_record(result, {"capture_status": "captured"})
                    fsync.assert_called_once_with(result.fileno())
                self.assertEqual(json.loads(output.read_text())["capture_status"], "captured")

    def test_remove_created_retires_each_page_immediately(self):
        snapshot = {
            "slug": "run-owned:ftml-diff-20260726-001",
            "identity": 42,
            "saved_source": "alpha",
            "saved_source_sha256": MODULE.sha256("alpha"),
        }
        created = [snapshot]
        with tempfile.TemporaryDirectory() as root:
            ledger = Path(root) / "ledger.jsonl"
            with mock.patch.object(MODULE, "remove_exact") as remove_exact:
                MODULE.remove_created(object(), snapshot, created, ledger)
            remove_exact.assert_called_once()
            self.assertEqual(created, [])
            self.assertEqual(json.loads(ledger.read_text())["event"], "removed")

    def test_capture_is_appended_only_after_immediate_cleanup_succeeds(self):
        events = []
        with mock.patch.object(
            MODULE,
            "remove_created",
            side_effect=lambda *_args: events.append("removed"),
        ), mock.patch.object(
            MODULE,
            "append_capture_record",
            side_effect=lambda *_args: events.append("appended"),
        ):
            MODULE.retire_and_append_capture(
                object(),
                {"slug": "run-owned:ftml-diff-20260726-001"},
                [],
                Path("ledger.jsonl"),
                object(),
                {"capture_status": "captured"},
            )
        self.assertEqual(events, ["removed", "appended"])

        with mock.patch.object(
            MODULE,
            "remove_created",
            side_effect=RuntimeError("cleanup failed"),
        ), mock.patch.object(MODULE, "append_capture_record") as append:
            with self.assertRaisesRegex(RuntimeError, "cleanup failed"):
                MODULE.retire_and_append_capture(
                    object(),
                    {"slug": "run-owned:ftml-diff-20260726-001"},
                    [],
                    Path("ledger.jsonl"),
                    object(),
                    {"capture_status": "captured"},
                )
            append.assert_not_called()

    def test_interrupt_flag_defers_the_signal_until_an_explicit_boundary(self):
        interrupt = MODULE.InterruptFlag()

        interrupt.request(signal.SIGINT, None)

        self.assertEqual(interrupt.signum, signal.SIGINT)
        with self.assertRaisesRegex(InterruptedError, "signal 2"):
            interrupt.raise_if_requested()

    def test_signal_during_cleanup_keeps_prior_and_current_records_and_no_residual_page(self):
        interrupt = MODULE.InterruptFlag()
        pages = {}

        def snapshot(identity: int) -> dict:
            slug = f"run-owned:ftml-diff-20260726-{identity:03d}"
            value = {
                "slug": slug,
                "identity": identity,
                "title": f"FTML differential {identity:03d}",
                "saved_source": f"source {identity}",
                "saved_source_sha256": MODULE.sha256(f"source {identity}"),
            }
            page = SimpleNamespace(
                id=identity,
                title=value["title"],
                source=SimpleNamespace(wiki_text=value["saved_source"]),
            )
            page.refresh_source = mock.Mock(
                side_effect=(
                    lambda: interrupt.request(signal.SIGINT, None)
                    if identity == 2
                    else None
                )
            )
            page.destroy = mock.Mock(side_effect=lambda: pages.pop(slug))
            pages[slug] = page
            return value

        prior = snapshot(1)
        current = snapshot(2)
        created = [prior, current]
        site = SimpleNamespace(
            page=SimpleNamespace(
                get=mock.Mock(side_effect=lambda slug, **_kwargs: pages.get(slug)),
            )
        )

        with tempfile.TemporaryDirectory() as root:
            ledger = Path(root) / "ledger.jsonl"
            output = Path(root) / "captures.jsonl"
            with output.open("x", encoding="utf-8") as result:
                MODULE.retire_and_append_capture(
                    site,
                    prior,
                    created,
                    ledger,
                    result,
                    {"capture_status": "captured", "page_identity": 1},
                    interrupt,
                )
                with self.assertRaisesRegex(InterruptedError, "signal 2"):
                    MODULE.retire_and_append_capture(
                        site,
                        current,
                        created,
                        ledger,
                        result,
                        {"capture_status": "captured", "page_identity": 2},
                        interrupt,
                    )

            records = [json.loads(line) for line in output.read_text().splitlines()]
            removals = [json.loads(line) for line in ledger.read_text().splitlines()]

        self.assertEqual([record["page_identity"] for record in records], [1, 2])
        self.assertEqual([record["identity"] for record in removals], [1, 2])
        self.assertEqual(created, [])
        self.assertEqual(pages, {})

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

    def test_create_error_recovery_accepts_only_one_removed_trailing_lf(self):
        plan = page_plan("alpha\n")
        page = SimpleNamespace(
            id=42,
            title=plan["title"],
            source=SimpleNamespace(wiki_text="alpha"),
            refresh_source=mock.Mock(),
        )
        site = SimpleNamespace(page=SimpleNamespace(get=mock.Mock(return_value=page)))

        snapshot = MODULE.recover_snapshot_after_create_error(site, plan)

        self.assertEqual(snapshot["identity"], 42)
        self.assertEqual(snapshot["saved_source"], "alpha")
        page.refresh_source.assert_called_once_with()

    def test_create_error_recovery_refuses_title_or_source_mismatch(self):
        plan = page_plan("alpha\n")
        for title, source in [
            ("Different title", "alpha"),
            (plan["title"], "different"),
            (plan["title"], "alpha\n\n"),
        ]:
            with self.subTest(title=title, source=source):
                page = SimpleNamespace(
                    id=42,
                    title=title,
                    source=SimpleNamespace(wiki_text=source),
                    refresh_source=mock.Mock(),
                )
                site = SimpleNamespace(page=SimpleNamespace(get=mock.Mock(return_value=page)))
                with self.assertRaisesRegex(RuntimeError, "cleanup refused"):
                    MODULE.recover_snapshot_after_create_error(site, plan)

    def test_create_error_recovery_returns_none_when_no_page_was_created(self):
        plan = page_plan()
        site = SimpleNamespace(page=SimpleNamespace(get=mock.Mock(return_value=None)))
        self.assertIsNone(MODULE.recover_snapshot_after_create_error(site, plan))

    def test_partial_create_success_continues_with_recovered_snapshot(self):
        plan = page_plan("alpha\n")
        page = SimpleNamespace(
            id=42,
            title=plan["title"],
            source=SimpleNamespace(wiki_text="alpha"),
            refresh_source=mock.Mock(),
        )
        pages = SimpleNamespace(
            create=mock.Mock(side_effect=RuntimeError("AMC not_ok")),
            get=mock.Mock(return_value=page),
        )

        snapshot, event = MODULE.create_or_recover_snapshot(
            SimpleNamespace(page=pages),
            plan,
        )

        self.assertEqual(snapshot["identity"], 42)
        self.assertEqual(event, "created-after-save-error")

    def test_create_error_without_created_page_reraises_original_error(self):
        plan = page_plan()
        pages = SimpleNamespace(
            create=mock.Mock(side_effect=RuntimeError("AMC not_ok")),
            get=mock.Mock(return_value=None),
        )
        with self.assertRaisesRegex(RuntimeError, "AMC not_ok"):
            MODULE.create_or_recover_snapshot(SimpleNamespace(page=pages), plan)

    def test_cleanup_refuses_recovered_page_identity_change(self):
        snapshot = {
            "slug": "run-owned:ftml-diff-20260726-001",
            "identity": 42,
            "title": "FTML differential 001",
            "saved_source": "alpha",
            "saved_source_sha256": MODULE.sha256("alpha"),
        }
        page = SimpleNamespace(
            id=43,
            title=snapshot["title"],
            source=SimpleNamespace(wiki_text="alpha"),
            refresh_source=mock.Mock(),
            destroy=mock.Mock(),
        )
        site = SimpleNamespace(page=SimpleNamespace(get=mock.Mock(return_value=page)))

        with self.assertRaisesRegex(RuntimeError, "cleanup refused"):
            MODULE.remove_exact(site, snapshot)
        page.destroy.assert_not_called()


if __name__ == "__main__":
    unittest.main()
