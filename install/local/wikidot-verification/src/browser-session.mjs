import {createRequire} from "node:module";
import path from "node:path";
import {fileURLToPath} from "node:url";

import {
  createBrowserResponseCache,
  installBrowserRequestGate,
} from "./browser-request-gate.mjs";

const MODULE_DIR = path.dirname(fileURLToPath(import.meta.url));

export function defaultBrowserRoot() {
  return path.resolve(MODULE_DIR, "../../../..", "framerail");
}

export function loadPlaywright(browserRoot) {
  const root = browserRoot ?? defaultBrowserRoot();
  const requireFromRoot = createRequire(path.join(root, "package.json"));
  try {
    return requireFromRoot("playwright");
  } catch (error) {
    try {
      return requireFromRoot("@playwright/test");
    } catch (fallbackError) {
      throw new AggregateError(
        [error, fallbackError],
        `could not load playwright or @playwright/test from ${root}; pass --browser-root pointing at a package with Playwright installed`,
      );
    }
  }
}

export function resolveStorageStates({storageState = null, sourceStorageState = null, localStorageState = null}) {
  return {
    sourceStorageState: sourceStorageState ?? storageState ?? null,
    localStorageState: localStorageState ?? storageState ?? null,
  };
}

export function browserContextOptions({ignoreHttpsErrors, storageState = null, proxyServer = null, blockServiceWorkers = false}) {
  return {
    ignoreHTTPSErrors: ignoreHttpsErrors,
    ...(blockServiceWorkers ? {serviceWorkers: "block"} : {}),
    ...(storageState ? {storageState} : {}),
    ...(proxyServer ? {proxy: {server: proxyServer, bypass: "<-loopback>"}} : {}),
  };
}

async function newContextPair({browser, ignoreHttpsErrors, sourceStorageState, localStorageState, sourceProxyServer, localProxyServer, requestGate = null, localOrigins = []}) {
  let sourceContext = null;
  let localContext = null;
  const sourceResponseCache = requestGate ? createBrowserResponseCache() : null;
  try {
    sourceContext = await browser.newContext(
      browserContextOptions({
        ignoreHttpsErrors,
        storageState: sourceStorageState,
        proxyServer: sourceProxyServer,
        blockServiceWorkers: Boolean(requestGate),
      }),
    );
    if (requestGate) await installBrowserRequestGate(sourceContext, {gate: requestGate, responseCache: sourceResponseCache});
    localContext = await browser.newContext(
      browserContextOptions({
        ignoreHttpsErrors,
        storageState: localStorageState,
        proxyServer: localProxyServer,
        blockServiceWorkers: Boolean(requestGate),
      }),
    );
    if (requestGate) await installBrowserRequestGate(localContext, {gate: requestGate, exemptOrigins: localOrigins});
    return {sourceContext, localContext, sourceResponseCache};
  } catch (error) {
    if (localContext && localContext !== sourceContext) {
      await localContext.close().catch(() => {});
    }
    if (sourceContext) {
      await sourceContext.close().catch(() => {});
    }
    throw error;
  }
}

async function closeContextPair({sourceContext, localContext}) {
  const failures = [];
  if (localContext && localContext !== sourceContext) {
    await localContext.close().catch((error) => failures.push(error));
  }
  if (sourceContext) {
    await sourceContext.close().catch((error) => failures.push(error));
  }
  if (failures.length > 0) {
    throw new AggregateError(failures, "browser contexts failed to close");
  }
}

function browserSession({browser, sourceContext, localContext, sourceResponseCache, ignoreHttpsErrors, sourceStorageState, localStorageState, sourceProxyServer, localProxyServer, requestGate, localOrigins}) {
  return {
    browser,
    context: sourceContext,
    sourceContext,
    localContext,
    sourceResponseCache,
    async newContextPair() {
      return await newContextPair({
        browser,
        ignoreHttpsErrors,
        sourceStorageState,
        localStorageState,
        sourceProxyServer,
        localProxyServer,
        requestGate,
        localOrigins,
      });
    },
    async close() {
      let contextError = null;
      try {
        await closeContextPair({sourceContext, localContext});
      } catch (error) {
        contextError = error;
      }
      let browserError = null;
      try {
        await browser.close();
      } catch (error) {
        browserError = error;
      }
      if (contextError !== null && browserError !== null) {
        throw new AggregateError([contextError, browserError], "browser session failed to close");
      }
      if (contextError !== null) throw contextError;
      if (browserError !== null) throw browserError;
    },
  };
}

export async function openBrowser({
  chromium,
  cdpEndpoint,
  browserExecutable,
  ignoreHttpsErrors,
  storageState = null,
  sourceStorageState = null,
  localStorageState = null,
  createInitialContexts = true,
  sourceProxyServer = null,
  localProxyServer = null,
  requestGate = null,
  localOrigins = [],
}) {
  const resolvedStates = resolveStorageStates({storageState, sourceStorageState, localStorageState});
  let browser = null;
  if (cdpEndpoint) {
    if (sourceProxyServer || localProxyServer) throw new Error("CDP capture cannot enforce the owned egress proxy");
    browser = await chromium.connectOverCDP(cdpEndpoint);
  } else {
    browser = await chromium.launch({
      executablePath: browserExecutable,
    });
  }

  try {
    const contexts = createInitialContexts
      ? await newContextPair({
          browser,
          ignoreHttpsErrors,
          sourceStorageState: resolvedStates.sourceStorageState,
          localStorageState: resolvedStates.localStorageState,
          sourceProxyServer,
          localProxyServer,
          requestGate,
          localOrigins,
        })
      : {sourceContext: null, localContext: null};
    return browserSession({
      browser,
      ...contexts,
      ignoreHttpsErrors,
      sourceStorageState: resolvedStates.sourceStorageState,
      localStorageState: resolvedStates.localStorageState,
      sourceProxyServer,
      localProxyServer,
      requestGate,
      localOrigins,
    });
  } catch (error) {
    if (browser) {
      await browser.close().catch(() => {});
    }
    throw error;
  }
}
