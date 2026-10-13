import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import {test} from "node:test";
import {fileURLToPath} from "node:url";

import {
  createBrowserRequestGate,
  createBrowserResponseCache,
  installBrowserRequestGate,
} from "../src/browser-request-gate.mjs";

const TEST_DIR = path.dirname(fileURLToPath(import.meta.url));
const VERIFICATION_ROOT = path.resolve(TEST_DIR, "..");

function createContext() {
  const routes = [];
  return {
    routes,
    async route(pattern, handler) {
      routes.push({pattern, handler});
    },
    async routeWebSocket() {},
    on() {},
  };
}

function createResponse(body) {
  return {
    status: () => 200,
    headers: () => ({
      "cache-control": "public, max-age=600",
      "content-length": String(Buffer.byteLength(body)),
    }),
    body: async () => Buffer.from(body),
  };
}

function createRoute(url, {response = null} = {}) {
  const actions = [];
  const request = {
    url: () => url,
    method: () => "GET",
    resourceType: () => "stylesheet",
    headers: () => ({}),
    frame: () => null,
  };
  return {
    actions,
    request: () => request,
    async fetch(options) {
      actions.push({type: "fetch", options});
      if (response === null) throw new Error("unexpected external fetch");
      return response;
    },
    async fulfill(options) {
      actions.push({type: "fulfill", status: options.status ?? options.response?.status() ?? null});
    },
    async abort(reason) {
      actions.push({type: "abort", reason});
    },
    async continue() {
      actions.push({type: "continue"});
    },
  };
}

test("candidate Wikidot-family dependency miss fetches once and is reused from cache", async () => {
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache();
  const context = createContext();
  await installBrowserRequestGate(context, {gate, responseCache});
  const handler = context.routes[0].handler;
  const url = "https://scp-wiki.wdfiles.com/local--files/component:theme/font-bauhaus.css";
  const miss = createRoute(url, {response: createResponse("cached-wikidot-asset")});
  const hit = createRoute(url);

  await handler(miss);
  await handler(hit);

  assert.deepEqual(miss.actions, [
    {type: "fetch", options: {maxRedirects: 0}},
    {type: "fulfill", status: 200},
  ]);
  assert.deepEqual(hit.actions, [{type: "fulfill", status: 200}]);
  assert.equal(gate.snapshot().public_requests, 1);
  assert.equal(gate.snapshot().enforcement_failed, false);
  assert.equal(responseCache.snapshot().misses, 1);
  assert.equal(responseCache.snapshot().hits, 1);
  assert.equal(responseCache.snapshot().stores, 1);
});

test("candidate route overrides fall through to the central request gate instead of continuing directly", async () => {
  const sourceDirectory = path.join(VERIFICATION_ROOT, "src");
  const open43Sources = (await fs.readdir(sourceDirectory))
    .filter((name) => /^open43-.*\.mjs$/u.test(name))
    .map((name) => path.join(sourceDirectory, name));
  const routeOwners = [
    ...open43Sources,
    path.join(sourceDirectory, "standing-browser-parity-browser-session.mjs"),
    path.join(VERIFICATION_ROOT, "scripts", "capture-framerail-route-action-temporal.mjs"),
  ];

  for (const filePath of routeOwners) {
    const source = await fs.readFile(filePath, "utf8");
    assert.doesNotMatch(
      source,
      /route\.continue\(/u,
      `${path.relative(VERIFICATION_ROOT, filePath)} must use route.fallback() so the central gate remains authoritative`,
    );
  }
});

test("the 47-case temporal runner reuses the candidate cache override instead of opening a second default cache", async () => {
  const temporalPath = path.join(VERIFICATION_ROOT, "scripts", "capture-framerail-route-action-temporal.mjs");
  const source = await fs.readFile(temporalPath, "utf8");
  assert.match(source, /candidatePublicEvidenceResponseCacheOptions\(\)/u);
  assert.match(source, /sourceResponseCacheOptions,/u);
  assert.match(source, /source_response_cache:/u);
});
