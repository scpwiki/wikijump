#!/usr/bin/env python3
"""Persistent, fail-closed Wikidot page helper for theme localization canaries.

Protocol v1 is UTF-8 JSON Lines over stdin and stdout. Every request is an object with an integer ``id`` and an ``action``; every response repeats that ``id`` and contains either ``{"ok": true, "result": ...}`` or ``{"ok": false, "error": {"code": ..., "message": ...}}``. ``ping`` takes no resource fields, ``inspect`` requires ``slug`` and optionally ``kind``, ``create`` requires ``slug``, ``title``, ``source``, ``source_sha256``, and ``tags``, ``remove`` requires ``slug`` and an exact ``expected`` record, and ``shutdown`` ends the loop after its response.

The helper accepts credentials only through ``WIKIDOT_USERNAME`` and ``WIKIDOT_PASSWORD``, removes them from its environment during initialization, and refuses secret-shaped request fields. Page snapshots expose the Wikidot page ID as ``identity`` because the JavaScript execution interface uses that backend-neutral field for exact cleanup comparisons. It exits 0 after orderly EOF or shutdown and exits 2 when initialization, stream delivery, or cleanup fails; operation failures stay inside the public error envelope. The ``wikijump.theme_wikidot_helper.v1`` ping result is the compatibility identifier for this contract.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
import time
from typing import Any, TextIO, TypedDict

try:
    import httpx as _httpx
    from bs4 import BeautifulSoup as _BeautifulSoup
    from wikidot import Client as _WikidotClient
    from wikidot.module.page_source import extract_page_source_text as _extract_page_source_text
except ImportError:
    _httpx = None
    _BeautifulSoup = None
    _WikidotClient = None
    _extract_page_source_text = None
ALLOWED_SITE = "scpaiueouiuiuiui"
ALLOWED_DOMAIN = f"{ALLOWED_SITE}.wikidot.com"
ALLOWED_ORIGIN = f"https://{ALLOWED_DOMAIN}"
CURRENT_RUN_OWNED_SLUG = re.compile(r"^codex-l10n:[a-z0-9][a-z0-9-]+-(?:yossistyle|ashes-to-ashes|basalt)$")
LEGACY_RUN_OWNED_SLUG = re.compile(r"^theme:codex-l10n-[a-z0-9][a-z0-9-]+-(?:yossistyle|ashes-to-ashes|basalt)$")
# Legacy names remain read/delete-only for cleanup of pages created before the current slug contract. Remove this compatibility path only after the run ledger proves no legacy page remains and the sandbox operator signs off.
REFERENCE_PREREQUISITE_SLUGS = {"component:image-block-base", "component:image-block"}
PAGE_ID = re.compile(r"WIKIREQUEST\.info\.pageId\s*=\s*([0-9]+)\s*;")
SITE_ID = re.compile(r"WIKIREQUEST\.info\.siteId\s*=\s*([0-9]+)\s*;")
SITE_UNIX_NAME = re.compile(r'WIKIREQUEST\.info\.siteUnixName\s*=\s*"([^"]+)"\s*;')
SITE_DOMAIN = re.compile(r'WIKIREQUEST\.info\.domain\s*=\s*"([^"]+)"\s*;')
MAX_REQUEST_BYTES = 1_000_000
WIKIDOT_PAGE_SLUG_MAX_LENGTH = 60
class PageSnapshot(TypedDict):
    identity: int
    title: str
    source_sha256: str
    tags: list[str]
class RemovalResult(TypedDict):
    removed: bool
    already_absent: bool
class PublicError(Exception):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code
        self.message = message

class PrimaryCleanupError(Exception):
    def __init__(self, primary_error: Exception, cleanup_error: Exception):
        super().__init__("Wikidot helper operation and cleanup both failed")
        self.primary_error = primary_error
        self.cleanup_error = cleanup_error

def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()

def wikidot_round_trip_sha256(value: str) -> str:
    # Live Wikidot removes exactly one terminal LF when saving page source.
    return sha256(value[:-1] if value.endswith("\n") else value)

def validate_kind(value: object) -> str:
    if value not in ("theme_page", "reference_prerequisite"):
        raise PublicError("resource_not_allowed", "resource kind is outside the theme execution contract")
    return str(value)

def validate_slug(value: object, *, kind: str = "theme_page", allow_legacy: bool = False) -> str:
    kind = validate_kind(kind)
    if kind == "reference_prerequisite":
        if value not in REFERENCE_PREREQUISITE_SLUGS:
            raise PublicError("resource_not_allowed", "reference prerequisite is outside the read-only contract")
        return str(value)
    pattern = (CURRENT_RUN_OWNED_SLUG, LEGACY_RUN_OWNED_SLUG) if allow_legacy else (CURRENT_RUN_OWNED_SLUG,)
    if not isinstance(value, str) or len(value) > WIKIDOT_PAGE_SLUG_MAX_LENGTH or not any(candidate.fullmatch(value) for candidate in pattern):
        raise PublicError("resource_not_allowed", "resource is not a run-owned theme page")
    return value

def require_text(value: object, field: str, maximum: int) -> str:
    if not isinstance(value, str) or not value or len(value) > maximum:
        raise PublicError("invalid_request", f"{field} is invalid")
    return value

def validate_tags(value: object, slug: str) -> list[str]:
    expected = ["テーマ"] if slug.endswith("-yossistyle") else ["theme"]
    if value != expected:
        raise PublicError("invalid_request", "run-owned page tags are invalid")
    return expected

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
    @staticmethod
    def _dependencies() -> tuple[Any, Any, Any, Any]:
        dependencies = (_httpx, _BeautifulSoup, _WikidotClient, _extract_page_source_text)
        if any(dependency is None for dependency in dependencies):
            raise PublicError("initialization_failed", "Wikidot helper dependencies are unavailable")
        return dependencies

    def _verify_site_identity(self) -> None:
        root_html = self._get("")
        if root_html is None:
            raise PublicError(
                "site_identity_mismatch",
                "authenticated site root was not found",
            )
        site_id = SITE_ID.search(root_html)
        site_name = SITE_UNIX_NAME.search(root_html)
        domain = SITE_DOMAIN.search(root_html)
        if not site_id or not site_name or not domain or site_name.group(1) != ALLOWED_SITE or domain.group(1) != ALLOWED_DOMAIN:
            raise PublicError(
                "site_identity_mismatch",
                "authenticated site identity is outside the hard allowlist",
            )

    def __init__(self, *, username: str, password: str) -> None:
        httpx, beautiful_soup, wikidot_client, extract_source = self._dependencies()
        try:
            self.client = wikidot_client(username=username, password=password, logging_level="CRITICAL")
        except Exception as exc:
            raise PublicError("authentication_failed", "Wikidot authentication failed") from exc
        finally:
            password = ""
        self.httpx = httpx
        self.soup = beautiful_soup
        self.extract_source = extract_source
        try:
            self.headers = self.client.amc_client.header.get_header()
            self._verify_site_identity()
        except Exception as initialization_error:
            try:
                self.close()
            except Exception as cleanup_error:
                raise PrimaryCleanupError(initialization_error, cleanup_error) from initialization_error
            raise

    def close(self) -> None:
        client = getattr(self, "client", None)
        if client is not None:
            try:
                client.close()
            except Exception as exc:
                raise PublicError(
                    "cleanup_failed",
                    "Wikidot helper cleanup failed",
                ) from exc

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

    def _request_ajax_module_connector(self, form_fields: dict[str, Any]) -> dict[str, Any]:
        cookie = self.client.amc_client.header.cookie
        session_id = str(cookie.get("WIKIDOT_SESSION_ID", "")).strip()
        token = str(cookie.get("wikidot_token7", "")).strip()
        if not session_id or not token:
            raise PublicError("authentication_failed", "authenticated Wikidot session is unavailable")
        headers = {
            "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
            "User-Agent": "WikidotPy",
            "Referer": f"{ALLOWED_ORIGIN}/",
            "Cookie": f"wikidot_token7={token};WIKIDOT_SESSION_ID={session_id};",
        }
        try:
            with self.httpx.Client(follow_redirects=False, timeout=30.0, trust_env=False) as client:
                response = client.post(
                    f"{ALLOWED_ORIGIN}/ajax-module-connector.php",
                    headers=headers,
                    data={"wikidot_token7": token, **form_fields},
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

    def inspect(self, slug: str, kind: str = "theme_page") -> PageSnapshot | None:
        html = self._get(validate_slug(slug, kind=kind, allow_legacy=True))
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
        data = self._request_ajax_module_connector({"moduleName": "viewsource/ViewSourceModule", "page_id": page_id})
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
            "tags": [
                element.get_text(" ", strip=True)
                for element in self.soup(html, "html.parser").select(".page-tags a")
            ],
        }

    def page_tags(self, slug: str, kind: str = "theme_page") -> list[str] | None:
        html = self._get(validate_slug(slug, kind=kind))
        if html is None:
            return None
        return [element.get_text(" ", strip=True) for element in self.soup(html, "html.parser").select(".page-tags a")]

    def _acquire_create_lock(self, slug: str) -> tuple[Any, Any]:
        lock = self._request_ajax_module_connector({"mode": "page", "wiki_page": slug, "moduleName": "edit/PageEditModule"})
        if lock.get("status") not in (None, "ok") or lock.get("locked") or lock.get("other_locks"):
            raise PublicError("page_lock_refused", "PageEditModule did not grant an uncontested lock")
        if lock.get("page_revision_id") not in (None, "") or lock.get("page_id") not in (None, "") or lock.get("pageId") not in (None, ""):
            raise PublicError("page_exists", "PageEditModule found an existing revision")
        lock_id = lock.get("lock_id")
        lock_secret = lock.get("lock_secret")
        if not lock_id or not lock_secret:
            raise PublicError("page_lock_refused", "PageEditModule omitted required lock fields")
        return lock_id, lock_secret

    def _save_tags(self, slug: str, kind: str, identity: Any, tags: list[str]) -> None:
        tagged = self._request_ajax_module_connector(
            {
                "tags": " ".join(tags),
                "action": "WikiPageAction",
                "event": "saveTags",
                "pageId": identity,
                "moduleName": "Empty",
            }
        )
        if tagged.get("status") not in (None, "ok"):
            raise PublicError("save_tags_failed", "Wikidot page tags were not saved")
        for _ in range(5):
            if self.page_tags(slug, kind) == tags:
                return
            time.sleep(0.4)
        raise PublicError("save_tags_not_visible", "Wikidot page tags did not round-trip")

    def _await_created_page(self, slug: str, kind: str, title: str, source: str, tags: list[str]) -> PageSnapshot:
        for _ in range(5):
            actual = self.inspect(slug, kind)
            if actual is None:
                time.sleep(0.4)
                continue
            if actual["title"] != title or actual["source_sha256"] != wikidot_round_trip_sha256(source):
                raise PublicError(
                    "round_trip_mismatch",
                    "created page did not match the accepted title and source",
                )
            self._save_tags(slug, kind, actual["identity"], tags)
            actual = {**actual, "tags": list(tags)}
            return actual
        raise PublicError("create_not_visible", "created page was not visible after save")

    def create(
        self,
        slug: str,
        *,
        title: str,
        source: str,
        expected_source_sha256: str,
        tags: list[str],
        kind: str = "theme_page",
    ) -> PageSnapshot:
        kind = validate_kind(kind)
        slug = validate_slug(slug, kind=kind)
        title = require_text(title, "title", 200)
        if kind != "theme_page":
            raise PublicError("resource_not_allowed", "reference prerequisites are read-only")
        source = require_text(source, "source", 500_000)
        tags = validate_tags(tags, slug)
        if not isinstance(expected_source_sha256, str) or not re.fullmatch(r"[0-9a-f]{64}", expected_source_sha256):
            raise PublicError("invalid_request", "source_sha256 is invalid")
        if sha256(source) != expected_source_sha256:
            raise PublicError(
                "source_hash_mismatch",
                "submitted source does not match its accepted hash",
            )
        if self.inspect(slug, kind) is not None:
            raise PublicError("page_exists", "create-only preflight found an existing page")
        lock_id, lock_secret = self._acquire_create_lock(slug)
        saved = self._request_ajax_module_connector(
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
        return self._await_created_page(slug, kind, title, source, tags)

    def remove(self, slug: str, expected: PageSnapshot, kind: str = "theme_page") -> RemovalResult:
        kind = validate_kind(kind)
        if kind != "theme_page":
            raise PublicError("resource_not_allowed", "reference prerequisites are read-only")
        slug = validate_slug(slug, kind=kind, allow_legacy=True)
        actual = self.inspect(slug, kind)
        if actual is None:
            return {"removed": False, "already_absent": True}
        if actual != expected:
            raise PublicError(
                "page_changed",
                "delete refused a page whose identity, title, or source changed",
            )
        # Wikidot exposes no revision-CAS delete operation. Keep the exact snapshot check immediately adjacent to deletion and confirm absence afterward; this is the narrowest available race window.
        deleted = self._request_ajax_module_connector(
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
            if self.inspect(slug, kind) is None:
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
    if action not in ("inspect", "create", "remove"):
        raise PublicError("invalid_action", "unknown helper action")
    kind = validate_kind(request.get("kind", "theme_page"))
    slug = validate_slug(request.get("slug"), kind=kind, allow_legacy=action in ("inspect", "remove"))
    if action == "inspect":
        return {"page": backend.inspect(slug, kind)}, False
    if kind != "theme_page":
        raise PublicError("resource_not_allowed", "reference prerequisites are read-only")
    if action == "create":
        return {
            "page": backend.create(
                slug,
                title=request.get("title"),
                source=request.get("source"),
                expected_source_sha256=request.get("source_sha256"),
                tags=request.get("tags"),
                kind=kind,
            )
        }, False
    if action == "remove":
        expected = request.get("expected")
        if not isinstance(expected, dict) or set(expected) != {
            "identity",
            "title",
            "source_sha256",
            "tags",
        }:
            raise PublicError(
                "invalid_request",
                "remove requires exact expected identity, title, tags, and source hash",
            )
        return {"removal": backend.remove(slug, expected, kind)}, False
    raise AssertionError("validated helper action was not dispatched")


def serve(input_stream: TextIO, output_stream: TextIO, backend: Any) -> int:
    primary_error: Exception | None = None
    try:
        for raw_line in input_stream:
            request_id: int | None = None
            stop = False
            try:
                if len(raw_line.encode("utf-8")) > MAX_REQUEST_BYTES:
                    raise PublicError("request_too_large", "helper request exceeded its size limit")
                try:
                    request = json.loads(raw_line)
                except json.JSONDecodeError as exc:
                    raise PublicError("invalid_request", "helper request must be valid JSON") from exc
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
        return 0
    except Exception as error:
        primary_error = error
        raise
    finally:
        try:
            backend.close()
        except Exception as cleanup_error:
            if primary_error is not None:
                raise PrimaryCleanupError(primary_error, cleanup_error) from primary_error
            raise


def main() -> int:
    try:
        username = os.environ.pop("WIKIDOT_USERNAME", "")
        password = os.environ.pop("WIKIDOT_PASSWORD", "")
        if not username or not password:
            raise PublicError("initialization_failed", "Wikidot helper environment is incomplete")
        try:
            backend = WikidotBackend(username=username, password=password)
        finally:
            password = ""
        return serve(sys.stdin, sys.stdout, backend)
    except Exception:
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
