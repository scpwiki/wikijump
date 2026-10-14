from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import unittest


ROOT = Path(__file__).parents[3]
SCRIPT = Path(__file__).parents[1] / "render_ahead.py"
SPEC = importlib.util.spec_from_file_location("standing_render_ahead", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
RENDER_AHEAD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RENDER_AHEAD)


class RenderAheadMigrationTest(unittest.TestCase):
    def test_the_shipped_migration_is_idempotent(self) -> None:
        migration = ROOT / "deepwell" / "migrations" / Path(RENDER_AHEAD.MIGRATION_PATH).name
        RENDER_AHEAD.require_idempotent_migration(migration.read_text(encoding="utf-8"))

    def test_non_idempotent_statements_are_rejected(self) -> None:
        for sql in (
            "CREATE TABLE page_revision_render_ahead (revision_id BIGINT);",
            "CREATE TABLE IF NOT EXISTS a (b INT);\nALTER TABLE page DROP COLUMN slug;",
            "-- comment only\n",
        ):
            with self.subTest(sql=sql), self.assertRaises(ValueError):
                RENDER_AHEAD.require_idempotent_migration(sql)

    def test_comments_are_not_statements(self) -> None:
        self.assertEqual(
            RENDER_AHEAD.sql_statements(
                "-- a; b\nCREATE TABLE IF NOT EXISTS a (\n  b INT -- c\n);\n"
            ),
            ["CREATE TABLE IF NOT EXISTS a (\n  b INT -- c\n)"],
        )


class RenderAheadRuntimeTest(unittest.TestCase):
    def test_environment_drops_inherited_runtime_actions(self) -> None:
        inspected = {
            "Config": {
                "Env": [
                    "DATABASE_URL=postgres://db",
                    "REDIS_URL=redis://cache",
                    "DEEPWELL_RPC_TOKEN=" + "0" * 64,
                    "DEEPWELL_RUNTIME_ACTION=render-finalize",
                    "DEEPWELL_RENDER_PASS=pass2",
                    "PATH=/usr/bin",
                ]
            }
        }
        self.assertEqual(
            RENDER_AHEAD.runtime_environment(inspected),
            {
                "DATABASE_URL": "postgres://db",
                "REDIS_URL": "redis://cache",
                "DEEPWELL_RPC_TOKEN": "0" * 64,
                "PATH": "/usr/bin",
            },
        )

    def test_environment_requires_backing_services(self) -> None:
        with self.assertRaises(RuntimeError):
            RENDER_AHEAD.runtime_environment({"Config": {"Env": ["REDIS_URL=x"]}})

    def test_config_mount_is_required(self) -> None:
        self.assertEqual(
            RENDER_AHEAD.config_mount(
                {"Mounts": [{"Destination": "/etc/deepwell.toml", "Source": "/srv/config.toml"}]}
            ),
            "/srv/config.toml",
        )
        with self.assertRaises(RuntimeError):
            RENDER_AHEAD.config_mount({"Mounts": []})

    def test_settings_only_name_not_before_for_a_second_pass(self) -> None:
        first = RENDER_AHEAD.render_ahead_settings(8, 100, None)
        self.assertEqual(first["DEEPWELL_RUNTIME_ACTION"], "render-ahead")
        self.assertEqual(first["DEEPWELL_RENDER_AHEAD_DRY_RUN"], "false")
        self.assertNotIn("DEEPWELL_RENDER_AHEAD_NOT_BEFORE", first)
        second = RENDER_AHEAD.render_ahead_settings(8, 100, "2026-10-14T00:00:00Z")
        self.assertEqual(second["DEEPWELL_RENDER_AHEAD_NOT_BEFORE"], "2026-10-14T00:00:00Z")

    def test_link_digest_ignores_row_timestamps(self) -> None:
        self.assertNotIn("_at", RENDER_AHEAD.LINK_DIGEST_QUERY)
        for table in ("page_connection", "page_connection_missing", "page_link"):
            self.assertIn(f"FROM {table}", RENDER_AHEAD.LINK_DIGEST_QUERY)

    def test_changed_pages_include_added_and_removed_link_sets(self) -> None:
        self.assertEqual(
            RENDER_AHEAD.changed_pages({"1": "a", "2": "b", "3": "c"}, {"1": "a", "2": "x", "4": "d"}),
            [2, 3, 4],
        )

    def test_settings_limit_pages_and_dry_runs(self) -> None:
        settings = RENDER_AHEAD.render_ahead_settings(1, 100, None, [5, 9], dry_run=True)
        self.assertEqual(settings["DEEPWELL_RENDER_AHEAD_PAGE_IDS"], "5,9")
        self.assertEqual(settings["DEEPWELL_RENDER_AHEAD_DRY_RUN"], "true")
        with self.assertRaises(ValueError):
            RENDER_AHEAD.render_ahead_settings(1, 100, None, [])

    def test_only_deadline_failures_are_retried(self) -> None:
        deadline = {"error": "failed to render\n`-- deadline has elapsed, at service.rs"}
        other = {"error": "include expansion exceeded maximum total includes 256"}
        self.assertTrue(RENDER_AHEAD.deadline_only({"failed": 1, "failures": [deadline]}))
        self.assertFalse(
            RENDER_AHEAD.deadline_only({"failed": 2, "failures": [deadline, other]})
        )
        self.assertFalse(
            RENDER_AHEAD.deadline_only({"failed": 51, "failures": [deadline] * 50}),
            "truncated failure details cannot prove every failure is a deadline",
        )

    def test_extended_deadline_replaces_exactly_one_setting(self) -> None:
        config = "[main]\nrender-timeout-ms = 2000\nother = 1\n"
        self.assertEqual(
            RENDER_AHEAD.config_with_render_timeout(config, 300000),
            "[main]\nrender-timeout-ms = 300000\nother = 1\n",
        )
        for bad in ("[main]\n", "render-timeout-ms = 1\nrender-timeout-ms = 2\n"):
            with self.subTest(config=bad), self.assertRaises(ValueError):
                RENDER_AHEAD.config_with_render_timeout(bad, 300000)

    def test_summary_must_be_one_real_run(self) -> None:
        summary = {"action": "render-ahead", "dry_run": False, "failed": 0}
        output = "log line\n" + json.dumps(summary) + "\n"
        self.assertEqual(RENDER_AHEAD.parse_summary(output), summary)
        dry = {**summary, "dry_run": True}
        self.assertEqual(RENDER_AHEAD.parse_summary(json.dumps(dry), dry_run=True), dry)
        for bad in (
            "no summary\n",
            json.dumps(summary) + "\n" + json.dumps(summary) + "\n",
            json.dumps(dry) + "\n",
        ):
            with self.subTest(output=bad), self.assertRaises(RuntimeError):
                RENDER_AHEAD.parse_summary(bad)


class RenderUntilLinksSettleTest(unittest.TestCase):
    """Fake site: page 1 is stable, page 2 links differently on every render,
    page 3 reads page 1's links and changes once they move."""

    def setUp(self) -> None:
        self.clock = iter(f"t{index}" for index in range(100))
        self.original_now = RENDER_AHEAD.utc_now
        RENDER_AHEAD.utc_now = lambda: next(self.clock)
        self.links = {"1": "old", "2": "r0", "3": "old"}
        self.renders = 0
        self.calls = []

    def tearDown(self) -> None:
        RENDER_AHEAD.utc_now = self.original_now

    def render(self, name, cutoff, pages=None):
        self.calls.append((name, cutoff, pages))
        selected = pages or [1, 2, 3]
        self.renders += 1
        for page in selected:
            if page == 1:
                self.links["1"] = "new"
            elif page == 2:
                self.links["2"] = f"r{self.renders}"
            elif page == 3 and self.renders > 1:
                self.links["3"] = "new"
        return {"candidates": len(selected), "failed_page_ids": []}

    def settle(self, not_before=None):
        passes = [{}]
        def render(name, cutoff, pages=None):
            passes.append({})
            return self.render(name, cutoff, pages)
        return RENDER_AHEAD.render_until_links_settle(render, lambda: dict(self.links), passes, not_before)

    def test_dependency_changes_render_everything_again(self) -> None:
        not_before, _, nondeterministic = self.settle()
        self.assertEqual(nondeterministic, [2])
        self.assertEqual(
            [call[0] for call in self.calls],
            ["full-1", "link-probe-1", "full-2", "link-probe-2", "full-3", "link-probe-3"],
        )
        self.assertIsNotNone(not_before, "the second pass must not accept rows from the first")

    def test_probe_uses_a_fresh_cutoff(self) -> None:
        self.settle(not_before="t-start")
        probe_cutoffs = [cutoff for name, cutoff, _ in self.calls if name.startswith("link-probe")]
        full_cutoffs = {cutoff for name, cutoff, _ in self.calls if name.startswith("full")}
        self.assertTrue(probe_cutoffs)
        self.assertTrue(full_cutoffs.isdisjoint(probe_cutoffs))

    def test_stable_links_need_one_pass(self) -> None:
        self.links["1"] = "new"
        self.render = lambda name, cutoff, pages=None: (
            self.calls.append((name, cutoff, pages)) or {"candidates": 3, "failed_page_ids": []}
        )
        _, _, nondeterministic = self.settle()
        self.assertEqual(nondeterministic, [])
        self.assertEqual([call[0] for call in self.calls], ["full-1"])

    def test_links_that_keep_moving_fail_the_run(self) -> None:
        render = self.render
        def moving(name, cutoff, pages=None):
            result = render(name, cutoff, pages)
            if not pages:
                # Page 4's links move on every full pass but are stable when
                # page 4 is rendered alone: a dependency that never settles.
                self.links["4"] = f"moved-{self.renders}"
            return result
        self.render = moving
        with self.assertRaises(RuntimeError):
            self.settle()

    def test_probe_must_select_every_changed_page(self) -> None:
        render = self.render
        self.render = lambda name, cutoff, pages=None: (
            {"candidates": 0, "failed_page_ids": []} if pages else render(name, cutoff, pages)
        )
        with self.assertRaises(RuntimeError):
            self.settle()


if __name__ == "__main__":
    unittest.main()
