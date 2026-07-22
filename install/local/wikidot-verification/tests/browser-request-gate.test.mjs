import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";
import {
  acquireBrowserCaptureLock,
  createBrowserRequestGate,
  createBrowserResponseCache,
  createPersistentBrowserRequestGate,
  installBrowserRequestGate,
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

function createRoute(url, {continueError = null, method = "GET", resourceType = "script", headers = {}, fetchResponse = null} = {}) {
  const actions = [];
  return {
    actions,
    request() {
      return {url: () => url, method: () => method, resourceType: () => resourceType, headers: () => headers};
    },
    async continue() {
      actions.push({type: "continue"});
      if (continueError) throw continueError;
    },
    async abort(reason) {
      actions.push({type: "abort", reason});
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

function createFetchResponse({status = 200, headers = {}, body = "asset"} = {}) {
  return {
    status: () => status,
    headers: () => ({"content-length": String(Buffer.byteLength(body)), ...headers}),
    body: async () => Buffer.from(body),
  };
}

test("shared gate admits concurrent public requests one per four seconds", async () => {
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
  await localContext.routes[0].handler(localExact);
  const source = createRoute("https://scp-wiki.wikidot.com/scp-173");
  const wrongPort = createRoute("https://scp-wiki.wikijump.localhost:18443/scp-173");
  await Promise.all([sourceContext.routes[0].handler(source), localContext.routes[0].handler(wrongPort)]);
  let connected = false;
  await sourceContext.webSocketRoutes[0].handler({connectToServer() { connected = true; }});

  assert.deepEqual(localExact.actions, [{type: "continue"}]);
  assert.deepEqual(source.actions, [{type: "continue"}]);
  assert.deepEqual(wrongPort.actions, [{type: "continue"}]);
  assert.equal(connected, false);
  assert.deepEqual(gate.snapshot().grants.map((grant) => grant.released_at_epoch_ms), [0, 4_000]);
  assert.equal(gate.snapshot().local_exempt_requests, 1);
  assert.equal(gate.snapshot().websocket_connections_blocked, 1);
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
    fetchResponse: createFetchResponse({headers: {"cache-control": "private, no-cache"}, body: "body{}"}),
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
  });
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

test("unsupported, malformed, and failed request paths fail closed and leave the queue usable", async () => {
  const clock = createClock({failSleeps: 1});
  const gate = createBrowserRequestGate({intervalMs: 4_000, now: clock.now, sleep: clock.sleep});
  const context = createContext();
  await installBrowserRequestGate(context, {gate});
  const handler = context.routes[0].handler;

  const dataUrl = createRoute("data:text/plain,unmetered");
  await handler(dataUrl);
  assert.deepEqual(dataUrl.actions, [{type: "abort", reason: "blockedbyclient"}]);

  await gate.acquire();
  const blockedAfterSleepFailure = createRoute("https://scp-wiki.wikidot.com/queued");
  await handler(blockedAfterSleepFailure);
  assert.deepEqual(blockedAfterSleepFailure.actions, [{type: "abort", reason: "blockedbyclient"}]);

  const continuationFailure = createRoute("https://scp-wiki.wikidot.com/continue-failure", {continueError: new Error("route disposed")});
  await handler(continuationFailure);
  assert.deepEqual(continuationFailure.actions, [
    {type: "continue"},
    {type: "abort", reason: "blockedbyclient"},
  ]);
  const recovery = await gate.acquire();

  assert.equal(recovery.released_at_epoch_ms, 8_000);
  assert.equal(gate.snapshot().unsupported_requests_blocked, 3);
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
    headers: () => ({"retry-after": "7"}),
  });
  await gate.flush();
  const route = createRoute("https://scp-wiki.wikidot.com/after-popup");
  await context.routes[0].handler(route);

  assert.deepEqual(route.actions, [{type: "continue"}]);
  assert.deepEqual(gate.snapshot().grants.map((grant) => grant.released_at_epoch_ms), [7_000]);
  assert.equal(gate.snapshot().retry_after_honored, 1);
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

test("only canonical standing Wikijump origins can become local exemptions", () => {
  assert.deepEqual(localBrowserCaptureOrigins("https://scp-wiki.wikijump.localhost/scp-173?x=1"), [
    "https://scp-wiki.wikijump.localhost",
    "https://scp-wiki.wjfiles.localhost",
  ]);
  assert.throws(() => localBrowserCaptureOrigins("https://public.example/scp-173"), /\.wikijump\.localhost/);
  assert.throws(() => localBrowserCaptureOrigins("https://user@scp-wiki.wikijump.localhost/scp-173"), /without credentials/);
  assert.throws(() => localBrowserCaptureOrigins("https://scp-wiki.wikijump.localhost:18443/scp-173"), /non-default port/);
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
        /unconfirmed request-gate state from run stale-run; operator review is required/
      );
    });
  }
});
