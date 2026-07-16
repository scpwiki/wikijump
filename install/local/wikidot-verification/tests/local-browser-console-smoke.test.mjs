import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";
import {parseArgs} from "../scripts/local-browser-console-smoke.mjs";
import {captureLocalSmoke, classifyFailureUrl, inspectLedger, localSmokeUrl, preflightEnShardManifest, RECORD_SCHEMA, RUNTIME_IDENTITY_SCHEMA, runLocalBrowserSmoke, sha256Value, validateRuntimeIdentity} from "../src/local-browser-console-smoke.mjs";

const SHA_A = "a".repeat(64);
const SHA_B = "b".repeat(64);
const identity = {
  schema: RUNTIME_IDENTITY_SCHEMA,
  wikijump_sha: "1".repeat(40),
  ftml_sha: "2".repeat(40),
  rustc_vv: "rustc 1.88.0\nbinary: rustc\ncommit-hash: 0123456789abcdef\ncommit-date: 2026-06-01\nhost: x86_64-unknown-linux-gnu\nrelease: 1.88.0\nLLVM version: 20.1.0",
  profile: "release",
  features: ["alpha", "beta"],
  deepwell_binary_or_image_sha256: "3".repeat(64),
  framerail_assets_sha256: "4".repeat(64),
  render_run_id: "render-20260716T120000Z",
};
const rowA = {fixture_id: "EN:a", family: "EN", slug: "a", local_https_url: "https://scp-wiki.wikijump.localhost/a"};

test("CLI and runtime identity require all authority inputs", () => {
  assert.throws(() => parseArgs(["--inventory", "i", "--shard-manifest", "s", "--shard-id", "en", "--output", "o", "--runtime-identity", "r"]), /--browser-executable is required/);
  assert.deepEqual(parseArgs(["--inventory", "i", "--shard-manifest", "s", "--shard-id", "en", "--output", "o", "--runtime-identity", "r", "--browser-executable", "b", "--browser-arg", "--host-resolver-rules=MAP x 127.0.0.2"]).browserArgs, ["--host-resolver-rules=MAP x 127.0.0.2"]);
  assert.equal(validateRuntimeIdentity(identity), identity);
  assert.throws(() => validateRuntimeIdentity({...identity, ftml_sha: "short"}), /ftml_sha/);
  assert.throws(() => validateRuntimeIdentity({...identity, features: ["beta", "alpha"]}), /sorted unique/);
  assert.throws(() => validateRuntimeIdentity({...identity, render_run_id: ""}), /render_run_id/);
});

test("slug and local URL validation reject traversal and URL syntax", () => {
  assert.equal(localSmokeUrl(rowA), "https://scp-wiki.wikijump.localhost/a");
  assert.equal(localSmokeUrl({...rowA, fixture_id: "EN:_404", slug: "_404", local_https_url: "https://scp-wiki.wikijump.localhost/_404"}), "https://scp-wiki.wikijump.localhost/_404");
  assert.equal(localSmokeUrl({...rowA, fixture_id: "EN:_template", slug: "_template", local_https_url: "https://scp-wiki.wikijump.localhost/_template"}), "https://scp-wiki.wikijump.localhost/_template");
  for (const slug of ["../a", "a/b", "a\\b", "a%2fb", "a.b", "a?x", "a#x", "-a", "a-"]) {
    assert.throws(() => localSmokeUrl({...rowA, slug, local_https_url: ""}), /safe ASCII grammar/);
  }
  assert.throws(() => localSmokeUrl({...rowA, local_https_url: "https://scp-wiki.wikijump.localhost:444/a"}), /must be exactly/);
  assert.throws(() => localSmokeUrl({...rowA, local_https_url: "https://scp-wiki.wikijump.localhost:443/a"}), /must be exactly/);
  assert.throws(() => localSmokeUrl({...rowA, local_https_url: "https://user@scp-wiki.wikijump.localhost/a"}), /must be exactly/);
});

test("failure attribution uses a narrow local allowlist and never treats unknown as local", () => {
  assert.equal(classifyFailureUrl("https://scp-wiki.wikijump.localhost/a"), "fail");
  assert.equal(classifyFailureUrl("https://scp-wiki.wikijump.localhost:443/a"), "external_unclassified");
  assert.equal(classifyFailureUrl("https://files.wjfiles.localhost/a.png"), "fail");
  assert.equal(classifyFailureUrl("https://bad.label.wjfiles.localhost/a.png"), "external_unclassified");
  assert.equal(classifyFailureUrl("http://127.0.0.1:3393/a"), "fail");
  assert.equal(classifyFailureUrl("http://[::1]:3393/a"), "fail");
  assert.equal(classifyFailureUrl("blob:https://scp-wiki.wikijump.localhost/id"), "fail");
  assert.equal(classifyFailureUrl("https://api.wikijump.localhost/rpc"), "external_unclassified");
  assert.equal(classifyFailureUrl("data:text/plain,x"), "unknown");
  assert.equal(classifyFailureUrl(null), "unknown");
});

test("shard preflight enforces unique IDs, ownership, and exact EN union", () => {
  const inventory = [rowA, {...rowA, fixture_id: "EN:b", slug: "b", local_https_url: "https://scp-wiki.wikijump.localhost/b"}];
  const manifest = {schema: "wikijump_full_parity.corpus_shard_manifest.v1", shards: [{shard_id: "s1", fixture_ids: ["EN:a"]}, {shard_id: "s2", fixture_ids: ["EN:b"]}]};
  assert.deepEqual(preflightEnShardManifest(inventory, manifest, "s2").map((row) => row.fixture_id), ["EN:b"]);
  assert.throws(() => preflightEnShardManifest(inventory, {...manifest, shards: [...manifest.shards, {shard_id: "s2", fixture_ids: []}]}, "s1"), /duplicate shard_id/);
  assert.throws(() => preflightEnShardManifest(inventory, {...manifest, shards: [{shard_id: "s1", fixture_ids: ["EN:a"]}]}, "s1"), /missing from the shard union/);
  assert.throws(() => preflightEnShardManifest(inventory, {...manifest, shards: [{shard_id: "s1", fixture_ids: ["EN:a"]}, {shard_id: "s2", fixture_ids: ["EN:a", "EN:b"]}]}, "s1"), /multiple shards/);
});

function makePage({finalUrl = localSmokeUrl(rowA), emit = null, frames = null} = {}) {
  const handlers = new Map();
  const removed = [];
  const mainFrame = {url: () => finalUrl, evaluate: async () => []};
  const page = {
    on(event, handler) { handlers.set(event, handler); },
    off(event, handler) { assert.equal(handlers.get(event), handler); removed.push(event); handlers.delete(event); },
    mainFrame() { return mainFrame; },
    async goto(url) {
      await emit?.(handlers, page, url);
      return {status: () => 200, url: () => finalUrl};
    },
    async waitForTimeout() {},
    locator() { return {count: async () => 1}; },
    frames() { return frames ?? [mainFrame]; },
    url() { return finalUrl; },
    removed,
  };
  return page;
}

test("capture records redirects, requires exact final URL, and detaches listeners", async () => {
  const redirected = "https://scp-wiki.wikijump.localhost/b";
  const page = makePage({
    finalUrl: redirected,
    emit: async (handlers, target) => {
      const request = {isNavigationRequest: () => true, frame: () => target.mainFrame(), resourceType: () => "document"};
      handlers.get("response")({url: () => localSmokeUrl(rowA), status: () => 302, request: () => request});
      handlers.get("response")({url: () => redirected, status: () => 200, request: () => request});
    },
  });
  const record = await captureLocalSmoke(page, rowA, {timeoutMs: 100, settleMs: 0});
  assert.equal(record.final_url, redirected);
  assert.deepEqual(record.redirect_chain.map((entry) => entry.status), [302, 200]);
  assert.equal(record.result, "fail");
  assert.deepEqual(page.removed.sort(), ["console", "pageerror", "requestfailed", "response"]);
});

test("external and unknown failures require review while frame inspection failures fail", async () => {
  const external = makePage({emit: async (handlers) => handlers.get("console")({type: () => "error", text: () => "cdn error", location: () => ({url: "https://cdn.example/x.js"})})});
  assert.equal((await captureLocalSmoke(external, rowA, {timeoutMs: 100, settleMs: 0})).result, "review");
  const brokenFrame = {url: () => "https://scp-wiki.wikijump.localhost/frame", async evaluate() { throw new Error("detached"); }};
  const page = makePage({frames: [makePage().mainFrame(), brokenFrame]});
  const record = await captureLocalSmoke(page, rowA, {timeoutMs: 100, settleMs: 0});
  assert.equal(record.result, "fail");
  assert.match(record.page_errors[0].message, /image inspection failed/);
});

function validRecord(fingerprint, overrides = {}) {
  return {schema: RECORD_SCHEMA, fixture_id: "EN:a", slug: "a", url: localSmokeUrl(rowA), status: 200, final_url: localSmokeUrl(rowA), redirect_chain: [{url: localSmokeUrl(rowA), status: 200}], page_content_count: 1, visible_broken_images: [], console_errors: [], page_errors: [], request_failures: [], http_error_responses: [], navigation_error: null, result: "pass", run_fingerprint_sha256: fingerprint, ...overrides};
}

test("resume truncates only an unterminated tail and rejects middle or fabricated records", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-smoke-ledger-"));
  const output = path.join(root, "records.jsonl");
  const fingerprint = SHA_A;
  await fs.writeFile(output, `${JSON.stringify(validRecord(fingerprint))}\n{"partial"`, "utf8");
  const repaired = await inspectLedger(output, fingerprint, [rowA]);
  assert.equal(repaired.truncatedTail, true);
  assert.deepEqual(repaired.observed, ["EN:a"]);
  assert.equal((await fs.readFile(output, "utf8")).endsWith("\n"), true);
  await fs.writeFile(output, `${JSON.stringify(validRecord(fingerprint))}\n{broken}\n`, "utf8");
  assert.equal((await inspectLedger(output, fingerprint, [rowA])).ledgerErrors.length, 1);
  await fs.writeFile(output, `${JSON.stringify({schema: RECORD_SCHEMA, fixture_id: "EN:a", run_fingerprint_sha256: fingerprint})}\n`, "utf8");
  assert.match((await inspectLedger(output, fingerprint, [rowA])).ledgerErrors[0], /slug\/url contract|status/);
  await fs.writeFile(output, `${JSON.stringify(validRecord(fingerprint, {status: 500, redirect_chain: [{url: localSmokeUrl(rowA), status: 500}], result: "fail"}))}\n`, "utf8");
  assert.match((await inspectLedger(output, fingerprint, [rowA])).ledgerErrors[0], /only complete exact-URL 2xx pass/);
});

function fakeChromium(version = "Chromium 130") {
  let contexts = 0;
  let launchOptions = null;
  return {
    get contexts() { return contexts; },
    get launchOptions() { return launchOptions; },
    async launch(options) {
      launchOptions = options;
      return {
        version: () => version,
        async newContext() {
          contexts += 1;
          return {async newPage() { return {...makePage(), async close() {}}; }, async close() {}};
        },
        async close() {},
      };
    },
  };
}

function driverOptions(root, chromium) {
  return {chromium, rows: [rowA], outputPath: path.join(root, "smoke.jsonl"), runtimeIdentity: identity, inventoryPath: "/inventory.json", inventorySha256: SHA_A, shardManifestPath: "/shards.json", shardManifestSha256: SHA_B, shardId: "en", browserExecutable: "/browser/chrome", browserExecutableSha256: "c".repeat(64), browserArgs: ["--host-resolver-rules=MAP x 127.0.0.2"], workers: 2, timeoutMs: 100, settleMs: 0, ignoreHttpsErrors: true};
}

test("driver fingerprints browser/config/inputs, resumes exact records, and owner-locks output", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wj-smoke-driver-"));
  const chromium = fakeChromium();
  const options = driverOptions(root, chromium);
  const first = await runLocalBrowserSmoke(options);
  assert.equal(first.summary.status, "pass");
  assert.equal(first.summary.run_contract.browser.version, "Chromium 130");
  assert.equal(first.summary.run_contract.inventory_sha256, SHA_A);
  assert.deepEqual(first.summary.run_contract.capture_config.browser_args, options.browserArgs);
  assert.deepEqual(chromium.launchOptions.args, options.browserArgs);
  assert.equal(chromium.contexts, 1);
  const resumed = await runLocalBrowserSmoke(options);
  assert.equal(resumed.summary.status, "pass");
  assert.equal(chromium.contexts, 1);
  const changed = await runLocalBrowserSmoke({...options, chromium: fakeChromium("Chromium 131")});
  assert.equal(changed.summary.status, "fail");
  assert.match(changed.summary.ledger_errors[0], /run fingerprint/);
  await fs.writeFile(`${options.outputPath}.lock`, "owned\n", "utf8");
  await assert.rejects(() => runLocalBrowserSmoke(options), /owner lock already exists/);
  await fs.rm(`${options.outputPath}.lock`);
});

test("run contract selected-row hash is stable and excludes unselected rows", () => {
  assert.equal(sha256Value([{fixture_id: rowA.fixture_id, family: rowA.family, slug: rowA.slug, url: localSmokeUrl(rowA)}]).length, 64);
});
