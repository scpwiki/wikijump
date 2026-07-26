#!/usr/bin/env python3
"""Freeze anonymous Wikidot PagePreviewModule results for syntax cases."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

CASE_SCHEMA = "wikijump_syntax_differential.syntax_case.v1"
LIVE_CASE_SCHEMA = "wikijump_syntax_differential.live_case.v1"
REFERENCE_SCHEMA = "wikijump_syntax_differential.wikidot_reference.v1"
PAGE_PREVIEW_TIER = "page-preview"
LOCAL_EXECUTION_TIERS = {"ftml", "wikijump-runtime"}
REQUIREMENTS_PATH = Path(__file__).parents[1] / "requirements.txt"


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def validate_case(value: object, live_execution_classes: set[str] | None = None) -> dict[str, str] | None:
    if not isinstance(value, dict):
        raise ValueError("syntax case must be an object")
    if value.get("schema") == LIVE_CASE_SCHEMA:
        selected_classes = live_execution_classes or {"page-preview-isolated"}
        execution_class = value.get("execution_class")
        if execution_class not in selected_classes:
            return None
        value = {
            "schema": CASE_SCHEMA,
            "case_id": value.get("case_id"),
            "source": value.get("source"),
            "title": value.get("case_id"),
            "wikidot_observation_tier": PAGE_PREVIEW_TIER,
            "local_execution_tier": (
                "wikijump-runtime"
                if execution_class == "wikijump-runtime"
                else "ftml"
            ),
        }
    required = {"schema", "case_id", "source", "wikidot_observation_tier", "local_execution_tier"}
    missing = required - value.keys()
    if missing:
        raise ValueError(f"syntax case is missing {sorted(missing)}")
    if value["schema"] != CASE_SCHEMA:
        raise ValueError("syntax case schema is unsupported")
    case_id = value["case_id"]
    source = value["source"]
    observation_tier = value["wikidot_observation_tier"]
    local_execution_tier = value["local_execution_tier"]
    title = value.get("title", case_id)
    if not isinstance(case_id, str) or not case_id:
        raise ValueError("syntax case case_id is invalid")
    if not isinstance(source, str):
        raise ValueError(f"syntax case {case_id} source is invalid")
    if observation_tier != PAGE_PREVIEW_TIER:
        raise ValueError(f"syntax case {case_id} requires unsupported Wikidot observation tier {observation_tier!r}")
    if local_execution_tier not in LOCAL_EXECUTION_TIERS:
        raise ValueError(f"syntax case {case_id} requires unsupported local execution tier {local_execution_tier!r}")
    if not isinstance(title, str) or not title:
        raise ValueError(f"syntax case {case_id} title is invalid")
    return {
        "schema": CASE_SCHEMA,
        "case_id": case_id,
        "source": source,
        "wikidot_observation_tier": PAGE_PREVIEW_TIER,
        "local_execution_tier": local_execution_tier,
        "title": title,
    }


def load_cases(path: Path, live_execution_classes: set[str] | None = None) -> list[dict[str, str]]:
    cases = [
        case
        for line in path.read_text().split("\n")
        if line.strip()
        if (case := validate_case(json.loads(line), live_execution_classes)) is not None
    ]
    case_ids = [case["case_id"] for case in cases]
    if len(case_ids) != len(set(case_ids)):
        raise ValueError("syntax case IDs must be unique")
    if not cases:
        raise ValueError("at least one syntax case is required")
    return cases


def preview_body(case: dict[str, str]) -> dict[str, str]:
    return {
        "moduleName": "edit/PagePreviewModule",
        "mode": "page",
        "source": case["source"],
        "title": case["title"],
    }


def reference_record(
    case: dict[str, str],
    body: str,
    *,
    site: str,
    captured_at: str,
    site_domain: str,
    wikidot_version: str,
    wikidot_commit: str,
    requirements_sha256: str,
) -> dict[str, Any]:
    return {
        "schema": REFERENCE_SCHEMA,
        "syntax_case": case,
        "source_sha256": sha256(case["source"]),
        "captured_at": captured_at,
        "provenance": {
            "site": site,
            "site_domain": site_domain,
            "module": "edit/PagePreviewModule",
            "wikidot_py_version": wikidot_version,
            "wikidot_py_commit": wikidot_commit,
            "requirements_sha256": requirements_sha256,
            "authenticated": False,
            "mutated": False,
        },
        "raw_html": body,
        "raw_html_sha256": sha256(body),
    }


def acquire(cases: list[dict[str, str]], *, site_name: str, batch_size: int) -> list[dict[str, Any]]:
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
            [preview_body(case) for case in cases],
            batch_size=batch_size,
            max_retries=3,
        )
    records = []
    for case, response in zip(cases, responses, strict=True):
        if response is None:
            raise RuntimeError(f"Wikidot preview retries were exhausted for {case['case_id']}")
        data = response.json()
        body = data.get("body")
        if not isinstance(body, str):
            raise RuntimeError(f"Wikidot preview returned no HTML body for {case['case_id']}")
        records.append(
            reference_record(
                case,
                body,
                site=site.unix_name,
                site_domain=site.domain,
                wikidot_version=wikidot.__version__,
                wikidot_commit=commit_match.group(1),
                requirements_sha256=requirements_sha256,
                captured_at=captured_at,
            )
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
    parser.add_argument("--cases", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--site", default="sandbox-for-codex")
    parser.add_argument("--batch-size", type=int, default=4)
    parser.add_argument(
        "--live-execution-class",
        action="append",
        choices=["saved-page-batch", "page-preview-isolated", "wikijump-runtime"],
    )
    args = parser.parse_args()
    if args.batch_size <= 0:
        parser.error("--batch-size must be positive")
    return args


def main() -> int:
    args = parse_args()
    if args.output.exists():
        raise FileExistsError(f"frozen Wikidot reference already exists: {args.output}")
    execution_classes = set(args.live_execution_class) if args.live_execution_class else None
    cases = load_cases(args.cases, execution_classes)
    records = acquire(cases, site_name=args.site, batch_size=args.batch_size)
    write_frozen(args.output, records)
    print(json.dumps({"cases": len(records), "output": str(args.output)}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
