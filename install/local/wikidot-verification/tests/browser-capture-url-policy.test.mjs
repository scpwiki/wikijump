import assert from "node:assert/strict";
import {execFile} from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";
import {fileURLToPath} from "node:url";
import {promisify} from "node:util";

import {
  assertBrowserCaptureUrl,
  BrowserCaptureUrlPolicyError,
  createBrowserCaptureUrlPolicy,
  guardMainFrameNavigation,
} from "../src/browser-capture-url-policy.mjs";
import {capturePage} from "../scripts/capture-browser-rendering.mjs";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const scriptPath = path.resolve(
  __dirname,
  "../scripts/capture-browser-rendering.mjs",
);

test("capture policies accept intended Wikidot and Wikijump origins", () => {
  const source = createBrowserCaptureUrlPolicy(
    "source",
    "https://scp-wiki.wikidot.com/alpha",
  );
  const local = createBrowserCaptureUrlPolicy(
    "local",
    "https://scp-wiki.wikijump.localhost/alpha",
  );

  assert.equal(source.origin, "https://scp-wiki.wikidot.com");
  assert.equal(local.origin, "https://scp-wiki.wikijump.localhost");
  assert.equal(
    assertBrowserCaptureUrl(source, "https://scp-wiki.wikidot.com/beta").pathname,
    "/beta",
  );
  assert.doesNotThrow(() =>
    createBrowserCaptureUrlPolicy(
      "source",
      "http://scp-jp.wikidot.com/tag-list",
    ),
  );
  assert.doesNotThrow(() =>
    createBrowserCaptureUrlPolicy(
      "local",
      "http://scp-wiki.wikijump.localhost/alpha",
    ),
  );
});

test("capture policies reject unsafe inventory URLs", () => {
  const sourceUrls = [
    "file:///etc/passwd",
    "data:text/plain,secret",
    "http://localhost/admin",
    "http://127.0.0.1/admin",
    "http://[::1]/admin",
    "http://10.0.0.1/admin",
    "http://169.254.169.254/latest/meta-data",
    "https://metadata.google.internal/computeMetadata/v1",
    "https://evil.example/page",
    "https://scp-wiki.wikidot.com:8443/page",
    "https://user:password@scp-wiki.wikidot.com/page",
  ];
  const localUrls = [
    "file:///etc/passwd",
    "http://localhost:8080/admin",
    "http://127.0.0.1/admin",
    "http://192.168.1.10/admin",
    "http://169.254.169.254/latest/meta-data",
    "https://wikijump.localhost.evil.example/page",
    "https://scp-wiki.wikijump.localhost:8443/page",
  ];

  for (const url of sourceUrls) {
    assert.throws(
      () => createBrowserCaptureUrlPolicy("source", url),
      BrowserCaptureUrlPolicyError,
      url,
    );
  }
  for (const url of localUrls) {
    assert.throws(
      () => createBrowserCaptureUrlPolicy("local", url),
      BrowserCaptureUrlPolicyError,
      url,
    );
  }
});

test("capture policies reject a different post-redirect origin", () => {
  const source = createBrowserCaptureUrlPolicy(
    "source",
    "https://scp-wiki.wikidot.com/alpha",
  );
  assert.doesNotThrow(() =>
    assertBrowserCaptureUrl(
      source,
      "http://scp-wiki.wikidot.com/alpha",
      "final URL",
    ),
  );
  for (const url of [
    "https://other.wikidot.com/alpha",
    "http://127.0.0.1/admin",
    "http://169.254.169.254/latest/meta-data",
  ]) {
    assert.throws(
      () => assertBrowserCaptureUrl(source, url, "final URL"),
      BrowserCaptureUrlPolicyError,
      url,
    );
  }
});

function fakeRoute(url, frame, mainFrame, isNavigationRequest = true) {
  const calls = [];
  return {
    calls,
    request() {
      return {
        frame: () => frame,
        isNavigationRequest: () => isNavigationRequest,
        url: () => url,
      };
    },
    async continue() {
      calls.push("continue");
    },
    async abort(reason) {
      calls.push(["abort", reason]);
    },
    mainFrame,
  };
}

test("navigation guard blocks disallowed main-frame redirects", async () => {
  const mainFrame = {};
  let handler;
  let blockedError;
  const page = {
    mainFrame: () => mainFrame,
    async route(_pattern, routeHandler) {
      handler = routeHandler;
    },
    async unroute() {},
  };
  const policy = createBrowserCaptureUrlPolicy(
    "source",
    "https://scp-wiki.wikidot.com/alpha",
  );
  const remove = await guardMainFrameNavigation(
    page,
    policy,
    (error) => {
      blockedError = error;
    },
  );

  const sameOrigin = fakeRoute(
    "https://scp-wiki.wikidot.com/beta",
    mainFrame,
    mainFrame,
  );
  await handler(sameOrigin);
  assert.deepEqual(sameOrigin.calls, ["continue"]);

  const redirect = fakeRoute(
    "http://169.254.169.254/latest/meta-data",
    mainFrame,
    mainFrame,
  );
  await handler(redirect);
  assert.deepEqual(redirect.calls, [["abort", "blockedbyclient"]]);
  assert.ok(blockedError instanceof BrowserCaptureUrlPolicyError);

  blockedError = null;
  const subframe = fakeRoute(
    "http://127.0.0.1/admin",
    {},
    mainFrame,
  );
  await handler(subframe);
  assert.deepEqual(subframe.calls, ["continue"]);
  assert.equal(blockedError, null);
  await remove();
});

test("capturePage persists nothing after an unsafe final redirect", async () => {
  let currentUrl = "about:blank";
  let evaluations = 0;
  let contentReads = 0;
  const mainFrame = {
    url: () => currentUrl,
    async evaluate() {
      evaluations += 1;
      return "private text";
    },
  };
  const page = {
    on() {},
    mainFrame: () => mainFrame,
    async goto() {
      currentUrl = "http://169.254.169.254/latest/meta-data";
      return {status: () => 200};
    },
    async waitForLoadState() {},
    async content() {
      contentReads += 1;
      return "<html>private text</html>";
    },
    url: () => currentUrl,
  };

  const result = await capturePage(
    page,
    "https://scp-wiki.wikidot.com/alpha",
    {
      captureSide: "source",
      timeoutMs: 100,
      waitUntil: "domcontentloaded",
      settleMs: 0,
      screenshotPath: null,
    },
  );

  assert.match(result.error, /final URL origin is not allowlisted/u);
  assert.equal(result.finalUrl, null);
  assert.equal(result.visibleText, "");
  assert.equal(result.html, "");
  assert.equal(evaluations, 0);
  assert.equal(contentReads, 0);
});

test("all-frame capture rechecks origin after a frame visibility probe", async () => {
  const pageUrl = "https://scp-wiki.wikijump.localhost/page";
  const mainFrame = {url: () => pageUrl, evaluate: async () => "main text"};
  let childUrl = pageUrl;
  let childEvaluations = 0;
  const childFrame = {
    url: () => childUrl,
    async frameElement() {
      return {
        async evaluate() {
          childUrl = "http://127.0.0.1/admin";
          return true;
        },
      };
    },
    async evaluate() {
      childEvaluations += 1;
      return "private text";
    },
  };
  const page = {
    on() {},
    mainFrame: () => mainFrame,
    goto: async () => ({status: () => 200}),
    waitForLoadState: async () => {},
    frames: () => [mainFrame, childFrame],
    content: async () => "<html>main text</html>",
    url: () => pageUrl,
  };

  const result = await capturePage(page, pageUrl, {
    captureSide: "local",
    timeoutMs: 100,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
    visibleTextScope: "all-frames",
  });

  assert.equal(result.visibleText, "main text");
  assert.equal(childEvaluations, 0);
});

test("all-frame capture preserves inherited-origin srcdoc text", async () => {
  const pageUrl = "https://scp-wiki.wikijump.localhost/page";
  const mainFrame = {url: () => pageUrl, evaluate: async () => "main text"};
  const childFrame = {
    url: () => "about:srcdoc",
    parentFrame: () => mainFrame,
    frameElement: async () => ({evaluate: async () => true}),
    evaluate: async () => "srcdoc text",
  };
  const page = {
    on() {},
    mainFrame: () => mainFrame,
    goto: async () => ({status: () => 200}),
    waitForLoadState: async () => {},
    frames: () => [mainFrame, childFrame],
    content: async () => "<html>main text</html>",
    url: () => pageUrl,
  };

  const result = await capturePage(page, pageUrl, {
    captureSide: "local",
    timeoutMs: 100,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
    visibleTextScope: "all-frames",
  });

  assert.equal(result.visibleText, "main text\nsrcdoc text");
});

test("CLI rejects private inventory URLs before loading Playwright", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wikijump-browser-url-policy-"),
  );
  const outputDir = path.join(root, "out");
  for (const [field, value, message] of [
    ["source_url", "http://169.254.169.254/latest/meta-data", /source URL host/u],
    ["local_https_url", "http://127.0.0.1/admin", /local URL host/u],
  ]) {
    const row = {
      fixture_id: `EN:${field}`,
      source_url: "https://scp-wiki.wikidot.com/alpha",
      local_https_url: "https://scp-wiki.wikijump.localhost/alpha",
      [field]: value,
    };
    const inventoryPath = path.join(root, `${field}.json`);
    await fs.writeFile(
      inventoryPath,
      JSON.stringify({
        schema: "wikijump_full_parity.corpus_inventory_lock.v1",
        rows: [row],
      }),
      "utf8",
    );
    await assert.rejects(
      execFileAsync(process.execPath, [
        scriptPath,
        "--inventory",
        inventoryPath,
        "--output-dir",
        outputDir,
        "--browser-root",
        path.join(root, "missing-browser-root"),
        "--json",
      ]),
      (error) => {
        assert.match(error.stderr, message);
        assert.doesNotMatch(error.stderr, /could not load playwright/u);
        return true;
      },
    );
  }
});
