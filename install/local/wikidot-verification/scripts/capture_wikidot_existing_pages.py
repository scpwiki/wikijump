#!/usr/bin/env python3
"""Capture immutable identities and rendered fragments from existing Wikidot pages."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

PLAN_SCHEMA = "wikijump_syntax_differential.saved_page_plan.v1"
REFERENCE_SCHEMA = "wikijump_syntax_differential.wikidot_saved_page_reference.v1"
ALLOWED_SITES = {"scp-wiki", "scp-jp", "sandbox-for-codex"}
SLUG_PATTERN = re.compile(r"^[a-z0-9][a-z0-9:_-]*$")
REQUIREMENTS_PATH = Path(__file__).parents[1] / "requirements.txt"
REQUIREMENTS_LOCK_PATH = Path(__file__).parents[1] / "requirements.lock"
WIKIDOT_PIN = re.compile(r"Rokurolize/wikidot\.py@([0-9a-f]{40})")


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def pinned_dependency_identity(
    requirements_path: Path = REQUIREMENTS_PATH,
    requirements_lock_path: Path = REQUIREMENTS_LOCK_PATH,
) -> dict[str, str]:
    requirements = requirements_path.read_text()
    requirements_lock = requirements_lock_path.read_text()
    requirement_match = WIKIDOT_PIN.search(requirements)
    lock_match = WIKIDOT_PIN.search(requirements_lock)
    if requirement_match is None or lock_match is None:
        raise RuntimeError("Python dependency files must pin Rokurolize/wikidot.py to a full commit")
    if requirement_match.group(1) != lock_match.group(1):
        raise RuntimeError("requirements.txt and requirements.lock pin different wikidot.py commits")
    return {
        "wikidot_py_commit": requirement_match.group(1),
        "requirements_sha256": hashlib.sha256(requirements_path.read_bytes()).hexdigest(),
        "requirements_lock_sha256": hashlib.sha256(requirements_lock_path.read_bytes()).hexdigest(),
    }


def validate_plan(value: object) -> dict[str, Any]:
    if not isinstance(value, dict) or value.get("schema") != PLAN_SCHEMA:
        raise ValueError("saved-page plan schema is unsupported")
    if not isinstance(value.get("case_id"), str) or not value["case_id"]:
        raise ValueError("saved-page plan case_id is invalid")
    if value.get("site") not in ALLOWED_SITES:
        raise ValueError(f"saved-page plan site is outside the read allowlist: {value.get('site')}")
    if not isinstance(value.get("slug"), str) or SLUG_PATTERN.fullmatch(value["slug"]) is None:
        raise ValueError(f"saved-page plan slug is invalid: {value.get('slug')}")
    selector = value.get("selector")
    if not isinstance(selector, str) or not selector.startswith(".") or len(selector) < 2:
        raise ValueError(f"saved-page plan selector must be one class selector: {value['case_id']}")
    expected = value.get("expected")
    if not isinstance(expected, dict):
        raise ValueError(f"saved-page plan expected contract is invalid: {value['case_id']}")
    for field in ("required_class_tokens", "forbidden_literals"):
        items = expected.get(field)
        if not isinstance(items, list) or any(not isinstance(item, str) or not item for item in items):
            raise ValueError(f"saved-page plan expected {field} is invalid: {value['case_id']}")
    if not expected["required_class_tokens"]:
        raise ValueError(f"saved-page plan expected required_class_tokens is empty: {value['case_id']}")
    return value


def load_plans(path: Path) -> list[dict[str, Any]]:
    plans = [validate_plan(json.loads(line)) for line in path.read_text().splitlines() if line.strip()]
    case_ids = [plan["case_id"] for plan in plans]
    if not plans or len(case_ids) != len(set(case_ids)):
        raise ValueError("saved-page plans must have unique non-empty case IDs")
    return plans


def capture(plans: list[dict[str, Any]]) -> list[dict[str, Any]]:
    from bs4 import BeautifulSoup
    import httpx
    import wikidot

    dependency_identity = pinned_dependency_identity()
    records = []
    with wikidot.Client() as client, httpx.Client(
        follow_redirects=False, timeout=30.0, trust_env=False
    ) as anonymous:
        for plan in plans:
            site = client.site.get(plan["site"])
            page = site.page.get(plan["slug"], raise_when_not_found=False)
            if page is None:
                raise RuntimeError(f"Wikidot page does not exist: {plan['site']}:{plan['slug']}")
            page.refresh_source()
            source = page.source.wiki_text
            revision = page.latest_revision
            response = anonymous.get(f"https://{site.domain}/{plan['slug']}")
            response.raise_for_status()
            document = BeautifulSoup(response.text, "lxml")
            page_content = document.select_one("#page-content")
            if page_content is None:
                raise RuntimeError(f"Wikidot page has no #page-content: {plan['case_id']}")
            selected = page_content.select(plan["selector"])
            if len(selected) != 1:
                raise RuntimeError(
                    f"Wikidot selector returned {len(selected)} nodes for {plan['case_id']}"
                )
            captured_at = datetime.now(UTC).isoformat()
            fragment = str(selected[0])
            page_content_html = str(page_content)
            records.append(
                {
                    "schema": REFERENCE_SCHEMA,
                    "case": plan,
                    "captured_at": captured_at,
                    "actor": {"authenticated": False},
                    "site": {"unix_name": site.unix_name, "domain": site.domain},
                    "page": {
                        "slug": page.fullname,
                        "identity": page.id,
                        "title": page.title,
                        "revision_identity": revision.id,
                        "revision_number": revision.rev_no,
                        "source_wikitext": source,
                        "source_sha256": sha256(source),
                    },
                    "page_content_html": page_content_html,
                    "page_content_html_sha256": sha256(page_content_html),
                    "selected_html": fragment,
                    "selected_html_sha256": sha256(fragment),
                    "provenance": {
                        "transport": "anonymous-https",
                        "mutated": False,
                        "wikidot_py_version": wikidot.__version__,
                        **dependency_identity,
                    },
                }
            )
    return records


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plans", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(f"frozen Wikidot reference already exists: {args.output}")
    records = capture(load_plans(args.plans))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x", encoding="utf-8") as output:
        for record in records:
            output.write(json.dumps(record, ensure_ascii=False, sort_keys=True, separators=(",", ":")))
            output.write("\n")
    print(json.dumps({"captured": len(records), "mutated_pages": 0, "output": str(args.output)}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
