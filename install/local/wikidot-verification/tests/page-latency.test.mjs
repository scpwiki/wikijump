import { strict as assert } from "node:assert";
import test from "node:test";

import { parseArgs, percentile, runPageLatency, summarizeSamples } from "../src/page-latency.mjs";

function fakeResponse(body, status = 200) {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: {
      get(name) {
        return name === "content-type" ? "text/html" : null;
      },
    },
    async arrayBuffer() {
      const buffer = Buffer.from(body);
      return buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength);
    },
  };
}

test("parseArgs accepts required latency options", () => {
  const args = parseArgs([
    "node",
    "script",
    "--url",
    "http://127.0.0.1/scp-173",
    "--requests",
    "50",
    "--warmups",
    "5",
    "--header",
    "Host: scp-wiki.wikijump.local",
    "--require-stable-body",
    "--output",
    "latency.json",
  ]);
  assert.equal(args.url, "http://127.0.0.1/scp-173");
  assert.equal(args.requests, 50);
  assert.equal(args.warmups, 5);
  assert.deepEqual(args.headers, { Host: "scp-wiki.wikijump.local" });
  assert.equal(args.requireStableBody, true);
  assert.equal(args.output, "latency.json");
});

test("parseArgs rejects zero measured requests", () => {
  assert.throws(() => parseArgs(["node", "script", "--url", "http://example.test", "--requests", "0"]), /greater than 0/);
});

test("percentile uses nearest rank", () => {
  assert.equal(percentile([10, 20, 30, 40], 50), 20);
  assert.equal(percentile([10, 20, 30, 40], 95), 40);
});

test("summarizeSamples reports duration percentiles and stable body hash", () => {
  const samples = [
    { status: 200, ok: true, duration_ms: 10.1119, bytes: 5, body_sha256: "a" },
    { status: 200, ok: true, duration_ms: 20.2229, bytes: 5, body_sha256: "a" },
    { status: 404, ok: false, duration_ms: 30.3339, bytes: 5, body_sha256: "a" },
  ];
  const summary = summarizeSamples(samples);
  assert.equal(summary.requests, 3);
  assert.equal(summary.ok, 2);
  assert.deepEqual(summary.status_counts, { 200: 2, 404: 1 });
  assert.equal(summary.duration_ms.p50, 20.223);
  assert.equal(summary.duration_ms.p95, 30.334);
  assert.equal(summary.body_stable, true);
  assert.equal(summary.body_sha256, "a");
});

test("runPageLatency excludes warmups and detects body changes", async () => {
  const bodies = ["warmup", "alpha", "beta"];
  const calls = [];
  const report = await runPageLatency({
    url: "http://example.test/page",
    warmups: 1,
    requests: 2,
    fetchImpl: async (url, options) => {
      calls.push({ url, options });
      return fakeResponse(bodies.shift());
    },
  });
  assert.equal(calls.length, 3);
  assert.equal(report.samples.length, 2);
  assert.equal(report.summary.body_stable, false);
  assert.equal(report.summary.body_sha256_values.length, 2);
});
