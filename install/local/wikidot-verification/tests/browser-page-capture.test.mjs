import assert from "node:assert/strict";
import {test} from "node:test";

import {applyReadOnlyCaptureAction, capturePage} from "../scripts/capture-browser-rendering.mjs";

test("source-state capture actions are restricted to non-mutating theme controls", async () => {
  const calls = [];
  const page = {
    locator(selector) {
      const locator = {
        first() { return this; },
        nth() { return this; },
        async count() { return 1; },
        async isVisible() { return true; },
        async isEnabled() { return true; },
        async click(options) { calls.push({kind: "click", selector, options}); },
        async focus(options) { calls.push({kind: "focus", selector, options}); },
        async hover(options) { calls.push({kind: "hover", selector, options}); },
        async fill(value) { calls.push({kind: "fill", selector, value}); },
      };
      return locator;
    },
    async evaluate(fn, value) { calls.push({kind: "evaluate", value}); },
  };

  await applyReadOnlyCaptureAction(page, {kind: "click", selector: "#more-options-button"});
  await applyReadOnlyCaptureAction(page, {kind: "focus", selector: ".page-rate-widget-box a"});
  await applyReadOnlyCaptureAction(page, {kind: "fill-search", selector: "#search-top-box-input", value: "SCP test"});
  await applyReadOnlyCaptureAction(page, {kind: "set-hash", value: "#u-credit-view"});
  await assert.rejects(applyReadOnlyCaptureAction(page, {kind: "click", selector: "#delete-button"}), /not an approved read-only control/u);
  await assert.rejects(applyReadOnlyCaptureAction(page, {kind: "fill-search", selector: "#search-top-box-input", value: "line one\nline two"}), /limited to a short/u);
  assert.deepEqual(calls.map((row) => row.kind), ["click", "focus", "evaluate", "evaluate"]);
  assert.deepEqual(calls[2].value, {selector: "#search-top-box-input", value: "SCP test"});
  assert.equal(calls[3].value, "#u-credit-view");
});

test("capturePage records page errors and failed subframe responses", async () => {
  const handlers = new Map();
  const mainFrame = {name: "main", async evaluate() { return "visible"; }};
  const childFrame = {name: "child", async evaluate() { return "sensitive child frame text"; }};
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
      return [mainFrame, childFrame];
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
  assert.equal(result.visibleText, "visible");
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

test("capturePage does not collect visible text from child frames", async () => {
  const mainFrame = {async evaluate() { return "main frame text"; }};
  const childFrame = {async evaluate() { return "sensitive iframe text"; }};
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
  });

  assert.equal(result.visibleText, "main frame text");
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

test("capturePage delegates non-excluded requests to context request gates", async () => {
  let pageRouteHandler;
  const mainFrame = {async evaluate() { return "visible"; }};
  const page = {
    on() {},
    mainFrame() { return mainFrame; },
    route(_pattern, handler) { pageRouteHandler = handler; },
    async goto() { return {status: () => 200}; },
    async waitForLoadState() {},
    frames() { return [mainFrame]; },
    async content() { return "<html>visible</html>"; },
    url() { return "https://scp-wiki.wikidot.com/scp-9506"; },
  };

  await capturePage(page, "https://scp-wiki.wikidot.com/scp-9506", {
    timeoutMs: 100,
    waitUntil: "domcontentloaded",
    settleMs: 0,
    screenshotPath: null,
    blockedRequestPrefixes: ["https://stats.g.doubleclick.net/r/"],
  });

  const delegated = [];
  const aborted = [];
  await pageRouteHandler({
    request() { return {url: () => "https://scp-wiki.wikidot.com/scp-9506"}; },
    async fallback() { delegated.push("https://scp-wiki.wikidot.com/scp-9506"); },
    async abort() { assert.fail("the Wikidot page must not be excluded"); },
  });
  await pageRouteHandler({
    request() { return {url: () => "https://stats.g.doubleclick.net/r/track.gif"}; },
    async fallback() { assert.fail("excluded ad requests must not reach the context gate"); },
    async abort(reason) { aborted.push(reason); },
  });

  assert.deepEqual(delegated, ["https://scp-wiki.wikidot.com/scp-9506"]);
  assert.deepEqual(aborted, ["blockedbyclient"]);
});
