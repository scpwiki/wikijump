import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";
import {parseArgs} from "../scripts/local-browser-console-smoke.mjs";
import {buildSmokeSummary, captureLocalSmoke, classifyFailureUrl, inspectLedger, localSmokeUrl, RECORD_SCHEMA, runLocalBrowserSmoke, runtimeIdentityFingerprint} from "../src/local-browser-console-smoke.mjs";

test("CLI requires an explicit runtime identity", () => {
  assert.throws(() => parseArgs(["--inventory", "i", "--shard-manifest", "s", "--shard-id", "en", "--output", "o"]), /--runtime-identity is required/);
  const parsed = parseArgs(["--inventory", "i", "--shard-manifest", "s", "--shard-id", "en", "--output", "o", "--runtime-identity", "r", "--browser-root", "b", "--browser-executable", "chrome", "--workers", "3", "--timeout-ms", "10", "--settle-ms", "0", "--ignore-https-errors"]);
  assert.deepEqual({shardId: parsed.shardId, workers: parsed.workers, timeoutMs: parsed.timeoutMs, settleMs: parsed.settleMs, ignoreHttpsErrors: parsed.ignoreHttpsErrors}, {shardId: "en", workers: 3, timeoutMs: 10, settleMs: 0, ignoreHttpsErrors: true});
});

test("local URL validation rejects redirects, ports, and inventory URL drift", () => {
  assert.equal(localSmokeUrl({fixture_id: "EN:a", slug: "a", local_https_url: "https://scp-wiki.wikijump.localhost/a"}), "https://scp-wiki.wikijump.localhost/a");
  assert.throws(() => localSmokeUrl({fixture_id: "EN:a", slug: "a", local_https_url: "https://example.com/a"}), /must be exactly/);
  assert.throws(() => localSmokeUrl({fixture_id: "EN:a", slug: "a", local_https_url: "https://scp-wiki.wikijump.localhost:444/a"}), /must be exactly/);
});

test("failure URLs distinguish local WJFiles from unclassified external hosts", () => {
  assert.equal(classifyFailureUrl("https://scp-wiki.wikijump.localhost/a"), "fail");
  assert.equal(classifyFailureUrl("https://abc.wjfiles.localhost/a.png"), "fail");
  assert.equal(classifyFailureUrl("https://api.wikijump.localhost/rpc"), "fail");
  assert.equal(classifyFailureUrl("https://cdn.example/a.png"), "external_unclassified");
  assert.equal(classifyFailureUrl(null), "fail");
});

test("capture records requested smoke signals without DOM or text", async () => {
  const handlers = new Map();
  const page = {
    on(event, handler) { handlers.set(event, handler); },
    async goto() {
      handlers.get("console")({type: () => "error", text: () => "external script", location: () => ({url: "https://cdn.example/x.js"})});
      handlers.get("requestfailed")({url: () => "https://files.wjfiles.localhost/missing.png", resourceType: () => "image", failure: () => ({errorText: "net::ERR_FAILED"})});
      handlers.get("response")({url: () => "https://cdn.example/nope", status: () => 404, request: () => ({resourceType: () => "script"})});
      return {status: () => 200};
    },
    async waitForTimeout() {},
    locator(selector) { assert.equal(selector, "#page-content"); return {count: async () => 1}; },
    async evaluate() { return [{url: "https://cdn.example/broken.png"}]; },
    url() { return "https://scp-wiki.wikijump.localhost/a"; },
  };
  const record = await captureLocalSmoke(page, {fixture_id: "EN:a", slug: "a"}, {timeoutMs: 100, settleMs: 0});
  assert.equal(record.result, "fail");
  assert.equal(record.console_errors[0].classification, "external_unclassified");
  assert.equal(record.request_failures[0].classification, "fail");
  assert.equal(record.http_error_responses[0].classification, "external_unclassified");
  assert.equal(record.visible_broken_images[0].classification, "external_unclassified");
  assert.equal(Object.hasOwn(record, "html"), false);
  assert.equal(Object.hasOwn(record, "text"), false);
});

test("ledger resume and summary fail closed on duplicate, missing, and unexpected fixtures", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-local-smoke-"));
  const output = path.join(root, "smoke.jsonl");
  const identity = {wikijump_sha: "abc", profile: "local"};
  const fingerprint = runtimeIdentityFingerprint(identity);
  const record = (fixtureId, result = "pass") => ({schema: RECORD_SCHEMA, fixture_id: fixtureId, runtime_identity_sha256: fingerprint, result});
  await fs.writeFile(output, `${JSON.stringify(record("EN:a"))}\n${JSON.stringify(record("EN:a"))}\n${JSON.stringify(record("EN:extra"))}\n`, "utf8");
  const ledger = await inspectLedger(output, fingerprint);
  assert.deepEqual(ledger.duplicate, ["EN:a"]);
  const summary = buildSmokeSummary({expected: ["EN:a", "EN:b"], records: ledger.records, runtimeIdentity: identity, inventory: "/i", shardManifest: "/s", shardId: "en"});
  assert.equal(summary.status, "fail");
  assert.deepEqual(summary.missing, ["EN:b"]);
  assert.deepEqual(summary.duplicate, ["EN:a"]);
  assert.deepEqual(summary.unexpected, ["EN:extra"]);
});

test("ledger rejects runtime identity changes and malformed JSON", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-local-smoke-bad-"));
  const output = path.join(root, "smoke.jsonl");
  await fs.writeFile(output, `${JSON.stringify({schema: RECORD_SCHEMA, fixture_id: "EN:a", runtime_identity_sha256: "wrong", result: "pass"})}\n{`, "utf8");
  const ledger = await inspectLedger(output, runtimeIdentityFingerprint({sha: "expected"}));
  assert.equal(ledger.records.length, 0);
  assert.equal(ledger.ledgerErrors.length, 2);
});

test("driver saves one resumable JSONL record per fixture and injects identity into summary", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-local-smoke-driver-"));
  const output = path.join(root, "smoke.jsonl");
  const makePage = () => ({
    on() {},
    async goto() { return {status: () => 200}; },
    locator() { return {count: async () => 1}; },
    async evaluate() { return []; },
    url() { return "https://scp-wiki.wikijump.localhost/a"; },
    async close() {},
  });
  const chromium = {async launch() { return {async newContext() { return {async newPage() { return makePage(); }, async close() {}}; }, async close() {}}; }};
  const identity = {wikijump_sha: "8645394", profile: "local"};
  const result = await runLocalBrowserSmoke({chromium, rows: [{fixture_id: "EN:a", family: "EN", slug: "a"}], outputPath: output, runtimeIdentity: identity, inventoryPath: "/inventory.json", shardManifestPath: "/shards.json", shardId: "en", workers: 2, timeoutMs: 100, settleMs: 0, ignoreHttpsErrors: true});
  assert.equal(result.summary.status, "pass");
  assert.deepEqual(result.summary.runtime_identity, identity);
  assert.equal((await fs.readFile(output, "utf8")).trim().split("\n").length, 1);
  const resumed = await runLocalBrowserSmoke({chromium: {async launch() { throw new Error("resume launched browser"); }}, rows: [{fixture_id: "EN:a", family: "EN", slug: "a"}], outputPath: output, runtimeIdentity: identity, inventoryPath: "/inventory.json", shardManifestPath: "/shards.json", shardId: "en", workers: 1, timeoutMs: 100, settleMs: 0, ignoreHttpsErrors: true});
  assert.equal(resumed.summary.status, "pass");
});
