import io
import json
import unittest

from scripts import wikidot_theme_page_helper as HELPER


class FakeBackend:
    def __init__(self) -> None:
        self.closed = False
        self.calls: list[tuple[object, ...]] = []

    def close(self) -> None:
        self.closed = True

    def inspect(self, slug: str, kind: str = "theme_page") -> dict[str, object]:
        self.calls.append(("inspect", slug, kind))
        return {"identity": 7, "title": "fixture", "source_sha256": "0" * 64, "tags": []}

    def create(self, slug: str, title: str, source: str, expected_source_sha256: str, tags: list[str], kind: str = "theme_page") -> dict[str, object]:
        self.calls.append(("create", slug, title, source, expected_source_sha256, tags, kind))
        return {"identity": 8, "title": title, "source_sha256": expected_source_sha256, "tags": tags}

    def remove(self, slug: str, expected: dict[str, object], kind: str = "theme_page") -> dict[str, bool]:
        self.calls.append(("remove", slug, expected, kind))
        return {"removed": True, "already_absent": False}


class WikidotThemePageHelperTests(unittest.TestCase):
    def test_dispatch_rejects_secret_fields_before_action_handling(self) -> None:
        with self.assertRaisesRegex(HELPER.PublicError, "credentials are accepted"):
            HELPER.dispatch(FakeBackend(), {"action": "ping", "session_token": "secret"})

    def test_serve_returns_public_error_and_closes_backend(self) -> None:
        backend = FakeBackend()
        output = io.StringIO()

        result = HELPER.serve(io.StringIO('{"id":1,"action":"unknown"}\n'), output, backend)

        self.assertEqual(result, 0)
        self.assertTrue(backend.closed)
        response = json.loads(output.getvalue())
        self.assertFalse(response["ok"])
        self.assertEqual(response["error"]["code"], "invalid_action")
        self.assertNotIn("secret", output.getvalue())

    def test_dispatch_action_matrix_preserves_contract_fields(self) -> None:
        backend = FakeBackend()
        slug = "codex-l10n:20260723-helper-yossistyle"
        source = "fixture"
        source_sha256 = HELPER.sha256(source)
        expected = {"identity": 8, "title": "fixture", "source_sha256": source_sha256, "tags": ["テーマ"]}

        ping, stop = HELPER.dispatch(backend, {"action": "ping"})
        self.assertEqual(ping["protocol"], "wikijump.theme_wikidot_helper.v1")
        self.assertFalse(stop)
        inspected, stop = HELPER.dispatch(backend, {"action": "inspect", "slug": slug})
        self.assertEqual(inspected["page"]["identity"], 7)
        self.assertFalse(stop)
        created, stop = HELPER.dispatch(
            backend,
            {
                "action": "create",
                "slug": slug,
                "title": "fixture",
                "source": source,
                "source_sha256": source_sha256,
                "tags": ["テーマ"],
            },
        )
        self.assertEqual(created["page"], expected)
        self.assertFalse(stop)
        removed, stop = HELPER.dispatch(backend, {"action": "remove", "slug": slug, "expected": expected})
        self.assertTrue(removed["removal"]["removed"])
        self.assertFalse(stop)
        shutdown, stop = HELPER.dispatch(backend, {"action": "shutdown"})
        self.assertEqual(shutdown, {"closed": True})
        self.assertTrue(stop)
        self.assertEqual([call[0] for call in backend.calls], ["inspect", "create", "remove"])

    def test_serve_rejects_malformed_requests_and_stops_after_shutdown(self) -> None:
        backend = FakeBackend()
        output = io.StringIO()
        requests = io.StringIO('not-json\n{"id":true,"action":"ping"}\n{"id":3,"action":"shutdown"}\n{"id":4,"action":"ping"}\n')

        self.assertEqual(HELPER.serve(requests, output, backend), 0)

        responses = [json.loads(line) for line in output.getvalue().splitlines()]
        self.assertEqual([response["id"] for response in responses], [None, None, 3])
        self.assertEqual([response.get("error", {}).get("code") for response in responses], ["invalid_request", "invalid_request", None])
        self.assertTrue(backend.closed)

    def test_serve_preserves_stream_and_cleanup_failures(self) -> None:
        class BrokenOutput:
            def write(self, value: str) -> None:
                raise OSError("write failed")

            def flush(self) -> None:
                raise AssertionError("flush must not follow failed write")

        class BrokenCleanupBackend(FakeBackend):
            def close(self) -> None:
                raise RuntimeError("close failed")

        with self.assertRaises(HELPER.PrimaryCleanupError) as caught:
            HELPER.serve(io.StringIO('{"id":1,"action":"ping"}\n'), BrokenOutput(), BrokenCleanupBackend())
        self.assertIsInstance(caught.exception.primary_error, OSError)
        self.assertIsInstance(caught.exception.cleanup_error, RuntimeError)

    def test_slug_contract_distinguishes_mutable_and_read_only_resources(self) -> None:
        current = "codex-l10n:20260723-helper-yossistyle"
        self.assertEqual(HELPER.validate_slug(current), current)
        self.assertEqual(
            HELPER.validate_slug("component:image-block", kind="reference_prerequisite"),
            "component:image-block",
        )
        with self.assertRaises(HELPER.PublicError):
            HELPER.validate_slug("component:other", kind="reference_prerequisite")
        legacy = "theme:codex-l10n-20260723-helper-yossistyle"
        self.assertEqual(HELPER.validate_slug(legacy, allow_legacy=True), legacy)
        with self.assertRaises(HELPER.PublicError):
            HELPER.validate_slug(legacy)

    def test_backend_refuses_direct_reference_prerequisite_removal(self) -> None:
        backend = object.__new__(HELPER.WikidotBackend)
        with self.assertRaisesRegex(HELPER.PublicError, "read-only"):
            backend.remove("component:image-block", {}, kind="reference_prerequisite")

    def test_create_lock_rejects_existing_or_contested_pages(self) -> None:
        backend = object.__new__(HELPER.WikidotBackend)
        for response, code in (
            ({"locked": True}, "page_lock_refused"),
            ({"page_revision_id": 9}, "page_exists"),
            ({"status": "ok"}, "page_lock_refused"),
        ):
            backend._request_ajax_module_connector = lambda fields, response=response: response
            with self.assertRaises(HELPER.PublicError) as caught:
                backend._acquire_create_lock("codex-l10n:20260723-helper-yossistyle")
            self.assertEqual(caught.exception.code, code)

    def test_created_page_snapshot_includes_confirmed_tags(self) -> None:
        backend = object.__new__(HELPER.WikidotBackend)
        backend.inspect = lambda slug, kind: {
            "identity": 7,
            "title": "fixture",
            "source_sha256": HELPER.sha256("source"),
            "tags": [],
        }
        saved: list[tuple[object, ...]] = []
        backend._save_tags = lambda *arguments: saved.append(arguments)

        actual = backend._await_created_page(
            "codex-l10n:20260723-helper-yossistyle",
            "theme_page",
            "fixture",
            "source",
            ["テーマ"],
        )

        self.assertEqual(actual["tags"], ["テーマ"])
        self.assertEqual(len(saved), 1)

    def test_authenticated_get_refuses_redirects_and_empty_successes(self) -> None:
        class Response:
            def __init__(self, status_code: int, text: str, *, redirect: bool = False):
                self.status_code = status_code
                self.text = text
                self.is_redirect = redirect

        class Session:
            def __init__(self, response: Response):
                self.response = response

            def __enter__(self) -> "Session":
                return self

            def __exit__(self, *arguments: object) -> None:
                return None

            def get(self, *arguments: object, **keywords: object) -> Response:
                return self.response

        class Httpx:
            def __init__(self, response: Response):
                self.response = response

            def Client(self, **keywords: object) -> Session:
                return Session(self.response)

        backend = object.__new__(HELPER.WikidotBackend)
        backend.headers = {}
        backend.httpx = Httpx(Response(404, ""))
        self.assertIsNone(backend._get("missing"))
        for response, code in (
            (Response(302, "", redirect=True), "redirect_refused"),
            (Response(200, ""), "authenticated_get_failed"),
            (Response(503, "unavailable"), "authenticated_get_failed"),
        ):
            backend.httpx = Httpx(response)
            with self.assertRaises(HELPER.PublicError) as caught:
                backend._get("fixture")
            self.assertEqual(caught.exception.code, code)

    def test_site_identity_requires_the_allowlisted_authenticated_site(self) -> None:
        backend = object.__new__(HELPER.WikidotBackend)
        backend._get = lambda slug: (
            'WIKIREQUEST.info.pageId = 7; WIKIREQUEST.info.siteId = 8; '
            'WIKIREQUEST.info.siteUnixName = "scpaiueouiuiuiui"; '
            'WIKIREQUEST.info.domain = "scpaiueouiuiuiui.wikidot.com";'
        )
        backend._verify_site_identity()
        backend._get = lambda slug: 'WIKIREQUEST.info.siteId = 8; WIKIREQUEST.info.siteUnixName = "other"; WIKIREQUEST.info.domain = "other.wikidot.com";'
        with self.assertRaises(HELPER.PublicError) as caught:
            backend._verify_site_identity()
        self.assertEqual(caught.exception.code, "site_identity_mismatch")

    def test_create_and_remove_enforce_exact_snapshots(self) -> None:
        backend = object.__new__(HELPER.WikidotBackend)
        source = "fixture source"
        snapshot = {"identity": 7, "title": "fixture", "source_sha256": HELPER.sha256(source), "tags": ["テーマ"]}
        inspections = iter([None, {**snapshot, "tags": []}])
        backend.inspect = lambda slug, kind="theme_page": next(inspections)
        backend._acquire_create_lock = lambda slug: ("lock", "secret")
        events: list[str] = []
        backend._request_ajax_module_connector = lambda fields: events.append(fields.get("event", fields.get("moduleName"))) or {"status": "ok"}
        backend._save_tags = lambda slug, kind, identity, tags: events.append("confirmedTags")

        created = backend.create(
            "codex-l10n:20260723-helper-yossistyle",
            title="fixture",
            source=source,
            expected_source_sha256=HELPER.sha256(source),
            tags=["テーマ"],
        )

        self.assertEqual(created, snapshot)
        self.assertEqual(events, ["savePage", "confirmedTags"])
        backend.inspect = lambda slug, kind="theme_page": {**snapshot, "title": "changed"}
        with self.assertRaises(HELPER.PublicError) as caught:
            backend.remove("codex-l10n:20260723-helper-yossistyle", snapshot)
        self.assertEqual(caught.exception.code, "page_changed")


if __name__ == "__main__":
    unittest.main()
