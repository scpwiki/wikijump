from contextlib import redirect_stderr
import io
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import ANY, patch


LOCAL_INSTALL = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LOCAL_INSTALL))

import deepwell_hot_reload as hot_reload  # noqa: E402


class DeepwellHotReloadTest(unittest.TestCase):
    def hot_reload_arguments(self):
        return hot_reload.build_parser().parse_args(
            [
                "--container",
                "runtime50x-deepwell-1",
                "--source-root",
                "/unused-test-worktree",
                "--timeout",
                "1",
                "--settle",
                "0",
            ]
        )

    def run_patches(self, *, rollback_error=None):
        inspected = {
            "Id": "0123456789abcdef",
            "Name": "/runtime50x-deepwell-1",
            "State": {"Running": True},
            "Mounts": [],
            "Config": {"Image": "runtime50x-deepwell:test"},
        }
        patches = {
            "validate_source_root": patch.object(
                hot_reload,
                "validate_source_root",
                return_value=(Path("/candidate"), [Path("/candidate/deepwell/src")]),
            ),
            "discover_container": patch.object(
                hot_reload,
                "discover_container",
                return_value=(inspected["Id"], inspected),
            ),
            "validate_container": patch.object(hot_reload, "validate_container"),
            "preflight_runtime": patch.object(hot_reload, "preflight_runtime"),
            "daemon_pid": patch.object(hot_reload, "daemon_pid", return_value=41),
            "health_check": patch.object(hot_reload, "health_check", return_value=True),
            "copy_inputs": patch.object(hot_reload, "copy_inputs"),
            "commit_inputs": patch.object(hot_reload, "commit_inputs"),
            "wait_for_replacement": patch.object(
                hot_reload,
                "wait_for_replacement",
                side_effect=hot_reload.HotReloadError("candidate failed"),
            ),
            "zombie_deepwell_process_count": patch.object(
                hot_reload,
                "zombie_deepwell_process_count",
                return_value=0,
            ),
            "rollback_inputs": patch.object(
                hot_reload,
                "rollback_inputs",
                side_effect=rollback_error,
            ),
            "restart_container": patch.object(hot_reload, "restart_container"),
            "wait_for_healthy_daemon": patch.object(
                hot_reload,
                "wait_for_healthy_daemon",
                return_value=73,
            ),
            "cleanup_stage": patch.object(hot_reload, "cleanup_stage"),
        }
        return patches

    def test_success_restarts_when_cargo_watch_leaves_a_zombie_daemon(self):
        patches = self.run_patches()
        patches["wait_for_replacement"] = patch.object(
            hot_reload,
            "wait_for_replacement",
            return_value=52,
        )
        patches["zombie_deepwell_process_count"] = patch.object(
            hot_reload,
            "zombie_deepwell_process_count",
            side_effect=[1, 0],
        )
        with (
            patches["validate_source_root"],
            patches["discover_container"],
            patches["validate_container"],
            patches["preflight_runtime"],
            patches["daemon_pid"],
            patches["health_check"],
            patches["copy_inputs"],
            patches["commit_inputs"],
            patches["wait_for_replacement"],
            patches["zombie_deepwell_process_count"] as zombie_count,
            patches["rollback_inputs"] as rollback,
            patches["restart_container"] as restart,
            patches["wait_for_healthy_daemon"] as wait_for_healthy,
            patches["cleanup_stage"] as cleanup,
        ):
            result = hot_reload.run(self.hot_reload_arguments(), hot_reload.Docker())

        rollback.assert_not_called()
        restart.assert_called_once_with(ANY, "0123456789abcdef")
        wait_for_healthy.assert_called_once()
        self.assertEqual(zombie_count.call_count, 2)
        cleanup.assert_called_once()
        self.assertEqual(result["new_daemon_pid"], 73)
        self.assertTrue(result["container_restarted"])
        self.assertEqual(result["zombies_after_replacement"], 1)

    def test_success_keeps_the_replacement_when_no_zombie_remains(self):
        patches = self.run_patches()
        patches["wait_for_replacement"] = patch.object(
            hot_reload,
            "wait_for_replacement",
            return_value=52,
        )
        with (
            patches["validate_source_root"],
            patches["discover_container"],
            patches["validate_container"],
            patches["preflight_runtime"],
            patches["daemon_pid"],
            patches["health_check"],
            patches["copy_inputs"],
            patches["commit_inputs"],
            patches["wait_for_replacement"],
            patches["zombie_deepwell_process_count"],
            patches["rollback_inputs"] as rollback,
            patches["restart_container"] as restart,
            patches["wait_for_healthy_daemon"] as wait_for_healthy,
            patches["cleanup_stage"] as cleanup,
        ):
            result = hot_reload.run(self.hot_reload_arguments(), hot_reload.Docker())

        rollback.assert_not_called()
        restart.assert_not_called()
        wait_for_healthy.assert_not_called()
        cleanup.assert_called_once()
        self.assertEqual(result["new_daemon_pid"], 52)
        self.assertFalse(result["container_restarted"])
        self.assertEqual(result["zombies_after_replacement"], 0)

    def test_paths_overlap_for_parent_child_and_exact_paths(self):
        self.assertTrue(hot_reload.paths_overlap("/src", "/src/deepwell/src"))
        self.assertTrue(
            hot_reload.paths_overlap("/src/deepwell/src", "/src/deepwell/src")
        )
        self.assertFalse(hot_reload.paths_overlap("/opt/locales", "/src/deepwell/src"))

    def test_validate_container_rejects_a_source_bind_mount(self):
        inspected = {
            "Name": "/runtime50x-deepwell-1",
            "State": {"Running": True},
            "Mounts": [
                {
                    "Destination": "/src/deepwell/src",
                    "Source": "/host/deepwell/src",
                    "RW": False,
                }
            ],
        }
        with self.assertRaisesRegex(hot_reload.HotReloadError, "bind-mount"):
            hot_reload.validate_container(inspected)

    def test_validate_container_allows_unrelated_locale_mount(self):
        inspected = {
            "Name": "/runtime50x-deepwell-1",
            "State": {"Running": True},
            "Mounts": [{"Destination": "/opt/locales", "RW": False}],
        }
        hot_reload.validate_container(inspected)

    def test_validate_source_root_returns_explicit_build_inputs(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            deepwell = root / "deepwell"
            (deepwell / "src").mkdir(parents=True)
            (deepwell / "Cargo.toml").write_text(
                '[package]\nname = "deepwell"\n', encoding="utf-8"
            )
            for filename in ("Cargo.lock", "build.rs", "askama.toml"):
                (deepwell / filename).touch()

            source_root, entries = hot_reload.validate_source_root(root)

        self.assertEqual(source_root, root.resolve())
        self.assertEqual(
            [entry.name for entry in entries], list(hot_reload.SYNC_ENTRIES)
        )

    def test_argument_parser_rejects_zero_timeout(self):
        parser = hot_reload.build_parser()
        with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            parser.parse_args(["--timeout", "0"])

    def test_argument_parser_rejects_non_finite_wait_values(self):
        parser = hot_reload.build_parser()
        for flag in ("--timeout", "--settle"):
            for value in ("nan", "inf", "-inf", "1e309"):
                with (
                    self.subTest(flag=flag, value=value),
                    redirect_stderr(io.StringIO()),
                    self.assertRaises(SystemExit),
                ):
                    parser.parse_args([flag, value])

    def test_container_lock_rejects_a_concurrent_copy_and_releases(self):
        container = "unit-test-container"
        lock_path = Path(tempfile.gettempdir()) / (
            f"wikijump-deepwell-hot-reload-{container}.lock"
        )
        try:
            with hot_reload.container_lock(container):
                with self.assertRaisesRegex(hot_reload.HotReloadError, "another"):
                    with hot_reload.container_lock(container):
                        self.fail("the second lock unexpectedly succeeded")
            with hot_reload.container_lock(container):
                pass
        finally:
            lock_path.unlink(missing_ok=True)

    def test_commit_keeps_rollback_trap_armed_through_reload_trigger(self):
        with patch.object(hot_reload, "exec_shell") as execute:
            hot_reload.commit_inputs(
                hot_reload.Docker(),
                "runtime50x-deepwell-1",
                "/tmp/hot-reload-stage",
            )

        script = execute.call_args.args[2]
        trigger = script.index('touch "$destination/Cargo.toml"')
        committed = script.index("committed=yes")
        disarmed = script.index("trap - HUP INT TERM EXIT")
        self.assertLess(trigger, committed)
        self.assertLess(committed, disarmed)

    def test_failed_candidate_rolls_back_restarts_and_waits_for_stable_daemon(self):
        patches = self.run_patches()
        with (
            patches["validate_source_root"],
            patches["discover_container"],
            patches["validate_container"],
            patches["preflight_runtime"],
            patches["daemon_pid"],
            patches["health_check"],
            patches["copy_inputs"],
            patches["commit_inputs"],
            patches["wait_for_replacement"],
            patches["rollback_inputs"] as rollback,
            patches["restart_container"] as restart,
            patches["wait_for_healthy_daemon"] as wait_for_healthy,
            patches["cleanup_stage"] as cleanup,
            self.assertRaisesRegex(hot_reload.HotReloadError, "recovered as PID 73"),
        ):
            hot_reload.run(self.hot_reload_arguments(), hot_reload.Docker())

        rollback.assert_called_once()
        restart.assert_called_once_with(ANY, "0123456789abcdef")
        wait_for_healthy.assert_called_once()
        cleanup.assert_called_once()

    def test_recovery_health_wait_requires_a_stable_daemon_pid(self):
        with (
            patch.object(
                hot_reload,
                "daemon_pid",
                side_effect=[51, 52, 73, 73],
            ),
            patch.object(hot_reload, "health_check", return_value=True),
            patch.object(hot_reload.time, "sleep"),
        ):
            recovered = hot_reload.wait_for_healthy_daemon(
                hot_reload.Docker(),
                "runtime50x-deepwell-1",
                timeout=1,
                settle=0.1,
            )

        self.assertEqual(recovered, 73)

    def test_interrupted_candidate_rolls_back_and_restarts_before_returning(self):
        patches = self.run_patches()
        patches["wait_for_replacement"] = patch.object(
            hot_reload,
            "wait_for_replacement",
            side_effect=KeyboardInterrupt(),
        )
        with (
            patches["validate_source_root"],
            patches["discover_container"],
            patches["validate_container"],
            patches["preflight_runtime"],
            patches["daemon_pid"],
            patches["health_check"],
            patches["copy_inputs"],
            patches["commit_inputs"],
            patches["wait_for_replacement"],
            patches["rollback_inputs"] as rollback,
            patches["restart_container"] as restart,
            patches["wait_for_healthy_daemon"] as wait_for_healthy,
            patches["cleanup_stage"] as cleanup,
            self.assertRaises(KeyboardInterrupt),
        ):
            hot_reload.run(self.hot_reload_arguments(), hot_reload.Docker())

        rollback.assert_called_once()
        restart.assert_called_once_with(ANY, "0123456789abcdef")
        wait_for_healthy.assert_called_once()
        cleanup.assert_called_once()

    def test_failed_rollback_preserves_staged_backup(self):
        rollback_error = hot_reload.HotReloadError("rollback failed")
        patches = self.run_patches(rollback_error=rollback_error)
        with (
            patches["validate_source_root"],
            patches["discover_container"],
            patches["validate_container"],
            patches["preflight_runtime"],
            patches["daemon_pid"],
            patches["health_check"],
            patches["copy_inputs"],
            patches["commit_inputs"],
            patches["wait_for_replacement"],
            patches["rollback_inputs"] as rollback,
            patches["restart_container"] as restart,
            patches["wait_for_healthy_daemon"] as wait_for_healthy,
            patches["cleanup_stage"] as cleanup,
            self.assertRaisesRegex(hot_reload.HotReloadError, "backup preserved"),
        ):
            hot_reload.run(self.hot_reload_arguments(), hot_reload.Docker())

        rollback.assert_called_once()
        restart.assert_not_called()
        wait_for_healthy.assert_not_called()
        cleanup.assert_not_called()

    def test_interrupted_failed_rollback_reports_preserved_backup(self):
        patches = self.run_patches(
            rollback_error=hot_reload.HotReloadError("rollback failed")
        )
        patches["wait_for_replacement"] = patch.object(
            hot_reload,
            "wait_for_replacement",
            side_effect=KeyboardInterrupt(),
        )
        with (
            patches["validate_source_root"],
            patches["discover_container"],
            patches["validate_container"],
            patches["preflight_runtime"],
            patches["daemon_pid"],
            patches["health_check"],
            patches["copy_inputs"],
            patches["commit_inputs"],
            patches["wait_for_replacement"],
            patches["rollback_inputs"] as rollback,
            patches["restart_container"] as restart,
            patches["wait_for_healthy_daemon"] as wait_for_healthy,
            patches["cleanup_stage"] as cleanup,
            self.assertRaisesRegex(
                hot_reload.HotReloadError,
                r"KeyboardInterrupt; automatic rollback failed: rollback failed; "
                r"backup preserved in /tmp/wikijump-deepwell-hot-reload-",
            ),
        ):
            hot_reload.run(self.hot_reload_arguments(), hot_reload.Docker())

        rollback.assert_called_once()
        restart.assert_not_called()
        wait_for_healthy.assert_not_called()
        cleanup.assert_not_called()


if __name__ == "__main__":
    unittest.main()
