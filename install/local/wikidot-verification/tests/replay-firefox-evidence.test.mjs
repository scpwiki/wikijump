import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";

import {
  FIREFOX_REPLAY_NORMALIZATIONS,
  buildReplayVerdict,
  parseArgs,
  replayNormalizations,
  runFirefoxEvidenceReplay,
  usage,
} from "../scripts/replay-firefox-evidence.mjs";

function baseArgs(overrides = {}) {
  return {
    cacheDir: "/tmp/firefox-replay-cache",
    cacheIdentity: "test-identity",
    url: "https://scp-wiki.wikidot.com/scp-9506",
    outputDir: "/tmp/firefox-replay-out",
    firefoxExecutable: null,
    browserRoot: "/tmp/browser-root",
    viewport: {width: 1366, height: 900},
    timeoutMs: 120_000,
    settleMs: 1_500,
    excludePrefixes: [],
    ...overrides,
  };
}

function baseVerdictInput(overrides = {}) {
  return {
    url: "https://scp-wiki.wikidot.com/scp-9506",
    cacheDir: "/tmp/firefox-replay-cache",
    cacheIdentity: "test-identity",
    manifestSha256Before: "before",
    manifestSha256After: "before",
    browser: {engine: "firefox", version: "153.0", executable_sha256: "abc"},
    navigationStatus: 200,
    navigationError: null,
    images: {images: 36, broken: 0, incomplete_with_dimensions: 0, zero_natural: 0},
    requestCount: 89,
    failedRequests: 0,
    gateSnapshot: {external_network_requests: 0, enforcement_failed: false, unsupported_requests_blocked: 0},
    cacheSnapshot: {misses: 0, stores: 0},
    ...overrides,
  };
}

test("firefox evidence replay CLI parses cache, url, and output options", () => {
  const parsed = parseArgs([
    "--cache-dir", "cache",
    "--cache-identity", "scp9506-live-20261012-chromium149",
    "--url", "https://scp-wiki.wikidot.com/scp-9506",
    "--output-dir", "out",
    "--viewport", "1366x900",
  ]);
  assert.equal(parsed.cacheIdentity, "scp9506-live-20261012-chromium149");
  assert.equal(parsed.url, "https://scp-wiki.wikidot.com/scp-9506");
  assert.deepEqual(parsed.viewport, {width: 1366, height: 900});
  assert.equal(parsed.timeoutMs, 120_000);
  assert.equal(parsed.settleMs, 1_500);
  assert.ok(path.isAbsolute(parsed.cacheDir));
  assert.ok(path.isAbsolute(parsed.outputDir));
});

function baseArgsToArgv() {
  return [
    "--cache-dir", "cache",
    "--cache-identity", "id",
    "--url", "https://scp-wiki.wikidot.com/scp-9506",
    "--output-dir", "out",
  ];
}

test("firefox evidence replay CLI rejects missing inputs and non-evidence urls", () => {
  assert.throws(() => parseArgs(["--cache-dir", "c", "--cache-identity", "i", "--url", "https://x.test/"]), /--output-dir is required/u);
  assert.throws(() => parseArgs([...baseArgsToArgv(), "--unknown", "x"]), /unknown argument/u);
  assert.throws(() => parseArgs([...baseArgsToArgv(), "--viewport", "wide"]), /--viewport must be WIDTHxHEIGHT/u);
  assert.throws(() => parseArgs([...baseArgsToArgv(), "--url", "data:text/plain,hi"]), /absolute http\(s\) URL/u);
  assert.match(usage(), /--cache-dir/u);
  assert.match(usage(), /never fetches live evidence/u);
  assert.match(usage(), /--exclude-prefix/u);
});

test("firefox evidence replay CLI accumulates repeatable exclude prefixes", () => {
  const parsed = parseArgs([
    ...baseArgsToArgv(),
    "--exclude-prefix", "https://s.nitropay.com/",
    "--exclude-prefix", "https://ad.doubleclick.net/",
  ]);
  assert.deepEqual(parsed.excludePrefixes, ["https://s.nitropay.com/", "https://ad.doubleclick.net/"]);
  assert.deepEqual(parseArgs(baseArgsToArgv()).excludePrefixes, []);
  assert.deepEqual(replayNormalizations([]), FIREFOX_REPLAY_NORMALIZATIONS);
  const withExclusions = replayNormalizations(["https://s.nitropay.com/"]);
  assert.equal(withExclusions.length, FIREFOX_REPLAY_NORMALIZATIONS.length + 1);
  assert.equal(withExclusions.at(-1).kind, "excluded_request_prefixes_aborted");
});

test("firefox evidence replay verdict passes only on a sealed replay", () => {
  const clean = buildReplayVerdict(baseVerdictInput());
  assert.equal(clean.status, "pass");
  assert.equal(clean.schema, "wikijump.firefox_evidence_replay.v1");
  assert.deepEqual(clean.harness_normalizations, FIREFOX_REPLAY_NORMALIZATIONS);
  assert.ok(clean.harness_normalizations.length > 0);

  assert.equal(buildReplayVerdict(baseVerdictInput({cacheSnapshot: {misses: 1, stores: 0}})).status, "fail");
  assert.equal(buildReplayVerdict(baseVerdictInput({gateSnapshot: {external_network_requests: 1, enforcement_failed: false, unsupported_requests_blocked: 0}})).status, "fail");
  assert.equal(buildReplayVerdict(baseVerdictInput({gateSnapshot: {external_network_requests: 0, enforcement_failed: true, unsupported_requests_blocked: 0}})).status, "fail");
  assert.equal(buildReplayVerdict(baseVerdictInput({navigationStatus: null, navigationError: "timeout"})).status, "fail");
  assert.equal(buildReplayVerdict(baseVerdictInput({navigationStatus: 500})).status, "fail");
});

test("firefox evidence replay runs a cache-only page through the injected launcher", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-firefox-replay-test-"));
  const cacheDir = path.join(root, "cache");
  const outputDir = path.join(root, "out");
  const executable = path.join(root, "firefox");
  await fs.mkdir(cacheDir, {recursive: true, mode: 0o700});
  await fs.chmod(cacheDir, 0o700);
  await fs.writeFile(executable, "fake-firefox-bytes");
  const contextOptions = [];
  const routeHandlers = [];
  const fakePage = {
    on() {},
    async route(pattern, handler) {
      routeHandlers.push({pattern, handler});
    },
    async goto() {
      return {status: () => 200};
    },
    async waitForTimeout() {},
    async evaluate() {
      return [
        {complete: true, naturalWidth: 16, naturalHeight: 16},
        {complete: false, naturalWidth: 10, naturalHeight: 10},
      ];
    },
  };
  const fakeContext = {
    async route() {},
    async routeWebSocket() {},
    on() {},
    async newPage() {
      return fakePage;
    },
    async close() {},
  };
  const fakeBrowser = {
    version: () => "153.0-test",
    async newContext(options) {
      contextOptions.push(options);
      return fakeContext;
    },
    async close() {},
  };

  const {verdict} = await runFirefoxEvidenceReplay(
    baseArgs({cacheDir, outputDir, excludePrefixes: ["https://s.nitropay.com/"]}),
    {launchFirefox: async () => ({browser: fakeBrowser, executable})},
  );
  assert.equal(verdict.status, "pass");
  assert.equal(verdict.browser.engine, "firefox");
  assert.equal(verdict.browser.version, "153.0-test");
  assert.deepEqual(verdict.images, {images: 2, broken: 1, incomplete_with_dimensions: 1, zero_natural: 0});
  assert.equal(verdict.navigation_status, 200);
  assert.deepEqual(contextOptions[0].viewport, {width: 1366, height: 900});
  assert.equal(contextOptions[0].serviceWorkers, "block");
  assert.equal(contextOptions[0].ignoreHTTPSErrors, true);
  const written = JSON.parse(await fs.readFile(path.join(outputDir, "firefox-evidence-replay.json"), "utf8"));
  assert.equal(written.status, "pass");
  assert.deepEqual(written.excluded_request_prefixes, ["https://s.nitropay.com/"]);
  assert.equal(written.harness_normalizations.length, FIREFOX_REPLAY_NORMALIZATIONS.length + 1);

  assert.equal(routeHandlers.length, 1);
  const outcomes = [];
  const fakeRoute = (url) => ({
    request: () => ({url: () => url}),
    async abort(reason) {
      outcomes.push({url, outcome: `aborted:${reason}`});
    },
    async fallback() {
      outcomes.push({url, outcome: "fallback"});
    },
  });
  await routeHandlers[0].handler(fakeRoute("https://s.nitropay.com/ads-143.js"));
  await routeHandlers[0].handler(fakeRoute("https://scp-wiki.wdfiles.com/local--files/x/y.css"));
  assert.deepEqual(outcomes, [
    {url: "https://s.nitropay.com/ads-143.js", outcome: "aborted:blockedbyclient"},
    {url: "https://scp-wiki.wdfiles.com/local--files/x/y.css", outcome: "fallback"},
  ]);

  await assert.rejects(
    () => runFirefoxEvidenceReplay(baseArgs({cacheDir, outputDir}), {launchFirefox: async () => ({browser: fakeBrowser, executable})}),
    /EEXIST/,
  );
  await fs.rm(root, {recursive: true, force: true});
});
