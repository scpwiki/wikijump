#!/usr/bin/env python3
"""Render saved pages ahead of a Deepwell renderer-generation switch.

A renderer epoch or FTML bump makes every saved page stale, and Deepwell
refuses stale output. Rerendering only after activation therefore holds the
site in maintenance for the whole rerender. This command runs the `render-ahead`
action of the image that is about to be activated against the live database
while the current Deepwell keeps serving. The new Deepwell promotes the rows
when it starts, so activation costs a container restart.

The one-off container copies the environment and configuration mount of the
running Deepwell container and joins its network namespace, so it reaches the
same database, cache, and object store. Its entrypoint is the Deepwell binary,
not the image start script, so no migration is recorded: the command only runs
the image's idempotent render-ahead table migration, which keeps the serving
build's own migration check (and a rollback to it) working.

Rendering also refreshes link rows. If those rows change during the run, a page
rendered early may have read links that a later page changed, so every page is
rendered again; the links must then be stable. Pages that miss the serving
render deadline under concurrent load are retried alone and then once with a
longer one-shot deadline (docs/agents/compatibility/execution.md); any other
failure stops the run. A final dry run checks the whole site.
"""

from __future__ import annotations

import argparse
from datetime import UTC, datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time


MIGRATION_PATH = "/opt/deepwell/migrations/20261014120000_page_revision_render_ahead.sql"
DEEPWELL_BINARY = "/usr/local/bin/deepwell"
CONFIG_PATH = "/etc/deepwell.toml"
# Link content only: rendering rewrites rows and their timestamps even when
# the links are unchanged.
LINK_TABLES = {
    "page_connection": ("from_page_id", "to_page_id", "connection_type", "count"),
    "page_connection_missing": (
        "from_page_id",
        "to_site_id",
        "to_page_slug",
        "connection_type",
        "count",
    ),
    "page_link": ("page_id", "url", "count"),
}
IMAGE_ID = re.compile(r"sha256:[0-9a-f]{64}")
IDEMPOTENT_STATEMENT = re.compile(
    r"^(CREATE TABLE IF NOT EXISTS|CREATE INDEX IF NOT EXISTS)\s", re.IGNORECASE
)
SUMMARY_SCHEMA = "wikijump.render_ahead_receipt.v1"
RENDER_TIMEOUT = re.compile(r"^render-timeout-ms\s*=\s*[0-9]+\s*$", re.MULTILINE)
DEADLINE_MARKERS = ("deadline has elapsed", "due to timeout")


def run(*args: str, input_text: str | None = None) -> str:
    result = subprocess.run(
        args, check=True, text=True, capture_output=True, input=input_text
    )
    return result.stdout


def sql_statements(sql: str) -> list[str]:
    """Split a migration into statements, dropping comments and blank lines."""
    lines = [
        line for line in sql.splitlines() if line.strip() and not line.lstrip().startswith("--")
    ]
    return [statement.strip() for statement in "\n".join(lines).split(";") if statement.strip()]


def require_idempotent_migration(sql: str) -> None:
    statements = sql_statements(sql)
    if not statements:
        raise ValueError("render-ahead migration is empty")
    for statement in statements:
        if IDEMPOTENT_STATEMENT.match(statement) is None:
            raise ValueError(
                "render-ahead migration must only create tables or indexes IF NOT EXISTS: "
                + statement.splitlines()[0]
            )


def container_inspect(container: str) -> dict[str, object]:
    [inspected] = json.loads(run("docker", "inspect", container))
    state = inspected.get("State") or {}
    if not state.get("Running"):
        raise RuntimeError(f"{container} is not running")
    return inspected


def config_mount(inspected: dict[str, object]) -> str:
    for mount in inspected.get("Mounts") or []:
        if mount.get("Destination") == CONFIG_PATH:
            return str(mount["Source"])
    raise RuntimeError(f"Deepwell container has no {CONFIG_PATH} mount")


def runtime_environment(inspected: dict[str, object]) -> dict[str, str]:
    environment = {}
    for entry in (inspected.get("Config") or {}).get("Env") or []:
        key, _, value = entry.partition("=")
        if key.startswith("DEEPWELL_RENDER") or key == "DEEPWELL_RUNTIME_ACTION":
            continue
        environment[key] = value
    for required in ("DATABASE_URL", "REDIS_URL", "DEEPWELL_RPC_TOKEN"):
        if required not in environment:
            raise RuntimeError(f"Deepwell container environment lacks {required}")
    return environment


def render_ahead_settings(
    concurrency: int,
    batch_size: int,
    not_before: str | None,
    page_ids: list[int] | None = None,
    dry_run: bool = False,
) -> dict[str, str]:
    settings = {
        "DEEPWELL_RUNTIME_ACTION": "render-ahead",
        "DEEPWELL_RENDER_AHEAD_DRY_RUN": "true" if dry_run else "false",
        "DEEPWELL_RENDER_AHEAD_CONCURRENCY": str(concurrency),
        "DEEPWELL_RENDER_AHEAD_BATCH_SIZE": str(batch_size),
    }
    if not_before is not None:
        settings["DEEPWELL_RENDER_AHEAD_NOT_BEFORE"] = not_before
    if page_ids is not None:
        if not page_ids:
            raise ValueError("a page-limited render-ahead run needs page IDs")
        settings["DEEPWELL_RENDER_AHEAD_PAGE_IDS"] = ",".join(map(str, page_ids))
    return settings


def deadline_only(summary: dict[str, object]) -> bool:
    """Whether every failure of a pass is the renderer deadline."""
    failures = summary["failures"]
    return len(failures) == summary["failed"] and all(
        any(marker in failure["error"] for marker in DEADLINE_MARKERS)
        for failure in failures
    )


def config_with_render_timeout(config: str, timeout_ms: int) -> str:
    matches = RENDER_TIMEOUT.findall(config)
    if len(matches) != 1:
        raise ValueError("Deepwell config must set render-timeout-ms exactly once")
    return RENDER_TIMEOUT.sub(f"render-timeout-ms = {timeout_ms}", config)


def parse_summary(output: str, dry_run: bool = False) -> dict[str, object]:
    summaries = []
    for line in output.splitlines():
        if not line.startswith("{"):
            continue
        value = json.loads(line)
        if value.get("action") == "render-ahead":
            summaries.append(value)
    if len(summaries) != 1:
        raise RuntimeError("render-ahead did not print exactly one summary")
    summary = summaries[0]
    if summary.get("dry_run") is not dry_run:
        raise RuntimeError("render-ahead summary does not match the requested mode")
    return summary


def psql(database_container: str, sql: str) -> str:
    return run(
        "docker",
        "exec",
        "-i",
        database_container,
        "sh",
        "-c",
        'PGPASSWORD="$POSTGRES_PASSWORD" exec psql -h 127.0.0.1 -U "$POSTGRES_USER" '
        '-d "$POSTGRES_DB" -At -v ON_ERROR_STOP=1 --single-transaction',
        input_text=sql,
    ).strip()


def link_fingerprint(database_container: str) -> dict[str, str]:
    queries = "\n".join(link_fingerprint_query(table, columns) for table, columns in LINK_TABLES.items())
    rows = psql(database_container, queries).splitlines()
    fingerprint = dict(row.split("|", 1) for row in rows)
    if set(fingerprint) != set(LINK_TABLES):
        raise RuntimeError("link fingerprint query returned unexpected rows")
    return fingerprint


def link_fingerprint_query(table: str, columns: tuple[str, ...]) -> str:
    row = "ROW(" + ", ".join(columns) + ")::text"
    return (
        f"SELECT '{table}', md5(COALESCE(string_agg({row}, E'\\n' ORDER BY {row}), '')) "
        f"FROM {table};"
    )


def render_pass(
    image: str,
    deepwell_container: str,
    config_source: str,
    environment: dict[str, str],
    settings: dict[str, str],
    log_path: Path,
) -> dict[str, object]:
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", prefix="render-ahead-env.", delete=False
    ) as env_file:
        os.chmod(env_file.name, 0o600)
        for key, value in {**environment, **settings}.items():
            if "\n" in value:
                raise ValueError(f"{key} is not a single-line environment value")
            env_file.write(f"{key}={value}\n")
    try:
        result = subprocess.run(
            (
                "docker",
                "run",
                "--rm",
                "--network",
                f"container:{deepwell_container}",
                "--env-file",
                env_file.name,
                "--volume",
                f"{config_source}:{CONFIG_PATH}:ro",
                "--entrypoint",
                DEEPWELL_BINARY,
                image,
                CONFIG_PATH,
            ),
            check=False,
            text=True,
            capture_output=True,
        )
    finally:
        os.unlink(env_file.name)
    log_path.write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode != 0:
        raise RuntimeError(f"render-ahead exited {result.returncode}; see {log_path}")
    return parse_summary(
        result.stdout, settings["DEEPWELL_RENDER_AHEAD_DRY_RUN"] == "true"
    )


def image_identity(image: str) -> str:
    image_id = run("docker", "image", "inspect", "--format", "{{.Id}}", image).strip()
    if IMAGE_ID.fullmatch(image_id) is None:
        raise RuntimeError(f"{image} has no immutable image ID")
    return image_id


def prepared_deepwell_image(prepared_receipt: Path) -> str:
    prepared = json.loads(prepared_receipt.read_text(encoding="utf-8"))
    image = prepared["images"]["deepwell"]
    if IMAGE_ID.fullmatch(image.get("id", "")) is None:
        raise ValueError("prepared receipt has no immutable Deepwell image ID")
    return image["id"]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--prepared-receipt", type=Path)
    source.add_argument("--image", help="Deepwell image ID to render with")
    parser.add_argument("--deepwell-container", default="wikijump-standing-deepwell-1")
    parser.add_argument("--database-container", default="wikijump-standing-database-1")
    parser.add_argument("--concurrency", type=int, default=16)
    parser.add_argument("--batch-size", type=int, default=200)
    parser.add_argument(
        "--tail-render-timeout-ms",
        type=int,
        default=300_000,
        help="one-shot deadline for pages that still miss the serving deadline alone",
    )
    parser.add_argument(
        "--rerender-all",
        action="store_true",
        help="ignore existing ahead rows and render every stale page again",
    )
    parser.add_argument("--output-dir", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if min(args.concurrency, args.batch_size, args.tail_render_timeout_ms) < 1:
        raise ValueError("numeric options must be positive")
    image = (
        prepared_deepwell_image(args.prepared_receipt)
        if args.prepared_receipt is not None
        else args.image
    )
    image_id = image_identity(image)
    args.output_dir.mkdir(mode=0o700, parents=True, exist_ok=False)
    started_at = datetime.now(UTC)
    started = time.monotonic()

    inspected = container_inspect(args.deepwell_container)
    serving_image_id = str(inspected.get("Image"))
    if serving_image_id == image_id:
        raise ValueError("the serving Deepwell already runs the render-ahead image")
    environment = runtime_environment(inspected)
    config_source = config_mount(inspected)

    migration = run("docker", "run", "--rm", "--entrypoint", "cat", image_id, MIGRATION_PATH)
    require_idempotent_migration(migration)
    psql(args.database_container, migration)

    passes: list[dict[str, object]] = []

    def render(name: str, settings: dict[str, str], config: str = config_source) -> dict[str, object]:
        pass_started = time.monotonic()
        summary = render_pass(
            image_id,
            args.deepwell_container,
            config,
            environment,
            settings,
            args.output_dir / f"{name}.log",
        )
        passes.append(
            {
                "pass": name,
                "settings": {
                    key: value
                    for key, value in settings.items()
                    if key != "DEEPWELL_RUNTIME_ACTION"
                },
                "config": "serving" if config == config_source else "extended-deadline",
                "duration_seconds": time.monotonic() - pass_started,
                "summary": summary,
            }
        )
        return summary

    # Full passes: render every stale page, and again if links moved.
    not_before = (
        datetime.now(UTC).isoformat().replace("+00:00", "Z") if args.rerender_all else None
    )
    links_before = link_fingerprint(args.database_container)
    for number in (1, 2):
        summary = render(
            f"full-{number}",
            render_ahead_settings(args.concurrency, args.batch_size, not_before),
        )
        links_after = link_fingerprint(args.database_container)
        passes[-1]["links_before"] = links_before
        passes[-1]["links_after"] = links_after
        if links_after == links_before:
            break
        if number == 2:
            raise RuntimeError("link rows still changed during the second full pass")
        # Rows rendered before this point may have read links that changed.
        not_before = datetime.now(UTC).isoformat().replace("+00:00", "Z")
        links_before = links_after

    # Tail: pages that missed the deadline under load are retried alone, then
    # once with a longer one-shot deadline. Any other failure stops the run.
    failed = summary["failed_page_ids"]
    if failed:
        if not deadline_only(summary):
            raise RuntimeError("render-ahead has failures other than the render deadline")
        summary = render(
            "tail-serial",
            render_ahead_settings(1, args.batch_size, not_before, failed),
        )
        failed = summary["failed_page_ids"]
    if failed:
        if not deadline_only(summary):
            raise RuntimeError("render-ahead tail has failures other than the render deadline")
        with tempfile.NamedTemporaryFile(
            "w", encoding="utf-8", prefix="render-ahead-config.", suffix=".toml", delete=False
        ) as config_file:
            config_file.write(
                config_with_render_timeout(
                    Path(config_source).read_text(encoding="utf-8"),
                    args.tail_render_timeout_ms,
                )
            )
        os.chmod(config_file.name, 0o644)
        try:
            summary = render(
                "tail-extended-deadline",
                render_ahead_settings(1, args.batch_size, not_before, failed),
                config_file.name,
            )
        finally:
            os.unlink(config_file.name)
        failed = summary["failed_page_ids"]

    # Verify the whole site, not just the last pass's pages.
    final = render(
        "verify",
        render_ahead_settings(args.concurrency, args.batch_size, not_before, dry_run=True),
    )
    receipt = {
        "schema": SUMMARY_SCHEMA,
        "status": "pass" if final["stale_without_valid_ahead"] == 0 else "incomplete",
        "started_at": started_at.isoformat(),
        "completed_at": datetime.now(UTC).isoformat(),
        "duration_seconds": time.monotonic() - started,
        "render_image_id": image_id,
        "serving_image_id": serving_image_id,
        "compiled_generator": final["compiled_generator"],
        "deepwell_container": args.deepwell_container,
        "database_container": args.database_container,
        "migration_sha256": hashlib.sha256(migration.encode()).hexdigest(),
        "tail_render_timeout_ms": args.tail_render_timeout_ms,
        "passes": passes,
        "unrendered_page_ids": failed,
        "stale_without_valid_ahead": final["stale_without_valid_ahead"],
    }
    receipt_path = args.output_dir / "render-ahead-receipt.json"
    with receipt_path.open("x", encoding="utf-8") as output:
        json.dump(receipt, output, indent=2, sort_keys=True)
        output.write("\n")
    print(
        json.dumps(
            {
                key: receipt[key]
                for key in ("status", "duration_seconds", "stale_without_valid_ahead")
            }
        )
    )
    return 0 if receipt["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
