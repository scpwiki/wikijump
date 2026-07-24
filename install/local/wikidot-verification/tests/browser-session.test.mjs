import assert from "node:assert/strict";
import path from "node:path";
import {test} from "node:test";

import {
  browserContextOptions,
  defaultBrowserRoot,
  openBrowser,
  resolveStorageStates,
} from "../src/browser-session.mjs";
import {browserRenderTestDirectory as __dirname} from "./support/browser-render-evidence-fixture.mjs";

test("default browser root is resolved from the repository, not cwd", () => {
  const originalCwd = process.cwd();
  process.chdir(__dirname);
  try {
    assert.equal(defaultBrowserRoot(), path.resolve(__dirname, "../../../..", "framerail"));
  } finally {
    process.chdir(originalCwd);
  }
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

test("browser session close reports context and browser shutdown failures", async () => {
  let closeAttempts = 0;
  const browser = {
    async newContext() {
      const ordinal = closeAttempts;
      closeAttempts += 1;
      return {
        async close() {
          throw new Error(`context ${ordinal} close failed`);
        },
      };
    },
    async close() {
      throw new Error("browser close failed");
    },
  };
  const chromium = {
    async launch() {
      return browser;
    },
  };
  const session = await openBrowser({
    chromium,
    browserExecutable: "/usr/bin/google-chrome",
    ignoreHttpsErrors: false,
  });

  await assert.rejects(
    () => session.close(),
    (error) => {
      assert(error instanceof AggregateError);
      assert.match(error.message, /browser session failed to close/);
      assert.equal(error.errors.length, 2);
      assert(error.errors[0] instanceof AggregateError);
      assert.match(error.errors[1].message, /browser close failed/);
      return true;
    },
  );
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
