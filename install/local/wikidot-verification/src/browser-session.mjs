import {createRequire} from "node:module";
import path from "node:path";
import {fileURLToPath} from "node:url";

import {
  createBrowserResponseCache,
  defaultPublicEvidenceResponseCacheOptions,
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

async function newContextPair({browser, ignoreHttpsErrors, sourceStorageState, localStorageState, sourceProxyServer, localProxyServer, requestGate = null, localOrigins = [], sourceResponseCache = null, sourceResponseCacheOptions = null}) {
  let sourceContext = null;
  let localContext = null;
  const responseCache = requestGate
    ? (sourceResponseCache ?? createBrowserResponseCache(sourceResponseCacheOptions ?? defaultPublicEvidenceResponseCacheOptions()))
    : null;
  try {
    await responseCache?.load();
    sourceContext = await browser.newContext(
      browserContextOptions({
        ignoreHttpsErrors,
        storageState: sourceStorageState,
        proxyServer: sourceProxyServer,
        blockServiceWorkers: Boolean(requestGate),
      }),
    );
    if (requestGate) await installBrowserRequestGate(sourceContext, {gate: requestGate, responseCache});
    localContext = await browser.newContext(
      browserContextOptions({
        ignoreHttpsErrors,
        storageState: localStorageState,
        proxyServer: localProxyServer,
        blockServiceWorkers: Boolean(requestGate),
      }),
    );
    if (requestGate) await installBrowserRequestGate(localContext, {
      gate: requestGate,
      exemptOrigins: localOrigins,
      responseCache,
      cacheOnly: true,
    });
    return {sourceContext, localContext, sourceResponseCache: responseCache};
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

function browserSession({browser, sourceContext, localContext, sourceResponseCache, ignoreHttpsErrors, sourceStorageState, localStorageState, sourceProxyServer, localProxyServer, requestGate, localOrigins, sourceResponseCacheOptions}) {
  const responseCaches = new Set(sourceResponseCache ? [sourceResponseCache] : []);
  return {
    browser,
    context: sourceContext,
    sourceContext,
    localContext,
    sourceResponseCache,
    async newContextPair() {
      const pair = await newContextPair({
        browser,
        ignoreHttpsErrors,
        sourceStorageState,
        localStorageState,
        sourceProxyServer,
        localProxyServer,
        requestGate,
        localOrigins,
        sourceResponseCache,
        sourceResponseCacheOptions,
      });
      return pair;
    },
    async close() {
      let contextError = null;
      try {
        await closeContextPair({sourceContext, localContext});
      } catch (error) {
        contextError = error;
      }
      let cacheError = null;
      try {
        await Promise.all([...responseCaches].map((cache) => cache.flush()));
      } catch (error) {
        cacheError = error;
      }
      let browserError = null;
      try {
        await browser.close();
      } catch (error) {
        browserError = error;
      }
      if (cacheError !== null && browserError !== null) {
        browserError = new AggregateError([cacheError, browserError], "browser session failed to close");
      } else if (cacheError !== null) {
        browserError = cacheError;
      }
      if (contextError !== null && browserError !== null) {
        throw new AggregateError([contextError, browserError], "browser session failed to close");
      }
      if (contextError !== null) throw contextError;
      if (browserError !== null) throw browserError;
    },
  };
}

/**
 * Firefox launch options for a retained-evidence replay.
 *
 * Firefox upgrades an insecure subresource through an internal redirect that Playwright's
 * route interception never observes, so an evidence replay would never be asked for the
 * page's `http://` subresources and would silently fetch them from the network instead.
 * Disabling that upgrade makes the browser issue the request as written, which keeps every
 * request observable and therefore serveable from retained evidence. Chromium and WebKit do
 * not need this: they either rewrite the subresource URL before requesting it or keep a
 * request the gate can route.
 */
export function firefoxEvidenceReplayLaunchOptions(executable) {
  if (typeof executable !== "string" || executable === "") {
    throw new Error("firefox evidence replay launch options require an executable path");
  }
  return {
    executablePath: executable,
    firefoxUserPrefs: {
      "dom.security.https_only_mode": false,
      "dom.security.https_only_mode_pbm": false,
      "dom.security.https_only_mode_ever_enabled": false,
      "security.mixed_content.upgrade_display_content": false,
    },
  };
}

export async function openBrowser({
  chromium,
  cdpEndpoint,
  browserExecutable,
  headless,
  browserEnvironment = null,
  browserArgs = [],
  ignoreHttpsErrors,
  storageState = null,
  sourceStorageState = null,
  localStorageState = null,
  createInitialContexts = true,
  sourceProxyServer = null,
  localProxyServer = null,
  requestGate = null,
  localOrigins = [],
  sourceResponseCacheOptions = null,
}) {
  const resolvedStates = resolveStorageStates({storageState, sourceStorageState, localStorageState});
  let browser = null;
  if (cdpEndpoint) {
    if (sourceProxyServer || localProxyServer) throw new Error("CDP capture cannot enforce the owned egress proxy");
    browser = await chromium.connectOverCDP(cdpEndpoint);
  } else {
    const launchOptions = {executablePath: browserExecutable};
    if (headless !== undefined) launchOptions.headless = headless;
    if (browserEnvironment !== null) launchOptions.env = browserEnvironment;
    if (browserArgs.length > 0) launchOptions.args = browserArgs;
    browser = await chromium.launch(launchOptions);
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
          sourceResponseCacheOptions,
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
      sourceResponseCacheOptions,
    });
  } catch (error) {
    if (browser) {
      await browser.close().catch(() => {});
    }
    throw error;
  }
}
