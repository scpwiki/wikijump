import assert from "node:assert/strict";
import { test } from "node:test";

import {
  captureFeedEndpointCases,
  summarizeFeedEndpointResponse,
  validateFeedEndpointCases,
  verifyFeedEndpointCaptures,
} from "../src/listpages-feed-endpoint-oracle.mjs";

const fixture = {
  schema: "wikijump_listpages_compat.feed_endpoint_cases.v1",
  site: "sandbox-for-codex",
  cases: [
    {
      case_id: "baseline",
      path: "/feed/pages/t/Test",
      expected_status: 200,
      expected_kind: "rss",
      expected_item_count: 1,
      expected_title: "Test",
    },
    {
      case_id: "same",
      path: "/feed/pages/limit/1/t/Test",
      expected_status: 200,
      expected_kind: "rss",
      expected_item_count: 1,
      same_item_guids_as: "baseline",
    },
  ],
};

const rss = `<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wikidot="http://www.wikidot.com/rss-namespace">
<channel>
<title>Test</title><link>http://sandbox-for-codex.wikidot.com</link>
<description>A &amp; B</description>
<lastBuildDate>Mon, 27 Jul 2026 00:00:00 +0000</lastBuildDate>
<item><guid>http://sandbox-for-codex.wikidot.com/one</guid></item>
</channel></rss>`;

test("summarizes RSS and live ProcessException responses", () => {
  assert.deepEqual(summarizeFeedEndpointResponse(rss, "text/xml"), {
    kind: "rss",
    title: "Test",
    link: "http://sandbox-for-codex.wikidot.com",
    description: "A & B",
    last_build_date: "Mon, 27 Jul 2026 00:00:00 +0000",
    item_count: 1,
    item_guids: ["http://sandbox-for-codex.wikidot.com/one"],
    has_content_encoded_namespace: true,
    has_wikidot_namespace: true,
  });
  assert.deepEqual(
    summarizeFeedEndpointResponse(
      "exception 'ProcessException' with message 'Invalid range argument.' Stack trace:",
      "text/xml",
    ),
    {
      kind: "error",
      error: "Invalid range argument.",
      leaked_server_stack: true,
    },
  );
});

test("validates case scope and verifies relational expectations", () => {
  assert.equal(validateFeedEndpointCases(fixture), fixture);
  assert.throws(
    () =>
      validateFeedEndpointCases({
        ...fixture,
        cases: [{ ...fixture.cases[0], path: "https://example.test/" }],
      }),
    /unsafe/u,
  );
});

test("captures immutable anonymous evidence through an injected fetch", async () => {
  const calls = [];
  const captures = await captureFeedEndpointCases(fixture, {
    capturedAt: "2026-07-27T00:00:00.000Z",
    requestImpl: async (path, options) => {
      calls.push({ path, options });
      return new Response(rss, {
        status: 200,
        headers: { "content-type": "text/xml;charset=utf-8" },
      });
    },
  });
  assert.equal(captures.length, 2);
  assert.equal(captures[0].provenance.authenticated, false);
  assert.equal(captures[0].provenance.mutated, false);
  assert.equal(captures[0].body, rss);
  assert.equal(captures[0].body_sha256.length, 64);
  assert.equal(calls[0].path, "/feed/pages/t/Test");
  assert.equal(calls[0].options.headers.accept.includes("application/rss+xml"), true);
  assert.deepEqual(verifyFeedEndpointCaptures(fixture, captures), []);
});
