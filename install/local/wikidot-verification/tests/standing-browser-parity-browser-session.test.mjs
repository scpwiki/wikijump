import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createBrowserResponseCache } from "../src/browser-request-gate.mjs";

import {
  candidateLocalOriginSets,
  createParityBrowserControls,
  isParityBrowserPublicOrigin,
  installCandidateFilePortRoute,
  parityBrowserLaunchOptions,
  parityBrowserExecutionMode,
  parityBrowserRequestIntervalMs,
  parityBrowserThrottleConfig,
} from "../src/standing-browser-parity-browser-session.mjs";

const hash = (character) => character.repeat(64);

test("parity browser starts with a cold native HTTP cache", () => {
  assert.deepEqual(parityBrowserLaunchOptions("/browser/chrome"), {
    executablePath: "/browser/chrome",
    args: ["--disable-http-cache"],
  });
});

test("candidate browser local origins include every sealed site on an editable candidate", () => {
  const sets = candidateLocalOriginSets({
    candidate: {
      endpoint: {
        allowed_origin_set: [
          "https://scpaiueouiuiuiui.wikijump.localhost:18449",
          "https://scpaiueouiuiuiui.wjfiles.localhost:18449",
        ],
      },
      site_origins: {
        "scp-wiki": {
          page: "https://scp-wiki.wikijump.localhost:18449",
          files: "https://scp-wiki.wjfiles.localhost:18449",
        },
        scpaiueouiuiuiui: {
          page: "https://scpaiueouiuiuiui.wikijump.localhost:18449",
          files: "https://scpaiueouiuiuiui.wjfiles.localhost:18449",
        },
      },
    },
  });

  assert.deepEqual(
    sets.localOrigins,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
      "https://scpaiueouiuiuiui.wikijump.localhost:18449",
      "https://scpaiueouiuiuiui.wjfiles.localhost:18449",
    ].sort(),
  );
  assert.deepEqual(sets.fileRouteOriginSets, [
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    [
      "https://scpaiueouiuiuiui.wikijump.localhost:18449",
      "https://scpaiueouiuiuiui.wjfiles.localhost:18449",
    ],
  ]);
});

test("browser throttle receipt binds exact case-set public origins", () => {
  const base = {
    args: { mode: "candidate-case" },
    runId: "fixture-run",
    lock: { path: "/private/lock", owner: "fixture" },
    policy: { sha256: hash("a"), value: { policy_version: "fixture-v1" } },
    localOrigins: [],
    candidate: null,
  };
  const config = parityBrowserThrottleConfig({
    ...base,
    publicOrigins: ["https://www.youtube.com", "https://embed.acast.com"],
  });

  assert.equal(parityBrowserExecutionMode("candidate-case"), "candidate");
  assert.equal(parityBrowserRequestIntervalMs("candidate-case"), 0);
  assert.equal(parityBrowserExecutionMode("live-reference"), "live");
  assert.equal(parityBrowserRequestIntervalMs("live-reference"), 0);
  assert.equal(config.execution_mode, "candidate");
  assert.equal(config.interval_ms, 0);
  assert.deepEqual(config.case_set_public_origins, [
    "https://www.youtube.com",
    "https://embed.acast.com",
  ]);
  const liveConfig = parityBrowserThrottleConfig({
    ...base,
    args: { mode: "live-reference" },
    publicOrigins: [],
  });
  assert.equal(liveConfig.execution_mode, "live");
  assert.equal(liveConfig.interval_ms, 0);
  assert.throws(
    () =>
      parityBrowserThrottleConfig({
        ...base,
        publicOrigins: ["https://www.youtube.com/watch"],
      }),
    /exact HTTPS origins/u,
  );
  assert.equal(
    isParityBrowserPublicOrigin(
      "https://www.youtube.com/embed/video",
      "document",
      "GET",
      config.case_set_public_origins,
    ),
    true,
  );
  assert.equal(
    isParityBrowserPublicOrigin(
      "https://www.youtube.com/embed/video",
      "document",
      "POST",
      config.case_set_public_origins,
    ),
    false,
  );
  assert.equal(
    isParityBrowserPublicOrigin(
      "https://youtube.com/embed/video",
      "document",
      "GET",
      config.case_set_public_origins,
    ),
    false,
  );
});

test("resumed browser controls reuse the sealed throttle receipt identity", async () => {
  const outputDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-resume-"));
  const policy = {sha256: hash("a"), value: {policy_version: "fixture-v1"}};
  const args = {mode: "candidate", timeoutMs: 1_000};
  const first = await createParityBrowserControls({args, outputDir, policy, candidate: null});
  const firstConfigSha256 = first.configSha256;
  await first.close();
  const resumed = await createParityBrowserControls({args, outputDir, policy, candidate: null, resume: true});
  assert.equal(resumed.configSha256, firstConfigSha256);
  await resumed.close();
});

test("candidate file routing maps only the exact canonical file authority to its sealed port", async () => {
  const patterns = [];
  let handler;
  const context = {
    async route(value, callback) {
      patterns.push(value);
      handler = callback;
    },
  };
  const origins = [
    "https://scp-wiki.wikijump.localhost:18449",
    "https://scp-wiki.wjfiles.localhost:18449",
  ];

  assert.equal(await installCandidateFilePortRoute(context, origins), true);
  assert.deepEqual(patterns, [
    "https://scp-wiki.wjfiles.localhost/**",
    "https://scp-wiki.wjfiles.localhost:18449/**",
  ]);

  const response = { status: 200 };
  let fetchOptions;
  let fulfillment;
  await handler({
    request() {
      return {
        url() {
          return "https://scp-wiki.wjfiles.localhost/local--files/scp-9506/NFSI.png?download=true";
        },
      };
    },
    async fetch(options) {
      fetchOptions = options;
      return response;
    },
    async fulfill(options) {
      fulfillment = options;
    },
  });
  assert.deepEqual(fetchOptions, {
    url: "https://scp-wiki.wjfiles.localhost:18449/local--files/scp-9506/NFSI.png?download=true",
    maxRedirects: 0,
      headers: {"accept-encoding": "identity"},
  });
  assert.deepEqual(fulfillment, { response });
});

test("candidate file routing also gates an already-localized sealed-port file request", async () => {
  const handlers = new Map();
  const context = {
    async route(value, callback) {
      handlers.set(value, callback);
    },
  };
  let admissions = 0;
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {
      sourceRequestGate: {
        async acquire() {
          admissions += 1000;
        },
        recordSyntheticPublicAdmission(count = 1) {
          admissions += count;
        },
      },
    },
  );
  const handler = handlers.get("https://scp-wiki.wjfiles.localhost:18449/**");
  assert.equal(typeof handler, "function");
  const response = { status: () => 200, headers: () => ({}) };
  let fetchOptions;
  await handler({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "image",
        url: () =>
          "https://scp-wiki.wjfiles.localhost:18449/local--files/theme%3Abasalt/basalt-theme-logo.svg",
      };
    },
    async fetch(options) {
      fetchOptions = options;
      return response;
    },
    async fulfill() {},
  });

  assert.deepEqual(fetchOptions, {
    url: "https://scp-wiki.wjfiles.localhost:18449/local--files/theme%3Abasalt/basalt-theme-logo.svg",
    maxRedirects: 0,
      headers: {"accept-encoding": "identity"},
  });
  assert.equal(admissions, 2);
});

test("candidate file routing follows only same-authority redirects on the sealed port", async () => {
  let handler;
  const context = {
    async route(_value, callback) {
      handler = callback;
    },
  };
  await installCandidateFilePortRoute(context, [
    "https://scp-wiki.wikijump.localhost:18449",
    "https://scp-wiki.wjfiles.localhost:18449",
  ]);

  const fetches = [];
  const finalResponse = { status: () => 200, headers: () => ({}) };
  const responses = [
    {
      status: () => 301,
      headers: () => ({ location: "/-/file/scp-9506/NFSI.png" }),
    },
    finalResponse,
  ];
  let fulfillment;
  await handler({
    request() {
      return {
        method: () => "GET",
        url: () =>
          "https://scp-wiki.wjfiles.localhost/local--files/scp-9506/NFSI.png",
      };
    },
    async fetch(options) {
      fetches.push(options);
      return responses.shift();
    },
    async fulfill(options) {
      fulfillment = options;
    },
  });

  assert.deepEqual(fetches, [
    {
      url: "https://scp-wiki.wjfiles.localhost:18449/local--files/scp-9506/NFSI.png",
      maxRedirects: 0,
      headers: {"accept-encoding": "identity"},
    },
    {
      url: "https://scp-wiki.wjfiles.localhost:18449/-/file/scp-9506/NFSI.png",
      maxRedirects: 0,
      headers: {"accept-encoding": "identity"},
    },
  ]);
  assert.deepEqual(fulfillment, { response: finalResponse });
});

test("candidate file routing returns public redirects to Chromium for gate enforcement", async () => {
  let handler;
  const context = {
    async route(_value, callback) {
      handler = callback;
    },
  };
  await installCandidateFilePortRoute(context, [
    "https://scp-wiki.wikijump.localhost:18449",
    "https://scp-wiki.wjfiles.localhost:18449",
  ]);

  const redirect = {
    status: () => 302,
    headers: () => ({ location: "https://cdn.example.invalid/asset.png" }),
  };
  let fetchCount = 0;
  let fulfillment;
  await handler({
    request() {
      return {
        method: () => "GET",
        url: () => "https://scp-wiki.wjfiles.localhost/local--files/a.png",
      };
    },
    async fetch() {
      fetchCount += 1;
      return redirect;
    },
    async fulfill(options) {
      fulfillment = options;
    },
  });

  assert.equal(fetchCount, 1);
  assert.deepEqual(fulfillment, { response: redirect });
});

test("candidate file routing resolves public image redirects through evidence replay before Chromium can bypass routing", async () => {
  let handler;
  const context = {
    async route(_value, callback) {
      handler = callback;
    },
  };
  const responseCache = createBrowserResponseCache({ evidenceReplay: true });
  let externalAdmissions = 0;
  let syntheticAdmissions = 0;
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {
      responseCache,
      sourceRequestGate: {
        async acquire() {
          externalAdmissions += 1;
        },
        recordSyntheticPublicAdmission(count = 1) {
          syntheticAdmissions += count;
        },
      },
    },
  );

  const targetUrl =
    "https://scp-wiki.wdfiles.com/local--files/theme%3Abasalt/logo.svg";
  const redirect = {
    status: () => 302,
    headers: () => ({ location: targetUrl }),
  };
  const target = {
    status: () => 200,
    headers: () => ({ "content-type": "image/svg+xml" }),
    body: async () => Buffer.from("<svg/>") ,
  };
  const fetches = [];
  let fulfillment;
  await handler({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "image",
        url: () =>
          "https://scp-wiki.wjfiles.localhost/local--files/theme%3Abasalt/logo.svg",
      };
    },
    async fetch(options) {
      fetches.push(options);
      return fetches.length === 1 ? redirect : target;
    },
    async fulfill(options) {
      fulfillment = options;
    },
  });

  assert.deepEqual(fetches, [
    {
      url: "https://scp-wiki.wjfiles.localhost:18449/local--files/theme%3Abasalt/logo.svg",
      maxRedirects: 0,
      headers: {"accept-encoding": "identity"},
    },
    { url: targetUrl, maxRedirects: 0 },
  ]);
  assert.equal(externalAdmissions, 1);
  assert.equal(syntheticAdmissions, 1);
  assert.equal(responseCache.get(targetUrl)?.status, 200);
  assert.deepEqual(fulfillment, {
    status: 200,
    headers: { "content-type": "image/svg+xml" },
    body: Buffer.from("<svg/>"),
  });
});

test("candidate file routing preserves the live public admission before a Wikidot fallback redirect", async () => {
  let handler;
  const context = {
    async route(_value, callback) {
      handler = callback;
    },
  };
  let admissions = 0;
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {
      sourceRequestGate: {
        async acquire() {
          admissions += 1000;
        },
        recordSyntheticPublicAdmission(count = 1) {
          admissions += count;
        },
      },
    },
  );

  const redirect = {
    status: () => 302,
    headers: () => ({
      location:
        "https://scp-wiki.wdfiles.com/local--files/theme%3Abasalt/basalt-theme-logo.svg",
    }),
  };
  let fulfillment;
  await handler({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "image",
        url: () =>
          "https://scp-wiki.wjfiles.localhost/local--files/theme%3Abasalt/basalt-theme-logo.svg",
      };
    },
    async fetch() {
      return redirect;
    },
    async fulfill(options) {
      fulfillment = options;
    },
  });

  assert.equal(admissions, 1);
  assert.deepEqual(fulfillment, { response: redirect });
});

test("candidate file routing preserves both Wikidot local-file admissions on a mirror hit", async () => {
  let handler;
  const context = {
    async route(_value, callback) {
      handler = callback;
    },
  };
  let admissions = 0;
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {
      sourceRequestGate: {
        async acquire() {
          admissions += 1000;
        },
        recordSyntheticPublicAdmission(count = 1) {
          admissions += count;
        },
      },
    },
  );

  const response = { status: () => 200, headers: () => ({}) };
  await handler({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "image",
        url: () =>
          "https://scp-wiki.wjfiles.localhost/local--files/theme%3Abasalt/basalt-theme-logo.svg",
      };
    },
    async fetch() {
      return response;
    },
    async fulfill() {},
  });

  assert.equal(admissions, 2);
});

test("candidate file routing preserves one direct wdfiles admission on a local-code mirror hit", async () => {
  let handler;
  const context = {
    async route(_value, callback) {
      handler = callback;
    },
  };
  let admissions = 0;
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {
      sourceRequestGate: {
        async acquire() {
          admissions += 1000;
        },
        recordSyntheticPublicAdmission(count = 1) {
          admissions += count;
        },
      },
    },
  );

  const response = { status: () => 200, headers: () => ({}) };
  await handler({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "stylesheet",
        url: () =>
          "https://scp-wiki.wjfiles.localhost/local--code/theme%3Abasalt/1",
      };
    },
    async fetch() {
      return response;
    },
    async fulfill() {},
  });

  assert.equal(admissions, 1);
});

test("candidate file routing replays a retained wdfiles asset when the local mirror misses", async () => {
  const handlers = new Map();
  const context = {
    async route(value, callback) {
      handlers.set(value, callback);
    },
  };
  const cached = {
    status: 200,
    headers: {"content-type": "image/png"},
    body: Buffer.from("retained-asset"),
  };
  const lookups = [];
  const responseCache = {
    evidenceReplay: true,
    cacheDocuments: true,
    lookup(key) {
      lookups.push(key);
      return cached;
    },
  };
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {responseCache},
  );

  let fulfillment;
  const route = (url, response) => ({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "image",
        url: () => url,
      };
    },
    async fetch() {
      return response;
    },
    async fulfill(options) {
      fulfillment = options;
    },
  });
  await handlers.get("https://scp-wiki.wjfiles.localhost:18449/**")(
    route(
      "https://scp-wiki.wjfiles.localhost:18449/local--files/scp-8980/femalescientist.png",
      {status: () => 404, headers: () => {}},
    ),
  );

  assert.deepEqual(lookups, [
    "https://scp-wiki.wdfiles.com/local--files/scp-8980/femalescientist.png",
  ]);
  assert.deepEqual(fulfillment, {status: cached.status, headers: cached.headers, body: cached.body});

  fulfillment = undefined;
  await handlers.get("https://scp-wiki.wikijump.localhost:18449/local--files/**")(
    route(
      "https://scp-wiki.wikijump.localhost:18449/local--files/scp-8980/femalescientist.png",
      {status: () => 404, headers: () => {}},
    ),
  );
  assert.deepEqual(fulfillment, {status: cached.status, headers: cached.headers, body: cached.body});
  assert.deepEqual(lookups, [
    "https://scp-wiki.wdfiles.com/local--files/scp-8980/femalescientist.png",
    "https://scp-wiki.wdfiles.com/local--files/scp-8980/femalescientist.png",
  ]);
});

test("candidate file fallback replay keeps GET, HEAD, Range, and credential identities separate", async () => {
  const handlers = new Map();
  const context = {
    async route(value, callback) {
      handlers.set(value, callback);
    },
  };
  const targetUrl = "https://scp-wiki.wdfiles.com/local--files/scp-8980/femalescientist.png";
  const cached = {
    status: 200,
    headers: {"content-type": "image/png"},
    body: Buffer.from("retained-get"),
  };
  const responseCache = createBrowserResponseCache({evidenceReplay: true});
  assert.equal(responseCache.store(targetUrl, cached, {requestHeaders: {}}), true);
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {responseCache},
  );
  const handler = handlers.get("https://scp-wiki.wjfiles.localhost:18449/**");
  const localUrl = "https://scp-wiki.wjfiles.localhost:18449/local--files/scp-8980/femalescientist.png";

  async function observe(method, headers = {}) {
    const localMiss = {status: () => 404, headers: () => ({})};
    let fulfillment = null;
    await handler({
      request() {
        return {
          method: () => method,
          resourceType: () => "image",
          url: () => localUrl,
          headers: () => headers,
          allHeaders: async () => headers,
        };
      },
      async fetch() {
        return localMiss;
      },
      async fulfill(options) {
        fulfillment = options;
      },
    });
    return {fulfillment, localMiss};
  }

  const get = await observe("GET");
  assert.deepEqual(get.fulfillment, {status: cached.status, headers: cached.headers, body: cached.body});
  for (const [method, headers] of [
    ["HEAD", {}],
    ["GET", {range: "bytes=0-9"}],
    ["GET", {cookie: "session=secret"}],
  ]) {
    const observation = await observe(method, headers);
    assert.deepEqual(observation.fulfillment, {response: observation.localMiss});
  }
});

test("candidate file redirect acquisition leaves a durable no-refetch barrier after transport uncertainty", async (t) => {
  const handlers = new Map();
  const context = {
    async route(value, callback) {
      handlers.set(value, callback);
    },
  };
  const cacheDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-file-redirect-barrier-"));
  t.after(() => fs.rm(cacheDir, {recursive: true, force: true}));
  const responseCache = createBrowserResponseCache({
    persistentDir: cacheDir,
    persistentIdentity: "evidence:file-redirect-barrier",
    evidenceReplay: true,
  });
  await responseCache.load();
  let externalAdmissions = 0;
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {
      responseCache,
      sourceRequestGate: {
        async acquire() {
          externalAdmissions += 1;
        },
        recordSyntheticPublicAdmission() {},
      },
    },
  );
  const handler = handlers.get("https://scp-wiki.wjfiles.localhost:18449/**");
  const localUrl = "https://scp-wiki.wjfiles.localhost/local--files/theme%3Abasalt/logo.svg";
  const targetUrl = "https://scp-wiki.wdfiles.com/local--files/theme%3Abasalt/logo.svg";
  const redirect = {status: () => 302, headers: () => ({location: targetUrl})};
  let targetFetches = 0;
  const makeRoute = () => ({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "image",
        url: () => localUrl,
        headers: () => ({}),
        allHeaders: async () => ({}),
      };
    },
    async fetch(options) {
      if (options.url === targetUrl) {
        targetFetches += 1;
        throw new Error("simulated uncertain redirect transport");
      }
      return redirect;
    },
    async fulfill() {},
  });

  await assert.rejects(handler(makeRoute()), /simulated uncertain redirect transport/u);
  await assert.rejects(handler(makeRoute()), /refuses to reacquire an identity with uncertain prior acquisition/u);
  assert.equal(targetFetches, 1);
  assert.equal(externalAdmissions, 1);
  assert.equal(responseCache.snapshot().acquisition_barriers, 1);
  const manifest = JSON.parse(await fs.readFile(path.join(cacheDir, "manifest.json"), "utf8"));
  assert.deepEqual(manifest.acquisition_barriers, [targetUrl]);
});

test("candidate file public redirects never replay anonymous target evidence for credential-bearing requests", async () => {
  const targetUrl = "https://scp-wiki.wdfiles.com/local--files/theme%3Abasalt/logo.svg";
  for (const variant of [
    {name: "cookie", headers: {cookie: "session=secret"}, userinfo: ""},
    {name: "authorization", headers: {authorization: "Bearer secret"}, userinfo: ""},
    {name: "userinfo", headers: {}, userinfo: "user:secret@"},
  ]) {
    const handlers = new Map();
    const context = {
      async route(value, callback) {
        handlers.set(value, callback);
      },
    };
    const responseCache = createBrowserResponseCache({evidenceReplay: true});
    assert.equal(responseCache.store(targetUrl, {
      status: 200,
      headers: {"content-type": "image/svg+xml"},
      body: Buffer.from("anonymous-retained-target"),
    }, {requestHeaders: {}}), true);
    let externalAdmissions = 0;
    await installCandidateFilePortRoute(
      context,
      [
        "https://scp-wiki.wikijump.localhost:18449",
        "https://scp-wiki.wjfiles.localhost:18449",
      ],
      {
        responseCache,
        sourceRequestGate: {
          async acquire() {
            externalAdmissions += 1;
          },
          recordSyntheticPublicAdmission() {},
        },
      },
    );
    const redirect = {status: () => 302, headers: () => ({location: targetUrl})};
    let fulfillment = null;
    let targetFetches = 0;
    await handlers.get("https://scp-wiki.wjfiles.localhost:18449/**")({
      request() {
        return {
          method: () => "GET",
          resourceType: () => "image",
          url: () => `https://${variant.userinfo}scp-wiki.wjfiles.localhost/local--files/theme%3Abasalt/logo.svg`,
          headers: () => variant.headers,
          allHeaders: async () => variant.headers,
        };
      },
      async fetch(options) {
        if (options.url === targetUrl) targetFetches += 1;
        return redirect;
      },
      async fulfill(options) {
        fulfillment = options;
      },
    });

    assert.deepEqual(fulfillment, {response: redirect}, variant.name);
    assert.equal(targetFetches, 0, variant.name);
    assert.equal(externalAdmissions, 0, variant.name);
  }
});

test("candidate file routing preserves collapsed source admissions when a redirect is outside the public gate", async () => {
  let handler;
  const context = {
    async route(_value, callback) {
      handler = callback;
    },
  };
  let admissions = 0;
  await installCandidateFilePortRoute(
    context,
    [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost:18449",
    ],
    {
      sourceRequestGate: {
        async acquire() {
          admissions += 1000;
        },
        recordSyntheticPublicAdmission(count = 1) {
          admissions += count;
        },
      },
    },
  );

  const redirect = {
    status: () => 302,
    headers: () => ({ location: "https://cdn.example.invalid/asset.png" }),
  };
  await handler({
    request() {
      return {
        method: () => "GET",
        resourceType: () => "image",
        url: () => "https://scp-wiki.wjfiles.localhost/local--files/a.png",
      };
    },
    async fetch() {
      return redirect;
    },
    async fulfill() {},
  });

  assert.equal(admissions, 2);
});

test("candidate file routing refuses malformed or ambiguous local origin declarations", async () => {
  const context = {
    async route() {
      throw new Error("must not install");
    },
  };
  await assert.rejects(
    installCandidateFilePortRoute(context, [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://other.wjfiles.localhost:18449",
    ]),
    /same site slug/u,
  );
  await assert.rejects(
    installCandidateFilePortRoute(context, [
      "https://scp-wiki.wikijump.localhost:18449",
      "https://scp-wiki.wjfiles.localhost",
    ]),
    /same explicit non-443 port/u,
  );
});

test("standing file routing preserves canonical HTTPS origins while replaying source redirects", async () => {
  const installed = [];
  const context = {
    async route(pattern, handler) {
      installed.push({pattern, handler});
    },
  };
  assert.equal(
    await installCandidateFilePortRoute(
      context,
      [
        "https://scp-wiki.wikijump.localhost",
        "https://scp-wiki.wjfiles.localhost",
      ],
      {allowDefaultHttpsPort: true},
    ),
    true,
  );
  assert.deepEqual(
    installed.map(({pattern}) => pattern),
    ["https://scp-wiki.wjfiles.localhost/**"],
  );
});


test("local file replay negotiates identity encoding while preserving request headers", async () => {
  for (const [encoding, pathname] of [
    ["gzip, deflate, br, zstd", "/local--code/theme:example/1"],
    ["gzip, deflate, br", "/local--files/example/image.png"],
    ["identity", "/local--code/component:example/2"],
    ["", "/local--files/example/font.woff2"],
  ]) {
    let handler;
    await installCandidateFilePortRoute({async route(_pattern, callback) {handler = callback;}}, [
      "https://example.wikijump.localhost:18449",
      "https://example.wjfiles.localhost:18449",
    ]);
    let observed;
    const response = {status: () => 200, headers: () => ({"content-type": "text/css"})};
    await handler({
      request: () => ({url: () => `https://example.wjfiles.localhost:18449${pathname}`, method: () => "GET", headers: () => ({"accept-encoding": encoding, origin: "https://example.wikijump.localhost:18449", "x-fixture-header": "preserved"})}),
      fetch: async options => {observed = options; return response;},
      fulfill: async () => {},
    });
    assert.equal(observed.headers["accept-encoding"], "identity");
    assert.equal(observed.headers.origin, "https://example.wikijump.localhost:18449");
    assert.equal(observed.headers["x-fixture-header"], "preserved");
  }
});
