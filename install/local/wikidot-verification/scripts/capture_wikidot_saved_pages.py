#!/usr/bin/env python3
"""Create, capture, and remove run-owned Wikidot differential pages."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import signal
import time
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

PAGE_PLAN_SCHEMA = "wikijump_syntax_differential.wikidot_page_plan.v1"
CAPTURE_SCHEMA = "wikijump_syntax_differential.wikidot_saved_page_capture.v1"
ALLOWED_SITE = "sandbox-for-codex"
ALLOWED_DOMAIN = f"{ALLOWED_SITE}.wikidot.com"
ALLOWED_ORIGIN = f"http://{ALLOWED_DOMAIN}"
RUN_OWNED_SLUG = re.compile(r"^run-owned:ftml-diff-[0-9]{8}-[0-9]{3}$")
MAX_SOURCE_CHARACTERS = 160_000
MAX_SOURCE_BYTES = 500_000


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def validate_plan(value: object) -> dict[str, Any]:
    if not isinstance(value, dict) or value.get("schema") != PAGE_PLAN_SCHEMA:
        raise ValueError("page plan schema is unsupported")
    slug = value.get("slug")
    title = value.get("title")
    source = value.get("source")
    source_sha256 = value.get("source_sha256")
    if not isinstance(slug, str) or RUN_OWNED_SLUG.fullmatch(slug) is None:
        raise ValueError("page plan slug is outside the run-owned contract")
    if not isinstance(title, str) or not title or len(title) > 200:
        raise ValueError(f"page plan title is invalid for {slug}")
    if not isinstance(source, str) or not source:
        raise ValueError(f"page plan source is invalid for {slug}")
    if len(source) > MAX_SOURCE_CHARACTERS or len(source.encode("utf-8")) > MAX_SOURCE_BYTES:
        raise ValueError(f"page plan source exceeds the saved-page limit for {slug}")
    if source_sha256 != sha256(source):
        raise ValueError(f"page plan source hash is invalid for {slug}")
    cases = value.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError(f"page plan has no cases for {slug}")
    for case in cases:
        if (
            not isinstance(case, dict)
            or not isinstance(case.get("case_id"), str)
            or not case["case_id"]
            or not isinstance(case.get("source_sha256"), str)
            or re.fullmatch(r"[0-9a-f]{64}", case["source_sha256"]) is None
            or case.get("page_scope", "batch-safe") not in {"batch-safe", "isolated"}
        ):
            raise ValueError(f"page plan case identity is invalid for {slug}")
    isolated = [case for case in cases if case.get("page_scope") == "isolated"]
    if isolated:
        case = isolated[0]
        if (
            len(cases) != 1
            or case.get("marker_begin") is not None
            or case.get("marker_end") is not None
            or case["source_sha256"] != source_sha256
        ):
            raise ValueError(f"isolated page plan is not sentinel-free for {slug}")
    elif any(
        not isinstance(case.get(field), str) or not case[field].startswith("WJDIFF_")
        for case in cases
        for field in ("marker_begin", "marker_end")
    ):
        raise ValueError(f"batch page plan marker is invalid for {slug}")
    return value


def load_plans(path: Path) -> list[dict[str, Any]]:
    plans = [validate_plan(json.loads(line)) for line in path.read_text().splitlines() if line.strip()]
    slugs = [value["slug"] for value in plans]
    if not plans or len(slugs) != len(set(slugs)):
        raise ValueError("page plans must have unique non-empty slugs")
    return plans


def append_ledger(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as output:
        output.write(json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")))
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())


def fetch_page_content(client: Any, slug: str, attempts: int) -> tuple[str, str]:
    from bs4 import BeautifulSoup

    for attempt in range(attempts):
        response = client.get(f"{ALLOWED_ORIGIN}/{slug}")
        if response.status_code == 200:
            page_content = BeautifulSoup(response.text, "lxml").select_one("#page-content")
            if page_content is not None:
                return str(page_content), response.text
        if attempt + 1 < attempts:
            time.sleep(0.4)
    raise RuntimeError(f"anonymous page fetch did not return rendered #page-content for {slug}")


def saved_snapshot(site: Any, plan: dict[str, Any]) -> dict[str, Any]:
    slug = plan["slug"]
    page = site.page.get(slug, raise_when_not_found=False)
    if page is None:
        raise RuntimeError(f"created page is not visible: {slug}")
    page.refresh_source()
    actual_source = page.source.wiki_text
    if page.title != plan["title"]:
        raise RuntimeError(f"created page title did not round-trip exactly: {slug}")
    return {
        "slug": slug,
        "identity": page.id,
        "title": page.title,
        "requested_source_sha256": plan["source_sha256"],
        "saved_source": actual_source,
        "saved_source_sha256": sha256(actual_source),
    }


def verify_saved_markers(plan: dict[str, Any], snapshot: dict[str, Any]) -> None:
    if plan["cases"][0].get("page_scope") == "isolated":
        return
    source = snapshot["saved_source"]
    markers = [
        marker
        for case in plan["cases"]
        for marker in (case.get("marker_begin"), case.get("marker_end"))
        if marker is not None
    ]
    if not markers or any(source.count(marker) != 1 for marker in markers):
        raise RuntimeError(f"created page did not preserve its case markers: {plan['slug']}")


def remove_exact(site: Any, snapshot: dict[str, Any]) -> None:
    page = site.page.get(snapshot["slug"], raise_when_not_found=False)
    if page is None:
        return
    page.refresh_source()
    if (
        page.id != snapshot["identity"]
        or page.title != snapshot["title"]
        or sha256(page.source.wiki_text) != snapshot["saved_source_sha256"]
    ):
        raise RuntimeError(f"cleanup refused a changed page: {snapshot['slug']}")
    page.destroy()
    for _ in range(5):
        if site.page.get(snapshot["slug"], raise_when_not_found=False) is None:
            return
        time.sleep(0.4)
    raise RuntimeError(f"cleanup did not remove page: {snapshot['slug']}")


def capture(
    plans: list[dict[str, Any]],
    *,
    output: Path,
    ledger: Path,
    fetch_timeout_seconds: float,
    fetch_attempts: int,
) -> list[dict[str, Any]]:
    import httpx
    import wikidot
    from wikidot.connector.ajax import AjaxModuleConnectorConfig

    username = os.environ.pop("WIKIDOT_USERNAME", None)
    password = os.environ.pop("WIKIDOT_PASSWORD", None)
    if not username or not password:
        raise RuntimeError("WIKIDOT_USERNAME and WIKIDOT_PASSWORD are required")
    if output.exists() or ledger.exists():
        raise FileExistsError("capture output and ledger must not already exist")
    config = AjaxModuleConnectorConfig(allow_insecure_session_transport_for=ALLOWED_SITE)
    created: list[dict[str, Any]] = []
    records: list[dict[str, Any]] = []
    cleanup_error: Exception | None = None
    with wikidot.Client(username=username, password=password, amc_config=config) as authenticated:
        site = authenticated.site.get(ALLOWED_SITE)
        if site.unix_name != ALLOWED_SITE or site.domain != ALLOWED_DOMAIN:
            raise RuntimeError("resolved Wikidot site is outside the exact allowlist")
        try:
            with httpx.Client(
                follow_redirects=False,
                timeout=fetch_timeout_seconds,
                trust_env=False,
            ) as anonymous:
                for plan in plans:
                    if site.page.get(plan["slug"], raise_when_not_found=False) is not None:
                        raise RuntimeError(f"create-only preflight found an existing page: {plan['slug']}")
                    append_ledger(
                        ledger,
                        {
                            "event": "create-intent",
                            "slug": plan["slug"],
                            "title": plan["title"],
                            "source_sha256": plan["source_sha256"],
                        },
                    )
                    site.page.create(
                        plan["slug"],
                        title=plan["title"],
                        source=plan["source"],
                        comment="run-owned FTML differential capture",
                    )
                    snapshot = saved_snapshot(site, plan)
                    created.append(snapshot)
                    append_ledger(
                        ledger,
                        {
                            "event": "created",
                            **{key: value for key, value in snapshot.items() if key != "saved_source"},
                        },
                    )
                    verify_saved_markers(plan, snapshot)
                    record = {
                        "schema": CAPTURE_SCHEMA,
                        "captured_at": datetime.now(UTC).isoformat(),
                        "page_plan": plan,
                        "site": site.unix_name,
                        "domain": site.domain,
                        "authenticated_capture": False,
                        "mutated": True,
                        "page_identity": snapshot["identity"],
                        "saved_source": snapshot["saved_source"],
                        "saved_source_sha256": snapshot["saved_source_sha256"],
                        "source_normalized": snapshot["saved_source_sha256"] != plan["source_sha256"],
                    }
                    try:
                        page_content, raw_html = fetch_page_content(
                            anonymous,
                            plan["slug"],
                            fetch_attempts,
                        )
                    except (httpx.HTTPError, RuntimeError) as error:
                        records.append(
                            {
                                **record,
                                "capture_status": "render-failed",
                                "render_error": type(error).__name__,
                            }
                        )
                    else:
                        records.append(
                            {
                                **record,
                                "capture_status": "captured",
                                "page_content_html": page_content,
                                "page_content_html_sha256": sha256(page_content),
                                "raw_page_html": raw_html,
                                "raw_page_html_sha256": sha256(raw_html),
                            }
                        )
        finally:
            for snapshot in reversed(created):
                try:
                    remove_exact(site, snapshot)
                    append_ledger(
                        ledger,
                        {
                            "event": "removed",
                            **{key: value for key, value in snapshot.items() if key != "saved_source"},
                        },
                    )
                except Exception as error:
                    cleanup_error = cleanup_error or error
            if cleanup_error is not None:
                raise cleanup_error
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x", encoding="utf-8") as result:
        for record in records:
            result.write(json.dumps(record, ensure_ascii=False, sort_keys=True, separators=(",", ":")))
            result.write("\n")
    return records


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pages", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--fetch-timeout-seconds", type=float, default=30.0)
    parser.add_argument("--fetch-attempts", type=int, default=5)
    return parser.parse_args()


def main() -> int:
    def interrupt(signum: int, frame: object) -> None:
        raise InterruptedError(f"capture interrupted by signal {signum}")

    signal.signal(signal.SIGINT, interrupt)
    signal.signal(signal.SIGTERM, interrupt)
    args = parse_args()
    if args.fetch_timeout_seconds <= 0 or args.fetch_attempts <= 0:
        raise ValueError("fetch timeout and attempts must be positive")
    plans = load_plans(args.pages)
    records = capture(
        plans,
        output=args.output,
        ledger=args.ledger,
        fetch_timeout_seconds=args.fetch_timeout_seconds,
        fetch_attempts=args.fetch_attempts,
    )
    captured = sum(record["capture_status"] == "captured" for record in records)
    failed = len(records) - captured
    print(
        json.dumps(
            {
                "captured": captured,
                "render_failed": failed,
                "mutated_pages": len(records),
                "residual_pages": 0,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
