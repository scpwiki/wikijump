#!/usr/bin/env python3
"""Render FTML differential page plans through anonymous Wikidot previews."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

PAGE_PLAN_SCHEMA = "wikijump_syntax_differential.wikidot_page_plan.v1"
CAPTURE_SCHEMA = "wikijump_syntax_differential.wikidot_preview_page_capture.v1"
REQUIREMENTS_PATH = Path(__file__).parents[1] / "requirements.txt"


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def validate_plan(value: object) -> dict[str, Any]:
    if not isinstance(value, dict) or value.get("schema") != PAGE_PLAN_SCHEMA:
        raise ValueError("page plan schema is unsupported")
    source = value.get("source")
    if not isinstance(source, str) or value.get("source_sha256") != sha256(source):
        raise ValueError("page plan source identity is invalid")
    cases = value.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError("page plan has no cases")
    return value


def load_plans(path: Path) -> list[dict[str, Any]]:
    plans = [validate_plan(json.loads(line)) for line in path.read_text().split("\n") if line.strip()]
    if not plans:
        raise ValueError("at least one page plan is required")
    return plans


def preview_body(plan: dict[str, Any]) -> dict[str, str]:
    return {
        "moduleName": "edit/PagePreviewModule",
        "mode": "page",
        "source": plan["source"],
        "title": plan["title"],
    }


def acquire(plans: list[dict[str, Any]], *, site_name: str, batch_size: int) -> list[dict[str, Any]]:
    import wikidot

    requirements = REQUIREMENTS_PATH.read_text()
    commit_match = re.search(r"Rokurolize/wikidot\.py@([0-9a-f]{40})", requirements)
    if commit_match is None:
        raise RuntimeError("requirements.txt does not pin Rokurolize/wikidot.py to a full commit")
    requirements_sha256 = hashlib.sha256(REQUIREMENTS_PATH.read_bytes()).hexdigest()
    captured_at = datetime.now(UTC).isoformat()
    with wikidot.Client() as client:
        site = client.site.get(site_name)
        responses = site.amc_request_with_retry(
            [preview_body(plan) for plan in plans],
            batch_size=batch_size,
            max_retries=3,
        )
    records = []
    for plan, response in zip(plans, responses, strict=True):
        if response is None:
            raise RuntimeError(f"Wikidot preview retries were exhausted for page {plan['page_index']}")
        body = response.json().get("body")
        if not isinstance(body, str):
            raise RuntimeError(f"Wikidot preview returned no HTML body for page {plan['page_index']}")
        records.append(
            {
                "schema": CAPTURE_SCHEMA,
                "captured_at": captured_at,
                "page_plan": plan,
                "site": site.unix_name,
                "domain": site.domain,
                "authenticated_capture": False,
                "mutated": False,
                "page_identity": None,
                "page_content_html": body,
                "page_content_html_sha256": sha256(body),
                "provenance": {
                    "module": "edit/PagePreviewModule",
                    "wikidot_py_version": wikidot.__version__,
                    "wikidot_py_commit": commit_match.group(1),
                    "requirements_sha256": requirements_sha256,
                },
            }
        )
    return records


def write_frozen(path: Path, records: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8") as output:
        for record in records:
            output.write(json.dumps(record, ensure_ascii=False, sort_keys=True, separators=(",", ":")))
            output.write("\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pages", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--site", default="sandbox-for-codex")
    parser.add_argument("--batch-size", type=int, default=4)
    args = parser.parse_args()
    if args.batch_size <= 0:
        parser.error("--batch-size must be positive")
    return args


def main() -> int:
    args = parse_args()
    if args.output.exists():
        raise FileExistsError(f"frozen Wikidot capture already exists: {args.output}")
    plans = load_plans(args.pages)
    records = acquire(plans, site_name=args.site, batch_size=args.batch_size)
    write_frozen(args.output, records)
    print(json.dumps({"pages": len(records), "mutated_pages": 0, "output": str(args.output)}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
