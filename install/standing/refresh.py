#!/usr/bin/env python3
"""Activate already-prepared standing application images without compiling."""

from __future__ import annotations

import argparse
from datetime import UTC, datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
import tomllib
import uuid

from merge_identity import validate_candidate_merge
from volume_contract import PROTECTED_VOLUMES


SERVICES = ("deepwell", "framerail", "wws")
RUNTIME_SERVICES = (
    "deepwell",
    "framerail",
    "wws",
    "caddy",
)
PROMOTION_PRECONDITION_SCHEMA = "wikijump.standing_promotion_precondition.v1"


def default_runtime_home() -> Path:
    state_root = os.environ.get("XDG_STATE_HOME")
    if state_root:
        root = Path(state_root).expanduser()
        if not root.is_absolute():
            raise ValueError("XDG_STATE_HOME must be absolute")
    else:
        root = Path.home() / ".local" / "state"
    return root / "wikijump" / "standing"


DEFAULT_RUNTIME_HOME = default_runtime_home()
RUNTIME_DIFFERENTIAL_IDENTITY = "runtime-differential-identity.json"
CANARY_URL = "http://scp-wiki.wikijump.localhost/scp-9506"
FTML_SOURCE = re.compile(
    r'source = "git\+https://github\.com/Rokurolize/ftml[^\"]*#([0-9a-f]{40})"'
)
ENVIRONMENT_KEY = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
RESOURCE_EXPIRY = re.compile(
    r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}"
    r"(?:\.[0-9]{1,6})?\+00:00"
)
CONTAINER_ID = re.compile(r"[0-9a-f]{64}")
IMAGE_ID = re.compile(r"sha256:[0-9a-f]{64}")


def command(*args: str, cwd: Path, capture: bool = True) -> str:
    result = subprocess.run(
        args, cwd=cwd, check=True, text=True, capture_output=capture
    )
    return result.stdout.strip() if capture else ""


def repository_identity(source_root: Path) -> dict[str, str]:
    if command("git", "status", "--porcelain", cwd=source_root):
        raise ValueError("source checkout must be clean")
    head = command("git", "rev-parse", "HEAD", cwd=source_root)
    develop = command(
        "git", "rev-parse", "refs/remotes/origin/develop^{commit}", cwd=source_root
    )
    if head != develop:
        raise ValueError(
            f"source HEAD {head} is not the fetched origin/develop head {develop}"
        )
    tree = command("git", "rev-parse", "HEAD^{tree}", cwd=source_root)
    lock_contents = (source_root / "deepwell" / "Cargo.lock").read_text(
        encoding="utf-8"
    )
    if not (source_root / "locales").is_dir():
        raise ValueError("source checkout is missing the locales directory")
    matches = set(FTML_SOURCE.findall(lock_contents))
    if len(matches) != 1:
        raise ValueError("deepwell/Cargo.lock must contain exactly one FTML revision")
    lockfile = source_root / "deepwell" / "Cargo.lock"
    return {
        "wikijump_sha": head,
        "wikijump_tree": tree,
        "ftml_sha": matches.pop(),
        "dependency_lock_sha256": file_sha256(lockfile),
    }


def read_environment(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        if not separator or not key or key in values:
            raise ValueError(
                f"invalid or duplicate environment entry in {path}: {line}"
            )
        values[key] = value
    return values


def write_environment(path: Path, values: dict[str, str]) -> None:
    for key, value in values.items():
        if not isinstance(key, str) or ENVIRONMENT_KEY.fullmatch(key) is None:
            raise ValueError(f"invalid environment key: {key!r}")
        if not isinstance(value, str) or not value.isprintable():
            raise ValueError(
                f"{key} must contain one printable, single-line environment value"
            )
    contents = "".join(f"{key}={value}\n" for key, value in sorted(values.items()))
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as temporary:
        temporary.write(contents)
        temporary.flush()
        os.fsync(temporary.fileno())
        temporary_path = Path(temporary.name)
    os.chmod(temporary_path, 0o600)
    os.replace(temporary_path, path)


def rollback_environment(
    previous: dict[str, str], images: dict[str, dict[str, object]]
) -> dict[str, str]:
    restored = dict(previous)
    for service in SERVICES:
        image_id = images[service]["id"]
        if not isinstance(image_id, str) or not IMAGE_ID.fullmatch(image_id):
            raise ValueError(f"rollback image {service} has no immutable image ID")
        restored[f"STANDING_{service.upper()}_IMAGE"] = image_id
    return restored


def compose_command(
    runtime_home: Path, *args: str, override_file: Path | None = None
) -> list[str]:
    command = [
        "docker",
        "compose",
        "--project-name",
        "wikijump-standing",
        "--env-file",
        str(runtime_home / ".env"),
        "--file",
        str(runtime_home / "compose.yaml"),
    ]
    if override_file is not None:
        command.extend(("--file", str(override_file)))
    command.extend(args)
    return command


def expected_compiled_generator(source_root: Path) -> str:
    lock = tomllib.loads((source_root / "deepwell" / "Cargo.lock").read_text(encoding="utf-8"))
    packages = [
        package
        for package in lock.get("package", [])
        if isinstance(package, dict) and package.get("name") == "ftml"
    ]
    versions = {package.get("version") for package in packages}
    if len(versions) != 1 or not all(isinstance(version, str) and version for version in versions):
        raise ValueError("deepwell/Cargo.lock must contain exactly one FTML package version")
    source = (
        source_root / "deepwell" / "src" / "services" / "render" / "generator.rs"
    ).read_text(encoding="utf-8")
    epochs = set(
        re.findall(r"DEEPWELL_RENDERER_EPOCH:\s*u32\s*=\s*([0-9]+)", source)
    )
    if len(epochs) != 1:
        raise ValueError("Deepwell renderer source must contain exactly one renderer epoch")
    return f"ftml v{versions.pop()}; deepwell-render/v{epochs.pop()}"


def standing_stale_saved_pages(
    runtime_home: Path, source_root: Path
) -> tuple[int, str]:
    expected = expected_compiled_generator(source_root)
    sql_expected = expected.replace("'", "''")
    sql = (
        "SELECT count(*) FROM page p "
        "JOIN page_revision pr ON pr.revision_id=p.latest_revision_id "
        "WHERE p.deleted_at IS NULL "
        f"AND pr.compiled_generator <> '{sql_expected}';"
    )
    output = command(
        "docker",
        "exec",
        "wikijump-standing-database-1",
        "sh",
        "-lc",
        'PGPASSWORD="$POSTGRES_PASSWORD" exec psql -h 127.0.0.1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" -At -v ON_ERROR_STOP=1 -c "$1"',
        "sh",
        sql,
        cwd=runtime_home,
    )
    try:
        stale = int(output)
    except ValueError as error:
        raise RuntimeError("standing stale saved-page count is not an integer") from error
    if stale < 0:
        raise RuntimeError("standing stale saved-page count is negative")
    return stale, expected


def prepare_saved_page_render_ahead(
    runtime_home: Path,
    source_root: Path,
    prepared_receipt_path: Path,
    deepwell_image_id: str,
) -> dict[str, object]:
    """Complete the slow renderer transition before taking the site offline.

    The serving Deepwell continues to use its current compiled pages while a
    one-off new-image process writes ahead rows to a separate table. The new
    image promotes them on startup. A failed or incomplete run must prevent
    activation; the retained log directory makes failure diagnosable.
    """
    stale, expected = standing_stale_saved_pages(runtime_home, source_root)
    if stale == 0:
        return {
            "status": "not-needed",
            "expected_compiled_generator": expected,
            "stale_pages_before": 0,
        }
    if IMAGE_ID.fullmatch(deepwell_image_id) is None:
        raise ValueError("render-ahead requires an immutable Deepwell image ID")

    evidence_root = runtime_home / "render-ahead-evidence"
    evidence_root.mkdir(mode=0o700, parents=True, exist_ok=True)
    output_dir = evidence_root / f"{source_root.name}-{uuid.uuid4().hex}"
    script = source_root / "install" / "standing" / "render_ahead.py"
    try:
        subprocess.run(
            [
                sys.executable,
                str(script),
                "--prepared-receipt",
                str(prepared_receipt_path),
                "--output-dir",
                str(output_dir),
            ],
            cwd=source_root,
            check=True,
        )
    except subprocess.CalledProcessError as error:
        raise RuntimeError(
            f"render-ahead failed; inspect retained evidence in {output_dir}"
        ) from error
    receipt_path = output_dir / "render-ahead-receipt.json"
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    if (
        receipt.get("schema") != "wikijump.render_ahead_receipt.v1"
        or receipt.get("status") != "pass"
        or receipt.get("render_image_id") != deepwell_image_id
        or receipt.get("compiled_generator") != expected
        or receipt.get("stale_without_valid_ahead") != 0
        or receipt.get("unrendered_page_ids")
    ):
        raise RuntimeError(f"render-ahead did not prove a fresh candidate: {receipt_path}")
    return {
        "status": "pass",
        "expected_compiled_generator": expected,
        "stale_pages_before": stale,
        "receipt": {"path": str(receipt_path), "sha256": file_sha256(receipt_path)},
    }


def assert_standing_unchanged_during_render_ahead(
    before: dict[str, dict[str, object]],
    after: dict[str, dict[str, object]],
    previous_port_443_owner: dict[str, object],
) -> None:
    """Refuse to park containers when another operator refreshed standing."""
    for service in RUNTIME_SERVICES:
        if (
            before[service]["container_id"] != after[service]["container_id"]
            or before[service]["image_id"] != after[service]["image_id"]
        ):
            raise RuntimeError(
                f"standing {service} changed during render-ahead; retry with a fresh precondition"
            )
    if port_443_owner(after)["container_id"] != previous_port_443_owner["container_id"]:
        raise RuntimeError("standing port-443 owner changed during render-ahead")


def wait_for_health(
    runtime_home: Path, override_file: Path, timeout_seconds: int
) -> dict[str, str]:
    deadline = time.monotonic() + timeout_seconds
    final: dict[str, str] = {}
    while time.monotonic() < deadline:
        final = {}
        for service in SERVICES:
            container_id = command(
                *compose_command(
                    runtime_home,
                    "ps",
                    "--all",
                    "--quiet",
                    service,
                    override_file=override_file,
                ),
                cwd=runtime_home,
            )
            if not container_id:
                final[service] = "missing"
                continue
            final[service] = command(
                "docker",
                "inspect",
                "--format",
                "{{if .State.Health}}{{.State.Health.Status}}{{else}}{{.State.Status}}{{end}}",
                container_id,
                cwd=runtime_home,
            )
        if all(status == "healthy" for status in final.values()):
            return final
        if any(status in {"dead", "exited"} for status in final.values()):
            raise RuntimeError(
                f"standing service stopped before becoming healthy: {final}"
            )
        time.sleep(5)
    raise TimeoutError(
        f"standing services did not become healthy within {timeout_seconds}s: {final}"
    )


def image_identity(reference: str, cwd: Path) -> dict[str, object]:
    raw = command(
        "docker", "image", "inspect", reference, "--format", "{{json .}}", cwd=cwd
    )
    image = json.loads(raw)
    image_id = image.get("Id")
    if not isinstance(image_id, str) or not IMAGE_ID.fullmatch(image_id):
        raise ValueError(f"image {reference} has no immutable SHA-256 image ID")
    return {
        "reference": reference,
        "id": image_id,
        "repo_digests": sorted(image.get("RepoDigests") or []),
        "labels": image.get("Config", {}).get("Labels") or {},
    }


def normalize_container_identity(value: object) -> dict[str, object]:
    if not isinstance(value, dict):
        raise ValueError("Docker container inspection is not an object")
    container_id = value.get("Id")
    image_id = value.get("Image")
    name = value.get("Name")
    state = value.get("State")
    if not isinstance(container_id, str) or not CONTAINER_ID.fullmatch(container_id):
        raise ValueError("Docker container has no immutable container ID")
    if not isinstance(image_id, str) or not IMAGE_ID.fullmatch(image_id):
        raise ValueError("Docker container has no immutable image ID")
    if not isinstance(name, str) or not name.startswith("/") or name == "/":
        raise ValueError("Docker container has no canonical name")
    status = state.get("Status") if isinstance(state, dict) else None
    if not isinstance(status, str) or not status:
        raise ValueError("Docker container has no state")
    network = value.get("NetworkSettings")
    ports = network.get("Ports", {}) if isinstance(network, dict) else {}
    if not isinstance(ports, dict):
        raise ValueError("Docker container ports are invalid")
    published_ports: list[dict[str, str]] = []
    for container_port, bindings in ports.items():
        if bindings is None:
            continue
        if not isinstance(container_port, str) or not isinstance(bindings, list):
            raise ValueError("Docker container ports are invalid")
        for binding in bindings:
            if not isinstance(binding, dict):
                raise ValueError("Docker container port binding is invalid")
            host_ip = binding.get("HostIp")
            host_port = binding.get("HostPort")
            if (
                not isinstance(host_ip, str)
                or not isinstance(host_port, str)
                or not host_port
            ):
                raise ValueError("Docker container port binding is invalid")
            published_ports.append(
                {
                    "container_port": container_port,
                    "host_ip": host_ip,
                    "host_port": host_port,
                }
            )
    published_ports.sort(
        key=lambda port: (port["host_port"], port["host_ip"], port["container_port"])
    )
    return {
        "container_id": container_id,
        "name": name[1:],
        "image_id": image_id,
        "status": status,
        "running": status == "running",
        "published_ports": published_ports,
    }


def container_identity(reference: str, cwd: Path) -> dict[str, object]:
    raw = command("docker", "inspect", "--format", "{{json .}}", reference, cwd=cwd)
    return normalize_container_identity(json.loads(raw))


def runtime_containers(runtime_home: Path) -> dict[str, dict[str, object]]:
    containers: dict[str, dict[str, object]] = {}
    for service in RUNTIME_SERVICES:
        identity = container_identity(f"wikijump-standing-{service}-1", runtime_home)
        identity["service"] = service
        containers[service] = identity
    return containers


def runtime_images(
    containers: dict[str, dict[str, object]], cwd: Path
) -> dict[str, dict[str, object]]:
    return {
        service: image_identity(container["image_id"], cwd)
        for service, container in containers.items()
    }


def port_443_owner(containers: dict[str, dict[str, object]]) -> dict[str, object]:
    owners = []
    for service, container in containers.items():
        bindings = [
            binding
            for binding in container["published_ports"]
            if binding["host_port"] == "443"
        ]
        if bindings:
            owners.append(
                {
                    "service": service,
                    "container_id": container["container_id"],
                    "name": container["name"],
                    "image_id": container["image_id"],
                    "bindings": bindings,
                }
            )
    if len(owners) != 1:
        raise ValueError(f"expected exactly one port-443 owner, found {len(owners)}")
    return owners[0]


def resource_exists(kind: str, reference: str, cwd: Path) -> bool:
    try:
        if kind == "container":
            command("docker", "inspect", reference, cwd=cwd)
        else:
            command("docker", "image", "inspect", reference, cwd=cwd)
    except subprocess.CalledProcessError:
        return False
    return True


def parked_container_name(run_id: str, service: str) -> str:
    if not isinstance(run_id, str) or not run_id:
        raise ValueError("promotion run ID is required for container parking")
    return (
        f"wikijump-standing-rollback-"
        f"{hashlib.sha256(run_id.encode('utf-8')).hexdigest()[:24]}-{service}"
    )


def restore_parked_containers(
    parked: dict[str, dict[str, object]], runtime_home: Path
) -> dict[str, dict[str, object]]:
    restored = {}
    for service, entry in parked.items():
        current = container_identity(entry["parked_name"], runtime_home)
        if (
            current["container_id"] != entry["container"]["container_id"]
            or current["image_id"] != entry["container"]["image_id"]
        ):
            raise RuntimeError(
                f"parked container identity changed before restore for {service}"
            )
        command(
            "docker",
            "rename",
            entry["parked_name"],
            entry["original_name"],
            cwd=runtime_home,
        )
        if entry["was_running"]:
            command(
                "docker",
                "start",
                entry["original_name"],
                cwd=runtime_home,
            )
        after = container_identity(entry["original_name"], runtime_home)
        if (
            after["container_id"] != entry["container"]["container_id"]
            or after["image_id"] != entry["container"]["image_id"]
            or after["running"] != entry["was_running"]
        ):
            raise RuntimeError(
                f"rollback did not restore container identity for {service}"
            )
        restored[service] = after
    return restored


def park_containers(
    previous: dict[str, dict[str, object]], runtime_home: Path, run_id: str
) -> dict[str, dict[str, object]]:
    parked: dict[str, dict[str, object]] = {}
    try:
        for service in SERVICES:
            old = previous[service]
            parked_name = parked_container_name(run_id, service)
            if resource_exists("container", parked_name, runtime_home):
                raise RuntimeError(f"rollback container already exists: {parked_name}")
            command(
                "docker",
                "rename",
                old["container_id"],
                parked_name,
                cwd=runtime_home,
            )
            renamed = container_identity(parked_name, runtime_home)
            if renamed["container_id"] != old["container_id"]:
                raise RuntimeError(f"parked container identity changed for {service}")
            parked[service] = {
                "service": service,
                "original_name": old["name"],
                "parked_name": parked_name,
                "was_running": old["running"],
                "container": renamed,
            }
            if old["running"]:
                command("docker", "stop", parked_name, cwd=runtime_home)
            parked[service]["container"] = container_identity(parked_name, runtime_home)
            if parked[service]["container"]["running"]:
                raise RuntimeError(f"parked container is still running for {service}")
    except Exception:
        try:
            restore_parked_containers(parked, runtime_home)
        except Exception as restore_error:
            raise RuntimeError(
                "failed to restore partially parked containers"
            ) from restore_error
        raise
    return parked


def remove_candidate_resources(
    candidate: dict[str, dict[str, object]] | None,
    candidate_images: dict[str, dict[str, object]],
    rollback_images: dict[str, dict[str, object]],
    runtime_home: Path,
) -> dict[str, object]:
    if candidate is None or any(service not in candidate for service in SERVICES):
        raise RuntimeError("candidate container inventory is unavailable")
    removed_containers = []
    for service in SERVICES:
        container_id = candidate[service]["container_id"]
        command("docker", "rm", "--force", container_id, cwd=runtime_home)
        if resource_exists("container", container_id, runtime_home):
            raise RuntimeError(
                f"candidate container remains after cleanup for {service}"
            )
        removed_containers.append({"service": service, "container_id": container_id})
    rollback_ids = {image["id"] for image in rollback_images.values()}
    removed_images = []
    retained_images = []
    processed_images = set()
    for service in SERVICES:
        image_id = candidate_images[service]["id"]
        if image_id in processed_images:
            continue
        processed_images.add(image_id)
        if image_id in rollback_ids:
            retained_images.append(
                {
                    "service": service,
                    "image_id": image_id,
                    "disposition": "rollback-image",
                }
            )
            continue
        command("docker", "image", "rm", image_id, cwd=runtime_home)
        if resource_exists("image", image_id, runtime_home):
            raise RuntimeError(f"candidate image remains after cleanup for {service}")
        removed_images.append({"service": service, "image_id": image_id})
    return {
        "status": "pass",
        "containers_removed": removed_containers,
        "images_removed": removed_images,
        "images_retained_as_rollback": retained_images,
    }


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepared_resource_expiry(receipt: dict[str, object]) -> str:
    disposition = receipt.get("resource_disposition")
    if not isinstance(disposition, dict):
        raise ValueError("prepared receipt resource expiry is invalid")
    expiry = disposition.get("expiry")
    if not isinstance(expiry, str) or RESOURCE_EXPIRY.fullmatch(expiry) is None:
        raise ValueError("prepared receipt resource expiry is invalid")
    try:
        datetime.fromisoformat(expiry)
    except ValueError as error:
        raise ValueError("prepared receipt resource expiry is invalid") from error
    return expiry


def load_prepared_receipt(
    path: Path, source_root: Path, identity: dict[str, str]
) -> tuple[dict[str, object], str, dict[str, object]]:
    receipt = json.loads(path.read_text(encoding="utf-8"))
    if (
        receipt.get("schema_version") != 1
        or receipt.get("kind") != "standing-image-preparation"
    ):
        raise ValueError("prepared receipt is not a standing image preparation receipt")
    if receipt.get("status") != "pass":
        raise ValueError("prepared receipt is not successful")
    proof_ref = receipt.get("promotion_precondition")
    if (
        not isinstance(proof_ref, dict)
        or not isinstance(proof_ref.get("path"), str)
        or not isinstance(proof_ref.get("sha256"), str)
    ):
        raise ValueError("prepared receipt has no promotion precondition")
    proof_path = Path(proof_ref["path"])
    if (
        not proof_path.is_file()
        or proof_path.is_symlink()
        or file_sha256(proof_path) != proof_ref["sha256"]
    ):
        raise ValueError("prepared receipt promotion precondition is stale")
    proof = json.loads(proof_path.read_text(encoding="utf-8"))
    if (
        proof.get("schema") != PROMOTION_PRECONDITION_SCHEMA
        or proof.get("status") != "pass"
        or not isinstance(proof.get("run_id"), str)
    ):
        raise ValueError(
            "prepared receipt promotion precondition is not a passing canonical receipt"
        )
    if receipt.get("run_id") != proof.get("run_id"):
        raise ValueError(
            "prepared receipt run ID does not match its promotion precondition"
        )
    validate_candidate_merge(
        source_root,
        identity,
        proof.get("candidate"),
        proof.get("build"),
        command,
    )
    for key in ("wikijump_sha", "wikijump_tree", "ftml_sha", "dependency_lock_sha256"):
        if receipt.get(key) != identity[key]:
            raise ValueError(
                f"prepared receipt {key} does not match the source checkout"
            )
    images = receipt.get("images")
    if not isinstance(images, dict) or set(images) != set(SERVICES):
        raise ValueError(
            "prepared receipt must contain exactly the three application images"
        )
    profiles = {"deepwell": "release", "framerail": "built", "wws": "release"}
    dockerfiles = receipt.get("dockerfiles")
    sealed_images = proof.get("build", {}).get("images")
    for service in SERVICES:
        image = images.get(service)
        if not isinstance(image, dict):
            raise ValueError(f"prepared receipt image {service} is invalid")
        reference = image.get("reference")
        image_id = image.get("id")
        if not isinstance(image_id, str) or not re.fullmatch(
            r"sha256:[0-9a-f]{64}", image_id
        ):
            raise ValueError(
                f"prepared image {service} is not bound to an image digest"
            )
        if reference != image_id:
            raise ValueError(
                f"prepared image {service} does not use its immutable image ID"
            )
        if not isinstance(sealed_images, dict) or sealed_images.get(service) != image_id:
            raise ValueError(
                f"prepared image {service} does not match the sealed candidate build"
            )
        if image.get("profile") != profiles[service]:
            raise ValueError(
                f"prepared image {service} profile is not {profiles[service]}"
            )
        dockerfile = source_root / "install" / "prod" / service / "Dockerfile"
        if not isinstance(dockerfiles, dict) or dockerfiles.get(service) != file_sha256(
            dockerfile
        ):
            raise ValueError(f"prepared image {service} Dockerfile identity is stale")
    prepared_resource_expiry(receipt)
    return receipt, file_sha256(path), proof


def runtime_differential_identity(
    source_root: Path,
    source_identity: dict[str, str],
    images: dict[str, dict[str, object]],
    runtime_config: str,
) -> dict[str, str]:
    deepwell_image_id = images["deepwell"]["id"]
    if not isinstance(deepwell_image_id, str) or not re.fullmatch(
        r"sha256:[0-9a-f]{64}", deepwell_image_id
    ):
        raise ValueError("Deepwell image identity is not one SHA-256 digest")
    return {
        "schema": "wikijump_syntax_differential.wikijump_runtime_identity.v1",
        "wikijump_sha": source_identity["wikijump_sha"],
        "ftml_sha": source_identity["ftml_sha"],
        "dependency_lock_sha256": file_sha256(source_root / "deepwell" / "Cargo.lock"),
        "executable_sha256": deepwell_image_id.removeprefix("sha256:"),
        "runtime_config_sha256": hashlib.sha256(
            runtime_config.encode("utf-8")
        ).hexdigest(),
    }


def atomic_json(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as temporary:
        json.dump(value, temporary, indent=2, sort_keys=True)
        temporary.write("\n")
        temporary.flush()
        os.fsync(temporary.fileno())
        temporary_path = Path(temporary.name)
    os.chmod(temporary_path, 0o600)
    os.replace(temporary_path, path)


def atomic_json_no_replace(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as temporary:
        json.dump(value, temporary, indent=2, sort_keys=True)
        temporary.write("\n")
        temporary.flush()
        os.fsync(temporary.fileno())
        temporary_path = Path(temporary.name)
    os.chmod(temporary_path, 0o600)
    try:
        os.link(temporary_path, path)
        directory_fd = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory_fd)
        finally:
            os.close(directory_fd)
    finally:
        temporary_path.unlink(missing_ok=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source-root", type=Path, default=Path(__file__).resolve().parents[2]
    )
    parser.add_argument("--runtime-home", type=Path, default=DEFAULT_RUNTIME_HOME)
    parser.add_argument("--prepared-receipt", type=Path, required=True)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--health-timeout-seconds", type=int, default=1800)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    source_root = args.source_root.resolve()
    runtime_home = args.runtime_home.resolve()
    requested_receipt_path = args.receipt.resolve() if args.receipt else None
    override_file = source_root / "install" / "standing" / "refresh.compose.yaml"
    if args.health_timeout_seconds <= 0:
        raise ValueError("--health-timeout-seconds must be positive")
    for required in (
        runtime_home / ".env",
        runtime_home / "compose.yaml",
        override_file,
    ):
        if not required.is_file():
            raise ValueError(f"required standing runtime file is missing: {required}")

    started_at = datetime.now(UTC)
    activation_started = time.monotonic()
    identity = repository_identity(source_root)
    receipt_path = (
        requested_receipt_path
        or runtime_home / f"refresh-receipt-{identity['wikijump_sha']}.json"
    )
    failure_receipt_path = receipt_path.with_name(
        f"{receipt_path.stem}-failure{receipt_path.suffix}"
    )
    if (
        receipt_path.exists()
        or receipt_path.is_symlink()
        or failure_receipt_path.exists()
        or failure_receipt_path.is_symlink()
    ):
        raise ValueError(
            "promotion receipt path already exists; use a fresh receipt path"
        )
    prepared_receipt_path = args.prepared_receipt.resolve()
    prepared_receipt, prepared_receipt_sha256, promotion_precondition = load_prepared_receipt(
        prepared_receipt_path, source_root, identity
    )
    environment = read_environment(runtime_home / ".env")
    previous_environment = dict(environment)
    if environment.get("STANDING_PROJECT_NAME") != "wikijump-standing":
        raise ValueError("runtime home is not the wikijump-standing project")
    rpc_token = environment.get("DEEPWELL_RPC_TOKEN", "")
    if not re.fullmatch(r"[0-9a-f]{64}", rpc_token):
        raise ValueError(
            "standing runtime environment must contain DEEPWELL_RPC_TOKEN as 64 lowercase hexadecimal characters"
        )
    network_name = environment.get("STANDING_NETWORK_NAME")
    if not network_name:
        raise ValueError("runtime environment does not name its standing network")
    command("docker", "network", "inspect", "--", network_name, cwd=runtime_home)
    previous_runtime = runtime_containers(runtime_home)
    previous_runtime_images = runtime_images(previous_runtime, runtime_home)
    previous_port_443_owner = port_443_owner(previous_runtime)
    rollback_images = {
        service: previous_runtime_images[service] for service in SERVICES
    }

    prepared_images = prepared_receipt["images"]
    if not isinstance(prepared_images, dict):
        raise ValueError("prepared receipt images are invalid")
    images = {
        service: image_identity(prepared_images[service]["reference"], runtime_home)
        for service in SERVICES
    }
    for service in SERVICES:
        expected = prepared_images[service]["id"]
        if images[service]["id"] != expected:
            raise RuntimeError(
                f"prepared image {service} changed: expected {expected}, got {images[service]['id']}"
            )
        labels = images[service].get("labels")
        candidate = promotion_precondition["candidate"]
        if (
            not isinstance(labels, dict)
            or labels.get("com.rokurolize.wikijump.sha")
            != candidate["wikijump_commit"]
            or labels.get("com.rokurolize.wikijump.tree")
            != candidate["wikijump_tree"]
            or labels.get("com.rokurolize.wikijump.ftml_sha")
            != candidate["ftml_sha"]
        ):
            raise RuntimeError(
                f"prepared image {service} is not labelled for the sealed candidate source"
            )

    expiry = prepared_resource_expiry(prepared_receipt)
    activation_verified_at = time.monotonic()
    differential_identity_path = runtime_home / RUNTIME_DIFFERENTIAL_IDENTITY
    previous_differential = None
    if differential_identity_path.exists():
        previous_differential = json.loads(
            differential_identity_path.read_text(encoding="utf-8")
        )
    parked: dict[str, dict[str, object]] = {}
    candidate_runtime = None
    candidate_runtime_images = None
    candidate_port_443_owner = None
    differential_identity_published = False
    switched = False
    render_ahead = None
    try:
        render_ahead = prepare_saved_page_render_ahead(
            runtime_home, source_root, prepared_receipt_path, images["deepwell"]["id"]
        )
        if repository_identity(source_root) != identity:
            raise RuntimeError("source identity changed during render-ahead")
        assert_standing_unchanged_during_render_ahead(
            previous_runtime,
            runtime_containers(runtime_home),
            previous_port_443_owner,
        )
        parked = park_containers(
            previous_runtime, runtime_home, prepared_receipt_sha256
        )
        environment.update(
            {
                "STANDING_DEEPWELL_IMAGE": prepared_images["deepwell"]["reference"],
                "STANDING_FRAMERAIL_IMAGE": prepared_images["framerail"]["reference"],
                "STANDING_WWS_IMAGE": prepared_images["wws"]["reference"],
                "STANDING_WIKIJUMP_SHA": identity["wikijump_sha"],
                "STANDING_FTML_SHA": identity["ftml_sha"],
                "STANDING_RESOURCE_EXPIRY": expiry,
            }
        )
        write_environment(runtime_home / ".env", environment)
        switched = True
        compose_started = time.monotonic()
        command(
            *compose_command(
                runtime_home,
                "up",
                "--detach",
                "--no-deps",
                "--no-build",
                *SERVICES,
                override_file=override_file,
            ),
            cwd=runtime_home,
        )
        health_started = time.monotonic()
        health = wait_for_health(
            runtime_home, override_file, args.health_timeout_seconds
        )
        health_completed = time.monotonic()
        stale_pages_remaining, expected_generator = standing_stale_saved_pages(
            runtime_home, source_root
        )
        if stale_pages_remaining != 0:
            raise RuntimeError(
                f"standing contains stale saved-page renders: {stale_pages_remaining}"
            )
        saved_page_render_freshness = {
            "status": "pass",
            "expected_compiled_generator": expected_generator,
            "stale_pages_remaining": stale_pages_remaining,
        }
        canary_started = time.monotonic()
        body = command(
            "curl",
            "--silent",
            "--show-error",
            "--fail",
            "--location",
            "--insecure",
            "--max-time",
            "30",
            CANARY_URL,
            cwd=runtime_home,
        )
        if "scp-9506" not in body.lower() or "page-content" not in body:
            raise RuntimeError(
                "standing scp-9506 canary returned an unexpected document"
            )
        canary_completed = time.monotonic()
        candidate_runtime = runtime_containers(runtime_home)
        for service in SERVICES:
            if (
                candidate_runtime[service]["container_id"]
                == previous_runtime[service]["container_id"]
            ):
                raise RuntimeError(
                    f"promotion did not create a new container for {service}"
                )
            if candidate_runtime[service]["image_id"] != images[service]["id"]:
                raise RuntimeError(f"candidate container image changed for {service}")
        candidate_runtime_images = runtime_images(candidate_runtime, runtime_home)
        candidate_port_443_owner = port_443_owner(candidate_runtime)
        effective_config = command(
            *compose_command(runtime_home, "config", override_file=override_file),
            cwd=runtime_home,
        )
        differential_identity = runtime_differential_identity(
            source_root, identity, images, effective_config
        )
        atomic_json(differential_identity_path, differential_identity)
        differential_identity_published = True
        receipt: dict[str, object] = {
            "schema_version": 3,
            "kind": "standing-promotion",
            "status": "pass",
            "run_id": prepared_receipt["run_id"],
            "started_at": started_at.isoformat(),
            "completed_at": datetime.now(UTC).isoformat(),
            "activation_duration_seconds": time.monotonic() - activation_started,
            "image_verification_duration_seconds": activation_verified_at
            - activation_started,
            "compose_activation_duration_seconds": health_started - compose_started,
            "health_duration_seconds": health_completed - health_started,
            "canary_duration_seconds": canary_completed - canary_started,
            **identity,
            "promotion_precondition": prepared_receipt["promotion_precondition"],
            "runtime_home": str(runtime_home),
            "prepared_receipt": {
                "path": str(prepared_receipt_path),
                "sha256": prepared_receipt_sha256,
            },
            "project_name": "wikijump-standing",
            "network_name": network_name,
            "images": images,
            "rollback_images": rollback_images,
            "protected_volumes": list(PROTECTED_VOLUMES),
            "runtime_differential_identity": {
                "path": str(differential_identity_path),
                "sha256": file_sha256(differential_identity_path),
                "identity": differential_identity,
            },
            "health": health,
            "saved_page_render_freshness": saved_page_render_freshness,
            "render_ahead": render_ahead,
            "canary": {
                "url": CANARY_URL,
                "status": "pass",
                "required_markers": ["scp-9506", "page-content"],
            },
            "cleanup": {
                "status": "pass",
                "candidate_receipt": {
                    "path": str(prepared_receipt_path),
                    "sha256": prepared_receipt_sha256,
                },
                "receipt": {
                    "path": str(prepared_receipt_path),
                    "sha256": prepared_receipt_sha256,
                },
                "superseded_images": [],
            },
            "resource_disposition": {
                "active": {
                    service: {
                        "owner": "standing-runtime",
                        "keep_until": expiry,
                        "id": images[service]["id"],
                    }
                    for service in SERVICES
                },
                "rollback": {
                    service: {
                        "owner": "standing-runtime-rollback",
                        "keep_until": expiry,
                        "id": rollback_images[service]["id"],
                    }
                    for service in SERVICES
                },
                "volumes": "protected-and-untouched",
                "worktrees": "none created",
                "target_directories": "none created",
            },
        }
        atomic_json_no_replace(receipt_path, receipt)
        print(
            json.dumps(
                {"status": "pass", "receipt": str(receipt_path), **identity},
                sort_keys=True,
            )
        )
        return 0
    except Exception as error:
        if switched:
            if candidate_runtime is None:
                try:
                    candidate_runtime = runtime_containers(runtime_home)
                except Exception:
                    candidate_runtime = None
            cleanup = None
            cleanup_error = None
            try:
                cleanup = remove_candidate_resources(
                    candidate_runtime, images, rollback_images, runtime_home
                )
            except Exception as cleanup_exception:
                cleanup_error = str(cleanup_exception)
        else:
            cleanup = {
                "status": "not-needed",
                "reason": "candidate activation did not switch the runtime",
            }
            cleanup_error = None
        restored = False
        restore_error = None
        restored_runtime = None
        restored_port_443_owner = None
        rollback_health = None
        rollback_canary = None
        if parked:
            try:
                if switched:
                    write_environment(
                        runtime_home / ".env",
                        rollback_environment(previous_environment, rollback_images),
                    )
                restored_runtime = restore_parked_containers(parked, runtime_home)
                final_runtime = runtime_containers(runtime_home)
                for service in RUNTIME_SERVICES:
                    if (
                        final_runtime[service]["container_id"]
                        != previous_runtime[service]["container_id"]
                    ):
                        raise RuntimeError(
                            f"rollback changed container identity for {service}"
                        )
                restored_port_443_owner = port_443_owner(final_runtime)
                if (
                    restored_port_443_owner["container_id"]
                    != previous_port_443_owner["container_id"]
                ):
                    raise RuntimeError("rollback changed the port-443 owner")
                rollback_health = wait_for_health(
                    runtime_home, override_file, args.health_timeout_seconds
                )
                rollback_body = command(
                    "curl",
                    "--silent",
                    "--show-error",
                    "--fail",
                    "--location",
                    "--insecure",
                    "--max-time",
                    "30",
                    CANARY_URL,
                    cwd=runtime_home,
                )
                if (
                    "scp-9506" not in rollback_body.lower()
                    or "page-content" not in rollback_body
                ):
                    raise RuntimeError(
                        "restored standing scp-9506 canary returned an unexpected document"
                    )
                rollback_canary = {
                    "url": CANARY_URL,
                    "status": "pass",
                    "required_markers": ["scp-9506", "page-content"],
                }
                if differential_identity_published:
                    if previous_differential is None:
                        differential_identity_path.unlink(missing_ok=True)
                    else:
                        atomic_json(differential_identity_path, previous_differential)
                restored = True
            except Exception as rollback_exception:
                restore_error = str(rollback_exception)
        failure = {
            "schema_version": 3,
            "status": "fail",
            "run_id": prepared_receipt_sha256,
            "started_at": started_at.isoformat(),
            "completed_at": datetime.now(UTC).isoformat(),
            "error": str(error),
            "render_ahead": render_ahead,
            "containers": {
                "before": previous_runtime,
                "candidate": candidate_runtime,
                "port_443_owner_before": previous_port_443_owner,
                "port_443_owner_candidate": candidate_port_443_owner,
            },
            "runtime_images": {
                "before": previous_runtime_images,
                "candidate": candidate_runtime_images,
            },
            "cleanup": cleanup,
            "cleanup_error": cleanup_error,
            "rollback": {
                "status": "pass" if restored else "fail",
                "containers": restored_runtime,
                "port_443_owner": restored_port_443_owner,
                "error": restore_error,
                "health": rollback_health,
                "canary": rollback_canary,
            },
            "rollback_images": rollback_images,
        }
        atomic_json_no_replace(failure_receipt_path, failure)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
