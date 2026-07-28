#!/usr/bin/env python3
"""Capture a run-owned, multi-page ListPages fixture graph from live Wikidot."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import signal
import time
from contextlib import ExitStack
from datetime import UTC, datetime
from pathlib import Path
from typing import Any
from urllib.parse import quote

PLAN_SCHEMA = "wikijump_listpages_compat.live_fixture_plan.v1"
CAPTURE_SCHEMA = "wikijump_listpages_compat.live_fixture_capture.v1"
ALLOWED_SITE = "sandbox-for-codex"
ALLOWED_DOMAIN = f"{ALLOWED_SITE}.wikidot.com"
ALLOWED_ORIGIN = f"http://{ALLOWED_DOMAIN}"
RUN_OWNED_SLUG = re.compile(
    r"^run-owned:lp-campaign-[0-9]{8}-[a-z0-9][a-z0-9-]*$"
)
ACCOUNT_LABELS = {"A", "B", "C"}
REQUIREMENTS_PATH = Path(__file__).parents[1] / "requirements.txt"


class InterruptFlag:
    def __init__(self) -> None:
        self.signum: int | None = None

    def request(self, signum: int, _frame: object) -> None:
        if self.signum is None:
            self.signum = signum

    def raise_if_requested(self) -> None:
        if self.signum is not None:
            raise InterruptedError(f"capture interrupted by signal {self.signum}")


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def append_jsonl(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as output:
        output.write(
            json.dumps(
                value,
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            )
        )
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())


def validate_plan(value: object) -> dict[str, Any]:
    if not isinstance(value, dict) or value.get("schema") != PLAN_SCHEMA:
        raise ValueError("ListPages live fixture plan schema is unsupported")
    pages = value.get("pages")
    captures = value.get("captures")
    if not isinstance(pages, list) or not pages:
        raise ValueError("ListPages live fixture plan has no pages")
    if not isinstance(captures, list) or not captures:
        raise ValueError("ListPages live fixture plan has no captures")

    page_keys: set[str] = set()
    fullnames: set[str] = set()
    for page in pages:
        if not isinstance(page, dict):
            raise ValueError("ListPages live fixture page is not an object")
        key = page.get("key")
        fullname = page.get("fullname")
        title = page.get("title")
        account = page.get("account")
        sources = page.get("sources")
        tags = page.get("tags", [])
        parent = page.get("parent")
        votes = page.get("votes", [])
        delay_after_seconds = page.get("delay_after_seconds", 0)
        if not isinstance(key, str) or not key or key in page_keys:
            raise ValueError("ListPages live fixture page key is invalid or duplicated")
        if (
            not isinstance(fullname, str)
            or RUN_OWNED_SLUG.fullmatch(fullname) is None
            or fullname in fullnames
        ):
            raise ValueError(f"ListPages live fixture fullname is invalid: {fullname!r}")
        if not isinstance(title, str) or not title or len(title) > 200:
            raise ValueError(f"ListPages live fixture title is invalid for {key}")
        if account not in ACCOUNT_LABELS:
            raise ValueError(f"ListPages live fixture account is invalid for {key}")
        if (
            not isinstance(sources, list)
            or not sources
            or any(not isinstance(source, str) for source in sources)
        ):
            raise ValueError(f"ListPages live fixture sources are invalid for {key}")
        if (
            not isinstance(tags, list)
            or any(not isinstance(tag, str) or not tag for tag in tags)
        ):
            raise ValueError(f"ListPages live fixture tags are invalid for {key}")
        if parent is not None and parent not in page_keys:
            raise ValueError(
                f"ListPages live fixture parent must precede its child for {key}"
            )
        if not isinstance(votes, list) or any(
            not isinstance(vote, dict)
            or vote.get("account") not in ACCOUNT_LABELS
            or vote.get("value") not in {-1, 1}
            for vote in votes
        ):
            raise ValueError(f"ListPages live fixture votes are invalid for {key}")
        if (
            not isinstance(delay_after_seconds, int | float)
            or isinstance(delay_after_seconds, bool)
            or delay_after_seconds < 0
            or delay_after_seconds > 5
        ):
            raise ValueError(
                f"ListPages live fixture creation delay is invalid for {key}"
            )
        page_keys.add(key)
        fullnames.add(fullname)

    case_ids: set[str] = set()
    for capture in captures:
        if not isinstance(capture, dict):
            raise ValueError("ListPages live fixture capture is not an object")
        case_id = capture.get("case_id")
        page = capture.get("page")
        suffix = capture.get("url_suffix", "")
        if (
            not isinstance(case_id, str)
            or not case_id
            or case_id in case_ids
            or page not in page_keys
        ):
            raise ValueError("ListPages live fixture capture identity is invalid")
        if (
            not isinstance(suffix, str)
            or (suffix and not suffix.startswith(("/", "?")))
            or "://" in suffix
            or "#" in suffix
            or any(character in suffix for character in "\r\n")
        ):
            raise ValueError(f"ListPages live fixture URL suffix is invalid: {suffix!r}")
        case_ids.add(case_id)
    return value


def load_plan(path: Path) -> dict[str, Any]:
    return validate_plan(json.loads(path.read_text(encoding="utf-8")))


def account_credentials(label: str) -> tuple[str, str]:
    username = os.environ.pop(f"WIKIDOT_{label}_USERNAME", None)
    password = os.environ.pop(f"WIKIDOT_{label}_PASSWORD", None)
    if not username or not password:
        raise RuntimeError(f"Wikidot sandbox account {label} is required")
    return username, password


def source_matches(actual: str, expected: str) -> bool:
    return actual == expected or (expected.endswith("\n") and actual == expected[:-1])


def snapshot_page(site: Any, plan_page: dict[str, Any]) -> dict[str, Any]:
    page = site.page.get(plan_page["fullname"], raise_when_not_found=False)
    if page is None:
        raise RuntimeError(f"created page is not visible: {plan_page['fullname']}")
    page.refresh_source()
    actual_source = page.source.wiki_text
    expected_source = plan_page["sources"][-1]
    if page.title != plan_page["title"] or not source_matches(
        actual_source, expected_source
    ):
        raise RuntimeError(
            f"created page did not round-trip its final title and source: {plan_page['fullname']}"
        )
    return {
        "key": plan_page["key"],
        "fullname": plan_page["fullname"],
        "identity": page.id,
        "title": page.title,
        "source": actual_source,
        "source_sha256": sha256(actual_source),
        "requested_source_sha256": sha256(expected_source),
        "source_normalized": actual_source != expected_source,
        "account": plan_page["account"],
        "tags": sorted(plan_page.get("tags", [])),
        "parent": plan_page.get("parent"),
        "revision_count_requested": len(plan_page["sources"]),
        "votes_requested": plan_page.get("votes", []),
    }


def remove_exact(site: Any, snapshot: dict[str, Any]) -> None:
    page = site.page.get(snapshot["fullname"], raise_when_not_found=False)
    if page is None:
        return
    page.refresh_source()
    if (
        page.id != snapshot["identity"]
        or page.title != snapshot["title"]
        or sha256(page.source.wiki_text) != snapshot["source_sha256"]
    ):
        raise RuntimeError(f"cleanup refused a changed page: {snapshot['fullname']}")
    page.destroy()
    for _ in range(5):
        if site.page.get(snapshot["fullname"], raise_when_not_found=False) is None:
            return
        time.sleep(0.4)
    raise RuntimeError(f"cleanup did not remove page: {snapshot['fullname']}")


def fetch_page_content(
    anonymous: Any,
    url: str,
    attempts: int,
) -> tuple[str | None, str, int]:
    from bs4 import BeautifulSoup

    last_response = None
    for attempt in range(attempts):
        response = anonymous.get(url)
        last_response = response
        if response.status_code == 200:
            page_content = BeautifulSoup(response.text, "lxml").select_one(
                "#page-content"
            )
            if page_content is not None:
                return str(page_content), response.text, response.status_code
        if attempt + 1 < attempts:
            time.sleep(0.4)
    if last_response is None:
        raise RuntimeError("anonymous page fetch returned no response")
    return None, last_response.text, last_response.status_code


def capture(
    plan: dict[str, Any],
    *,
    output: Path,
    ledger: Path,
    fetch_timeout_seconds: float,
    fetch_attempts: int,
    interrupt: InterruptFlag,
) -> list[dict[str, Any]]:
    import httpx
    import wikidot
    from wikidot.connector.ajax import AjaxModuleConnectorConfig

    if output.exists() or ledger.exists():
        raise FileExistsError("capture output and ledger must not already exist")
    required_labels = {
        page["account"] for page in plan["pages"]
    } | {
        vote["account"]
        for page in plan["pages"]
        for vote in page.get("votes", [])
    }
    required_labels.add("A")
    credentials = {
        label: account_credentials(label) for label in sorted(required_labels)
    }
    requirements_sha256 = hashlib.sha256(REQUIREMENTS_PATH.read_bytes()).hexdigest()
    commit_match = re.search(
        r"Rokurolize/wikidot\.py@([0-9a-f]{40})",
        REQUIREMENTS_PATH.read_text(encoding="utf-8"),
    )
    if commit_match is None:
        raise RuntimeError("requirements.txt does not pin Wikidot.py to a full commit")

    config = AjaxModuleConnectorConfig(
        allow_insecure_session_transport_for=ALLOWED_SITE
    )
    snapshots: list[dict[str, Any]] = []
    records: list[dict[str, Any]] = []
    cleanup_error: Exception | None = None
    output.parent.mkdir(parents=True, exist_ok=True)
    with ExitStack() as stack:
        clients = {
            label: stack.enter_context(
                wikidot.Client(
                    username=username,
                    password=password,
                    amc_config=config,
                )
            )
            for label, (username, password) in credentials.items()
        }
        sites = {label: client.site.get(ALLOWED_SITE) for label, client in clients.items()}
        if any(
            site.unix_name != ALLOWED_SITE or site.domain != ALLOWED_DOMAIN
            for site in sites.values()
        ):
            raise RuntimeError("resolved Wikidot site is outside the exact allowlist")
        admin_site = sites["A"]
        try:
            for page_plan in plan["pages"]:
                if (
                    admin_site.page.get(
                        page_plan["fullname"], raise_when_not_found=False
                    )
                    is not None
                ):
                    raise RuntimeError(
                        f"create-only preflight found an existing page: {page_plan['fullname']}"
                    )

            snapshots_by_key: dict[str, dict[str, Any]] = {}
            for page_plan in plan["pages"]:
                interrupt.raise_if_requested()
                append_jsonl(
                    ledger,
                    {
                        "event": "create-intent",
                        "key": page_plan["key"],
                        "fullname": page_plan["fullname"],
                        "source_sha256": sha256(page_plan["sources"][-1]),
                    },
                )
                site = sites[page_plan["account"]]
                parent_key = page_plan.get("parent")
                parent_fullname = (
                    snapshots_by_key[parent_key]["fullname"] if parent_key else None
                )
                try:
                    publish_arguments = {
                        "fullname": page_plan["fullname"],
                        "title": page_plan["title"],
                        "source": page_plan["sources"][0],
                        "comment": "run-owned ListPages compatibility fixture",
                        "tags": page_plan.get("tags", []),
                        "post_save_visibility_attempts": 5,
                        "post_save_visibility_interval": 0.4,
                    }
                    if parent_fullname is not None:
                        publish_arguments["parent_fullname"] = parent_fullname
                    result = site.page.publish(
                        **publish_arguments
                    )
                    page = result.page
                    for revision_source in page_plan["sources"][1:]:
                        page = page.edit(
                            title=page_plan["title"],
                            source=revision_source,
                            comment="run-owned ListPages compatibility fixture revision",
                        )
                except Exception:
                    recovered = admin_site.page.get(
                        page_plan["fullname"], raise_when_not_found=False
                    )
                    if recovered is not None:
                        recovered.refresh_source()
                        snapshots.append(
                            {
                                "key": page_plan["key"],
                                "fullname": page_plan["fullname"],
                                "identity": recovered.id,
                                "title": recovered.title,
                                "source": recovered.source.wiki_text,
                                "source_sha256": sha256(recovered.source.wiki_text),
                            }
                        )
                    raise
                snapshot = snapshot_page(admin_site, page_plan)
                snapshots.append(snapshot)
                snapshots_by_key[page_plan["key"]] = snapshot
                append_jsonl(
                    ledger,
                    {
                        "event": "created",
                        **{
                            key: value
                            for key, value in snapshot.items()
                            if key != "source"
                        },
                    },
                )
                if page_plan.get("delay_after_seconds", 0) > 0:
                    time.sleep(page_plan["delay_after_seconds"])

            for page_plan in plan["pages"]:
                for vote in page_plan.get("votes", []):
                    interrupt.raise_if_requested()
                    sites[vote["account"]].page.get(page_plan["fullname"]).vote(
                        vote["value"]
                    )
                    append_jsonl(
                        ledger,
                        {
                            "event": "voted",
                            "key": page_plan["key"],
                            "account": vote["account"],
                            "value": vote["value"],
                        },
                    )

            with httpx.Client(
                follow_redirects=False,
                timeout=fetch_timeout_seconds,
                trust_env=False,
            ) as anonymous:
                for capture_plan in plan["captures"]:
                    interrupt.raise_if_requested()
                    snapshot = snapshots_by_key[capture_plan["page"]]
                    url = (
                        f"{ALLOWED_ORIGIN}/"
                        f"{quote(snapshot['fullname'], safe=':')}"
                        f"{capture_plan.get('url_suffix', '')}"
                    )
                    page_content, raw_html, status = fetch_page_content(
                        anonymous,
                        url,
                        fetch_attempts,
                    )
                    record = {
                        "schema": CAPTURE_SCHEMA,
                        "captured_at": datetime.now(UTC).isoformat(),
                        "case": capture_plan,
                        "request": {
                            "url": url,
                            "method": "GET",
                            "authenticated": False,
                            "status": status,
                        },
                        "site": {
                            "unix_name": ALLOWED_SITE,
                            "domain": ALLOWED_DOMAIN,
                        },
                        "fixture_graph": [
                            {
                                key: value
                                for key, value in fixture.items()
                                if key != "source"
                            }
                            for fixture in snapshots
                        ],
                        "provenance": {
                            "mutated": True,
                            "wikidot_py_version": wikidot.__version__,
                            "wikidot_py_commit": commit_match.group(1),
                            "requirements_sha256": requirements_sha256,
                            "plan_sha256": hashlib.sha256(
                                json.dumps(
                                    plan,
                                    ensure_ascii=False,
                                    sort_keys=True,
                                    separators=(",", ":"),
                                ).encode("utf-8")
                            ).hexdigest(),
                        },
                        "capture_status": (
                            "captured"
                            if page_content is not None
                            else "no-page-content"
                        ),
                        "raw_page_html": raw_html,
                        "raw_page_html_sha256": sha256(raw_html),
                    }
                    if page_content is not None:
                        record["page_content_html"] = page_content
                        record["page_content_html_sha256"] = sha256(page_content)
                    append_jsonl(output, record)
                    records.append(record)
        finally:
            for snapshot in reversed(snapshots.copy()):
                try:
                    remove_exact(admin_site, snapshot)
                    append_jsonl(
                        ledger,
                        {
                            "event": "removed",
                            "key": snapshot["key"],
                            "fullname": snapshot["fullname"],
                            "identity": snapshot["identity"],
                        },
                    )
                except Exception as error:
                    cleanup_error = cleanup_error or error
            if cleanup_error is not None:
                raise cleanup_error
    interrupt.raise_if_requested()
    return records


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--fetch-timeout-seconds", type=float, default=30.0)
    parser.add_argument("--fetch-attempts", type=int, default=5)
    return parser.parse_args()


def main() -> int:
    interrupt = InterruptFlag()
    signal.signal(signal.SIGINT, interrupt.request)
    signal.signal(signal.SIGTERM, interrupt.request)
    args = parse_args()
    if args.fetch_timeout_seconds <= 0 or args.fetch_attempts <= 0:
        raise ValueError("fetch timeout and attempts must be positive")
    records = capture(
        load_plan(args.plan),
        output=args.output,
        ledger=args.ledger,
        fetch_timeout_seconds=args.fetch_timeout_seconds,
        fetch_attempts=args.fetch_attempts,
        interrupt=interrupt,
    )
    print(
        json.dumps(
            {
                "captured": len(records),
                "mutated_pages": len(
                    json.loads(args.plan.read_text(encoding="utf-8"))["pages"]
                ),
                "residual_pages": 0,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
