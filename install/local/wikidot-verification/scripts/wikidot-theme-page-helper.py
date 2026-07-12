#!/usr/bin/env python3
"""Persistent, fail-closed Wikidot page helper for theme localization canaries."""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
import time
from pathlib import Path
from typing import Any, TextIO

ALLOWED_SITE = "scpaiueouiuiuiui"
ALLOWED_DOMAIN = f"{ALLOWED_SITE}.wikidot.com"
ALLOWED_ORIGIN = f"http://{ALLOWED_DOMAIN}"
RUN_OWNED_SLUG = re.compile(r"^theme:codex-l10n-[a-z0-9][a-z0-9-]+-(?:yossistyle|ashes-to-ashes|basalt)$")
PAGE_ID = re.compile(r"WIKIREQUEST\.info\.pageId\s*=\s*([0-9]+)\s*;")
SITE_ID = re.compile(r"WIKIREQUEST\.info\.siteId\s*=\s*([0-9]+)\s*;")
SITE_UNIX_NAME = re.compile(r'WIKIREQUEST\.info\.siteUnixName\s*=\s*"([^"]+)"\s*;')
SITE_DOMAIN = re.compile(r'WIKIREQUEST\.info\.domain\s*=\s*"([^"]+)"\s*;')
MAX_REQUEST_BYTES = 1_000_000


class PublicError(Exception):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code
        self.message = message


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def validate_slug(value: object) -> str:
    if not isinstance(value, str) or len(value) > 100 or RUN_OWNED_SLUG.fullmatch(value) is None:
        raise PublicError("resource_not_allowed", "resource is not a run-owned theme page")
    return value


def require_text(value: object, field: str, maximum: int) -> str:
    if not isinstance(value, str) or not value or len(value) > maximum:
        raise PublicError("invalid_request", f"{field} is invalid")
    return value


def reject_secret_fields(value: object) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if re.search(r"password|cookie|credential|session|token", str(key), re.IGNORECASE):
                raise PublicError(
                    "secret_field_refused",
                    "credentials are accepted through environment variables only",
                )
            reject_secret_fields(child)
    elif isinstance(value, list):
        for child in value:
            reject_secret_fields(child)


class WikidotBackend:
    def __init__(self) -> None:
        username = os.environ.pop("WIKIDOT_USERNAME", "")
        password = os.environ.pop("WIKIDOT_PASSWORD", "")
        root = Path(os.environ.pop("WIKIDOT_PY_ROOT", "/home/roku/src/Rokurolize/wikidot.py/src"))
        if not username or not password or not root.is_dir():
            raise PublicError("initialization_failed", "Wikidot helper environment is incomplete")
        sys.path.insert(0, str(root))
        try:
            import httpx
            from bs4 import BeautifulSoup
            from wikidot import Client
            from wikidot.module.page_source import extract_page_source_text
        except Exception as exc:
            raise PublicError("initialization_failed", "Wikidot helper dependencies are unavailable") from exc
        try:
            self.client = Client(username=username, password=password, logging_level="CRITICAL")
        except Exception as exc:
            raise PublicError("authentication_failed", "Wikidot authentication failed") from exc
        finally:
            password = ""
        self.httpx = httpx
        self.soup = BeautifulSoup
        self.extract_source = extract_page_source_text
        self.headers = self.client.amc_client.header.get_header()
        root_html = self._get("")
        if root_html is None:
            self.close()
            raise PublicError(
                "site_identity_mismatch",
                "authenticated site root was not found",
            )
        site_id = SITE_ID.search(root_html)
        site_name = SITE_UNIX_NAME.search(root_html)
        domain = SITE_DOMAIN.search(root_html)
        if not site_id or not site_name or not domain or site_name.group(1) != ALLOWED_SITE or domain.group(1) != ALLOWED_DOMAIN:
            self.close()
            raise PublicError(
                "site_identity_mismatch",
                "authenticated site identity is outside the hard allowlist",
            )

    def close(self) -> None:
        client = getattr(self, "client", None)
        if client is not None:
            try:
                client.close()
            except Exception:
                pass

    def _get(self, slug: str) -> str | None:
        url = ALLOWED_ORIGIN if not slug else f"{ALLOWED_ORIGIN}/{slug}"
        try:
            with self.httpx.Client(follow_redirects=False, timeout=30.0, trust_env=False) as client:
                response = client.get(url, headers=self.headers)
        except Exception as exc:
            raise PublicError("authenticated_get_failed", "authenticated Wikidot GET failed") from exc
        if response.is_redirect:
            raise PublicError("redirect_refused", "authenticated Wikidot GET returned a redirect")
        if response.status_code == 404:
            return None
        if response.status_code != 200:
            raise PublicError(
                "authenticated_get_failed",
                f"authenticated Wikidot GET returned HTTP {response.status_code}",
            )
        if not response.text:
            raise PublicError(
                "authenticated_get_failed",
                "authenticated Wikidot GET returned an empty success response",
            )
        return response.text

    def _amc(self, body: dict[str, Any]) -> dict[str, Any]:
        cookie = self.client.amc_client.header.cookie
        session_id = str(cookie.get("WIKIDOT_SESSION_ID", "")).strip()
        token = str(cookie.get("wikidot_token7", "")).strip()
        if not session_id or not token:
            raise PublicError("authentication_failed", "authenticated Wikidot session is unavailable")
        headers = {
            "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
            "User-Agent": "WikidotPy",
            "Referer": "https://www.wikidot.com/",
            "Cookie": f"wikidot_token7={token};WIKIDOT_SESSION_ID={session_id};",
        }
        try:
            with self.httpx.Client(follow_redirects=False, timeout=30.0, trust_env=False) as client:
                response = client.post(
                    f"{ALLOWED_ORIGIN}/ajax-module-connector.php",
                    headers=headers,
                    data={"wikidot_token7": token, **body},
                )
            if response.is_redirect:
                raise PublicError(
                    "redirect_refused",
                    "authenticated Wikidot request returned a redirect",
                )
            response.raise_for_status()
            data = response.json()
        except PublicError:
            raise
        except Exception as exc:
            raise PublicError("wikidot_request_failed", "authenticated Wikidot request failed") from exc
        if not isinstance(data, dict):
            raise PublicError("malformed_response", "Wikidot returned a malformed response")
        return data

    def inspect(self, slug: str) -> dict[str, Any] | None:
        html = self._get(validate_slug(slug))
        if html is None:
            return None
        page_id_match = PAGE_ID.search(html)
        if page_id_match is None:
            raise PublicError(
                "page_identity_incomplete",
                "authenticated page GET did not contain a page id",
            )
        page_id = int(page_id_match.group(1))
        title_element = self.soup(html, "html.parser").select_one("#page-title")
        if title_element is None:
            raise PublicError(
                "page_identity_incomplete",
                "authenticated page GET did not contain a title",
            )
        data = self._amc({"moduleName": "viewsource/ViewSourceModule", "page_id": page_id})
        body = data.get("body")
        if not isinstance(body, str):
            raise PublicError("source_unavailable", "ViewSourceModule did not return a source body")
        source_element = self.soup(body.replace("&nbsp;", " "), "lxml").select_one("div.page-source")
        if source_element is None:
            raise PublicError("source_unavailable", "ViewSourceModule source body was malformed")
        source = self.extract_source(source_element)
        return {
            "identity": page_id,
            "title": title_element.get_text(" ", strip=True),
            "source_sha256": sha256(source),
        }

    def create(self, slug: str, title: str, source: str, expected_hash: str) -> dict[str, Any]:
        slug = validate_slug(slug)
        title = require_text(title, "title", 200)
        source = require_text(source, "source", 500_000)
        if not re.fullmatch(r"[0-9a-f]{64}", expected_hash) or sha256(source) != expected_hash:
            raise PublicError(
                "source_hash_mismatch",
                "submitted source does not match its accepted hash",
            )
        if self.inspect(slug) is not None:
            raise PublicError("page_exists", "create-only preflight found an existing page")
        lock = self._amc({"mode": "page", "wiki_page": slug, "moduleName": "edit/PageEditModule"})
        if lock.get("status") not in (None, "ok") or lock.get("locked") or lock.get("other_locks"):
            raise PublicError("page_lock_refused", "PageEditModule did not grant an uncontested lock")
        if lock.get("page_revision_id") not in (None, "") or lock.get("page_id") not in (None, "") or lock.get("pageId") not in (None, ""):
            raise PublicError("page_exists", "PageEditModule found an existing revision")
        lock_id = lock.get("lock_id")
        lock_secret = lock.get("lock_secret")
        if not lock_id or not lock_secret:
            raise PublicError("page_lock_refused", "PageEditModule omitted required lock fields")
        saved = self._amc(
            {
                "action": "WikiPageAction",
                "event": "savePage",
                "moduleName": "Empty",
                "mode": "page",
                "lock_id": lock_id,
                "lock_secret": lock_secret,
                "revision_id": "",
                "wiki_page": slug,
                "page_id": "",
                "title": title,
                "source": source,
                "comments": "run-owned theme localization E2E create",
            }
        )
        if saved.get("status") != "ok":
            raise PublicError("save_failed", "Wikidot create-only save failed")
        for _ in range(5):
            actual = self.inspect(slug)
            if actual is not None:
                if actual["title"] != title or actual["source_sha256"] != expected_hash:
                    raise PublicError(
                        "round_trip_mismatch",
                        "created page did not match the accepted title and source",
                    )
                return actual
            time.sleep(0.4)
        raise PublicError("create_not_visible", "created page was not visible after save")

    def remove(self, slug: str, expected: dict[str, Any]) -> dict[str, Any]:
        slug = validate_slug(slug)
        actual = self.inspect(slug)
        if actual is None:
            return {"removed": False, "already_absent": True}
        if actual != expected:
            raise PublicError(
                "page_changed",
                "delete refused a page whose identity, title, or source changed",
            )
        deleted = self._amc(
            {
                "action": "WikiPageAction",
                "event": "deletePage",
                "page_id": actual["identity"],
                "moduleName": "Empty",
            }
        )
        if deleted.get("status") not in (None, "ok"):
            raise PublicError("delete_failed", "Wikidot deletePage failed")
        for _ in range(5):
            if self.inspect(slug) is None:
                return {"removed": True, "already_absent": False}
            time.sleep(0.4)
        raise PublicError("delete_not_confirmed", "deleted page did not become absent")


def dispatch(backend: Any, request: dict[str, Any]) -> tuple[dict[str, Any], bool]:
    reject_secret_fields(request)
    action = request.get("action")
    if action == "ping":
        return {
            "protocol": "wikijump.theme_wikidot_helper.v1",
            "site": ALLOWED_SITE,
        }, False
    if action == "shutdown":
        return {"closed": True}, True
    slug = validate_slug(request.get("slug"))
    if action == "inspect":
        return {"page": backend.inspect(slug)}, False
    if action == "create":
        return {
            "page": backend.create(
                slug,
                request.get("title"),
                request.get("source"),
                request.get("source_sha256"),
            )
        }, False
    if action == "remove":
        expected = request.get("expected")
        if not isinstance(expected, dict) or set(expected) != {
            "identity",
            "title",
            "source_sha256",
        }:
            raise PublicError(
                "invalid_request",
                "remove requires exact expected identity, title, and source hash",
            )
        return backend.remove(slug, expected), False
    raise PublicError("invalid_action", "unknown helper action")


def serve(input_stream: TextIO, output_stream: TextIO, backend: Any) -> int:
    for raw_line in input_stream:
        request_id: int | None = None
        stop = False
        try:
            if len(raw_line.encode("utf-8")) > MAX_REQUEST_BYTES:
                raise PublicError("request_too_large", "helper request exceeded its size limit")
            request = json.loads(raw_line)
            if not isinstance(request, dict) or not isinstance(request.get("id"), int) or isinstance(request.get("id"), bool):
                raise PublicError("invalid_request", "helper request requires an integer id")
            request_id = request["id"]
            result, stop = dispatch(backend, request)
            response = {"id": request_id, "ok": True, "result": result}
        except PublicError as exc:
            response = {
                "id": request_id,
                "ok": False,
                "error": {"code": exc.code, "message": exc.message},
            }
        except Exception:
            response = {
                "id": request_id,
                "ok": False,
                "error": {
                    "code": "internal_error",
                    "message": "helper operation failed safely",
                },
            }
        output_stream.write(json.dumps(response, ensure_ascii=False, separators=(",", ":")) + "\n")
        output_stream.flush()
        if stop:
            break
    backend.close()
    return 0


def main() -> int:
    try:
        backend = WikidotBackend()
    except Exception:
        return 2
    return serve(sys.stdin, sys.stdout, backend)


if __name__ == "__main__":
    raise SystemExit(main())
