import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";

import {
  browserRenderExecFile as execFileAsync,
  browserRenderInventory as inventory,
  browserRenderScriptPath as scriptPath,
} from "./support/browser-render-evidence-fixture.mjs";

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
