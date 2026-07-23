import io
import json
import unittest

from scripts import wikidot_theme_page_helper as HELPER


class FakeBackend:
    def __init__(self) -> None:
        self.closed = False

    def close(self) -> None:
        self.closed = True


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
        self.assertEqual(response["error"]["code"], "resource_not_allowed")
        self.assertNotIn("secret", output.getvalue())

    def test_slug_contract_distinguishes_mutable_and_read_only_resources(self) -> None:
        current = "codex-l10n:20260723-helper-yossistyle"
        self.assertEqual(HELPER.validate_slug(current), current)
        self.assertEqual(
            HELPER.validate_slug("component:image-block", kind="reference_prerequisite"),
            "component:image-block",
        )
        with self.assertRaises(HELPER.PublicError):
            HELPER.validate_slug("component:other", kind="reference_prerequisite")


if __name__ == "__main__":
    unittest.main()
