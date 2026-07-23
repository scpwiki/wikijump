import assert from "node:assert/strict";
import {execFile} from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";
import {fileURLToPath} from "node:url";
import {promisify} from "node:util";
import {browserCaptureFailure, browserContextOptions, capturePage, defaultBrowserRoot, openBrowser, resolveStorageStates} from "../scripts/capture-browser-rendering.mjs";
import {
  buildEvidenceRecord,
  compactVisibleText,
  inventoryRows,
  rowLocalUrl,
  rowSourceUrl,
  safePathSegment,
  selectInventoryRows,
  writeEvidenceArtifacts,
} from "../src/browser-render-evidence.mjs";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = path.resolve(__dirname, "../scripts/capture-browser-rendering.mjs");

const inventory = {
  schema: "wikijump_full_parity.corpus_inventory_lock.v1",
  rows: [
    {
      fixture_id: "EN:alpha",
      family: "EN",
      slug: "alpha",
      source_url: "https://scp-wiki.wikidot.com/alpha",
      local_https_url: "https://scp-wiki.wikijump.localhost/alpha",
      required_browser: true,
    },
    {
      fixture_id: "EN:beta",
      family: "EN",
      slug: "beta",
      source_url: "https://scp-wiki.wikidot.com/beta",
      local_https_url: "https://scp-wiki.wikijump.localhost/beta",
      required_browser: true,
    },
  ],
};

test("browser capture failure preserves both operation and cleanup errors", () => {
  const captureError = new Error("capture failed");
  const cleanupError = new Error("cleanup failed");
  const combined = browserCaptureFailure(captureError, cleanupError);
  assert(combined instanceof AggregateError);
  assert.deepEqual(combined.errors, [captureError, cleanupError]);
  assert.equal(browserCaptureFailure(captureError, null), captureError);
  assert.equal(browserCaptureFailure(null, cleanupError), cleanupError);
  assert.equal(browserCaptureFailure(null, null), null);
});

test("selectInventoryRows intersects explicit fixture ids with shard membership", () => {
  const rows = inventoryRows(inventory);
  const selected = selectInventoryRows({
    rows,
    fixtureIds: ["EN:alpha", "EN:beta"],
    shardId: "en-0001",
    shardManifest: {
      schema: "wikijump_full_parity.corpus_shard_manifest.v1",
      shards: [{shard_id: "en-0001", fixture_ids: ["EN:beta"]}],
    },
  });

  assert.deepEqual(selected.map((row) => row.fixture_id), ["EN:beta"]);
});

test("selectInventoryRows rejects absent requested fixture ids", () => {
  const rows = inventoryRows(inventory);
  assert.throws(
    () => selectInventoryRows({rows, fixtureIds: ["EN:alpha", "EN:missing"]}),
    /requested fixture IDs were not found: EN:missing/
  );
});

test("inventoryRows rejects duplicate fixture ids", () => {
  assert.throws(
    () => inventoryRows({schema: inventory.schema, rows: [inventory.rows[0], {...inventory.rows[1], fixture_id: "EN:alpha"}]}),
    /inventory\.rows\[1\] duplicates fixture_id: EN:alpha/
  );
});

test("selectInventoryRows rejects shard fixture ids missing from the inventory", () => {
  const rows = inventoryRows(inventory);
  assert.throws(
    () =>
      selectInventoryRows({
        rows,
        shardId: "en-0001",
        shardManifest: {
          schema: "wikijump_full_parity.corpus_shard_manifest.v1",
          shards: [{shard_id: "en-0001", fixture_ids: ["EN:alpha", "EN:missing"]}],
        },
      }),
    /shard en-0001 fixture IDs were not found in inventory: EN:missing/
  );
});

test("buildEvidenceRecord emits fields accepted by the browser rendering validator", () => {
  const record = buildEvidenceRecord({
    row: inventory.rows[0],
    source: {status: 200, finalUrl: "https://scp-wiki.wikidot.com/alpha", visibleText: " Alpha\n page "},
    local: {status: 200, finalUrl: "https://scp-wiki.wikijump.localhost/alpha", visibleText: "Alpha page"},
    sourceArtifact: "/tmp/live.dom.html",
    localArtifact: "/tmp/local.dom.html",
    sourceScreenshot: "/tmp/live.png",
    localScreenshot: "/tmp/local.png",
  });

  assert.equal(record.evidence_type, "browser_rendering");
  assert.equal(record.fixture_id, "EN:alpha");
  assert.equal(record.source_visible_text, "Alpha page");
  assert.equal(record.local_visible_text, "Alpha page");
  assert.equal(record.source_browser_artifact, "/tmp/live.dom.html");
  assert.equal(record.local_browser_artifact, "/tmp/local.dom.html");
  assert.deepEqual(record.capture_errors, []);
});

test("writeEvidenceArtifacts keeps row artifacts under a safe fixture directory", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-evidence-"));
  const artifacts = await writeEvidenceArtifacts({
    outputDir: root,
    row: {fixture_id: "EN:../alpha beta"},
    source: {html: "<html>live</html>"},
    local: {html: "<html>local</html>"},
    screenshot: true,
  });

  assert.equal(path.dirname(artifacts.sourceArtifact), path.join(root, safePathSegment("EN:../alpha beta")));
  assert.equal(await fs.readFile(artifacts.sourceArtifact, "utf8"), "<html>live</html>");
  assert.equal(await fs.readFile(artifacts.localArtifact, "utf8"), "<html>local</html>");
  assert.equal(compactVisibleText(" one\n\t two "), "one two");
});

test("safePathSegment keeps colliding fixture IDs distinct", () => {
  assert.notEqual(safePathSegment("EN:a/b"), safePathSegment("EN:a_b"));
  assert.doesNotMatch(safePathSegment("EN:alpha"), /:/);
  assert.doesNotMatch(safePathSegment("EN:alpha."), /\.-[a-f0-9]{12}$/);
  assert.notEqual(
    safePathSegment(`EN:${"a".repeat(180)}1`),
    safePathSegment(`EN:${"a".repeat(180)}2`)
  );
});

test("inventoryRows rejects malformed rows before browser capture starts", () => {
  assert.throws(
    () => inventoryRows({schema: inventory.schema, rows: [{slug: "missing-fixture"}]}),
    /inventory\.rows\[0\] must be an object with a non-empty fixture_id/
  );
  assert.throws(
    () => inventoryRows({schema: inventory.schema, rows: [null]}),
    /inventory\.rows\[0\] must be an object with a non-empty fixture_id/
  );
});

test("row URL helpers skip blank preferred fields before falling back", () => {
  assert.equal(rowSourceUrl({source_url: "", live_url: "https://live.example/page"}), "https://live.example/page");
  assert.equal(
    rowLocalUrl({local_https_url: "", local_http_url: "http://local.example/page"}),
    "http://local.example/page"
  );
});

test("default browser root is resolved from the repository, not cwd", () => {
  const originalCwd = process.cwd();
  process.chdir(__dirname);
  try {
    assert.equal(defaultBrowserRoot(), path.resolve(__dirname, "../../../..", "framerail"));
  } finally {
    process.chdir(originalCwd);
  }
});

test("browser capture exits when Playwright cannot be loaded", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-missing-playwright-"));
  const outputDir = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-output-"));
  const inventoryPath = path.join(root, "inventory.json");
  await fs.writeFile(inventoryPath, JSON.stringify(inventory), "utf8");
  await fs.writeFile(path.join(root, "package.json"), "{}", "utf8");

  await assert.rejects(
    () =>
      execFileAsync(
        process.execPath,
        [
          scriptPath,
          "--inventory",
          inventoryPath,
          "--output-dir",
          outputDir,
          "--limit",
          "1",
          "--browser-root",
          root,
        ],
        {timeout: 1_000},
      ),
    (error) => {
      assert.equal(error.killed, false);
      assert.match(error.stderr, /could not load playwright or @playwright\/test/);
      return true;
    },
  );
});

test("openBrowser applies HTTPS ignore settings to a fresh CDP context", async () => {
  const newContextOptions = [];
  const closedContexts = [];
  let closedBrowser = false;
  const browser = {
    async newContext(options) {
      const context = {
        id: newContextOptions.length,
        async close() {
          closedContexts.push(this.id);
        },
      };
      newContextOptions.push(options);
      return context;
    },
    async close() {
      closedBrowser = true;
    },
  };
  const chromium = {
    async connectOverCDP(endpoint) {
      assert.equal(endpoint, "http://127.0.0.1:9222");
      return browser;
    },
  };

  const session = await openBrowser({
    chromium,
    cdpEndpoint: "http://127.0.0.1:9222",
    ignoreHttpsErrors: true,
  });
  assert.equal(session.context, session.sourceContext);
  assert.notEqual(session.sourceContext, session.localContext);
  assert.deepEqual(newContextOptions, [
    {ignoreHTTPSErrors: true},
    {ignoreHTTPSErrors: true},
  ]);

  await session.close();
  assert.deepEqual(closedContexts, [1, 0]);
  assert.equal(closedBrowser, true);
});

test("openBrowser can isolate source and local storage states", async () => {
  const newContextOptions = [];
  const closedContexts = [];
  let closedBrowser = false;
  const browser = {
    async newContext(options) {
      const context = {
        id: newContextOptions.length,
        async close() {
          closedContexts.push(this.id);
        },
      };
      newContextOptions.push(options);
      return context;
    },
    async close() {
      closedBrowser = true;
    },
  };
  const chromium = {
    async launch(options) {
      assert.deepEqual(options, {executablePath: "/usr/bin/google-chrome"});
      return browser;
    },
  };

  const session = await openBrowser({
    chromium,
    browserExecutable: "/usr/bin/google-chrome",
    ignoreHttpsErrors: true,
    sourceStorageState: "/private/source.json",
    localStorageState: "/private/local.json",
  });

  assert.notEqual(session.sourceContext, session.localContext);
  assert.deepEqual(newContextOptions, [
    {ignoreHTTPSErrors: true, storageState: "/private/source.json"},
    {ignoreHTTPSErrors: true, storageState: "/private/local.json"},
  ]);
  await session.close();
  assert.deepEqual(closedContexts, [1, 0]);
  assert.equal(closedBrowser, true);
});

test("openBrowser closes partial resources when context creation fails", async () => {
  let closedSourceContext = false;
  let closedBrowser = false;
  const browser = {
    async newContext(options) {
      if (options.storageState === "/private/source.json") {
        return {
          async close() {
            closedSourceContext = true;
          },
        };
      }
      throw new Error("local context failed");
    },
    async close() {
      closedBrowser = true;
    },
  };
  const chromium = {
    async launch() {
      return browser;
    },
  };

  await assert.rejects(
    () =>
      openBrowser({
        chromium,
        browserExecutable: "/usr/bin/google-chrome",
        ignoreHttpsErrors: true,
        sourceStorageState: "/private/source.json",
        localStorageState: "/private/local.json",
      }),
    /local context failed/
  );
  assert.equal(closedSourceContext, true);
  assert.equal(closedBrowser, true);
});

test("browser context options do not expose unset storage state", () => {
  assert.deepEqual(browserContextOptions({ignoreHttpsErrors: true}), {ignoreHTTPSErrors: true});
  assert.deepEqual(
    browserContextOptions({ignoreHttpsErrors: true, blockServiceWorkers: true}),
    {ignoreHTTPSErrors: true, serviceWorkers: "block"}
  );
  assert.deepEqual(
    browserContextOptions({ignoreHttpsErrors: false, storageState: "/private/state.json"}),
    {ignoreHTTPSErrors: false, storageState: "/private/state.json"}
  );
  assert.deepEqual(
    resolveStorageStates({storageState: "/private/shared.json", sourceStorageState: "/private/source.json"}),
    {sourceStorageState: "/private/source.json", localStorageState: "/private/shared.json"}
  );
});

test("capturePage records page errors and failed subframe responses", async () => {
  const handlers = new Map();
  const mainFrame = {name: "main"};
  const childFrame = {name: "child"};
  const page = {
    on(event, handler) {
      handlers.set(event, handler);
    },
    mainFrame() {
      return mainFrame;
    },
    async goto() {
      handlers.get("pageerror")?.(new Error("client render failed"));
      handlers.get("response")?.({
        status: () => 500,
        url: () => "https://local.example/main",
        request: () => ({
          isNavigationRequest: () => true,
          frame: () => mainFrame,
          resourceType: () => "document",
        }),
      });
      handlers.get("response")?.({
        status: () => 500,
        url: () => "https://local.example/frame",
        request: () => ({
          isNavigationRequest: () => true,
          frame: () => childFrame,
          resourceType: () => "document",
        }),
      });
      return {status: () => 200};
    },
    async waitForLoadState() {},
    frames() {
      return [
        {async evaluate() { return "visible"; }},
        {async evaluate() { return "child frame text"; }},
      ];
    },
    async content() {
      return "<html>visible</html>";
    },
    url() {
      return "https://local.example/page";
    },
  };

  const result = await capturePage(page, "https://local.example/page", {
    timeoutMs: 100,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
  });

  assert.deepEqual(result.consoleErrors, ["client render failed"]);
  assert.equal(result.visibleText, "visible\nchild frame text");
  assert.deepEqual(result.failedRequests, [
    {
      url: "https://local.example/frame",
      status: 500,
      resourceType: "document",
    },
  ]);
});

test("capturePage can scope visible text to the main frame", async () => {
  const mainFrame = {async evaluate() { return "main frame text"; }};
  const childFrame = {async evaluate() { return "child frame text"; }};
  const page = {
    on() {},
    mainFrame() {
      return mainFrame;
    },
    async goto() {
      return {status: () => 200};
    },
    async waitForLoadState() {},
    frames() {
      return [mainFrame, childFrame];
    },
    async content() {
      return "<html>main frame text</html>";
    },
    url() {
      return "https://local.example/page";
    },
  };

  const result = await capturePage(page, "https://local.example/page", {
    timeoutMs: 100,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
    visibleTextScope: "main-frame",
  });

  assert.equal(result.visibleText, "main frame text");
});

test("capturePage skips hidden child frames for all-frame visible text", async () => {
  const mainFrame = {async evaluate() { return "main frame text"; }};
  const visibleFrame = {
    async frameElement() {
      return {async evaluate() { return true; }};
    },
    async evaluate() {
      return "visible iframe text";
    },
  };
  const hiddenFrame = {
    async frameElement() {
      return {async evaluate() { return false; }};
    },
    async evaluate() {
      throw new Error("hidden frame text should not be read");
    },
  };
  const page = {
    on() {},
    mainFrame() {
      return mainFrame;
    },
    async goto() {
      return {status: () => 200};
    },
    async waitForLoadState() {},
    frames() {
      return [mainFrame, visibleFrame, hiddenFrame];
    },
    async content() {
      return "<html>main frame text</html>";
    },
    url() {
      return "https://local.example/page";
    },
  };

  const result = await capturePage(page, "https://local.example/page", {
    timeoutMs: 100,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
  });

  assert.equal(result.visibleText, "main frame text\nvisible iframe text");
});

test("capturePage records delayed main-frame navigation failures", async () => {
  const handlers = new Map();
  const mainFrame = {name: "main"};
  const page = {
    on(event, handler) {
      handlers.set(event, handler);
    },
    mainFrame() {
      return mainFrame;
    },
    async goto() {
      handlers.get("response")?.({
        status: () => 200,
        url: () => "https://local.example/initial",
        request: () => ({
          isNavigationRequest: () => true,
          frame: () => mainFrame,
          resourceType: () => "document",
        }),
      });
      return {status: () => 200};
    },
    async waitForLoadState(state) {
      if (state !== "load") return;
      handlers.get("response")?.({
        status: () => 404,
        url: () => "https://local.example/not-found",
        request: () => ({
          isNavigationRequest: () => true,
          frame: () => mainFrame,
          resourceType: () => "document",
        }),
      });
    },
    frames() {
      return [{async evaluate() { return "visible"; }}];
    },
    async content() {
      return "<html>visible</html>";
    },
    url() {
      return "https://local.example/not-found";
    },
  };

  const result = await capturePage(page, "https://local.example/page", {
    timeoutMs: 100,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
  });

  assert.deepEqual(result.failedRequests, [
    {
      url: "https://local.example/not-found",
      status: 404,
      resourceType: "document",
    },
  ]);
});

test("capturePage bounds post-navigation load-state waits", async () => {
  const loadStateTimeouts = [];
  const page = {
    on() {},
    mainFrame() {
      return {};
    },
    async goto() {
      return {status: () => 200};
    },
    async waitForLoadState(_state, options) {
      loadStateTimeouts.push(options.timeout);
    },
    frames() {
      return [{async evaluate() { return "visible"; }}];
    },
    async content() {
      return "<html>visible</html>";
    },
    url() {
      return "https://local.example/page";
    },
  };

  await capturePage(page, "https://local.example/page", {
    timeoutMs: 30_000,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
  });

  assert.deepEqual(loadStateTimeouts, [2_000, 2_000]);
});

test("capture CLI rejects an empty row selection before launching a browser", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-empty-selection-"));
  const inventoryPath = path.join(root, "inventory.json");
  await fs.writeFile(inventoryPath, JSON.stringify(inventory), "utf8");

  await assert.rejects(
    execFileAsync(process.execPath, [
      scriptPath,
      "--inventory",
      inventoryPath,
      "--output-dir",
      path.join(root, "out"),
      "--fixture-id",
      "EN:missing",
      "--json",
    ]),
    (error) => {
      assert.match(error.stderr, /requested fixture IDs were not found: EN:missing/);
      return true;
    }
  );
});

test("capture CLI rejects shard manifests without a shard id", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-missing-shard-id-"));
  const inventoryPath = path.join(root, "inventory.json");
  const shardManifestPath = path.join(root, "shards.json");
  await fs.writeFile(inventoryPath, JSON.stringify(inventory), "utf8");
  await fs.writeFile(shardManifestPath, JSON.stringify({shards: []}), "utf8");

  await assert.rejects(
    execFileAsync(process.execPath, [
      scriptPath,
      "--inventory",
      inventoryPath,
      "--output-dir",
      path.join(root, "out"),
      "--shard-manifest",
      shardManifestPath,
      "--json",
    ]),
    (error) => {
      assert.match(error.stderr, /--shard-id is required when --shard-manifest is provided/);
      return true;
    }
  );
});

test("capture CLI rejects public and credentialed local URLs before it can launch a browser", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-local-origin-policy-"));
  const inventoryPath = path.join(root, "inventory.json");
  const outputDir = path.join(root, "out");
  const invalidLocalUrls = [
    "https://public.example/scp-173",
    "https://user@scp-wiki.wikijump.localhost/scp-173",
  ];

  for (const localHttpsUrl of invalidLocalUrls) {
    await fs.writeFile(
      inventoryPath,
      JSON.stringify({
        schema: inventory.schema,
        rows: [{...inventory.rows[0], local_https_url: localHttpsUrl}],
      }),
      "utf8"
    );
    await assert.rejects(
      execFileAsync(process.execPath, [
        scriptPath,
        "--inventory",
        inventoryPath,
        "--output-dir",
        outputDir,
        "--json",
      ]),
      (error) => {
        assert.match(error.stderr, /invalid local capture URL/);
        return true;
      }
    );
  }
});

test("capture CLI reuses one source and local context across all rows", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-row-contexts-"));
  const browserRoot = path.join(root, "browser-root");
  const inventoryPath = path.join(root, "inventory.json");
  const outputDir = path.join(root, "out");
  const tracePath = path.join(root, "contexts.jsonl");
  await fs.mkdir(browserRoot);
  await fs.writeFile(path.join(browserRoot, "package.json"), "{}", "utf8");
  await fs.mkdir(path.join(browserRoot, "node_modules", "@playwright", "test"), {recursive: true});
  await fs.writeFile(
    path.join(browserRoot, "node_modules", "@playwright", "test", "index.js"),
    `
const fs = require("node:fs");
let nextContextId = 0;
function trace(entry) {
  fs.appendFileSync(process.env.WIKIJUMP_CONTEXT_TRACE, JSON.stringify(entry) + "\\n");
}
class Page {
  constructor(contextId) {
    this.contextId = contextId;
    this.handlers = new Map();
    this.currentUrl = "about:blank";
  }
  on(event, handler) { this.handlers.set(event, handler); }
  async goto(url) { this.currentUrl = url; return {status: () => 200}; }
  async waitForLoadState() {}
  frames() { return [{evaluate: async () => "context-" + this.contextId}]; }
  async content() { return "<html>context-" + this.contextId + "</html>"; }
  async screenshot({path}) { fs.writeFileSync(path, "png"); }
  url() { return this.currentUrl; }
  async close() {}
}
exports.chromium = {
  async launch() {
    return {
      async newContext(options) {
        const id = nextContextId++;
        trace({event: "newContext", id, options});
        return {
          async newPage() { trace({event: "newPage", id}); return new Page(id); },
          async route() {},
          async routeWebSocket() {},
          on() {},
          async close() { trace({event: "closeContext", id}); },
        };
      },
      async close() {},
    };
  },
};
`,
    "utf8"
  );
  await fs.writeFile(inventoryPath, JSON.stringify(inventory), "utf8");

  await execFileAsync(
    process.execPath,
    [
      scriptPath,
      "--inventory",
      inventoryPath,
      "--output-dir",
      outputDir,
      "--browser-root",
      browserRoot,
      "--json",
    ],
    {env: {...process.env, WIKIJUMP_CONTEXT_TRACE: tracePath}}
  );

  const trace = (await fs.readFile(tracePath, "utf8"))
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const records = JSON.parse(await fs.readFile(path.join(outputDir, "records.json"), "utf8"));
  assert.deepEqual(
    trace.filter((entry) => entry.event === "newContext").map((entry) => entry.id),
    [0, 1]
  );
  assert.deepEqual(
    trace.filter((entry) => entry.event === "newContext").map((entry) => entry.options.serviceWorkers),
    ["block", "block"]
  );
  assert.deepEqual(
    trace.filter((entry) => entry.event === "closeContext").map((entry) => entry.id),
    [1, 0]
  );
  assert.deepEqual(
    records.evidence.map((record) => [
      record.fixture_id,
      record.source_visible_text,
      record.local_visible_text,
    ]),
    [
      ["EN:alpha", "context-0", "context-1"],
      ["EN:beta", "context-0", "context-1"],
    ]
  );
  const requestGateConfig = JSON.parse(await fs.readFile(path.join(outputDir, "request-gate-config.json"), "utf8"));
  assert.equal(requestGateConfig.status, "sealed_before_browser_request");
  assert.equal(requestGateConfig.interval_ms, 4_000);
  assert.equal(records.capture.request_gate.public_requests, 0);
  assert.equal(records.capture.browser_context_scope, "run");
  assert.equal(records.capture.source_response_cache.entries, 0);
});

test("capture CLI records requested visible text scope", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-visible-scope-"));
  const browserRoot = path.join(root, "browser-root");
  const inventoryPath = path.join(root, "inventory.json");
  const outputDir = path.join(root, "out");
  await fs.mkdir(browserRoot);
  await fs.writeFile(path.join(browserRoot, "package.json"), "{}", "utf8");
  await fs.mkdir(path.join(browserRoot, "node_modules", "@playwright", "test"), {recursive: true});
  await fs.writeFile(
    path.join(browserRoot, "node_modules", "@playwright", "test", "index.js"),
    `
let nextContextId = 0;
class Page {
  constructor(contextId) {
    this.contextId = contextId;
    this.handlers = new Map();
    this.currentUrl = "about:blank";
    this.main = {evaluate: async () => "main-" + this.contextId};
    this.child = {evaluate: async () => "child-" + this.contextId};
  }
  on(event, handler) { this.handlers.set(event, handler); }
  async goto(url) { this.currentUrl = url; return {status: () => 200}; }
  async waitForLoadState() {}
  mainFrame() { return this.main; }
  frames() { return [this.main, this.child]; }
  async content() { return "<html>main-" + this.contextId + "</html>"; }
  async screenshot({path}) { require("node:fs").writeFileSync(path, "png"); }
  url() { return this.currentUrl; }
  async close() {}
}
exports.chromium = {
  async launch() {
    return {
      async newContext() {
        const id = nextContextId++;
        return {async newPage() { return new Page(id); }, async route() {}, async routeWebSocket() {}, on() {}, async close() {}};
      },
      async close() {},
    };
  },
};
`,
    "utf8"
  );
  await fs.writeFile(inventoryPath, JSON.stringify({schema: inventory.schema, rows: [inventory.rows[0]]}), "utf8");

  await execFileAsync(process.execPath, [
    scriptPath,
    "--inventory",
    inventoryPath,
    "--output-dir",
    outputDir,
    "--browser-root",
    browserRoot,
    "--visible-text-scope",
    "main-frame",
    "--json",
  ]);

  const records = JSON.parse(await fs.readFile(path.join(outputDir, "records.json"), "utf8"));
  const [record] = records.evidence;
  assert.equal(records.capture.visible_text_scope, "main-frame");
  assert.equal(record.source_visible_text, "main-0");
  assert.equal(record.local_visible_text, "main-1");
});

test("capture CLI keeps source evidence when local URL is missing", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-partial-capture-"));
  const browserRoot = path.join(root, "browser-root");
  const inventoryPath = path.join(root, "inventory.json");
  const outputDir = path.join(root, "out");
  await fs.mkdir(browserRoot);
  await fs.writeFile(path.join(browserRoot, "package.json"), "{}", "utf8");
  await fs.mkdir(path.join(browserRoot, "node_modules", "@playwright", "test"), {recursive: true});
  await fs.writeFile(
    path.join(browserRoot, "node_modules", "@playwright", "test", "index.js"),
    `
const sourceHtml = "<html><body>source ok</body></html>";
class Page {
  constructor() {
    this.handlers = new Map();
    this.currentUrl = "about:blank";
  }
  on(event, handler) { this.handlers.set(event, handler); }
  async goto(url) { this.currentUrl = url; return {status: () => 200}; }
  async waitForLoadState() {}
  async evaluate() { return "source ok"; }
  async content() { return sourceHtml; }
  async screenshot({path}) { require("node:fs").writeFileSync(path, "png"); }
  url() { return this.currentUrl; }
  async close() {}
}
exports.chromium = {
  async launch() {
    return {
      async newContext() {
        return {async newPage() { return new Page(); }, async route() {}, async routeWebSocket() {}, on() {}, async close() {}};
      },
      async close() {},
    };
  },
};
`,
    "utf8"
  );
  await fs.writeFile(
    inventoryPath,
    JSON.stringify({
      schema: inventory.schema,
      rows: [{
        fixture_id: "EN:partial",
        family: "EN",
        slug: "partial",
        source_url: "https://live.example/partial",
        local_https_url: "",
        required_browser: true,
      }],
    }),
    "utf8"
  );

  await assert.rejects(
    execFileAsync(process.execPath, [
      scriptPath,
      "--inventory",
      inventoryPath,
      "--output-dir",
      outputDir,
      "--browser-root",
      browserRoot,
      "--json",
    ]),
    (error) => {
      assert.match(error.stdout, /"selected_count":1/);
      return true;
    }
  );
  const records = JSON.parse(await fs.readFile(path.join(outputDir, "records.json"), "utf8"));
  const [record] = records.evidence;
  assert.equal(record.source_visible_text, "source ok");
  assert.equal(record.local_visible_text, "");
  assert.deepEqual(record.capture_errors, [{side: "local", message: "missing local URL"}]);
});
