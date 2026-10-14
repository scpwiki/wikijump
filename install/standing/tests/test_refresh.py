from __future__ import annotations

import importlib.util
import hashlib
import json
import os
from contextlib import contextmanager
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


SCRIPT = Path(__file__).parents[1] / "refresh.py"
sys.path.insert(0, str(SCRIPT.parent))
SPEC = importlib.util.spec_from_file_location("standing_refresh", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
REFRESH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REFRESH)


@contextmanager
def merged_candidate(identity: dict[str, str], candidate_commit: str = "9" * 40):
    original_command = REFRESH.command

    def fake_command(*args: str, cwd: Path, capture: bool = True) -> str:
        if args[:2] == ("git", "merge-base"):
            return candidate_commit
        if args[:3] == ("git", "diff", "--name-only"):
            return "install/standing/refresh.py"
        if args[:3] == ("git", "rev-list", "--parents"):
            return " ".join((identity["wikijump_sha"], "8" * 40, candidate_commit))
        raise AssertionError(args)

    REFRESH.command = fake_command
    try:
        yield
    finally:
        REFRESH.command = original_command



class RefreshStandingTest(unittest.TestCase):
    def test_default_runtime_home_is_user_state_not_wjlab(self) -> None:
        with mock.patch.dict(
            os.environ, {"XDG_STATE_HOME": "/tmp/wikijump-state"}, clear=False
        ):
            self.assertEqual(
                REFRESH.default_runtime_home(),
                Path("/tmp/wikijump-state/wikijump/standing"),
            )
        with mock.patch.dict(
            os.environ, {"XDG_STATE_HOME": "relative-state"}, clear=False
        ):
            with self.assertRaisesRegex(ValueError, "must be absolute"):
                REFRESH.default_runtime_home()

    def test_container_identity_rejects_mutable_identity(self) -> None:
        with self.assertRaisesRegex(ValueError, "immutable container ID"):
            REFRESH.normalize_container_identity(
                {
                    "Id": "container",
                    "Image": "sha256:" + "a" * 64,
                    "Name": "/wikijump-standing-caddy-1",
                    "State": {"Status": "running"},
                    "NetworkSettings": {"Ports": {}},
                }
            )

    def test_image_identity_rejects_mutable_image_id(self) -> None:
        original_command = REFRESH.command
        try:
            REFRESH.command = lambda *args, cwd, capture=True: json.dumps(
                {"Id": "latest"}
            )
            with self.assertRaisesRegex(ValueError, "immutable SHA-256 image ID"):
                REFRESH.image_identity("candidate", Path("/runtime"))
        finally:
            REFRESH.command = original_command

    def test_port_443_owner_rejects_zero_or_two_owners(self) -> None:
        owner = {
            "container_id": "a" * 64,
            "name": "wikijump-standing-caddy-1",
            "image_id": "sha256:" + "b" * 64,
            "published_ports": [
                {
                    "container_port": "443/tcp",
                    "host_ip": "127.0.0.1",
                    "host_port": "443",
                }
            ],
        }
        with self.assertRaisesRegex(ValueError, "exactly one"):
            REFRESH.port_443_owner({})
        with self.assertRaisesRegex(ValueError, "exactly one"):
            REFRESH.port_443_owner(
                {"one": owner, "two": {**owner, "container_id": "c" * 64}}
            )

    def test_atomic_receipt_does_not_replace_existing_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            path = Path(temporary_dir) / "receipt.json"
            REFRESH.atomic_json_no_replace(path, {"status": "pass"})
            original = path.read_text(encoding="utf-8")
            with self.assertRaises(FileExistsError):
                REFRESH.atomic_json_no_replace(path, {"status": "changed"})
            self.assertEqual(path.read_text(encoding="utf-8"), original)

    def test_candidate_cleanup_proves_container_and_image_absence(self) -> None:
        candidate = {
            service: {"container_id": chr(99 + index) * 64}
            for index, service in enumerate(REFRESH.SERVICES)
        }
        candidate_images = {
            service: {"id": "sha256:" + chr(100 + index) * 64}
            for index, service in enumerate(REFRESH.SERVICES)
        }
        rollback_images = {
            service: {"id": "sha256:" + "a" * 64} for service in REFRESH.SERVICES
        }
        containers = {value["container_id"] for value in candidate.values()}
        images = {value["id"] for value in candidate_images.values()}
        with self.assertRaisesRegex(RuntimeError, "inventory is unavailable"):
            REFRESH.remove_candidate_resources(
                None, candidate_images, rollback_images, Path("/runtime")
            )
        original_command = REFRESH.command

        def fake_command(*args: str, cwd: Path, capture: bool = True) -> str:
            if args[:2] == ("docker", "inspect") or args[:3] == (
                "docker",
                "image",
                "inspect",
            ):
                reference = args[-1]
                if reference in containers or reference in images:
                    return "present"
                raise subprocess.CalledProcessError(1, args)
            if args[:3] == ("docker", "rm", "--force"):
                containers.remove(args[3])
                return ""
            if args[:3] == ("docker", "image", "rm"):
                images.remove(args[3])
                return ""
            raise AssertionError(args)

        try:
            REFRESH.command = fake_command
            result = REFRESH.remove_candidate_resources(
                candidate, candidate_images, rollback_images, Path("/runtime")
            )
        finally:
            REFRESH.command = original_command
        self.assertEqual(result["status"], "pass")
        self.assertEqual(len(result["containers_removed"]), 3)
        self.assertEqual(len(result["images_removed"]), 3)
        self.assertEqual(containers, set())
        self.assertEqual(images, set())

    def test_rollback_restores_parked_container_identity(self) -> None:
        service = REFRESH.SERVICES[0]
        original_name = f"wikijump-standing-{service}-1"
        parked_name = REFRESH.parked_container_name("run", service)
        state = {
            parked_name: {
                "container_id": "a" * 64,
                "name": parked_name,
                "image_id": "sha256:" + "b" * 64,
                "running": False,
            }
        }
        parked = {
            service: {
                "parked_name": parked_name,
                "original_name": original_name,
                "was_running": True,
                "container": state[parked_name],
            }
        }
        original_command = REFRESH.command
        original_container_identity = REFRESH.container_identity

        def fake_command(*args: str, cwd: Path, capture: bool = True) -> str:
            if args[:2] == ("docker", "rename"):
                current, target = args[2:4]
                state[target] = {**state.pop(current), "name": target, "running": False}
                return ""
            if args[:2] == ("docker", "start"):
                state[args[2]]["running"] = True
                return ""
            raise AssertionError(args)

        def fake_identity(reference: str, cwd: Path) -> dict[str, object]:
            return state[reference]

        try:
            REFRESH.command = fake_command
            REFRESH.container_identity = fake_identity
            restored = REFRESH.restore_parked_containers(parked, Path("/runtime"))
        finally:
            REFRESH.command = original_command
            REFRESH.container_identity = original_container_identity
        self.assertEqual(restored[service]["container_id"], "a" * 64)
        self.assertTrue(restored[service]["running"])
        self.assertEqual(restored[service]["name"], original_name)

    def test_parking_records_old_container_ids_and_names(self) -> None:
        state = {}
        previous = {}
        for index, service in enumerate(REFRESH.SERVICES):
            container_id = chr(97 + index) * 64
            name = f"wikijump-standing-{service}-1"
            state[container_id] = {
                "container_id": container_id,
                "name": name,
                "image_id": "sha256:" + chr(100 + index) * 64,
                "running": True,
            }
            previous[service] = state[container_id]
        original_command = REFRESH.command
        original_container_identity = REFRESH.container_identity

        def fake_command(*args: str, cwd: Path, capture: bool = True) -> str:
            if args[:2] == ("docker", "inspect"):
                reference = args[-1]
                if reference in state:
                    return "present"
                raise subprocess.CalledProcessError(1, args)
            if args[:2] == ("docker", "rename"):
                current, target = args[2:4]
                state[target] = {**state.pop(current), "name": target}
                return ""
            if args[:2] == ("docker", "stop"):
                state[args[2]]["running"] = False
                return ""
            raise AssertionError(args)

        try:
            REFRESH.command = fake_command
            REFRESH.container_identity = lambda reference, _cwd: state[reference]
            parked = REFRESH.park_containers(previous, Path("/runtime"), "run")
        finally:
            REFRESH.command = original_command
            REFRESH.container_identity = original_container_identity
        self.assertEqual(
            {
                service: entry["container"]["container_id"]
                for service, entry in parked.items()
            },
            {service: value["container_id"] for service, value in previous.items()},
        )
        self.assertTrue(all(entry["parked_name"] in state for entry in parked.values()))
        self.assertTrue(
            all(not entry["container"]["running"] for entry in parked.values())
        )

    def prepared_receipt_fixture(
        self, root: Path, *, expiry: object = "2026-09-05T12:34:56.123456+00:00"
    ) -> tuple[dict[str, object], dict[str, str]]:
        for service in REFRESH.SERVICES:
            path = root / "install/prod" / service / "Dockerfile"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(service, encoding="utf-8")
        identity = {
            "wikijump_sha": "a" * 40,
            "wikijump_tree": "b" * 40,
            "ftml_sha": "c" * 40,
            "dependency_lock_sha256": "d" * 64,
        }
        proof_path = root / "promotion-precondition.json"
        proof_path.write_text(
            json.dumps(
                {
                    "schema": REFRESH.PROMOTION_PRECONDITION_SCHEMA,
                    "status": "pass",
                    "run_id": "candidate-run-001122334455",
                    "candidate": {
                        "wikijump_commit": "9" * 40,
                        "wikijump_tree": identity["wikijump_tree"],
                        "ftml_sha": identity["ftml_sha"],
                    },
                    "build": {
                        "wikijump_commit": "9" * 40,
                        "wikijump_tree": identity["wikijump_tree"],
                        "ftml_sha": identity["ftml_sha"],
                        "images": {
                            service: "sha256:" + "e" * 64
                            for service in REFRESH.SERVICES
                        },
                    },
                }
            ),
            encoding="utf-8",
        )
        receipt: dict[str, object] = {
            "schema_version": 1,
            "kind": "standing-image-preparation",
            "status": "pass",
            "run_id": "candidate-run-001122334455",
            **identity,
            "promotion_precondition": {
                "path": str(proof_path),
                "sha256": hashlib.sha256(proof_path.read_bytes()).hexdigest(),
            },
            "dockerfiles": {
                service: hashlib.sha256(service.encode()).hexdigest()
                for service in REFRESH.SERVICES
            },
            "images": {
                service: {
                    "reference": "sha256:" + "e" * 64,
                    "id": "sha256:" + "e" * 64,
                    "profile": "release" if service != "framerail" else "built",
                }
                for service in REFRESH.SERVICES
            },
            "resource_disposition": {
                "owner": "standing-image-preparation",
                "expiry": expiry,
            },
        }
        return receipt, identity

    def test_standing_runtime_uses_supported_release_profiles(self) -> None:
        compose = (SCRIPT.parent / "compose.yaml").read_text(encoding="utf-8")
        self.assertIn("DEEPWELL_BUILD_PROFILE: release", compose)
        self.assertIn("WWS_BUILD_PROFILE: release", compose)
        self.assertIn(
            "DEEPWELL_RPC_TOKEN: ${DEEPWELL_RPC_TOKEN:?DEEPWELL_RPC_TOKEN is required}",
            compose,
        )

    def test_standing_runtime_labels_include_lifecycle_provenance(self) -> None:
        compose = (SCRIPT.parent / "compose.yaml").read_text(encoding="utf-8")
        self.assertIn(
            "com.rokurolize.wikijump.lifecycle: prepared-exact-merged-head",
            compose,
        )

    def test_compose_restart_is_fixed_to_app_services_without_volume_flags(
        self,
    ) -> None:
        runtime_home = Path("/srv/wikijump-standing")
        command = REFRESH.compose_command(
            runtime_home,
            "up",
            "--detach",
            "--no-deps",
            "--no-build",
            *REFRESH.SERVICES,
            override_file=Path("/src/refresh.compose.yaml"),
        )
        self.assertEqual(
            command[-7:],
            [
                "up",
                "--detach",
                "--no-deps",
                "--no-build",
                "deepwell",
                "framerail",
                "wws",
            ],
        )
        self.assertNotIn("down", command)
        self.assertNotIn("-v", command)
        self.assertNotIn("--volumes", command)
        self.assertNotIn("--remove-volumes", command)

    def test_cli_rejects_every_volume_removal_spelling(self) -> None:
        for forbidden in ("-v", "--volumes", "--remove-volumes"):
            with self.subTest(forbidden=forbidden):
                result = subprocess.run(
                    (
                        sys.executable,
                        str(SCRIPT),
                        "--prepared-receipt",
                        "/tmp/absent",
                        forbidden,
                    ),
                    text=True,
                    capture_output=True,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("unrecognized arguments", result.stderr)

    def test_activation_has_no_build_path_and_uses_prepared_references(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertNotIn('docker", "build', source)
        self.assertIn("--prepared-receipt", source)
        self.assertIn("--no-build", source)
        prepare = (SCRIPT.parent / "prepare.py").read_text(encoding="utf-8")
        self.assertIn('"install" / "prod" / service / "Dockerfile"', prepare)

    def test_local_development_still_uses_watch_mode(self) -> None:
        deepwell_start = (
            SCRIPT.parents[1] / "local/deepwell/deepwell-start"
        ).read_text(encoding="utf-8")
        wws_start = (SCRIPT.parents[1] / "local/wws/wws-start").read_text(
            encoding="utf-8"
        )
        framerail_start = (
            SCRIPT.parents[1] / "local/framerail/framerail-start"
        ).read_text(encoding="utf-8")
        self.assertIn("cargo watch", deepwell_start)
        self.assertIn("cargo watch", wws_start)
        self.assertIn("pnpm dev", framerail_start)

    def test_refresh_checks_saved_page_generator_freshness_before_canary(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertIn("stale_pages_remaining", source)
        self.assertIn("saved_page_render_freshness", source)
        call = source.rindex("standing_stale_saved_pages(")
        self.assertLess(call, source.index('body = command('))

    def test_refresh_requires_zero_stale_saved_pages(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        self.assertIn("stale_pages_remaining", source)
        self.assertIn("standing contains stale saved-page renders", source)

    def test_render_ahead_skips_when_saved_pages_are_current(self) -> None:
        with mock.patch.object(
            REFRESH, "standing_stale_saved_pages", return_value=(0, "generator")
        ), mock.patch.object(REFRESH.subprocess, "run") as run:
            result = REFRESH.prepare_saved_page_render_ahead(
                Path("/runtime"), Path("/source"), Path("/prepared.json"),
                "sha256:" + "a" * 64,
            )
        self.assertEqual(result["status"], "not-needed")
        run.assert_not_called()

    def test_render_ahead_runs_before_activation_and_binds_its_proof(self) -> None:
        image = "sha256:" + "a" * 64
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            source = root / "source"
            source.mkdir()
            runtime = root / "runtime"
            runtime.mkdir()
            prepared = root / "prepared.json"

            def render_ahead(command: list[str], *, cwd: Path, check: bool) -> None:
                self.assertTrue(check)
                self.assertEqual(cwd, source)
                self.assertEqual(command[command.index("--prepared-receipt") + 1], str(prepared))
                output = Path(command[command.index("--output-dir") + 1])
                output.mkdir()
                (output / "render-ahead-receipt.json").write_text(json.dumps({
                    "schema": "wikijump.render_ahead_receipt.v1",
                    "status": "pass",
                    "render_image_id": image,
                    "compiled_generator": "ftml; deepwell-render/v24",
                    "stale_without_valid_ahead": 0,
                    "unrendered_page_ids": [],
                }), encoding="utf-8")

            with mock.patch.object(
                REFRESH, "standing_stale_saved_pages",
                return_value=(24573, "ftml; deepwell-render/v24"),
            ), mock.patch.object(REFRESH.subprocess, "run", side_effect=render_ahead):
                result = REFRESH.prepare_saved_page_render_ahead(
                    runtime, source, prepared, image
                )
            self.assertEqual(result["status"], "pass")
            self.assertEqual(result["stale_pages_before"], 24573)
            receipt = result["receipt"]
            self.assertEqual(receipt["sha256"], REFRESH.file_sha256(Path(receipt["path"])))

    def test_render_ahead_rejects_mismatched_candidate_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)

            def forged_receipt(command: list[str], *, cwd: Path, check: bool) -> None:
                output = Path(command[command.index("--output-dir") + 1])
                output.mkdir()
                (output / "render-ahead-receipt.json").write_text(json.dumps({
                    "schema": "wikijump.render_ahead_receipt.v1",
                    "status": "pass",
                    "render_image_id": "sha256:" + "b" * 64,
                    "compiled_generator": "old generator",
                    "stale_without_valid_ahead": 0,
                    "unrendered_page_ids": [],
                }), encoding="utf-8")

            with mock.patch.object(
                REFRESH, "standing_stale_saved_pages", return_value=(1, "new generator")
            ), mock.patch.object(REFRESH.subprocess, "run", side_effect=forged_receipt):
                with self.assertRaisesRegex(RuntimeError, "did not prove a fresh candidate"):
                    REFRESH.prepare_saved_page_render_ahead(
                        root, root, root / "prepared.json", "sha256:" + "a" * 64
                    )

    def test_render_ahead_failure_keeps_evidence_path_and_prevents_activation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            with mock.patch.object(
                REFRESH, "standing_stale_saved_pages", return_value=(10, "new generator")
            ), mock.patch.object(
                REFRESH.subprocess, "run",
                side_effect=subprocess.CalledProcessError(1, ["render_ahead.py"]),
            ):
                with self.assertRaisesRegex(RuntimeError, "inspect retained evidence in"):
                    REFRESH.prepare_saved_page_render_ahead(
                        root, root, root / "prepared.json", "sha256:" + "a" * 64
                    )

    def test_render_ahead_occurs_before_parking_containers(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        main = source[source.index("def main() -> int:"):]
        self.assertLess(
            main.index("render_ahead = prepare_saved_page_render_ahead("),
            main.index("parked = park_containers("),
        )

    def test_expected_compiled_generator_is_source_derived(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            cargo = root / "deepwell" / "Cargo.lock"
            cargo.parent.mkdir(parents=True)
            cargo.write_text(
                '[[package]]\nname = "ftml"\nversion = "1.2.3+test"\n',
                encoding="utf-8",
            )
            generator = root / "deepwell" / "src" / "services" / "render" / "generator.rs"
            generator.parent.mkdir(parents=True)
            generator.write_text(
                "pub(crate) const DEEPWELL_RENDERER_EPOCH: u32 = 17;\n",
                encoding="utf-8",
            )
            self.assertEqual(
                REFRESH.expected_compiled_generator(root),
                "ftml v1.2.3+test; deepwell-render/v17",
            )

    def test_standing_deepwell_migrations_are_explicit_in_the_image(self) -> None:
        dockerfile = (SCRIPT.parents[1] / "prod/deepwell/Dockerfile").read_text(
            encoding="utf-8"
        )
        start = (SCRIPT.parents[1] / "prod/deepwell/deepwell-start").read_text(
            encoding="utf-8"
        )
        self.assertIn("cargo install sqlx-cli --version 0.9.0 --locked", dockerfile)
        self.assertIn("COPY ./deepwell/migrations /opt/deepwell/migrations", dockerfile)
        self.assertIn("sqlx migrate run --source /opt/deepwell/migrations", start)
        self.assertIn("exec /usr/local/bin/deepwell", start)

    def test_standing_deepwell_uses_image_locales_without_a_host_bind(self) -> None:
        compose = (SCRIPT.parent / "compose.yaml").read_text(encoding="utf-8")
        dockerfile = (SCRIPT.parents[1] / "prod/deepwell/Dockerfile").read_text(
            encoding="utf-8"
        )
        renderer_source = (SCRIPT.parent / "render.py").read_text(encoding="utf-8")
        refresher = SCRIPT.read_text(encoding="utf-8")

        self.assertIn("COPY ./locales/fluent /opt/locales/fluent", dockerfile)
        self.assertNotIn("target: /opt/locales", compose)
        self.assertNotIn("STANDING_LOCALES_SOURCE", compose)
        self.assertNotIn("STANDING_LOCALES_SOURCE", renderer_source)
        self.assertNotIn("STANDING_LOCALES_SOURCE", refresher)

    def test_prepared_receipt_rejects_mutable_or_wrong_image_reference(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            receipt, identity = self.prepared_receipt_fixture(root)
            images = receipt["images"]
            assert isinstance(images, dict)
            deepwell = images["deepwell"]
            assert isinstance(deepwell, dict)
            deepwell["reference"] = "sha256:" + "f" * 64
            path = root / "prepared.json"
            path.write_text(json.dumps(receipt), encoding="utf-8")
            with merged_candidate(identity):
                with self.assertRaisesRegex(ValueError, "immutable image ID"):
                    REFRESH.load_prepared_receipt(path, root, identity)

    def test_prepared_receipt_accepts_candidate_proof_from_the_parent_of_the_merged_tree(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            receipt, identity = self.prepared_receipt_fixture(root)
            proof_ref = receipt["promotion_precondition"]
            assert isinstance(proof_ref, dict)
            proof_path = Path(str(proof_ref["path"]))
            proof = json.loads(proof_path.read_text(encoding="utf-8"))
            candidate_commit = "9" * 40
            for section in ("candidate", "build"):
                proof[section]["wikijump_commit"] = candidate_commit
            proof_path.write_text(json.dumps(proof), encoding="utf-8")
            proof_ref["sha256"] = hashlib.sha256(proof_path.read_bytes()).hexdigest()
            path = root / "prepared.json"
            path.write_text(json.dumps(receipt), encoding="utf-8")
            with merged_candidate(identity, candidate_commit):
                loaded, _, loaded_proof = REFRESH.load_prepared_receipt(
                    path, root, identity
                )
                self.assertEqual(loaded["run_id"], proof["run_id"])
                self.assertEqual(
                    loaded_proof["candidate"]["wikijump_commit"], candidate_commit
                )

    def test_prepared_receipt_binds_the_promotion_precondition_run_id(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            receipt, identity = self.prepared_receipt_fixture(root)
            receipt["run_id"] = "candidate-run-other"
            path = root / "prepared.json"
            path.write_text(json.dumps(receipt), encoding="utf-8")
            with merged_candidate(identity):
                with self.assertRaisesRegex(ValueError, "run ID"):
                    REFRESH.load_prepared_receipt(path, root, identity)

    def test_prepared_receipt_rejects_unsafe_resource_expiry(self) -> None:
        invalid_expiries: tuple[object, ...] = (
            "2026-09-05T12:34:56+00:00\nSTANDING_DEEPWELL_IMAGE=evil",
            "2026-09-05T12:34:56+00:00\rSTANDING_NETWORK_NAME=evil",
            "2026-09-05T12:34:56+00:00\x00suffix",
            "2026-09-05T12:34:56+00:00\tsuffix",
            " 2026-09-05T12:34:56+00:00",
            "2026-09-05T12:34:56+00:00 ",
            "2026-09-05T12:34:56",
            "not-a-timestamp",
            None,
            7,
        )
        for expiry in invalid_expiries:
            with (
                self.subTest(expiry=expiry),
                tempfile.TemporaryDirectory() as temporary_dir,
            ):
                root = Path(temporary_dir)
                receipt, identity = self.prepared_receipt_fixture(root, expiry=expiry)
                path = root / "prepared.json"
                path.write_text(json.dumps(receipt), encoding="utf-8")
                with merged_candidate(identity):
                    with self.assertRaisesRegex(ValueError, "resource expiry"):
                        REFRESH.load_prepared_receipt(path, root, identity)

    def test_prepared_receipt_accepts_canonical_resource_expiry(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            receipt, identity = self.prepared_receipt_fixture(root)
            path = root / "prepared.json"
            path.write_text(json.dumps(receipt), encoding="utf-8")
            with merged_candidate(identity):
                loaded, _, _ = REFRESH.load_prepared_receipt(path, root, identity)
            self.assertEqual(
                loaded["resource_disposition"], receipt["resource_disposition"]
            )

    def test_standing_framerail_keeps_csrf_origin_checks_enabled(self) -> None:
        compose = (SCRIPT.parent / "compose.yaml").read_text(encoding="utf-8")
        dockerfile = (SCRIPT.parents[1] / "prod/framerail/Dockerfile").read_text(
            encoding="utf-8"
        )
        self.assertIn('FRAMERAIL_CSRF_CHECK_ORIGIN: "true"', compose)
        self.assertIn("ARG FRAMERAIL_CSRF_CHECK_ORIGIN=true", dockerfile)

    def test_environment_rewrite_is_atomic_and_preserves_unrelated_values(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            path = Path(temporary_dir) / ".env"
            path.write_text("KEEP=value\nSTANDING_WIKIJUMP_SHA=old\n", encoding="utf-8")
            values = REFRESH.read_environment(path)
            values["STANDING_WIKIJUMP_SHA"] = "new"
            REFRESH.write_environment(path, values)
            self.assertEqual(
                path.read_text(encoding="utf-8"),
                "KEEP=value\nSTANDING_WIKIJUMP_SHA=new\n",
            )
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_environment_rewrite_rejects_multiline_values_without_replacing_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            root = Path(temporary_dir)
            path = root / ".env"
            original = "KEEP=value\nSTANDING_DEEPWELL_IMAGE=verified\n"
            path.write_text(original, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "single-line environment value"):
                REFRESH.write_environment(
                    path,
                    {
                        "KEEP": "value",
                        "STANDING_RESOURCE_EXPIRY": (
                            "2026-09-05T12:34:56+00:00\nSTANDING_DEEPWELL_IMAGE=evil"
                        ),
                    },
                )
            self.assertEqual(path.read_text(encoding="utf-8"), original)
            self.assertEqual(list(root.glob("..env.*")), [])

    def test_runtime_differential_identity_binds_lock_image_and_compose_config(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            source_root = Path(temporary_dir)
            lock_path = source_root / "deepwell" / "Cargo.lock"
            lock_path.parent.mkdir()
            lock_path.write_bytes(b"locked dependencies\n")
            source_identity = {
                "wikijump_sha": "a" * 40,
                "ftml_sha": "b" * 40,
            }
            images = {"deepwell": {"id": f"sha256:{'c' * 64}"}}
            config = "services:\n  deepwell:\n    image: candidate\n"
            identity = REFRESH.runtime_differential_identity(
                source_root, source_identity, images, config
            )
            self.assertEqual(
                identity,
                {
                    "schema": "wikijump_syntax_differential.wikijump_runtime_identity.v1",
                    **source_identity,
                    "dependency_lock_sha256": hashlib.sha256(
                        b"locked dependencies\n"
                    ).hexdigest(),
                    "executable_sha256": "c" * 64,
                    "runtime_config_sha256": hashlib.sha256(
                        config.encode("utf-8")
                    ).hexdigest(),
                },
            )

    def test_runtime_differential_identity_rejects_non_digest_image_id(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_dir:
            source_root = Path(temporary_dir)
            lock_path = source_root / "deepwell" / "Cargo.lock"
            lock_path.parent.mkdir()
            lock_path.write_text("", encoding="utf-8")
            with self.assertRaisesRegex(
                ValueError, "Deepwell image identity is not one SHA-256 digest"
            ):
                REFRESH.runtime_differential_identity(
                    source_root,
                    {"wikijump_sha": "a" * 40, "ftml_sha": "b" * 40},
                    {"deepwell": {"id": "candidate"}},
                    "services: {}",
                )


if __name__ == "__main__":
    unittest.main()
