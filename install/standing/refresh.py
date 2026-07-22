#!/usr/bin/env python3
"""Build and restart only the standing application services from merged develop."""

from __future__ import annotations

import argparse
from datetime import UTC, datetime, timedelta
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time


SERVICES = ("deepwell", "framerail", "wws")
DEFAULT_RUNTIME_HOME = Path("/home/roku/wjlab/runtime/wikijump-standing")
CANARY_URL = "http://scp-wiki.wikijump.localhost/scp-9506"
FTML_SOURCE = re.compile(
    r'source = "git\+https://github\.com/Rokurolize/ftml[^\"]*#([0-9a-f]{40})"'
)


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
    return {"wikijump_sha": head, "wikijump_tree": tree, "ftml_sha": matches.pop()}


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


def image_tag(wikijump_sha: str, service: str) -> str:
    return f"local/wikijump-standing-{wikijump_sha[:9]}-{service}:latest"


def build_command(
    source_root: Path, service: str, tag: str, identity: dict[str, str], expiry: str
) -> list[str]:
    args = [
        "docker",
        "build",
        "--file",
        str(source_root / "install" / "local" / service / "Dockerfile"),
        "--label",
        "com.rokurolize.wikijump.owner=standing-runtime",
        "--label",
        f"com.rokurolize.wikijump.expiry={expiry}",
        "--label",
        f"com.rokurolize.wikijump.sha={identity['wikijump_sha']}",
        "--label",
        f"com.rokurolize.wikijump.ftml_sha={identity['ftml_sha']}",
    ]
    if service == "framerail":
        args.extend(("--build-arg", "FRAMERAIL_ENV=local"))
    args.extend(("--tag", tag, str(source_root)))
    return args


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


def image_identity(tag: str, cwd: Path) -> dict[str, object]:
    raw = command("docker", "image", "inspect", tag, "--format", "{{json .}}", cwd=cwd)
    image = json.loads(raw)
    return {
        "tag": tag,
        "id": image["Id"],
        "repo_digests": sorted(image.get("RepoDigests") or []),
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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source-root", type=Path, default=Path(__file__).resolve().parents[2]
    )
    parser.add_argument("--runtime-home", type=Path, default=DEFAULT_RUNTIME_HOME)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--health-timeout-seconds", type=int, default=1800)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    source_root = args.source_root.resolve()
    runtime_home = args.runtime_home.resolve()
    receipt_path = (args.receipt or runtime_home / "refresh-receipt.json").resolve()
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
    expiry = (started_at + timedelta(days=30)).isoformat()
    identity = repository_identity(source_root)
    environment = read_environment(runtime_home / ".env")
    if environment.get("STANDING_PROJECT_NAME") != "wikijump-standing":
        raise ValueError("runtime home is not the wikijump-standing project")
    network_name = environment.get("STANDING_NETWORK_NAME")
    if not network_name:
        raise ValueError("runtime environment does not name its standing network")
    command("docker", "network", "inspect", "--", network_name, cwd=runtime_home)

    tags = {
        service: image_tag(identity["wikijump_sha"], service) for service in SERVICES
    }
    for service in SERVICES:
        command(
            *build_command(source_root, service, tags[service], identity, expiry),
            cwd=source_root,
            capture=False,
        )
    if repository_identity(source_root) != identity:
        raise RuntimeError("source identity changed during the image builds")

    environment.update(
        {
            "STANDING_DEEPWELL_IMAGE": tags["deepwell"],
            "STANDING_FRAMERAIL_IMAGE": tags["framerail"],
            "STANDING_WWS_IMAGE": tags["wws"],
            "STANDING_WIKIJUMP_SHA": identity["wikijump_sha"],
            "STANDING_FTML_SHA": identity["ftml_sha"],
            "STANDING_LOCALES_SOURCE": str(source_root / "locales"),
            "STANDING_RESOURCE_EXPIRY": expiry,
        }
    )
    write_environment(runtime_home / ".env", environment)
    command(
        *compose_command(
            runtime_home,
            "up",
            "--detach",
            "--no-deps",
            *SERVICES,
            override_file=override_file,
        ),
        cwd=runtime_home,
        capture=False,
    )
    health = wait_for_health(runtime_home, override_file, args.health_timeout_seconds)
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
        raise RuntimeError("standing scp-9506 canary returned an unexpected document")

    images = {
        service: image_identity(tag, runtime_home) for service, tag in tags.items()
    }
    receipt: dict[str, object] = {
        "schema_version": 1,
        "status": "pass",
        "started_at": started_at.isoformat(),
        "completed_at": datetime.now(UTC).isoformat(),
        **identity,
        "runtime_home": str(runtime_home),
        "project_name": "wikijump-standing",
        "network_name": network_name,
        "images": images,
        "health": health,
        "canary": {
            "url": CANARY_URL,
            "status": "pass",
            "required_markers": ["scp-9506", "page-content"],
        },
        "resource_disposition": {
            "containers": {
                service: {"owner": "standing-runtime", "keep_until": expiry}
                for service in SERVICES
            },
            "images": {
                service: {
                    "owner": "standing-runtime",
                    "keep_until": expiry,
                    "id": images[service]["id"],
                }
                for service in SERVICES
            },
            "volumes": "untouched",
            "worktrees": "none created",
            "target_directories": "none created",
        },
    }
    atomic_json(receipt_path, receipt)
    print(
        json.dumps(
            {"status": "pass", "receipt": str(receipt_path), **identity}, sort_keys=True
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
