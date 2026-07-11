#!/usr/bin/env python3
"""Copy Deepwell build inputs into a running cargo-watch container.

This is for prebuilt/no-dev local runtimes whose image already contains the
Rust toolchain, cargo-watch, and an initial Deepwell build.  Normal local-dev
containers use bind mounts and do not need this helper.
"""

from __future__ import annotations

import argparse
from contextlib import contextmanager
import fcntl
import json
import math
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tempfile
import time
import uuid


DESTINATION = PurePosixPath("/src/deepwell")
SYNC_ENTRIES = (
    "src",
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "askama.toml",
)


class HotReloadError(RuntimeError):
    """An expected, operator-actionable hot-reload failure."""


class Docker:
    """Small subprocess boundary that keeps Docker calls easy to test."""

    def run(
        self,
        arguments: list[str],
        *,
        check: bool = True,
    ) -> subprocess.CompletedProcess[str]:
        try:
            result = subprocess.run(
                ["docker", *arguments],
                check=False,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
        except FileNotFoundError as error:
            raise HotReloadError("docker was not found on PATH") from error

        if check and result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip() or "no output"
            raise HotReloadError(
                f"docker {' '.join(arguments[:2])} failed ({result.returncode}): {detail}"
            )
        return result


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Stage and replace Deepwell sources in a running cargo-watch "
            "container, then wait for the replacement daemon to become healthy."
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""Examples:
  # A prebuilt corpus runtime discovered through Compose labels
  %(prog)s --project runtime50x

  # Select a container directly and copy from another worktree
  %(prog)s --container runtime50x-deepwell-1 --source-root /path/to/wikijump

  # Validate discovery and safety checks without changing the container
  %(prog)s --project runtime50x --dry-run --json

The container must already run cargo-watch and contain /src/deepwell.  The
helper refuses source bind mounts so it cannot unexpectedly edit host files.
""",
    )
    parser.add_argument(
        "--project",
        default=os.environ.get("COMPOSE_PROJECT_NAME", "wikijump"),
        help="Compose project label used for discovery (default: %(default)s)",
    )
    parser.add_argument(
        "--service",
        default="deepwell",
        help="Compose service label used for discovery (default: %(default)s)",
    )
    parser.add_argument(
        "--container",
        help="Container name or ID; bypasses Compose label discovery",
    )
    parser.add_argument(
        "--source-root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
        help="Wikijump worktree to copy from (default: this script's worktree)",
    )
    parser.add_argument(
        "--timeout",
        type=positive_float,
        default=300.0,
        help="Seconds to wait for rebuild and health (default: %(default)s)",
    )
    parser.add_argument(
        "--settle",
        type=nonnegative_float,
        default=1.0,
        help="Seconds the new healthy daemon PID must remain stable (default: %(default)s)",
    )
    parser.add_argument(
        "--no-wait",
        action="store_true",
        help="Return after the copy triggers cargo-watch instead of waiting",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Run source/container preflight checks without copying anything",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Print one machine-readable JSON result",
    )
    return parser


def positive_float(value: str) -> float:
    parsed = float(value)
    if not math.isfinite(parsed) or parsed <= 0:
        raise argparse.ArgumentTypeError("must be greater than zero")
    return parsed


def nonnegative_float(value: str) -> float:
    parsed = float(value)
    if not math.isfinite(parsed) or parsed < 0:
        raise argparse.ArgumentTypeError("must be zero or greater")
    return parsed


def validate_source_root(source_root: Path) -> tuple[Path, list[Path]]:
    source_root = source_root.expanduser().resolve()
    manifest = source_root / "deepwell" / "Cargo.toml"
    if not manifest.is_file():
        raise HotReloadError(
            f"source root is not a Wikijump worktree (missing {manifest})"
        )
    if 'name = "deepwell"' not in manifest.read_text(encoding="utf-8"):
        raise HotReloadError(f"unexpected package manifest: {manifest}")

    entries = [source_root / "deepwell" / entry for entry in SYNC_ENTRIES]
    missing = [str(path) for path in entries if not path.exists()]
    if missing:
        raise HotReloadError(
            f"required Deepwell inputs are missing: {', '.join(missing)}"
        )
    return source_root, entries


def paths_overlap(first: str, second: str) -> bool:
    """Return whether either absolute POSIX path contains the other."""

    left = PurePosixPath(first)
    right = PurePosixPath(second)
    return left == right or left in right.parents or right in left.parents


def inspect_container(docker: Docker, container: str) -> dict:
    result = docker.run(["inspect", container])
    try:
        records = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise HotReloadError("docker inspect returned invalid JSON") from error
    if len(records) != 1:
        raise HotReloadError(f"docker inspect returned {len(records)} containers")
    return records[0]


def discover_container(
    docker: Docker,
    *,
    project: str,
    service: str,
    explicit_container: str | None,
) -> tuple[str, dict]:
    if explicit_container:
        inspected = inspect_container(docker, explicit_container)
        return inspected["Id"], inspected

    result = docker.run(
        [
            "ps",
            "--quiet",
            "--filter",
            f"label=com.docker.compose.project={project}",
            "--filter",
            f"label=com.docker.compose.service={service}",
        ]
    )
    matches = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    if not matches:
        raise HotReloadError(
            f"no running container matches Compose project={project!r}, service={service!r}"
        )
    if len(matches) != 1:
        raise HotReloadError(
            f"{len(matches)} running containers match project={project!r}, "
            f"service={service!r}; pass --container explicitly"
        )
    inspected = inspect_container(docker, matches[0])
    return inspected["Id"], inspected


def validate_container(inspected: dict) -> None:
    name = inspected.get("Name", "unknown").lstrip("/")
    if not inspected.get("State", {}).get("Running", False):
        raise HotReloadError(f"container {name} is not running")

    synchronized_paths = [str(DESTINATION / entry) for entry in SYNC_ENTRIES]
    blockers = []
    for mount in inspected.get("Mounts", []):
        mount_path = mount.get("Destination")
        if mount_path and any(
            paths_overlap(mount_path, synchronized)
            for synchronized in synchronized_paths
        ):
            blockers.append(mount_path)
    if blockers:
        joined = ", ".join(sorted(set(blockers)))
        raise HotReloadError(
            "refusing to copy across a container mount that overlaps Deepwell "
            f"build inputs ({joined}); use the existing bind-mount dev loop instead"
        )


def exec_shell(
    docker: Docker,
    container: str,
    script: str,
    *arguments: str,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    return docker.run(
        ["exec", container, "sh", "-eu", "-c", script, "hot-reload", *arguments],
        check=check,
    )


def preflight_runtime(docker: Docker, container: str) -> None:
    script = r"""
destination=$1
test -d "$destination"
test -f "$destination/Cargo.toml"
test -w "$destination"
test -x /usr/local/bin/wikijump-health-check
command -v timeout >/dev/null
for process in /proc/[0-9]*; do
    executable=$(readlink "$process/exe" 2>/dev/null || true)
    case "$executable" in
        */cargo-watch) exit 0 ;;
    esac
done
echo "cargo-watch is not running in the container" >&2
exit 1
"""
    result = exec_shell(docker, container, script, str(DESTINATION), check=False)
    if result.returncode != 0:
        detail = result.stderr.strip() or "runtime layout check failed"
        raise HotReloadError(detail)


def daemon_pid(docker: Docker, container: str) -> int | None:
    script = r"""
pid_file=/run/deepwell.pid
test -r "$pid_file" || exit 0
pid=$(cat "$pid_file")
case "$pid" in
    ''|*[!0-9]*) exit 0 ;;
esac
test -d "/proc/$pid" || exit 0
executable=$(readlink "/proc/$pid/exe" 2>/dev/null || true)
case "$executable" in
    */target/debug/deepwell|*/target/release/deepwell)
        printf '%s\n' "$pid"
        ;;
esac
exit 0
"""
    result = exec_shell(docker, container, script)
    output = result.stdout.strip()
    return int(output) if output else None


def health_check(docker: Docker, container: str) -> bool:
    result = docker.run(
        [
            "exec",
            container,
            "timeout",
            "5",
            "/usr/local/bin/wikijump-health-check",
        ],
        check=False,
    )
    return result.returncode == 0


def zombie_deepwell_process_count(docker: Docker, container: str) -> int:
    """Count unreaped Deepwell processes left under a non-init cargo-watch PID 1."""

    script = r"""
count=0
for process in /proc/[0-9]*; do
    test -r "$process/comm" || continue
    test -r "$process/stat" || continue
    name=$(cat "$process/comm" 2>/dev/null || true)
    test "$name" = deepwell || continue
    read -r _ _ state _ < "$process/stat" || continue
    if test "$state" = Z; then
        count=$((count + 1))
    fi
done
printf '%s\n' "$count"
"""
    result = exec_shell(docker, container, script)
    try:
        return int(result.stdout.strip())
    except ValueError as error:
        raise HotReloadError(
            "failed to count zombie Deepwell processes in the container"
        ) from error


@contextmanager
def container_lock(container: str):
    """Serialize copies per container; the OS releases this lock on exit."""

    lock_path = Path(tempfile.gettempdir()) / (
        f"wikijump-deepwell-hot-reload-{container}.lock"
    )
    with lock_path.open("a+", encoding="utf-8") as lock_file:
        try:
            fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise HotReloadError(
                f"another Deepwell hot reload is running for container {container[:12]}"
            ) from error
        yield


def copy_inputs(
    docker: Docker,
    container: str,
    entries: list[Path],
    stage: str,
) -> None:
    exec_shell(docker, container, 'mkdir -p "$1/payload"', stage)
    for source in entries:
        docker.run(["cp", str(source), f"{container}:{stage}/payload/"])


def commit_inputs(docker: Docker, container: str, stage: str) -> None:
    entry_words = " ".join(SYNC_ENTRIES)
    script = rf"""
stage=$1
destination=$2
payload="$stage/payload"
backup="$stage/backup"
entries="{entry_words}"

for entry in $entries; do
    test -e "$payload/$entry"
    test -e "$destination/$entry"
done
mkdir -p "$backup"

committed=no
rollback() {{
    for entry in $entries; do
        if [ -e "$backup/$entry" ]; then
            rm -rf "$destination/$entry"
            mv "$backup/$entry" "$destination/$entry"
        fi
    done
}}
finish() {{
    status=$?
    if [ "$committed" != yes ]; then
        rollback
    fi
    exit "$status"
}}
trap finish HUP INT TERM EXIT

for entry in $entries; do
    mv "$destination/$entry" "$backup/$entry"
    mv "$payload/$entry" "$destination/$entry"
done

# Guarantee one event after every input is in its final location.
touch "$destination/Cargo.toml"
committed=yes
trap - HUP INT TERM EXIT
"""
    exec_shell(docker, container, script, stage, str(DESTINATION))


def rollback_inputs(docker: Docker, container: str, stage: str) -> None:
    entry_words = " ".join(SYNC_ENTRIES)
    script = rf"""
stage=$1
destination=$2
backup="$stage/backup"
entries="{entry_words}"

for entry in $entries; do
    test -e "$backup/$entry"
done
for entry in $entries; do
    rm -rf "$destination/$entry"
    mv "$backup/$entry" "$destination/$entry"
done
touch "$destination/Cargo.toml"
"""
    exec_shell(docker, container, script, stage, str(DESTINATION))


def restart_container(docker: Docker, container: str) -> None:
    """Kill any superseded build/daemon before validating restored inputs."""

    docker.run(["restart", container])


def cleanup_stage(docker: Docker, container: str, stage: str) -> None:
    exec_shell(docker, container, 'rm -rf "$1"', stage, check=False)


def wait_for_replacement(
    docker: Docker,
    container: str,
    *,
    old_pid: int,
    timeout: float,
    settle: float,
) -> int:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        current_pid = daemon_pid(docker, container)
        if (
            current_pid is not None
            and current_pid != old_pid
            and health_check(docker, container)
        ):
            if settle:
                time.sleep(min(settle, max(0.0, deadline - time.monotonic())))
            settled_pid = daemon_pid(docker, container)
            if settled_pid == current_pid and health_check(docker, container):
                return current_pid
        time.sleep(min(0.5, max(0.0, deadline - time.monotonic())))

    logs = docker.run(["logs", "--tail", "160", container], check=False)
    excerpt = (logs.stdout + logs.stderr).strip()
    if len(excerpt) > 12000:
        excerpt = excerpt[-12000:]
    detail = f"\nRecent container logs:\n{excerpt}" if excerpt else ""
    raise HotReloadError(
        f"timed out after {timeout:g}s waiting for a new healthy Deepwell daemon{detail}"
    )


def wait_for_healthy_daemon(
    docker: Docker,
    container: str,
    *,
    timeout: float,
    settle: float,
) -> int:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        current_pid = daemon_pid(docker, container)
        if current_pid is not None and health_check(docker, container):
            if settle:
                time.sleep(min(settle, max(0.0, deadline - time.monotonic())))
            settled_pid = daemon_pid(docker, container)
            if settled_pid == current_pid and health_check(docker, container):
                return current_pid
        time.sleep(min(0.5, max(0.0, deadline - time.monotonic())))
    raise HotReloadError(
        f"timed out after {timeout:g}s waiting for Deepwell to recover after rollback"
    )


def display_name(inspected: dict) -> str:
    return inspected.get("Name", "unknown").lstrip("/")


def run(arguments: argparse.Namespace, docker: Docker) -> dict:
    started = time.monotonic()
    source_root, entries = validate_source_root(arguments.source_root)
    container, inspected = discover_container(
        docker,
        project=arguments.project,
        service=arguments.service,
        explicit_container=arguments.container,
    )
    validate_container(inspected)
    preflight_runtime(docker, container)

    with container_lock(container):
        # Capture the daemon identity under the same lock as the copy. This
        # prevents a concurrent helper from replacing the PID between our
        # health preflight and the subsequent restart wait.
        old_pid = daemon_pid(docker, container)
        if old_pid is None or not health_check(docker, container):
            raise HotReloadError(
                "Deepwell must have a running healthy daemon before hot reload; "
                "inspect container logs first"
            )

        base = {
            "status": "dry-run" if arguments.dry_run else "synced",
            "container": display_name(inspected),
            "container_id": container[:12],
            "image": inspected.get("Config", {}).get("Image"),
            "source_root": str(source_root),
            "destination": str(DESTINATION),
            "entries": list(SYNC_ENTRIES),
            "old_daemon_pid": old_pid,
        }
        if arguments.dry_run:
            return {
                **base,
                "elapsed_seconds": round(time.monotonic() - started, 3),
            }

        stage = f"/tmp/wikijump-deepwell-hot-reload-{uuid.uuid4().hex}"
        candidate_committed = False
        container_restarted = False
        zombies_after_replacement = None
        preserve_stage = False
        try:
            copy_inputs(docker, container, entries, stage)
            commit_inputs(docker, container, stage)
            candidate_committed = True
            new_pid = None
            if not arguments.no_wait:
                try:
                    new_pid = wait_for_replacement(
                        docker,
                        container,
                        old_pid=old_pid,
                        timeout=arguments.timeout,
                        settle=arguments.settle,
                    )
                    zombies_after_replacement = zombie_deepwell_process_count(
                        docker, container
                    )
                    if zombies_after_replacement:
                        # cargo-watch is PID 1 in some prebuilt stacks and does
                        # not reap the replaced daemon. Restarting preserves the
                        # copied source and target cache while clearing zombies.
                        restart_container(docker, container)
                        container_restarted = True
                        new_pid = wait_for_healthy_daemon(
                            docker,
                            container,
                            timeout=arguments.timeout,
                            settle=arguments.settle,
                        )
                        remaining_zombies = zombie_deepwell_process_count(
                            docker, container
                        )
                        if remaining_zombies:
                            raise HotReloadError(
                                "Deepwell recovered after zombie cleanup but "
                                f"{remaining_zombies} zombie process(es) remain"
                            )
                except HotReloadError as candidate_error:
                    try:
                        rollback_inputs(docker, container, stage)
                        candidate_committed = False
                    except HotReloadError as rollback_error:
                        candidate_committed = False
                        preserve_stage = True
                        raise HotReloadError(
                            f"{candidate_error}; automatic rollback failed: "
                            f"{rollback_error}; backup preserved in {stage}"
                        ) from rollback_error
                    try:
                        restart_container(docker, container)
                        recovered_pid = wait_for_healthy_daemon(
                            docker,
                            container,
                            timeout=arguments.timeout,
                            settle=arguments.settle,
                        )
                    except HotReloadError as recovery_error:
                        raise HotReloadError(
                            f"{candidate_error}; previous inputs were restored but "
                            f"the daemon did not recover: {recovery_error}"
                        ) from recovery_error
                    raise HotReloadError(
                        f"{candidate_error}; previous inputs were restored and "
                        f"Deepwell recovered as PID {recovered_pid}"
                    ) from candidate_error
        except BaseException as interrupted:
            if candidate_committed:
                try:
                    rollback_inputs(docker, container, stage)
                    candidate_committed = False
                except HotReloadError as rollback_error:
                    preserve_stage = True
                    interruption = type(interrupted).__name__
                    if str(interrupted):
                        interruption = f"{interruption}: {interrupted}"
                    raise HotReloadError(
                        f"{interruption}; automatic rollback failed: "
                        f"{rollback_error}; backup preserved in {stage}"
                    ) from rollback_error
                # An interrupt can arrive while cargo-watch is still compiling
                # the candidate. Restarting is required even though the source
                # files are already restored, otherwise that superseded build
                # can launch after this process returns.
                restart_container(docker, container)
                wait_for_healthy_daemon(
                    docker,
                    container,
                    timeout=arguments.timeout,
                    settle=arguments.settle,
                )
            raise
        finally:
            if not preserve_stage:
                cleanup_stage(docker, container, stage)

    return {
        **base,
        "status": "triggered" if arguments.no_wait else "healthy",
        "new_daemon_pid": new_pid,
        "container_restarted": container_restarted,
        "zombies_after_replacement": zombies_after_replacement,
        "elapsed_seconds": round(time.monotonic() - started, 3),
    }


def print_result(result: dict, *, as_json: bool) -> None:
    if as_json:
        print(json.dumps(result, indent=2, sort_keys=True))
        return
    print(
        f"Deepwell hot reload {result['status']}: {result['container']} "
        f"({result['elapsed_seconds']:.3f}s)"
    )
    print(f"  source: {result['source_root']}")
    print(f"  copied: {', '.join(result['entries'])}")
    if result.get("new_daemon_pid") is not None:
        print(
            f"  daemon: PID {result['old_daemon_pid']} -> "
            f"{result['new_daemon_pid']} (healthy)"
        )


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    arguments = parser.parse_args(argv)
    try:
        result = run(arguments, Docker())
    except HotReloadError as error:
        if arguments.json:
            print(json.dumps({"status": "error", "error": str(error)}))
        else:
            print(f"deepwell hot reload failed: {error}", file=sys.stderr)
        return 1
    print_result(result, as_json=arguments.json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
