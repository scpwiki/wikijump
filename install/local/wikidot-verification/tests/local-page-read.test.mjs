import assert from "node:assert/strict";
import test from "node:test";

import {LocalPageReadClient, LocalPageReadError, sameTimestamp} from "../src/local-page-read.mjs";

function response(result, id = 1) {
  const bytes = Buffer.from(JSON.stringify({jsonrpc: "2.0", id, result}));
  return {
    body: null,
    headers: {get: () => String(bytes.byteLength)},
    ok: true,
    async arrayBuffer() {
      return bytes;
    },
  };
}

test("local page reader compares timestamp precision and validates returned page identity", async () => {
  assert.equal(sameTimestamp("2026-01-01T00:00:00.1Z", "2026-01-01T00:00:00.100Z"), true);
  assert.equal(sameTimestamp("2026-01-01T00:00:00Z", "2026-01-01T00:00:01Z"), false);

  const requests = [];
  const client = new LocalPageReadClient({
    rpcUrl: "http://127.0.0.1:2747/jsonrpc",
    fetchImpl: async (_url, options) => {
      const request = JSON.parse(options.body);
      requests.push(request);
      if (request.method === "site_get") return response({site_id: 7}, request.id);
      return response({
        compiled_body_html: "<p>ok</p>",
        compiled_body_styles: [".x{}"],
        page_revision_count: 2,
        page_updated_at: "2026-01-01T00:00:00Z",
        site_id: 7,
        slug: "alpha",
        wikitext: "source",
      }, request.id);
    },
  });
  assert.equal(await client.siteId(), 7);
  assert.equal((await client.pageGet(7, "alpha")).wikitext, "source");
  assert.deepEqual(requests.map(({method}) => method), ["site_get", "page_get"]);
});

test("local page reader rejects mismatched RPC envelopes", async () => {
  const client = new LocalPageReadClient({
    rpcUrl: "http://127.0.0.1:2747/jsonrpc",
    fetchImpl: async () => response({site_id: 7}, 99),
  });
  await assert.rejects(() => client.siteId(), (error) => error instanceof LocalPageReadError && error.code === "rpc_envelope");
});
