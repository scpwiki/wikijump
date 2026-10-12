#!/usr/bin/env python3
"""Explicit read-only discovery; not an ordinary regression or CI acquisition."""

import argparse
import asyncio
import fcntl
import hashlib
import json
import logging
import os
import re
import subprocess
import time
from pathlib import Path
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
from urllib.parse import urlsplit
import httpx
import wikidot
from bs4 import BeautifulSoup
from wikidot.connector.ajax import AjaxModuleConnectorConfig
from wikidot.module.site import Site
from wikidot.module.forum_thread import ForumThreadCollection

parser = argparse.ArgumentParser(
    description="Acquire anonymous discussion identity metadata through the local Wikidot client. Existing complete responses are replayed; unsealed barriers forbid refetch."
)
parser.add_argument("--source-document", required=True, type=Path)
parser.add_argument("--source-url", required=True)
parser.add_argument("--output-dir", required=True, type=Path)
parser.add_argument("--private-response-cache", type=Path)
args = parser.parse_args()
OUT = args.output_dir
OUT.mkdir(mode=0o700, parents=True, exist_ok=True)
CACHE = args.private_response_cache or OUT / "private-anonymous-amc-cache"
CACHE.mkdir(mode=0o700, parents=True, exist_ok=True)
source = args.source_document.read_bytes()
source_url = urlsplit(args.source_url)
site_host = source_url.hostname
if (
    source_url.scheme != "https"
    or not site_host
    or not site_host.endswith(".wikidot.com")
    or source_url.username
    or source_url.password
    or source_url.query
    or source_url.fragment
):
    raise RuntimeError("Expected one credential-free HTTPS Wikidot thread reference")
site_name = site_host.removesuffix(".wikidot.com")
site_match = re.search(rb"WIKIREQUEST\.info\.siteId\s*=\s*(\d+)", source)
if not site_match:
    raise RuntimeError("Retained document has no source site identity")
site_id = int(site_match.group(1))
html = BeautifulSoup(source, "lxml")
head = html.select_one("#header h1")
site_title = head.get_text(" ", strip=True) if head else site_name
fork = next(
    (p for p in Path(wikidot.__file__).resolve().parents if (p / ".git").exists()), None
)
if fork is None:
    raise RuntimeError("Use the repository-pinned local Wikidot client environment")
commit = subprocess.check_output(
    ["git", "-C", str(fork), "rev-parse", "HEAD"], text=True
).strip()
if subprocess.check_output(
    ["git", "-C", str(fork), "status", "--porcelain"], text=True
).strip():
    raise RuntimeError("The Wikidot client checkout is dirty; acquisition refused")
plan = {
    "schema": "wikijump.source_discussion_acquisition_plan.v1",
    "source_document_sha256": hashlib.sha256(source).hexdigest(),
    "source_url": args.source_url,
    "source_site_id": site_id,
    "client_commit": commit,
    "actor": "anonymous",
    "operation": "ForumThread.posts high-level read; only ForumViewThreadPostsModule allowed; no action/event; attempt_limit=1; private response cache with durable barriers",
}


def durable(path, data):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, "wb") as file:
        file.write(data)
        file.flush()
        os.fsync(file.fileno())
    fd = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


plan_path = OUT / "plan.json"
if plan_path.exists():
    if json.loads(plan_path.read_text()) != plan:
        raise RuntimeError("Output directory belongs to a different observation")
else:
    durable(plan_path, json.dumps(plan, indent=2).encode())
original_post = httpx.AsyncClient.post
requests = []
fill_lock = asyncio.Lock()
retry_until = 0.0


async def cached_read(self, url, **kwargs):
    global retry_until
    data = kwargs.get("data", {})
    u = urlsplit(str(url))
    if (
        u.scheme != "https"
        or u.hostname != site_host
        or u.path != "/ajax-module-connector.php"
        or u.query
        or u.fragment
        or u.username
        or u.password
        or u.port not in (None, 443)
    ):
        raise RuntimeError("HTTP endpoint outside the declared source read")
    if (
        set(data) - {"wikidot_token7", "moduleName", "pageNo", "t"}
        or data.get("moduleName") != "forum/ForumViewThreadPostsModule"
    ):
        raise RuntimeError("AMC request outside the declared read module")
    headers = {k.lower(): v for k, v in kwargs.get("headers", {}).items()}
    if (
        "authorization" in headers
        or "wikidot_session_id" in headers.get("cookie", "").lower()
        or "authorization" in self.headers
        or any(key.lower() == "wikidot_session_id" for key in self.cookies.keys())
        or kwargs.get("auth")
    ):
        raise RuntimeError("Authenticated request refused")
    safe = {k: str(v) for k, v in data.items() if k != "wikidot_token7"}
    key = hashlib.sha256(
        json.dumps(
            {
                "url": str(url),
                "method": "POST",
                "body": safe,
                "actor": "anonymous",
                "headers": {k: v for k, v in headers.items() if k != "cookie"},
                "cookie_sha256": hashlib.sha256(
                    headers.get("cookie", "").encode()
                ).hexdigest(),
            },
            sort_keys=True,
        ).encode()
    ).hexdigest()
    if safe.get("t") != str(thread.id) or not re.fullmatch(
        r"[1-9]\d{0,3}", safe.get("pageNo", "")
    ):
        raise RuntimeError("Read parameters are outside the declared thread")
    receipt = CACHE / (key + ".json")
    body = CACHE / (key + ".body")
    barrier = CACHE / (key + ".barrier")
    async with fill_lock:
        with (CACHE / ".acquisition.lock").open("a") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            if receipt.exists():
                m = json.loads(receipt.read_text())
                if m["headers"].get("vary", "").strip() == "*" or m["status"] == 304:
                    raise RuntimeError(
                        "Historical-only response cannot be replayed or reacquired"
                    )
                payload = body.read_bytes()
                if hashlib.sha256(payload).hexdigest() != m["body_sha256"]:
                    raise RuntimeError("Retained response hash mismatch")
                requests.append(
                    {
                        "request": safe,
                        "mode": "retained",
                        "status": m["status"],
                        "captured_at": m["finished_at"],
                    }
                )
                return httpx.Response(
                    m["status"],
                    headers=m["headers"],
                    content=payload,
                    request=httpx.Request("POST", url),
                )
            if barrier.exists():
                raise RuntimeError("Unsealed acquisition barrier; no refetch permitted")
            durable(
                barrier,
                json.dumps(
                    {
                        "request": safe,
                        "started_at": datetime.now(timezone.utc).isoformat(),
                        "actor": "anonymous",
                    }
                ).encode(),
            )
            if retry_until > time.time():
                await asyncio.sleep(retry_until - time.time())
            response = await original_post(self, url, **kwargs)
            payload = response.content
            if b"WIKIDOT_SESSION_ID" in payload or b"wikidot_token7" in payload:
                raise RuntimeError(
                    "Response includes session material; sealing refused"
                )
            clean_headers = {
                k: v
                for k, v in response.headers.items()
                if k.lower()
                not in {
                    "set-cookie",
                    "set-cookie2",
                    "content-encoding",
                    "content-length",
                    "transfer-encoding",
                }
            }
            if response.status_code in (429, 503) and response.headers.get(
                "retry-after"
            ):
                value = response.headers["retry-after"]
                try:
                    retry_until = time.time() + max(0, int(value))
                except ValueError:
                    try:
                        retry_until = max(
                            time.time(), parsedate_to_datetime(value).timestamp()
                        )
                    except (ValueError, TypeError):
                        pass
            durable(body, payload)
            captured_at = datetime.now(timezone.utc).isoformat()
            durable(
                receipt,
                json.dumps(
                    {
                        "request": safe,
                        "status": response.status_code,
                        "headers": clean_headers,
                        "body_sha256": hashlib.sha256(payload).hexdigest(),
                        "finished_at": captured_at,
                    },
                    sort_keys=True,
                ).encode(),
            )
            barrier.unlink()
            fd = os.open(CACHE, os.O_RDONLY)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
            requests.append(
                {
                    "request": safe,
                    "mode": "acquired",
                    "status": response.status_code,
                    "captured_at": captured_at,
                }
            )
            return response


httpx.AsyncClient.post = cached_read
logging.getLogger("wikidot").setLevel(logging.CRITICAL)
with wikidot.Client(
    amc_config=AjaxModuleConnectorConfig(
        attempt_limit=1, retry_max_retries=0, semaphore_limit=1
    ),
    logging_level="CRITICAL",
) as client:
    site = Site(
        client=client,
        id=site_id,
        title=site_title,
        unix_name=site_name,
        domain=site_host,
        ssl_supported=True,
    )
    thread = ForumThreadCollection._parse_thread_page(site, html)
    if (
        not source_url.path.startswith(f"/forum/t-{thread.id}/")
        or thread.post_count > 1000
    ):
        raise RuntimeError(
            "Thread reference mismatch or source inventory exceeds the bounded importer"
        )
    if (OUT / "observation.json").exists():
        raise RuntimeError("Completed observation already exists; reuse it")
    posts = thread.posts
    if len(posts) != thread.post_count:
        raise RuntimeError("Source discussion acquisition is incomplete")

    def author(user):
        return {
            "type": type(user).__name__,
            "id": user.id,
            "name": user.name,
            "slug": user.unix_name,
        }

    rows = [
        {
            "id": p.id,
            "parent_id": p.parent_id,
            "created_at": p.created_at.replace(tzinfo=timezone.utc).isoformat(),
            "created_by": author(p.created_by),
            "edited_at": p.edited_at.replace(tzinfo=timezone.utc).isoformat()
            if p.edited_at
            else None,
            "edited_by": author(p.edited_by) if p.edited_by else None,
            "title": p.title,
            "rendered_html": p.text,
        }
        for p in posts
    ]
    record = {
        "schema": "wikijump.source_discussion_identity_observation.v1",
        "client_commit": commit,
        "source_reference": args.source_url,
        "source_site": site_name,
        "source_site_id": site_id,
        "source_document_sha256": hashlib.sha256(source).hexdigest(),
        "thread": {
            "id": thread.id,
            "title": thread.title,
            "description": thread.description,
            "created_at": thread.created_at.replace(tzinfo=timezone.utc).isoformat(),
            "created_by": author(thread.created_by),
            "post_count": thread.post_count,
        },
        "posts": rows,
        "requests": requests,
        "observed_at": max(request["captured_at"] for request in requests),
    }
    target = OUT / "observation.json"
    if target.exists():
        raise RuntimeError("Observation already exists; preserve it")
    durable(target, json.dumps(record, indent=2).encode())
    print(
        json.dumps(
            {
                "output": str(target),
                "posts": len(rows),
                "expected_posts": thread.post_count,
                "actors": len({r["created_by"]["id"] for r in rows}),
                "acquired": sum(r["mode"] == "acquired" for r in requests),
                "replayed": sum(r["mode"] == "retained" for r in requests),
            }
        )
    )
