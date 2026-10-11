import assert from "node:assert/strict";
import {spawn} from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";
import {
  acquireBrowserCaptureLock,
  createBrowserRequestGate,
  createBrowserResponseCache,
  createPersistentBrowserRequestGate,
  isWikidotCapturePublicOrigin,
  installBrowserRequestGate,
  isCaptureDependencyResourceType,
  localBrowserCaptureOrigins,
  parseRetryAfterMilliseconds,
} from "../src/browser-request-gate.mjs";

function createClock({failSleeps = 0} = {}) {
  let milliseconds = 0;
  let remainingFailures = failSleeps;
  const sleeps = [];
  return {
    now() {
      return milliseconds;
    },
    sleep: async (duration) => {
      sleeps.push(duration);
      if (remainingFailures > 0) {
        remainingFailures -= 1;
        throw new Error("simulated clock failure");
      }
      milliseconds += duration;
    },
    set(value) {
      milliseconds = value;
    },
    sleeps,
  };
}

function createContext() {
  const routes = [];
  const webSocketRoutes = [];
  const events = new Map();
  return {
    routes,
    webSocketRoutes,
    events,
    async route(pattern, handler) {
      routes.push({pattern, handler});
    },
    async routeWebSocket(pattern, handler) {
      webSocketRoutes.push({pattern, handler});
    },
    on(event, handler) {
      events.set(event, handler);
    },
  };
}

function createRoute(url, {abortError = null, continueError = null, method = "GET", resourceType = "script", headers = {}, visibleHeaders = headers, allHeaders = headers, fetchResponse = null, frameUrl = null} = {}) {
  const actions = [];
  const request = {url: () => url, method: () => method, resourceType: () => resourceType, headers: () => visibleHeaders, allHeaders: async () => allHeaders, frame: () => frameUrl === null ? null : {url: () => frameUrl}};
  return {
    actions,
    request() {
      return request;
    },
    async continue() {
      actions.push({type: "continue"});
      if (continueError) throw continueError;
    },
    async abort(reason) {
      actions.push({type: "abort", reason});
      if (abortError) throw abortError;
    },
    async fetch(options) {
      actions.push({type: "fetch", options});
      if (!fetchResponse) throw new Error("unexpected route fetch");
      return fetchResponse;
    },
    async fulfill(options) {
      actions.push({type: "fulfill", status: options.status ?? options.response?.status() ?? null});
    },
  };
}

function createFetchResponse({status = 200, headers = {}, visibleHeaders = null, allHeaders = null, body = "asset"} = {}) {
  const completeHeaders = {"content-length": String(Buffer.byteLength(body)), ...headers, ...(allHeaders ?? {})};
  return {
    status: () => status,
    headers: () => visibleHeaders ?? {"content-length": String(Buffer.byteLength(body)), ...headers},
    allHeaders: async () => completeHeaders,
    body: async () => Buffer.from(body),
  };
}

function runNodeChild(source) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ["--input-type=module", "--eval", source], {stdio: ["ignore", "pipe", "pipe"]});
    const stderr = [];
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", reject);
    child.on("close", (code, signal) => {
      if (code === 0 && signal === null) resolve();
      else reject(new Error(`child cache fill failed (${code ?? signal}): ${Buffer.concat(stderr).toString("utf8")}`));
    });
  });
}

test("default shared gate has no fixed delay", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({now: clock.now, sleep: clock.sleep});

  const grants = await Promise.all([gate.acquire(), gate.acquire(), gate.acquire()]);

  assert.deepEqual(grants.map((grant) => grant.released_at_epoch_ms), [0, 0, 0]);
  assert.deepEqual(clock.sleeps, []);
  assert.equal(gate.snapshot().interval_ms, 0);
});

test("synthetic local-source admissions never wait for public throttling or Retry-After", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});

  await gate.acquire();
  clock.set(100);
  assert.equal(await gate.deferForRetryAfter("5"), true);
  gate.recordSyntheticPublicAdmission(2);

  assert.deepEqual(clock.sleeps, []);
  assert.equal(gate.snapshot().public_requests, 3);
  assert.equal(gate.snapshot().external_network_requests, 1);
  assert.equal(gate.snapshot().synthetic_public_admissions, 2);
  assert.deepEqual(gate.snapshot().grants.map(({ released_at_epoch_ms }) => released_at_epoch_ms), [0, 100, 100]);
});

test("shared gate can still enforce an explicit four-second interval", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});

  const grants = await Promise.all([gate.acquire(), gate.acquire(), gate.acquire()]);

  assert.deepEqual(grants.map((grant) => grant.released_at_epoch_ms), [0, 4_000, 8_000]);
  assert.deepEqual(clock.sleeps, [4_000, 4_000]);
  assert.deepEqual(gate.snapshot().grants.map((grant) => grant.sequence), [1, 2, 3]);
});

test("Retry-After extends a shared gate without accepting an invalid or unbounded value", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});

  await gate.acquire();
  clock.set(100);
  assert.equal(await gate.deferForRetryAfter("5"), true);
  assert.equal(await gate.deferForRetryAfter("9".repeat(400)), false);
  const delayed = await gate.acquire();

  assert.equal(delayed.released_at_epoch_ms, 5_100);
  assert.equal(parseRetryAfterMilliseconds("3"), 3_000);
  assert.equal(parseRetryAfterMilliseconds("not-a-date"), null);
  assert.equal(gate.snapshot().retry_after_honored, 1);
  assert.equal(gate.snapshot().retry_after_invalid, 1);
});

test("source and local contexts share the gate while only the exact local origin is exempt", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const sourceContext = createContext();
  const localContext = createContext();
  await installBrowserRequestGate(sourceContext, {gate});
  await installBrowserRequestGate(localContext, {gate, exemptOrigins: ["https://scp-wiki.wikijump.localhost"]});

  const localExact = createRoute("https://scp-wiki.wikijump.localhost/scp-173");
  assert.equal(localContext.routes[0].pattern(new URL(localExact.request().url())), false);
  localContext.events.get("request")(localExact.request());
  const source = createRoute("https://scp-wiki.wikidot.com/scp-173");
  const wrongPort = createRoute("https://scp-wiki.wikijump.localhost:18443/scp-173");
  assert.equal(localContext.routes[0].pattern(new URL(wrongPort.request().url())), true);
  await Promise.all([sourceContext.routes[0].handler(source), localContext.routes[0].handler(wrongPort)]);
  let connected = false;
  await sourceContext.webSocketRoutes[0].handler({connectToServer() { connected = true; }});

  assert.deepEqual(localExact.actions, []);
  assert.deepEqual(source.actions, [{type: "continue"}]);
  assert.deepEqual(wrongPort.actions, [{type: "continue"}]);
  assert.equal(connected, false);
  assert.deepEqual(gate.snapshot().grants.map((grant) => grant.released_at_epoch_ms), [0, 4_000]);
  assert.equal(gate.snapshot().local_exempt_requests, 1);
  assert.equal(gate.snapshot().websocket_connections_blocked, 1);
});

test("the public gate admits Wikidot and css.wikidot.com but blocks unrelated public hosts before admission", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  gate.setActiveFixture("syntax-collapsible");
  const context = createContext();
  await installBrowserRequestGate(context, {gate, publicOriginPredicate: isWikidotCapturePublicOrigin});
  const handler = context.routes[0].handler;
  const ad = createRoute("https://api.rlcdn.com/api/identity");
  const css = createRoute("https://css.wikidot.com/local--code/1", {resourceType: "stylesheet"});
  const page = createRoute("http://sandbox-for-codex.wikidot.com/codex-oracle:fixture");
  const styleFrame = createRoute("https://interwiki.scpwiki.com/styleFrame.html?priority=1&theme=example", {resourceType: "document"});
  const styleFrameAsScript = createRoute("https://interwiki.scpwiki.com/styleFrame.html?priority=1&theme=example", {resourceType: "script"});
  const styleFramePost = createRoute("https://interwiki.scpwiki.com/styleFrame.html?priority=1&theme=example", {method: "POST", resourceType: "document"});
  const interwikiRuntime = createRoute("https://interwiki.scpwiki.com/interwiki.js", {resourceType: "script", frameUrl: "https://interwiki.scpwiki.com/interwikiFrame.html?lang=en"});
  const resizeRuntime = createRoute("https://interwiki.scpwiki.com/resizeIframe.js", {resourceType: "script", frameUrl: "https://interwiki.scpwiki.com/styleFrame.html?priority=1"});
  const unknownInterwikiScript = createRoute("https://interwiki.scpwiki.com/other.js", {resourceType: "script"});

  await handler(ad);
  await handler(css);
  await handler(page);
  await handler(styleFrame);
  await handler(styleFrameAsScript);
  await handler(styleFramePost);
  await handler(interwikiRuntime);
  await handler(resizeRuntime);
  await handler(unknownInterwikiScript);

  assert.deepEqual(ad.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.deepEqual(css.actions, [{type: "continue"}]);
  assert.deepEqual(page.actions, [{type: "continue"}]);
  assert.equal(gate.snapshot().public_requests, 5);
  assert.deepEqual(gate.snapshot().blocked_hosts, {"api.rlcdn.com": 1, "interwiki.scpwiki.com": 3});
  assert.deepEqual(gate.snapshot().blocked_hosts_by_fixture, {"syntax-collapsible": {"api.rlcdn.com": 1, "interwiki.scpwiki.com": 3}});
  assert.equal(isWikidotCapturePublicOrigin("https://css.wikidot.com"), true);
  assert.equal(isWikidotCapturePublicOrigin("https://wikidot.com"), true);
  assert.equal(isWikidotCapturePublicOrigin("http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--javascript/WIKIDOT.combined.js"), true);
  assert.equal(isWikidotCapturePublicOrigin("https://d3g0gp89917ko0.cloudfront.net/ads.js"), false);
  assert.deepEqual(styleFrameAsScript.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.deepEqual(styleFramePost.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.deepEqual(interwikiRuntime.actions, [{type: "continue"}]);
  assert.deepEqual(resizeRuntime.actions, [{type: "continue"}]);
  assert.deepEqual(unknownInterwikiScript.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/styleFrame.html?priority=1", "document", "GET"), true);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/interwikiFrame.html?lang=en", "document", "GET"), true);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/styleFrame.html", "script", "GET"), false);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/styleFrame.html", "document", "POST"), false);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/interwiki.js", "script", "GET", "https://interwiki.scpwiki.com/interwikiFrame.html"), true);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/resizeIframe.js", "script", "GET", "https://interwiki.scpwiki.com/styleFrame.html"), true);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/interwiki.js", "script", "GET"), false);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/other.js", "script", "GET"), false);
  assert.equal(isWikidotCapturePublicOrigin("https://interwiki.scpwiki.com/other.html", "document", "GET"), false);
  assert.equal(isWikidotCapturePublicOrigin("http://interwiki.scpwiki.com/styleFrame.html", "document", "GET"), false);
  assert.equal(isWikidotCapturePublicOrigin("https://example.com"), false);
});

test("theme dependencies are admitted by resource type while third-party execution is blocked", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, publicOriginPredicate: isWikidotCapturePublicOrigin});
  const handler = context.routes[0].handler;
  const stylesheet = createRoute("https://cdn.scpwiki.com/theme/en/sigma/theme.css", {resourceType: "stylesheet"});
  const font = createRoute("https://cdn.scpwiki.com/theme/en/sigma/font.woff2", {resourceType: "font"});
  const image = createRoute("https://cdn.scpwiki.com/theme/en/sigma/logo.svg", {resourceType: "image"});
  const script = createRoute("https://cdn.scpwiki.com/theme/en/sigma/theme.js", {resourceType: "script"});

  await handler(stylesheet);
  await handler(font);
  await handler(image);
  await handler(script);

  assert.deepEqual(stylesheet.actions, [{type: "continue"}]);
  assert.deepEqual(font.actions, [{type: "continue"}]);
  assert.deepEqual(image.actions, [{type: "continue"}]);
  assert.deepEqual(script.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(isCaptureDependencyResourceType("stylesheet"), true);
  assert.equal(isCaptureDependencyResourceType("font"), true);
  assert.equal(isCaptureDependencyResourceType("image"), true);
  assert.equal(isCaptureDependencyResourceType("script"), false);
  assert.equal(gate.snapshot().public_requests, 3);
  assert.deepEqual(gate.snapshot().blocked_hosts, {"cdn.scpwiki.com": 1});
});

test("a source response cache serves repeated cacheable assets without another gate grant", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const responseCache = createBrowserResponseCache();
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const handler = context.routes[0].handler;
  const url = "https://cdn.example.test/shared.css";
  const first = createRoute(url, {
    resourceType: "stylesheet",
    fetchResponse: createFetchResponse({headers: {"cache-control": "public, max-age=600"}, body: "body{}"}),
  });
  const second = createRoute(url, {resourceType: "stylesheet"});

  await handler(first);
  await handler(second);

  assert.deepEqual(first.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 200},
  ]);
  assert.deepEqual(second.actions, [{type: "fulfill", status: 200}]);
  assert.equal(gate.snapshot().public_requests, 1);
  assert.deepEqual(responseCache.snapshot(), {
    schema: "wikijump_full_parity.browser_response_cache.v1",
    entries: 1,
    bytes: 6,
    hits: 1,
    misses: 1,
    stores: 1,
    bypasses: 0,
    evictions: 0,
    max_entries: 512,
    max_bytes: 64 * 1024 * 1024,
    max_entry_bytes: 8 * 1024 * 1024,
    lookup_key: "exact_url",
    lifetime: "browser_context",
    documents_cached: false,
    evidence_replay: false,
    retention_policy: "bounded_lru",
    exact_variant_hits: 0,
    legacy_variant_misses: 0,
    variant_stores: 0,
  });
});

test("a persistent response cache replays a stable external 404 without another request", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-negative-cache-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const identity = "live-reference:avatar-19102600";
  const firstGate = createBrowserRequestGate({intervalMs: 4_000});
  const firstCache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: identity, evidenceReplay: true});
  await firstCache.load();
  const firstContext = createContext();
  await installBrowserRequestGate(firstContext, {gate: firstGate, responseCache: firstCache});
  await firstContext.routes[0].handler(createRoute("https://www.wikidot.com/avatar.php?userid=19102600&amp;size=small", {
    resourceType: "image",
    fetchResponse: createFetchResponse({status: 404, headers: {"cache-control": "public, max-age=600"}, body: "not found"}),
  }));
  await firstCache.flush();

  const secondGate = createBrowserRequestGate({intervalMs: 4_000});
  const secondCache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: identity, evidenceReplay: true});
  await secondCache.load();
  const secondContext = createContext();
  await installBrowserRequestGate(secondContext, {gate: secondGate, responseCache: secondCache, cacheOnly: true});
  const second = createRoute("https://www.wikidot.com/avatar.php?userid=19102600&amp;size=small", {resourceType: "image"});
  await secondContext.routes[0].handler(second);

  assert.deepEqual(second.actions, [{type: "fulfill", status: 404}]);
  assert.equal(secondGate.snapshot().public_requests, 0);
  assert.equal(secondGate.snapshot().external_network_requests, 0);
  assert.equal(secondGate.snapshot().synthetic_public_admissions, 0);
  assert.equal(secondCache.snapshot().persistent_entries_loaded, 1);

  const timestamped = createRoute("https://www.wikidot.com/avatar.php?userid=19102600&amp;size=small&amp;timestamp=1788341729", {resourceType: "image"});
  await secondContext.routes[0].handler(timestamped);
  assert.deepEqual(timestamped.actions, [{type: "fulfill", status: 404}]);
  assert.equal(secondGate.snapshot().public_requests, 0);
});

test("candidate cache misses for unsupported scripts abort without an external request or gate grant", async () => {
  const gate = createBrowserRequestGate({intervalMs: 4_000});
  const responseCache = createBrowserResponseCache();
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache, cacheOnly: true});
  const miss = createRoute("https://rsms.me/inter/inter.js", {resourceType: "script"});

  await context.routes[0].handler(miss);

  assert.deepEqual(miss.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(gate.snapshot().public_requests, 0);
  assert.equal(gate.snapshot().enforcement_failed, true);
  assert.equal(responseCache.snapshot().misses, 1);
});

test("candidate cacheOnly misses for public assets fail closed without fetch or manifest mutation", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-cacheonly-miss-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({
    persistentDir: cacheDir,
    persistentIdentity: "candidate-cacheonly-miss",
    evidenceReplay: true,
  });
  await responseCache.load();
  assert.equal(responseCache.store("https://cached.example.test/retained.css", {
    status: 200,
    headers: {"content-type": "text/css"},
    body: Buffer.from("retained"),
  }), true);
  await responseCache.flush();
  const manifestPath = path.join(cacheDir, "manifest.json");
  const manifestBefore = await fs.readFile(manifestPath);
  const storesBefore = responseCache.snapshot().stores;
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache, cacheOnly: true});
  const url = "https://scp-wiki.wdfiles.com/local--files/component:theme/missing.css";
  const miss = createRoute(url, {
    resourceType: "stylesheet",
    fetchResponse: createFetchResponse({headers: {"cache-control": "public, max-age=600"}, body: "must-not-be-fetched"}),
  });

  await context.routes[0].handler(miss);
  assert.deepEqual(miss.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(gate.snapshot().public_requests, 0);
  assert.equal(gate.snapshot().external_network_requests, 0);
  assert.equal(gate.snapshot().enforcement_failed, true);
  assert.equal(responseCache.snapshot().misses, 1);
  assert.equal(responseCache.snapshot().stores, storesBefore);
  assert.deepEqual(await fs.readFile(manifestPath), manifestBefore);
});

test("a fetched persistent evidence entry is durable before the routed response completes", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-immediate-persist-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const identity = "candidate-evidence:immediate-persist";
  const url = "https://scp-wiki.wdfiles.com/local--files/component:theme/durable.css";
  const firstGate = createBrowserRequestGate({intervalMs: 0});
  const firstCache = createBrowserResponseCache({
    persistentDir: cacheDir,
    persistentIdentity: identity,
    evidenceReplay: true,
  });
  await firstCache.load();
  const firstContext = createContext();
  await installBrowserRequestGate(firstContext, {
    gate: firstGate,
    responseCache: firstCache,
  });

  await firstContext.routes[0].handler(createRoute(url, {
    resourceType: "stylesheet",
    fetchResponse: createFetchResponse({body: "durable-before-fulfill"}),
  }));

  const secondGate = createBrowserRequestGate({intervalMs: 0});
  const secondCache = createBrowserResponseCache({
    persistentDir: cacheDir,
    persistentIdentity: identity,
    evidenceReplay: true,
  });
  await secondCache.load();
  const secondContext = createContext();
  await installBrowserRequestGate(secondContext, {
    gate: secondGate,
    responseCache: secondCache,
    cacheOnly: true,
  });
  const replay = createRoute(url, {resourceType: "stylesheet"});
  await secondContext.routes[0].handler(replay);

  assert.deepEqual(replay.actions, [{type: "fulfill", status: 200}]);
  assert.equal(secondGate.snapshot().external_network_requests, 0);
  assert.equal(secondCache.snapshot().persistent_entries_loaded, 1);
});

test("concurrent persistent cache instances merge distinct retained responses without losing either", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-persistent-merge-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {
    persistentDir: cacheDir,
    persistentIdentity: "evidence:concurrent-merge",
    evidenceReplay: true,
  };
  const first = createBrowserResponseCache(options);
  const second = createBrowserResponseCache(options);
  await Promise.all([first.load(), second.load()]);
  assert.equal(first.store("https://cdn.example.test/first.css", {status: 200, headers: {}, body: Buffer.from("first")}), true);
  assert.equal(second.store("https://cdn.example.test/second.css", {status: 200, headers: {}, body: Buffer.from("second")}), true);

  await Promise.all([first.flush(), second.flush()]);

  const reloaded = createBrowserResponseCache(options);
  await reloaded.load();
  assert.equal(reloaded.get("https://cdn.example.test/first.css")?.body.toString(), "first");
  assert.equal(reloaded.get("https://cdn.example.test/second.css")?.body.toString(), "second");
  await assert.rejects(fs.lstat(path.join(cacheDir, "manifest.json.lock")), {code: "ENOENT"});
});

test("persistent cache instances fail closed when one request identity resolves to conflicting retained bytes", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-persistent-conflict-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {
    persistentDir: cacheDir,
    persistentIdentity: "evidence:conflicting-response",
    evidenceReplay: true,
  };
  const first = createBrowserResponseCache(options);
  const second = createBrowserResponseCache(options);
  await Promise.all([first.load(), second.load()]);
  const url = "https://cdn.example.test/same.css";
  assert.equal(first.store(url, {status: 200, headers: {}, body: Buffer.from("first")}), true);
  assert.equal(second.store(url, {status: 200, headers: {}, body: Buffer.from("second")}), true);
  await first.flush();

  await assert.rejects(second.flush(), /conflicts with persisted evidence/u);

  const reloaded = createBrowserResponseCache(options);
  await reloaded.load();
  assert.equal(reloaded.get(url)?.body.toString(), "first");
});

test("concurrent candidate cache misses abort without external fetch or evidence write", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache, cacheOnly: true});
  const url = "https://scp-wiki.wdfiles.com/local--files/component:theme/shared.css";
  const routes = [
    createRoute(url, {resourceType: "stylesheet", fetchResponse: createFetchResponse({body: "must-not-be-fetched"})}),
    createRoute(url, {resourceType: "stylesheet", fetchResponse: createFetchResponse({body: "must-not-be-fetched"})}),
  ];

  await Promise.all(routes.map((route) => context.routes[0].handler(route)));

  assert.deepEqual(routes.map(({actions}) => actions), [
    [{type: "abort", reason: "blockedbyclient"}],
    [{type: "abort", reason: "blockedbyclient"}],
  ]);
  assert.equal(gate.snapshot().external_network_requests, 0);
  assert.equal(gate.snapshot().public_requests, 0);
  assert.equal(responseCache.snapshot().stores, 0);
});

test("separate gate installations sharing one cache single-flight the same evidence identity", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const firstContext = createContext();
  const secondContext = createContext();
  await installBrowserRequestGate(firstContext, {gate, responseCache});
  await installBrowserRequestGate(secondContext, {gate, responseCache});
  const url = "https://scp-wiki.wdfiles.com/local--files/component:theme/cross-context.css";
  let releaseFetch;
  const pendingResponse = new Promise((resolve) => {
    releaseFetch = () => resolve(createFetchResponse({body: "cross-context-single-flight"}));
  });
  const first = createRoute(url, {resourceType: "stylesheet", fetchResponse: pendingResponse});
  const second = createRoute(url, {resourceType: "stylesheet"});

  const firstRun = firstContext.routes[0].handler(first);
  const secondRun = secondContext.routes[0].handler(second);
  await new Promise((resolve) => setImmediate(resolve));
  releaseFetch();
  await Promise.all([firstRun, secondRun]);

  assert.deepEqual(first.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 200},
  ]);
  assert.deepEqual(second.actions, [{type: "fulfill", status: 200}]);
  assert.equal(gate.snapshot().external_network_requests, 1);
  assert.equal(responseCache.snapshot().stores, 1);
});

test("concurrent live evidence misses for one request identity perform one external fetch", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://scp-wiki.wdfiles.com/local--files/component:theme/live-shared.css";
  let releaseFetch;
  const pendingResponse = new Promise((resolve) => {
    releaseFetch = () => resolve(createFetchResponse({body: "live-single-flight"}));
  });
  const first = createRoute(url, {resourceType: "stylesheet", fetchResponse: pendingResponse});
  const second = createRoute(url, {resourceType: "stylesheet"});

  const firstRun = context.routes[0].handler(first);
  const secondRun = context.routes[0].handler(second);
  await new Promise((resolve) => setImmediate(resolve));
  releaseFetch();
  await Promise.all([firstRun, secondRun]);

  assert.deepEqual(first.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 200},
  ]);
  assert.deepEqual(second.actions, [{type: "fulfill", status: 200}]);
  assert.equal(gate.snapshot().external_network_requests, 1);
  assert.equal(responseCache.snapshot().stores, 1);
});

test("evidence acquisition caches Range GETs by exact range identity", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://scp-wiki.wdfiles.com/local--files/scp-173/video.bin";
  const first = createRoute(url, {
    resourceType: "image",
    headers: {range: "bytes=0-3"},
    fetchResponse: createFetchResponse({status: 206, body: "ABCD"}),
  });
  const sameRange = createRoute(url, {
    resourceType: "image",
    headers: {range: "bytes=0-3"},
  });
  const otherRange = createRoute(url, {
    resourceType: "image",
    headers: {range: "bytes=4-7"},
    fetchResponse: createFetchResponse({status: 206, body: "EFGH"}),
  });

  await context.routes[0].handler(first);
  await context.routes[0].handler(sameRange);
  await context.routes[0].handler(otherRange);

  assert.deepEqual(first.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 206},
  ]);
  assert.deepEqual(sameRange.actions, [{type: "fulfill", status: 206}]);
  assert.deepEqual(otherRange.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 206},
  ]);
  assert.equal(gate.snapshot().external_network_requests, 2);
  assert.equal(responseCache.snapshot().entries, 2);
});

test("evidence acquisition keys HTTP conditional headers independently of response Vary", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const conditionalHeaders = [
    "if-match",
    "if-none-match",
    "if-modified-since",
    "if-unmodified-since",
    "if-range",
  ];

  for (const [index, name] of conditionalHeaders.entries()) {
    const url = `https://cdn.example.test/conditional-${index}.css`;
    const first = createRoute(url, {
      resourceType: "stylesheet",
      headers: {[name]: "value-a"},
      fetchResponse: createFetchResponse({body: `first-${name}`}),
    });
    const same = createRoute(url, {
      resourceType: "stylesheet",
      headers: {[name]: "value-a"},
    });
    const different = createRoute(url, {
      resourceType: "stylesheet",
      headers: {[name]: "value-b"},
      fetchResponse: createFetchResponse({body: `second-${name}`}),
    });

    await context.routes[0].handler(first);
    await context.routes[0].handler(same);
    await context.routes[0].handler(different);

    assert.deepEqual(first.actions, [
      {type: "fetch", options: {maxRedirects: 0}},
      {type: "fulfill", status: 200},
    ]);
    assert.deepEqual(same.actions, [{type: "fulfill", status: 200}]);
    assert.deepEqual(different.actions, [
      {type: "fetch", options: {maxRedirects: 0}},
      {type: "fulfill", status: 200},
    ]);
  }

  assert.equal(gate.snapshot().external_network_requests, conditionalHeaders.length * 2);
  assert.equal(responseCache.snapshot().entries, conditionalHeaders.length * 2);
});

test("candidate cache-only mode never refetches credential-bearing external requests", async (t) => {
  for (const [name, headers] of [
    ["authorization", {authorization: "Bearer secret"}],
    ["cookie", {cookie: "session=secret"}],
  ]) {
    await t.test(name, async () => {
      const gate = createBrowserRequestGate({intervalMs: 0});
      const responseCache = createBrowserResponseCache({evidenceReplay: true});
      const context = createContext();
      await installBrowserRequestGate(context, {gate, responseCache, cacheOnly: true});
      const route = createRoute(`https://scp-wiki.wdfiles.com/${name}.css`, {
        resourceType: "stylesheet",
        headers,
        fetchResponse: createFetchResponse({body: "must-not-fetch"}),
      });

      await context.routes[0].handler(route);

      assert.deepEqual(route.actions, [{type: "abort", reason: "blockedbyclient"}]);
      assert.equal(gate.snapshot().external_network_requests, 0);
      assert.equal(responseCache.snapshot().stores, 0);
      assert.equal(gate.snapshot().enforcement_failed, true);
    });
  }
});

test("cache eligibility uses Playwright allHeaders for security-sensitive request headers", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache, cacheOnly: true});
  const route = createRoute("https://scp-wiki.wdfiles.com/security-hidden.css", {
    resourceType: "stylesheet",
    visibleHeaders: {},
    allHeaders: {cookie: "session=secret"},
    fetchResponse: createFetchResponse({body: "must-not-fetch"}),
  });

  await context.routes[0].handler(route);

  assert.deepEqual(route.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(gate.snapshot().external_network_requests, 0);
  assert.equal(responseCache.snapshot().stores, 0);
});

test("normal cache eligibility uses Playwright allHeaders for Set-Cookie", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache();
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://cdn.example.test/security-hidden.css";
  const response = createFetchResponse({
    visibleHeaders: {"content-length": "5", "cache-control": "public, max-age=600"},
    allHeaders: {"content-length": "5", "cache-control": "public, max-age=600", "set-cookie": "session=secret"},
    body: "asset",
  });

  await context.routes[0].handler(createRoute(url, {resourceType: "stylesheet", fetchResponse: response}));
  await context.routes[0].handler(createRoute(url, {resourceType: "stylesheet", fetchResponse: response}));

  assert.equal(gate.snapshot().external_network_requests, 2);
  assert.equal(responseCache.snapshot().stores, 0);
});

test("evidence replay strips Set-Cookie from retained and replayed response headers", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://cdn.example.test/public-token.css";
  const first = createRoute(url, {
    resourceType: "stylesheet",
    fetchResponse: createFetchResponse({
      allHeaders: {
        "content-length": "5",
        "set-cookie": "public_token=stale; Path=/",
        "content-type": "text/css",
      },
      body: "asset",
    }),
  });
  await context.routes[0].handler(first);

  const retained = responseCache.get(url);
  assert.equal(retained.headers["set-cookie"], undefined);
  assert.equal(retained.headers["content-type"], "text/css");
});

test("persistent evidence cache load sanitizes legacy Set-Cookie without reacquisition", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-set-cookie-migration-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const url = "https://cdn.example.test/legacy.css";
  await fs.writeFile(path.join(cacheDir, "manifest.json"), `${JSON.stringify({
    schema: "wikijump_full_parity.browser_response_cache_store.v1",
    identity: "evidence:set-cookie-migration",
    entries: [{
      key: url,
      status: 200,
      headers: {"set-cookie": "legacy=stale", "content-type": "text/css"},
      body_base64: Buffer.from("legacy").toString("base64"),
    }],
  })}\n`, {mode: 0o600});
  await fs.chmod(cacheDir, 0o700);
  const responseCache = createBrowserResponseCache({
    persistentDir: cacheDir,
    persistentIdentity: "evidence:set-cookie-migration",
    evidenceReplay: true,
  });
  await responseCache.load();
  assert.equal(responseCache.get(url)?.headers["set-cookie"], undefined);
  await responseCache.flush();
  const migrated = JSON.parse(await fs.readFile(path.join(cacheDir, "manifest.json"), "utf8"));
  assert.equal(migrated.entries[0].headers["set-cookie"], undefined);
});

test("persistent evidence replay keeps HEAD identities distinct across reloads", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-head-replay-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const identity = "evidence:head-variant";
  const url = "https://cdn.example.test/head-target";
  const firstCache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: identity, evidenceReplay: true});
  await firstCache.load();
  const firstGate = createBrowserRequestGate({intervalMs: 0});
  const firstContext = createContext();
  await installBrowserRequestGate(firstContext, {gate: firstGate, responseCache: firstCache});
  await firstContext.routes[0].handler(createRoute(url, {
    method: "HEAD",
    fetchResponse: createFetchResponse({status: 204, body: ""}),
  }));

  const secondCache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: identity, evidenceReplay: true});
  await secondCache.load();
  const secondGate = createBrowserRequestGate({intervalMs: 0});
  const secondContext = createContext();
  await installBrowserRequestGate(secondContext, {gate: secondGate, responseCache: secondCache});
  const replay = createRoute(url, {method: "HEAD"});
  await secondContext.routes[0].handler(replay);

  assert.deepEqual(replay.actions, [{type: "fulfill", status: 204}]);
  assert.equal(secondGate.snapshot().external_network_requests, 0);
  assert.equal(secondCache.get(url), null);
});

test("persistent evidence replay binds Vary Origin to the exact anonymous request and replays each variant", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-vary-origin-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {
    persistentDir: cacheDir,
    persistentIdentity: "evidence:vary-origin",
    evidenceReplay: true,
  };
  const url = "https://cdn.example.test/vary.css";
  const firstCache = createBrowserResponseCache(options);
  await firstCache.load();
  const firstGate = createBrowserRequestGate({intervalMs: 0});
  const firstContext = createContext();
  await installBrowserRequestGate(firstContext, {gate: firstGate, responseCache: firstCache});

  const originA = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {origin: "https://a.example.test"},
    fetchResponse: createFetchResponse({headers: {vary: "Origin"}, body: "from-a"}),
  });
  const originAReplay = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {origin: "https://a.example.test"},
  });
  const originB = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {origin: "https://b.example.test"},
    fetchResponse: createFetchResponse({headers: {vary: "Origin"}, body: "from-b"}),
  });
  await firstContext.routes[0].handler(originA);
  await firstContext.routes[0].handler(originAReplay);
  await firstContext.routes[0].handler(originB);
  assert.equal(firstGate.snapshot().external_network_requests, 2);
  assert.equal(firstCache.snapshot().variant_stores, 2);
  assert.equal(firstCache.snapshot().exact_variant_hits, 1);

  const manifest = JSON.parse(await fs.readFile(path.join(cacheDir, "manifest.json"), "utf8"));
  assert.equal(manifest.entries.length, 2);
  assert.deepEqual(
    manifest.entries.map(({key, vary_binding}) => ({key, vary_binding})),
    [
      {key: url, vary_binding: {names: ["origin"], values: {origin: "https://a.example.test"}}},
      {key: url, vary_binding: {names: ["origin"], values: {origin: "https://b.example.test"}}},
    ],
  );

  const secondCache = createBrowserResponseCache(options);
  await secondCache.load();
  const secondGate = createBrowserRequestGate({intervalMs: 0});
  const secondContext = createContext();
  await installBrowserRequestGate(secondContext, {gate: secondGate, responseCache: secondCache, cacheOnly: true});
  for (const origin of ["https://a.example.test", "https://b.example.test"]) {
    const replay = createRoute(url, {resourceType: "stylesheet", allHeaders: {origin}});
    await secondContext.routes[0].handler(replay);
    assert.deepEqual(replay.actions, [{type: "fulfill", status: 200}]);
  }
  assert.equal(secondGate.snapshot().external_network_requests, 0);
  assert.equal(secondCache.snapshot().exact_variant_hits, 2);
});

test("Vary bindings distinguish an absent request header from an explicitly empty value", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://cdn.example.test/optional-origin.css";
  const absent = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {},
    fetchResponse: createFetchResponse({headers: {vary: "Origin"}, body: "absent"}),
  });
  const empty = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {origin: ""},
    fetchResponse: createFetchResponse({headers: {vary: "Origin"}, body: "empty"}),
  });
  const absentReplay = createRoute(url, {resourceType: "stylesheet", allHeaders: {}});

  await context.routes[0].handler(absent);
  await context.routes[0].handler(empty);
  await context.routes[0].handler(absentReplay);

  assert.equal(gate.snapshot().external_network_requests, 2);
  assert.equal(responseCache.snapshot().variant_stores, 2);
  assert.equal(responseCache.snapshot().exact_variant_hits, 1);
});

test("Vary binds the complete CORS preflight header tuple", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://cdn.example.test/cors-font.woff2";
  const vary = "Origin, Access-Control-Request-Headers, Access-Control-Request-Method";
  const firstHeaders = {
    origin: "https://a.example.test",
    "access-control-request-headers": "x-example",
    "access-control-request-method": "GET",
  };
  const secondHeaders = {...firstHeaders, "access-control-request-headers": "x-other"};
  await context.routes[0].handler(createRoute(url, {
    resourceType: "font",
    allHeaders: firstHeaders,
    fetchResponse: createFetchResponse({headers: {vary}, body: "first-cors"}),
  }));
  await context.routes[0].handler(createRoute(url, {resourceType: "font", allHeaders: firstHeaders}));
  await context.routes[0].handler(createRoute(url, {
    resourceType: "font",
    allHeaders: secondHeaders,
    fetchResponse: createFetchResponse({headers: {vary}, body: "second-cors"}),
  }));

  assert.equal(gate.snapshot().external_network_requests, 2);
  assert.equal(responseCache.snapshot().variant_stores, 2);
  assert.equal(responseCache.snapshot().exact_variant_hits, 1);
});

test("Vary binds Sec-Fetch request context without collapsing distinct destinations", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://cdn.example.test/context.svg";
  const vary = "Sec-Fetch-Dest, Sec-Fetch-Mode, Sec-Fetch-Site";
  const imageHeaders = {"sec-fetch-dest": "image", "sec-fetch-mode": "no-cors", "sec-fetch-site": "cross-site"};
  const styleHeaders = {"sec-fetch-dest": "style", "sec-fetch-mode": "no-cors", "sec-fetch-site": "cross-site"};
  await context.routes[0].handler(createRoute(url, {
    resourceType: "image",
    allHeaders: imageHeaders,
    fetchResponse: createFetchResponse({headers: {vary}, body: "image-context"}),
  }));
  await context.routes[0].handler(createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: styleHeaders,
    fetchResponse: createFetchResponse({headers: {vary}, body: "style-context"}),
  }));
  await context.routes[0].handler(createRoute(url, {resourceType: "image", allHeaders: imageHeaders}));

  assert.equal(gate.snapshot().external_network_requests, 2);
  assert.equal(responseCache.snapshot().exact_variant_hits, 1);
});

test("Vary star is retained once but fails closed instead of being reacquired or replayed", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const acquisitionContext = createContext();
  await installBrowserRequestGate(acquisitionContext, {gate, responseCache});
  const url = "https://cdn.example.test/vary-star.css";
  const firstHeaders = {accept: "text/css", "sec-fetch-mode": "no-cors"};
  const first = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: firstHeaders,
    fetchResponse: createFetchResponse({headers: {vary: "*"}, body: "star-one"}),
  });
  const second = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: firstHeaders,
    fetchResponse: createFetchResponse({headers: {vary: "*"}, body: "must-not-refetch"}),
  });
  await acquisitionContext.routes[0].handler(first);
  const replayContext = createContext();
  await installBrowserRequestGate(replayContext, {gate, responseCache, cacheOnly: true});
  await replayContext.routes[0].handler(second);

  assert.deepEqual(first.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 200},
  ]);
  assert.deepEqual(second.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(gate.snapshot().external_network_requests, 1);
  assert.equal(gate.snapshot().enforcement_failed, true);
  assert.equal(responseCache.snapshot().variant_stores, 1);
});

test("legacy Vary evidence is retained without bulk reacquisition and is replaced on demand by one exact variant", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-vary-legacy-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const url = "https://cdn.example.test/legacy-vary.css";
  const identity = "evidence:legacy-vary";
  await fs.writeFile(path.join(cacheDir, "manifest.json"), `${JSON.stringify({
    schema: "wikijump_full_parity.browser_response_cache_store.v1",
    identity,
    entries: [{
      key: url,
      status: 200,
      headers: {vary: "Origin", "content-type": "text/css"},
      body_base64: Buffer.from("legacy-uncertain").toString("base64"),
    }],
  })}\n`, {mode: 0o600});
  await fs.chmod(cacheDir, 0o700);

  const cache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: identity, evidenceReplay: true});
  await cache.load();
  assert.equal(cache.lookup(url, {requestHeaders: {origin: "https://a.example.test"}}), null);
  assert.equal(cache.snapshot().legacy_variant_misses, 1);

  const gate = createBrowserRequestGate({intervalMs: 0});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache: cache});
  const exact = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {origin: "https://a.example.test"},
    fetchResponse: createFetchResponse({headers: {vary: "Origin"}, body: "exact-a"}),
  });
  const replay = createRoute(url, {resourceType: "stylesheet", allHeaders: {origin: "https://a.example.test"}});
  await context.routes[0].handler(exact);
  await context.routes[0].handler(replay);
  assert.equal(gate.snapshot().external_network_requests, 1);
  assert.equal(cache.snapshot().variant_stores, 1);

  const manifest = JSON.parse(await fs.readFile(path.join(cacheDir, "manifest.json"), "utf8"));
  assert.equal(manifest.entries.length, 2);
  assert.equal(manifest.entries.filter((entry) => entry.vary_binding !== undefined).length, 1);
});

test("separate persistent cache instances coalesce the same exact miss before external acquisition", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-cross-instance-fill-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {persistentDir: cacheDir, persistentIdentity: "evidence:cross-instance-fill", evidenceReplay: true};
  const firstCache = createBrowserResponseCache(options);
  const secondCache = createBrowserResponseCache(options);
  await Promise.all([firstCache.load(), secondCache.load()]);
  const firstGate = createBrowserRequestGate({intervalMs: 0});
  const secondGate = createBrowserRequestGate({intervalMs: 0});
  const firstContext = createContext();
  const secondContext = createContext();
  await installBrowserRequestGate(firstContext, {gate: firstGate, responseCache: firstCache});
  await installBrowserRequestGate(secondContext, {gate: secondGate, responseCache: secondCache});
  const url = "https://cdn.example.test/cross-process.css";
  let releaseFetch;
  const pendingResponse = new Promise((resolve) => {
    releaseFetch = () => resolve(createFetchResponse({headers: {vary: "Origin"}, body: "single"}));
  });
  const first = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {origin: "https://same.example.test"},
    fetchResponse: pendingResponse,
  });
  const second = createRoute(url, {
    resourceType: "stylesheet",
    allHeaders: {origin: "https://same.example.test"},
    fetchResponse: pendingResponse,
  });

  const firstRun = firstContext.routes[0].handler(first);
  const secondRun = secondContext.routes[0].handler(second);
  await new Promise((resolve) => setImmediate(resolve));
  releaseFetch();
  await Promise.all([firstRun, secondRun]);

  assert.equal(firstGate.snapshot().external_network_requests + secondGate.snapshot().external_network_requests, 1);
  const actionSets = [first.actions, second.actions];
  assert.equal(actionSets.filter((actions) => actions.some(({type}) => type === "fetch")).length, 1);
  assert.equal(actionSets.filter((actions) => actions.some(({type}) => type === "abort")).length, 0);
  assert.deepEqual(actionSets.map((actions) => actions.at(-1)), [
    {type: "fulfill", status: 200},
    {type: "fulfill", status: 200},
  ]);
});

test("distinct persistent misses can acquire concurrently without losing evidence", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-parallel-fill-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {persistentDir: cacheDir, persistentIdentity: "evidence:parallel-fill", evidenceReplay: true};
  const caches = [createBrowserResponseCache(options), createBrowserResponseCache(options)];
  await Promise.all(caches.map((cache) => cache.load()));
  let release;
  const waitForRelease = new Promise((resolve) => { release = resolve; });
  let bothStarted;
  const waitForBoth = new Promise((resolve) => { bothStarted = resolve; });
  let active = 0;
  let peakActive = 0;
  const fills = caches.map((cache, index) => {
    const key = `https://cdn.example.test/parallel-${index}.css`;
    return cache.withFill(key, {requestHeaders: {}}, async ({markAcquisitionStarted}) => {
      await markAcquisitionStarted();
      active += 1;
      peakActive = Math.max(peakActive, active);
      if (active === 2) bothStarted();
      await waitForRelease;
      active -= 1;
      const entry = {status: 200, headers: {}, body: Buffer.from(`response-${index}`)};
      assert.equal(cache.store(key, entry, {requestHeaders: {}}), true);
      return entry;
    });
  });
  let timeout;
  try {
    await Promise.race([
      waitForBoth,
      new Promise((_, reject) => { timeout = setTimeout(() => reject(new Error("distinct fills serialized through acquisition")), 1_000); }),
    ]);
    assert.equal(peakActive, 2);
  } finally {
    clearTimeout(timeout);
    release();
    await Promise.allSettled(fills);
  }
  const reloaded = createBrowserResponseCache(options);
  await reloaded.load();
  for (let index = 0; index < 2; index += 1) {
    assert.equal(reloaded.get(`https://cdn.example.test/parallel-${index}.css`)?.body.toString(), `response-${index}`);
  }
  assert.equal(reloaded.snapshot().acquisition_barriers, 0);
});

test("a concurrent manifest write cannot replay another fill before it seals", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-unsealed-fill-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {persistentDir: cacheDir, persistentIdentity: "evidence:unsealed-fill", evidenceReplay: true};
  const first = createBrowserResponseCache(options);
  await first.load();
  const firstKey = "https://cdn.example.test/unsealed.css";
  const secondKey = "https://cdn.example.test/sealed.css";
  let stored;
  const firstStored = new Promise((resolve) => { stored = resolve; });
  let release;
  const waitForRelease = new Promise((resolve) => { release = resolve; });
  const firstFill = first.withFill(firstKey, {requestHeaders: {}}, async ({markAcquisitionStarted}) => {
    await markAcquisitionStarted();
    const entry = {status: 200, headers: {}, body: Buffer.from("unsealed")};
    assert.equal(first.store(firstKey, entry, {requestHeaders: {}}), true);
    stored();
    await waitForRelease;
    return entry;
  });
  try {
    await firstStored;
    await first.withFill(secondKey, {requestHeaders: {}}, async ({markAcquisitionStarted}) => {
      await markAcquisitionStarted();
      const entry = {status: 200, headers: {}, body: Buffer.from("sealed")};
      assert.equal(first.store(secondKey, entry, {requestHeaders: {}}), true);
      return entry;
    });
    const observer = createBrowserResponseCache(options);
    await observer.load();
    assert.throws(() => observer.get(firstKey), /refuses to reacquire an identity with uncertain prior acquisition/u);
    assert.equal(observer.get(secondKey)?.body.toString(), "sealed");
  } finally {
    release();
    await firstFill;
  }
  const observer = createBrowserResponseCache(options);
  await observer.load();
  assert.equal(observer.get(firstKey)?.body.toString(), "unsealed");
});

test("separate Node processes coalesce one persistent exact fill before its producer runs", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-cross-process-fill-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const counterPath = path.join(cacheDir, "producer-count.txt");
  const moduleUrl = new URL("../src/browser-request-gate.mjs", import.meta.url).href;
  const source = `
    import fs from "node:fs/promises";
    import {createBrowserResponseCache} from ${JSON.stringify(moduleUrl)};
    const cache = createBrowserResponseCache({persistentDir:${JSON.stringify(cacheDir)}, persistentIdentity:"evidence:cross-process-fill", evidenceReplay:true});
    await cache.load();
    const key = "https://cdn.example.test/cross-process-vary.css";
    const requestHeaders = {origin:"https://same.example.test"};
    await cache.withFill(key, {requestHeaders}, async ({markAcquisitionStarted}) => {
      await markAcquisitionStarted();
      await fs.appendFile(${JSON.stringify(counterPath)}, String(process.pid) + "\\n");
      await new Promise((resolve) => setTimeout(resolve, 100));
      const entry = {status:200, headers:{vary:"Origin"}, body:Buffer.from("one-process-fetch")};
      if (!cache.store(key, entry, {requestHeaders})) throw new Error("store rejected exact variant");
      return entry;
    });
  `;

  await Promise.all([runNodeChild(source), runNodeChild(source)]);

  const producerPids = (await fs.readFile(counterPath, "utf8")).trim().split("\n").filter(Boolean);
  assert.equal(producerPids.length, 1);
  const reloaded = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: "evidence:cross-process-fill", evidenceReplay: true});
  await reloaded.load();
  assert.equal(reloaded.lookup("https://cdn.example.test/cross-process-vary.css", {requestHeaders: {origin: "https://same.example.test"}})?.body.toString(), "one-process-fetch");
});

test("a failed persistent acquisition leaves a durable barrier and never retries the producer", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-acquisition-barrier-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {persistentDir: cacheDir, persistentIdentity: "evidence:acquisition-barrier", evidenceReplay: true};
  const key = "https://cdn.example.test/uncertain.css";
  const first = createBrowserResponseCache(options);
  await first.load();
  let producerCalls = 0;
  await assert.rejects(
    first.withFill(key, {requestHeaders: {}}, async ({markAcquisitionStarted}) => {
      await markAcquisitionStarted();
      producerCalls += 1;
      throw new Error("simulated transport failure after acquisition started");
    }),
    /simulated transport failure/u,
  );
  assert.equal(producerCalls, 1);
  assert.equal(first.snapshot().acquisition_barriers, 1);

  const second = createBrowserResponseCache(options);
  await second.load();
  await assert.rejects(
    second.withFill(key, {requestHeaders: {}}, async () => {
      producerCalls += 1;
      return {status: 200, headers: {}, body: Buffer.from("must-not-run")};
    }),
    /refuses to reacquire an identity with uncertain prior acquisition/u,
  );
  assert.equal(producerCalls, 1);
  assert.equal(second.snapshot().acquisition_barriers, 1);
});

test("a process crash after persistent acquisition starts leaves a barrier that blocks another process", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-crash-barrier-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const counterPath = path.join(cacheDir, "producer-count.txt");
  const moduleUrl = new URL("../src/browser-request-gate.mjs", import.meta.url).href;
  const key = "https://cdn.example.test/crash-before-store.css";
  const crashSource = `
    import fs from "node:fs/promises";
    import {createBrowserResponseCache} from ${JSON.stringify(moduleUrl)};
    const cache = createBrowserResponseCache({persistentDir:${JSON.stringify(cacheDir)}, persistentIdentity:"evidence:crash-barrier", evidenceReplay:true});
    await cache.load();
    await cache.withFill(${JSON.stringify(key)}, {requestHeaders:{}}, async ({markAcquisitionStarted}) => {
      await markAcquisitionStarted();
      await fs.appendFile(${JSON.stringify(counterPath)}, "attempt\\n");
      process.exit(77);
    });
  `;
  await assert.rejects(runNodeChild(crashSource), /child cache fill failed \(77\)/u);

  const cache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: "evidence:crash-barrier", evidenceReplay: true});
  await cache.load();
  let retried = false;
  await assert.rejects(
    cache.withFill(key, {requestHeaders: {}}, async () => {
      retried = true;
      return {status: 200, headers: {}, body: Buffer.from("must-not-run")};
    }),
    /refuses to reacquire an identity with uncertain prior acquisition/u,
  );
  assert.equal(retried, false);
  assert.equal((await fs.readFile(counterPath, "utf8")).trim(), "attempt");
  assert.equal(cache.snapshot().acquisition_barriers, 1);
});

test("a producer failure before acquisition starts does not poison the persistent identity", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-pre-acquisition-failure-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {persistentDir: cacheDir, persistentIdentity: "evidence:pre-acquisition-failure", evidenceReplay: true};
  const key = "https://cdn.example.test/not-yet-contacted.css";
  const cache = createBrowserResponseCache(options);
  await cache.load();
  await assert.rejects(
    cache.withFill(key, {requestHeaders: {}}, async () => {
      throw new Error("gate admission failed before network acquisition");
    }),
    /gate admission failed before network acquisition/u,
  );
  assert.equal(cache.snapshot().acquisition_barriers, 0);

  let producerCalls = 0;
  const entry = await cache.withFill(key, {requestHeaders: {}}, async ({markAcquisitionStarted}) => {
    await markAcquisitionStarted();
    producerCalls += 1;
    const retained = {status: 200, headers: {}, body: Buffer.from("first-real-attempt")};
    assert.equal(cache.store(key, retained, {requestHeaders: {}}), true);
    return retained;
  });
  assert.equal(entry.body.toString(), "first-real-attempt");
  assert.equal(producerCalls, 1);
  assert.equal(cache.snapshot().acquisition_barriers, 0);
});

test("persistent evidence caches remain append-only beyond their in-memory LRU limits", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-no-evidence-eviction-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const responseCache = createBrowserResponseCache({
    persistentDir: cacheDir,
    persistentIdentity: "evidence:no-eviction",
    evidenceReplay: true,
    maxEntries: 1,
    maxBytes: 64,
    maxEntryBytes: 64,
  });
  await responseCache.load();

  assert.equal(responseCache.store("https://cdn.example.test/first", {status: 200, headers: {}, body: Buffer.from("first")}), true);
  await responseCache.flush();
  assert.equal(responseCache.store("https://cdn.example.test/second", {status: 200, headers: {}, body: Buffer.from("second")}), true);
  await responseCache.flush();

  const reloaded = createBrowserResponseCache({
    persistentDir: cacheDir,
    persistentIdentity: "evidence:no-eviction",
    evidenceReplay: true,
    maxEntries: 1,
    maxBytes: 64,
    maxEntryBytes: 64,
  });
  await reloaded.load();
  assert.equal(reloaded.get("https://cdn.example.test/first")?.status, 200);
  assert.equal(reloaded.get("https://cdn.example.test/second")?.status, 200);
  assert.equal(responseCache.snapshot().evictions, 0);
  assert.equal(responseCache.snapshot().retention_policy, "append_only_no_eviction");
});

test("persistent evidence retains an acquired body beyond the configured in-memory per-entry ceiling", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-persistent-large-body-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {
    persistentDir: cacheDir,
    persistentIdentity: "evidence:large-body",
    evidenceReplay: true,
    maxEntries: 1,
    maxBytes: 64,
    maxEntryBytes: 64,
  };
  const cache = createBrowserResponseCache(options);
  await cache.load();
  const body = Buffer.alloc(65, 0x61);
  assert.equal(cache.store("https://cdn.example.test/large.bin", {status: 200, headers: {}, body}), true);
  await cache.flush();

  const reloaded = createBrowserResponseCache(options);
  await reloaded.load();
  assert.equal(reloaded.get("https://cdn.example.test/large.bin")?.body.length, 65);
  assert.equal(reloaded.snapshot().max_entry_bytes, Number.MAX_SAFE_INTEGER);
});

test("an explicitly identified persistent source cache reuses documents without a second Wikidot request", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-response-cache-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const identity = "live-reference:scp-8980:chromium-149:fixture-2026-08-27";
  const url = "https://scp-wiki.wikidot.com/scp-8980";
  const firstGate = createBrowserRequestGate({intervalMs: 4_000});
  const firstCache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: identity, cacheDocuments: true, evidenceReplay: true});
  await firstCache.load();
  const firstContext = createContext();
  await installBrowserRequestGate(firstContext, {gate: firstGate, responseCache: firstCache});
  await firstContext.routes[0].handler(createRoute(url, {
    resourceType: "document",
    fetchResponse: createFetchResponse({headers: {"cache-control": "public, max-age=600"}, body: "<html>retained</html>"}),
  }));
  await firstCache.flush();

  const secondGate = createBrowserRequestGate({intervalMs: 4_000});
  const secondCache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: identity, cacheDocuments: true, evidenceReplay: true});
  await secondCache.load();
  const secondContext = createContext();
  await installBrowserRequestGate(secondContext, {gate: secondGate, responseCache: secondCache});
  const second = createRoute(url, {resourceType: "document"});
  await secondContext.routes[0].handler(second);

  assert.deepEqual(second.actions, [{type: "fulfill", status: 200}]);
  assert.equal(secondGate.snapshot().public_requests, 0);
  assert.equal(secondCache.snapshot().lifetime, "persistent");
  assert.equal(secondCache.snapshot().persistent_entries_loaded, 1);
  await assert.rejects(
    createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity: "different-browser", evidenceReplay: true}).load(),
    /identity or schema mismatch/u,
  );
});

test("a URL-only source response cache bypasses revalidation and request-context responses", async (t) => {
  const cases = [
    ["no-cache", {"cache-control": "no-cache"}],
    ["private", {"cache-control": "private, max-age=600"}],
    ["max-age zero", {"cache-control": "public, max-age=0"}],
    ["must-revalidate", {"cache-control": "public, max-age=600, must-revalidate"}],
    ["cookie variance", {"cache-control": "public, max-age=600", vary: "Cookie"}],
    ["language variance", {"cache-control": "public, max-age=600", vary: "Accept-Language"}],
    ["response cookie", {"cache-control": "public, max-age=600", "set-cookie": "session=secret"}],
  ];

  for (const [name, responseHeaders] of cases) {
    await t.test(name, async () => {
      const clock = createClock();
      const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
      const responseCache = createBrowserResponseCache();
      const context = createContext();
      await installBrowserRequestGate(context, {gate, responseCache});
      const handler = context.routes[0].handler;
      const url = `https://cdn.example.test/${encodeURIComponent(name)}.css`;

      const first = createRoute(url, {
        resourceType: "stylesheet",
        fetchResponse: createFetchResponse({headers: responseHeaders, body: "first"}),
      });
      const second = createRoute(url, {
        resourceType: "stylesheet",
        fetchResponse: createFetchResponse({headers: responseHeaders, body: "second"}),
      });

      await handler(first);
      await handler(second);

      assert.deepEqual(first.actions, [
        {type: "fetch", options: {maxRedirects: 0}},
        {type: "fulfill", status: 200},
      ]);
      assert.deepEqual(second.actions, [
        {type: "fetch", options: {maxRedirects: 0}},
        {type: "fulfill", status: 200},
      ]);
      assert.equal(gate.snapshot().public_requests, 2);
      assert.equal(responseCache.snapshot().entries, 0);
      assert.equal(responseCache.snapshot().stores, 0);
      assert.equal(responseCache.snapshot().bypasses, 2);
    });
  }
});

test("an evidence replay cache reuses public responses even when HTTP cache headers forbid browser caching", async (t) => {
  const cases = [
    ["no-cache", {"cache-control": "no-cache"}],
    ["private", {"cache-control": "private, max-age=600"}],
    ["max-age zero", {"cache-control": "public, max-age=0"}],
    ["must-revalidate", {"cache-control": "public, max-age=600, must-revalidate"}],
    ["cookie variance", {"cache-control": "public, max-age=600", vary: "Cookie"}],
    ["language variance", {"cache-control": "public, max-age=600", vary: "Accept-Language"}],
    ["response cookie", {"cache-control": "public, max-age=600", "set-cookie": "session=secret"}],
  ];

  for (const [name, responseHeaders] of cases) {
    await t.test(name, async () => {
      const gate = createBrowserRequestGate({intervalMs: 0});
      const responseCache = createBrowserResponseCache({evidenceReplay: true});
      const context = createContext();
      await installBrowserRequestGate(context, {gate, responseCache});
      const handler = context.routes[0].handler;
      const url = `https://cdn.example.test/${encodeURIComponent(name)}.css`;
      const first = createRoute(url, {resourceType: "stylesheet", fetchResponse: createFetchResponse({headers: responseHeaders, body: "retained"})});
      const second = createRoute(url, {resourceType: "stylesheet", fetchResponse: createFetchResponse({headers: responseHeaders, body: "should-not-refetch"})});

      await handler(first);
      await handler(second);

      assert.deepEqual(first.actions, [
        {type: "fetch", options: {maxRedirects: 0}},
        {type: "fulfill", status: 200},
      ]);
      assert.deepEqual(second.actions, [{type: "fulfill", status: 200}]);
      assert.equal(gate.snapshot().public_requests, 1);
      assert.equal(responseCache.snapshot().entries, 1);
      assert.equal(responseCache.snapshot().stores, 1);
      assert.equal(responseCache.snapshot().hits, 1);
      assert.equal(responseCache.snapshot().bypasses, 0);
      assert.equal(responseCache.snapshot().evidence_replay, true);
    });
  }
});

test("redirect responses are replayed only in evidence mode and retain their Location header across persistent reloads", async (t) => {
  const replayableStatuses = [301, 302, 303, 307, 308];
  const terminalNonRedirectStatuses = [300, 305, 306];

  await t.test("normal response caching still bypasses redirects", async () => {
    const gate = createBrowserRequestGate({intervalMs: 0});
    const responseCache = createBrowserResponseCache();
    const context = createContext();
    await installBrowserRequestGate(context, {gate, responseCache});
    const handler = context.routes[0].handler;
    const url = "https://cdn.example.test/normal-redirect.css";
    const response = createFetchResponse({status: 302, headers: {location: "https://assets.example.test/final.css"}, body: "redirect"});
    const first = createRoute(url, {resourceType: "stylesheet", fetchResponse: response});
    const second = createRoute(url, {resourceType: "stylesheet", fetchResponse: response});

    await handler(first);
    await handler(second);

    assert.deepEqual(first.actions, [
      {type: "fetch", options: {maxRedirects: 0}},
      {type: "fulfill", status: 302},
    ]);
    assert.deepEqual(second.actions, [
      {type: "fetch", options: {maxRedirects: 0}},
      {type: "fulfill", status: 302},
    ]);
    assert.equal(responseCache.snapshot().stores, 0);
    assert.equal(responseCache.snapshot().bypasses, 2);
  });

  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-redirect-replay-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const persistentIdentity = "evidence-replay:redirect-status-matrix";
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity, evidenceReplay: true, cacheDocuments: true});
  await responseCache.load();
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const handler = context.routes[0].handler;

  for (const status of replayableStatuses) {
    const url = `https://cdn.example.test/replay-${status}.css`;
    const location = `https://assets.example.test/final-${status}.css`;
    const first = createRoute(url, {resourceType: "document", fetchResponse: createFetchResponse({status, headers: {location}, body: `redirect-${status}`})});
    const second = createRoute(url, {resourceType: "document", fetchResponse: createFetchResponse({status: 599, body: "must not refetch"})});

    await handler(first);
    await handler(second);

    assert.deepEqual(first.actions, [
      {type: "fetch", options: {maxRedirects: 0}},
      {type: "fulfill", status},
    ]);
    assert.deepEqual(second.actions, [{type: "fulfill", status}]);
    const retained = responseCache.get(url);
    assert.equal(retained.status, status);
    assert.equal(retained.headers.location, location);
  }

  for (const status of terminalNonRedirectStatuses) {
    const url = `https://cdn.example.test/no-replay-${status}.css`;
    const response = createFetchResponse({status, headers: {location: "https://assets.example.test/ignored.css"}, body: `status-${status}`});
    const first = createRoute(url, {resourceType: "document", fetchResponse: response});
    const second = createRoute(url, {resourceType: "document"});

    await handler(first);
    await handler(second);

    assert.deepEqual(first.actions, [
      {type: "fetch", options: {maxRedirects: 0}},
      {type: "fulfill", status},
    ]);
    assert.deepEqual(second.actions, [{type: "fulfill", status}]);
    assert.equal(responseCache.get(url)?.status, status);
  }

  await responseCache.flush();
  const reloaded = createBrowserResponseCache({persistentDir: cacheDir, persistentIdentity, evidenceReplay: true, cacheDocuments: true});
  await reloaded.load();
  assert.equal(reloaded.snapshot().persistent_entries_loaded, replayableStatuses.length + terminalNonRedirectStatuses.length);
  for (const status of replayableStatuses) {
    const retained = reloaded.get(`https://cdn.example.test/replay-${status}.css`);
    assert.equal(retained.status, status);
    assert.equal(retained.headers.location, `https://assets.example.test/final-${status}.css`);
  }
});

test("a retained 304 is preserved but never replayed or automatically reacquired", async (t) => {
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-304-replay-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const options = {
    persistentDir: cacheDir,
    persistentIdentity: "evidence:304-nonreplayable",
    evidenceReplay: true,
    cacheDocuments: true,
  };
  const url = "https://cdn.example.test/conditional-document";
  const firstCache = createBrowserResponseCache(options);
  await firstCache.load();
  const firstGate = createBrowserRequestGate({intervalMs: 0});
  const firstContext = createContext();
  await installBrowserRequestGate(firstContext, {gate: firstGate, responseCache: firstCache});
  const first = createRoute(url, {
    resourceType: "document",
    headers: {"if-none-match": "etag-v1"},
    fetchResponse: createFetchResponse({status: 304, body: ""}),
  });
  await firstContext.routes[0].handler(first);
  assert.deepEqual(first.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 304},
  ]);
  assert.equal(firstGate.snapshot().external_network_requests, 1);

  const secondCache = createBrowserResponseCache(options);
  await secondCache.load();
  const secondGate = createBrowserRequestGate({intervalMs: 0});
  const secondContext = createContext();
  await installBrowserRequestGate(secondContext, {gate: secondGate, responseCache: secondCache});
  const second = createRoute(url, {
    resourceType: "document",
    headers: {"if-none-match": "etag-v1"},
    fetchResponse: createFetchResponse({status: 200, body: "must-not-refetch"}),
  });
  await secondContext.routes[0].handler(second);

  assert.deepEqual(second.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(secondGate.snapshot().external_network_requests, 0);
  assert.equal(secondGate.snapshot().enforcement_failed, true);
  assert.throws(
    () => secondCache.lookup(`${url}#__wikijump_evidence_request=if-none-match=etag-v1`, {requestHeaders: {"if-none-match": "etag-v1"}}),
    /retained 304 response is not standalone-replayable/u,
  );
});

test("a conditional request reuses a retained unconditional representation", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true, cacheDocuments: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const url = "https://cdn.example.test/conditional-with-200";
  const first = createRoute(url, {
    resourceType: "document",
    fetchResponse: createFetchResponse({body: "retained-representation"}),
  });
  await context.routes[0].handler(first);

  const second = createRoute(url, {
    resourceType: "document",
    headers: {"if-none-match": "etag-v1"},
    fetchResponse: createFetchResponse({status: 304}),
  });
  await context.routes[0].handler(second);

  assert.deepEqual(second.actions, [{type: "fulfill", status: 200}]);
  assert.equal(gate.snapshot().external_network_requests, 1);
  assert.equal(responseCache.snapshot().entries, 1);
});

test("evidence replay retains terminal server failures instead of retrying them", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const handler = context.routes[0].handler;

  for (const status of [500, 503]) {
    const url = `https://cdn.example.test/failure-${status}.css`;
    const first = createRoute(url, {
      resourceType: "stylesheet",
      fetchResponse: createFetchResponse({status, body: `failure-${status}`}),
    });
    const second = createRoute(url, {resourceType: "stylesheet"});

    await handler(first);
    await handler(second);

    assert.deepEqual(first.actions, [
      {type: "fetch", options: {maxRedirects: 0}},
      {type: "fulfill", status},
    ]);
    assert.deepEqual(second.actions, [{type: "fulfill", status}]);
  }

  assert.equal(gate.snapshot().external_network_requests, 2);
  assert.equal(responseCache.snapshot().stores, 2);
  assert.equal(responseCache.snapshot().hits, 2);
});

test("candidate evidence replay resolves cached subresource redirects before Chromium can bypass routing", async (t) => {
  for (const resourceType of ["stylesheet", "image", "font"]) {
    await t.test(resourceType, async () => {
      const gate = createBrowserRequestGate({intervalMs: 0});
      const responseCache = createBrowserResponseCache({evidenceReplay: true});
      const sourceUrl = `https://cdn.example.test/${resourceType}/source`;
      const targetUrl = `https://assets.example.test/${resourceType}/final`;
      responseCache.store(sourceUrl, {
        status: 301,
        headers: {location: targetUrl},
        body: Buffer.alloc(0),
      });
      responseCache.store(targetUrl, {
        status: 200,
        headers: {"content-type": resourceType === "stylesheet" ? "text/css" : "application/octet-stream"},
        body: Buffer.from(resourceType === "stylesheet" ? ".target { display: block }" : "retained-target"),
      });
      const context = createContext();
      await installBrowserRequestGate(context, {
        gate,
        responseCache,
        cacheOnly: true,
      });
      const first = createRoute(sourceUrl, {resourceType});

      await context.routes[0].handler(first);

      assert.deepEqual(first.actions, [{type: "fulfill", status: 200}]);
      assert.equal(gate.snapshot().public_requests, 0);
      assert.equal(gate.snapshot().external_network_requests, 0);
      assert.equal(responseCache.get(targetUrl)?.status, 200);

      const second = createRoute(sourceUrl, {resourceType});
      await context.routes[0].handler(second);
      assert.deepEqual(second.actions, [{type: "fulfill", status: 200}]);
      assert.equal(gate.snapshot().public_requests, 0);
    });
  }
});

test("candidate cache-only redirect target misses fail closed without external fetch", async (t) => {
  for (const resourceType of ["stylesheet", "image", "font"]) {
    await t.test(resourceType, async () => {
      const gate = createBrowserRequestGate({intervalMs: 0});
      const responseCache = createBrowserResponseCache({evidenceReplay: true});
      const sourceUrl = `https://cdn.example.test/${resourceType}/source`;
      const targetUrl = `https://assets.example.test/${resourceType}/missing`;
      responseCache.store(sourceUrl, {
        status: 301,
        headers: {location: targetUrl},
        body: Buffer.alloc(0),
      });
      const context = createContext();
      await installBrowserRequestGate(context, {gate, responseCache, cacheOnly: true});
      const route = createRoute(sourceUrl, {
        resourceType,
        fetchResponse: createFetchResponse({body: "must-not-be-fetched"}),
      });

      await context.routes[0].handler(route);

      assert.deepEqual(route.actions, [{type: "abort", reason: "blockedbyclient"}]);
      assert.equal(gate.snapshot().public_requests, 0);
      assert.equal(gate.snapshot().external_network_requests, 0);
      assert.equal(gate.snapshot().enforcement_failed, true);
      assert.equal(responseCache.snapshot().stores, 1);
    });
  }
});

test("evidence replay never URL-caches requests carrying credentials", async (t) => {
  for (const [name, headers] of [
    ["authorization", {authorization: "Bearer secret"}],
    ["cookie", {cookie: "session=secret"}],
  ]) {
    await t.test(name, async () => {
      const gate = createBrowserRequestGate({intervalMs: 0});
      const responseCache = createBrowserResponseCache({evidenceReplay: true});
      const context = createContext();
      await installBrowserRequestGate(context, {gate, responseCache});
      const route = createRoute(`https://cdn.example.test/${name}.css`, {
        resourceType: "stylesheet",
        headers,
        fetchResponse: createFetchResponse({status: 302, headers: {location: "https://assets.example.test/final.css"}}),
      });

      await context.routes[0].handler(route);

      assert.deepEqual(route.actions, [{type: "continue"}]);
      assert.equal(responseCache.snapshot().entries, 0);
      assert.equal(responseCache.snapshot().stores, 0);
      assert.equal(responseCache.snapshot().bypasses, 1);
    });
  }
});

test("candidate evidence replay never caches URL userinfo credentials", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache, cacheOnly: true});
  const route = createRoute("https://user:secret@cdn.example.test/private.css", {
    resourceType: "stylesheet",
    fetchResponse: createFetchResponse({body: "must-not-fetch"}),
  });

  await context.routes[0].handler(route);

  assert.deepEqual(route.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(gate.snapshot().external_network_requests, 0);
  assert.equal(responseCache.snapshot().entries, 0);
  assert.equal(responseCache.snapshot().stores, 0);
});

test("documents and no-store assets keep using the unchanged request gate", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const responseCache = createBrowserResponseCache();
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const handler = context.routes[0].handler;
  const document = createRoute("https://example.test/page", {resourceType: "document"});
  const noStoreUrl = "https://example.test/dynamic.js";
  const noStoreResponse = createFetchResponse({headers: {"cache-control": "no-store"}});

  await handler(document);
  await handler(createRoute(noStoreUrl, {fetchResponse: noStoreResponse}));
  await handler(createRoute(noStoreUrl, {fetchResponse: noStoreResponse}));

  assert.deepEqual(document.actions, [{type: "continue"}]);
  assert.equal(gate.snapshot().public_requests, 3);
  assert.equal(responseCache.snapshot().entries, 0);
  assert.equal(responseCache.snapshot().stores, 0);
  assert.equal(responseCache.snapshot().bypasses, 3);
});

test("intentional unsupported-request attribution requires the exact request object", async () => {
  const gate = createBrowserRequestGate({intervalMs: 4_000});
  const context = createContext();
  const attribution = await installBrowserRequestGate(context, {gate});
  const dataUrl = createRoute("data:text/plain,unmetered");

  await context.routes[0].handler(dataUrl);

  assert.deepEqual(dataUrl.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.deepEqual(attribution.classifyRequestFailure(dataUrl.request()), {
    decision: "unsupported_protocol",
    abort_reason: "blockedbyclient",
  });
  assert.equal(
    attribution.classifyRequestFailure({
      url: () => "data:text/plain,unmetered",
      resourceType: () => "script",
    }),
    null,
  );
});

test("gate implementation failures latch enforcement closed", async () => {
  const clock = createClock({failSleeps: 1});
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const context = createContext();
  await installBrowserRequestGate(context, {gate});
  const handler = context.routes[0].handler;

  await gate.acquire();
  const blockedAfterSleepFailure = createRoute("https://scp-wiki.wikidot.com/queued");
  await handler(blockedAfterSleepFailure);
  assert.deepEqual(blockedAfterSleepFailure.actions, [{type: "abort", reason: "blockedbyclient"}]);
  assert.equal(gate.snapshot().enforcement_failed, true);
  await assert.rejects(gate.acquire(), /simulated clock failure/u);
});

test("a persisted gate prevents a later capture process from granting before the prior interval", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-request-state-"));
  const statePath = path.join(root, "campaign.state.json");
  const firstClock = createClock();
  const first = await createPersistentBrowserRequestGate({
    statePath,
    intervalMs: 4_000,
    now: firstClock.now,
    sleep: firstClock.sleep,
  });
  await first.acquire();
  await first.flush();

  const secondClock = createClock();
  const second = await createPersistentBrowserRequestGate({
    statePath,
    intervalMs: 4_000,
    now: secondClock.now,
    sleep: secondClock.sleep,
  });
  const grant = await second.acquire();
  await second.flush();

  assert.equal(grant.released_at_epoch_ms, 4_000);
  assert.deepEqual(secondClock.sleeps, [4_000]);
  const state = JSON.parse(await fs.readFile(statePath, "utf8"));
  assert.equal(state.next_admissible_at_epoch_ms, 8_000);
});

test("context-level response handling preserves Retry-After from a different page or popup", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const context = createContext();
  await installBrowserRequestGate(context, {gate});

  context.events.get("response")({
    url: () => "https://scp-wiki.wikidot.com/popup-response",
    status: () => 503,
    headers: () => ({"retry-after": "7"}),
  });
  await gate.flush();
  const route = createRoute("https://scp-wiki.wikidot.com/after-popup");
  await context.routes[0].handler(route);

  assert.deepEqual(route.actions, [{type: "continue"}]);
  assert.deepEqual(gate.snapshot().grants.map((grant) => grant.released_at_epoch_ms), [7_000]);
  assert.equal(gate.snapshot().retry_after_honored, 1);
});

test("a 200 cache-hint Retry-After does not throttle later browser captures", async () => {
  const clock = createClock();
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const context = createContext();
  await installBrowserRequestGate(context, {gate});

  context.events.get("response")({
    url: () => "https://sandbox-for-codex.wikidot.com/_admin",
    status: () => 200,
    headers: () => ({"retry-after": "600"}),
  });
  await gate.flush();
  const route = createRoute("https://sandbox-for-codex.wikidot.com/next-page");
  await context.routes[0].handler(route);

  assert.deepEqual(route.actions, [{type: "continue"}]);
  assert.deepEqual(gate.snapshot().grants.map((grant) => grant.released_at_epoch_ms), [0]);
  assert.equal(gate.snapshot().retry_after_honored, 0);
});

test("uninspectable public response metadata latches the gate closed", async () => {
  const gate = createBrowserRequestGate();
  const context = createContext();
  await installBrowserRequestGate(context, {gate});

  context.events.get("response")({
    url: () => "https://scp-wiki.wikidot.com/uninspectable",
    headers() {
      throw new Error("metadata unavailable");
    },
  });
  const route = createRoute("https://scp-wiki.wikidot.com/after-uninspectable");
  await context.routes[0].handler(route);

  assert.deepEqual(route.actions, [{type: "abort", reason: "blockedbyclient"}]);
  await assert.rejects(gate.flush(), /headers cannot be inspected/);
  assert.equal(gate.snapshot().enforcement_failed, true);
});

test("persistence failure latches the gate closed before a restart can be admitted", async () => {
  const clock = createClock();
  let writes = 0;
  const gate = createBrowserRequestGate({
    now: clock.now,
    sleep: clock.sleep,
    persistState: async () => {
      writes += 1;
      if (writes > 1) throw new Error("durable state write failed");
    },
  });
  await gate.acquire();
  await assert.rejects(gate.deferForRetryAfter("30"), /durable state write failed/);

  await assert.rejects(gate.flush(), /durable state write failed/);
  await assert.rejects(gate.acquire(), /durable state write failed/);
  assert.equal(gate.snapshot().enforcement_failed, true);
});

test("only canonical Wikijump local origins can become local exemptions", () => {
  assert.deepEqual(localBrowserCaptureOrigins("https://scp-wiki.wikijump.localhost/scp-173?x=1"), [
    "https://scp-wiki.wikijump.localhost",
    "https://scp-wiki.wjfiles.localhost",
  ]);
  assert.deepEqual(localBrowserCaptureOrigins("https://scp-wiki.wikijump.localhost:18443/scp-173"), [
    "https://scp-wiki.wikijump.localhost:18443",
    "https://scp-wiki.wjfiles.localhost:18443",
  ]);
  assert.throws(() => localBrowserCaptureOrigins("https://public.example/scp-173"), /\.wikijump\.localhost/);
  assert.throws(() => localBrowserCaptureOrigins("https://user@scp-wiki.wikijump.localhost/scp-173"), /without credentials/);
});

test("capture lock refuses a live owner regardless of state confirmation", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-request-lock-"));
  const lockPath = path.join(root, "campaign.lock");
  const ticks = new Map([[process.pid, "123"]]);
  const processStartTicks = async (pid) => ticks.get(pid) ?? null;
  const first = await acquireBrowserCaptureLock({
    lockPath,
    runId: "first",
    hostname: "test-host",
    processStartTicks,
    now: () => "2026-07-20T00:00:00.000Z",
  });

  await assert.rejects(
    () => acquireBrowserCaptureLock({lockPath, runId: "second", hostname: "test-host", processStartTicks}),
    /held by run first/
  );
  await first.confirmState();
  await assert.rejects(
    () => acquireBrowserCaptureLock({lockPath, runId: "second", hostname: "test-host", processStartTicks}),
    /held by run first/
  );
  await first.release();
});

test("capture lock safely replaces a sealed stale owner", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-request-lock-"));
  const lockPath = path.join(root, "campaign.lock");
  const processStartTicks = async (pid) => pid === process.pid ? "123" : null;
  await fs.writeFile(lockPath, `${JSON.stringify({
    schema: "wikijump_full_parity.browser_capture_lock.v1",
    hostname: "test-host",
    pid: 42,
    process_start_ticks: "456",
    run_id: "stale-run",
    state_confirmation: "sealed",
  })}\n`, {mode: 0o600});
  const replacement = await acquireBrowserCaptureLock({lockPath, runId: "replacement", hostname: "test-host", processStartTicks});
  assert.equal(replacement.owner.run_id, "replacement");
  await replacement.confirmState();
  await replacement.release();
  await assert.rejects(fs.lstat(lockPath), {code: "ENOENT"});
});

test("capture lock replaces an unsealed stale owner when durable state preserves the request floor", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-request-lock-"));
  const lockPath = path.join(root, "campaign.lock");
  const statePath = `${lockPath}.state.json`;
  const processStartTicks = async (pid) => pid === process.pid ? "123" : null;
  await fs.writeFile(lockPath, `${JSON.stringify({
    schema: "wikijump_full_parity.browser_capture_lock.v1",
    hostname: "test-host",
    pid: 42,
    process_start_ticks: "456",
    run_id: "stale-run",
    state_confirmation: "pending",
  })}\n`, {mode: 0o600});
  await fs.writeFile(statePath, `${JSON.stringify({
    schema: "wikijump_full_parity.browser_request_gate_state.v1",
    next_admissible_at_epoch_ms: 12_000,
    retry_after_until_epoch_ms: 0,
  })}\n`, {mode: 0o600});

  const replacement = await acquireBrowserCaptureLock({lockPath, runId: "replacement", hostname: "test-host", processStartTicks});
  assert.equal(replacement.owner.run_id, "replacement");
  assert.equal(replacement.statePath, statePath);
  const clock = createClock();
  const gate = await createPersistentBrowserRequestGate({statePath: replacement.statePath, intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const grant = await gate.acquire();
  assert.equal(grant.released_at_epoch_ms, 12_000);
  assert.deepEqual(clock.sleeps, [12_000]);
  await replacement.confirmState();
  await replacement.release();
  await assert.rejects(fs.lstat(lockPath), {code: "ENOENT"});
});

test("capture lock refuses an unsealed stale owner when durable state is unavailable or malformed", async (t) => {
  for (const state of [null, "malformed\n"]) {
    await t.test(state === null ? "missing state" : "malformed state", async () => {
      const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-request-lock-"));
      const lockPath = path.join(root, "campaign.lock");
      const processStartTicks = async (pid) => pid === process.pid ? "123" : null;
      await fs.writeFile(lockPath, `${JSON.stringify({
        schema: "wikijump_full_parity.browser_capture_lock.v1",
        hostname: "test-host",
        pid: 42,
        process_start_ticks: "456",
        run_id: "stale-run",
        state_confirmation: "pending",
      })}\n`, {mode: 0o600});
      if (state !== null) await fs.writeFile(`${lockPath}.state.json`, state, {mode: 0o600});

      await assert.rejects(
        () => acquireBrowserCaptureLock({lockPath, runId: "blocked", hostname: "test-host", processStartTicks}),
        (error) => {
          assert.match(error.message, /unconfirmed request-gate state from run stale-run; operator review is required/);
          if (state === null) {
            assert.equal(error.cause, undefined);
          } else {
            assert.match(error.cause.message, /browser request gate state is malformed/);
          }
          return true;
        },
      );
    });
  }
});
