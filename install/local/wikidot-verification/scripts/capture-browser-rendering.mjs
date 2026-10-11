#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";
import {startCaptureEgressProxy} from "../src/capture-egress-proxy.mjs";
import {
  DEFAULT_REQUEST_INTERVAL_MS,
  acquireBrowserCaptureLock,
  createPersistentBrowserRequestGate,
  defaultPublicEvidenceResponseCacheOptions,
  localBrowserCaptureOrigins,
} from "../src/browser-request-gate.mjs";
import {
  browserContextOptions,
  defaultBrowserRoot,
  loadPlaywright,
  openBrowser,
  resolveStorageStates,
} from "../src/browser-session.mjs";
import {
  buildEvidenceRecord,
  compactVisibleText,
  readJson,
  inventoryRows,
  rowLocalUrl,
  rowSourceUrl,
  safePathSegment,
  selectInventoryRows,
  writeEvidenceArtifacts,
} from "../src/browser-render-evidence.mjs";

const DEFAULT_TIMEOUT_MS = 120_000;
const DEFAULT_SETTLE_MS = 1_000;
const POST_NAVIGATION_STATE_TIMEOUT_MS = 2_000;
const VISIBLE_TEXT_SCOPES = new Set(["main-frame"]);
const SCRIPT_PATH = fileURLToPath(import.meta.url);

function nextArg(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function parseArgs(argv) {
  const args = {
    fixtureIds: [],
    timeoutMs: DEFAULT_TIMEOUT_MS,
    settleMs: DEFAULT_SETTLE_MS,
    localUrlField: "local_https_url",
    screenshot: true,
    ignoreHttpsErrors: false,
    waitUntil: "domcontentloaded",
    visibleTextScope: "main-frame",
    sourceResponseCacheDir: null,
    sourceResponseCacheIdentity: null,
    sourceResponseCacheDocuments: false,
    sourceOnly: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--inventory") {
      args.inventory = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--shard-manifest") {
      args.shardManifest = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--shard-id") {
      args.shardId = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--fixture-id") {
      args.fixtureIds.push(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--limit") {
      const raw = nextArg(argv, index, arg);
      if (!/^\d+$/u.test(raw) || Number.parseInt(raw, 10) <= 0) {
        throw new Error("--limit must be a positive integer");
      }
      args.limit = Number.parseInt(raw, 10);
      index += 1;
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--local-url-field") {
      args.localUrlField = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--browser-root") {
      args.browserRoot = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--browser-executable") {
      args.browserExecutable = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--cdp-endpoint") {
      args.cdpEndpoint = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--storage-state") {
      args.storageState = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--source-storage-state") {
      args.sourceStorageState = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--local-storage-state") {
      args.localStorageState = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--source-response-cache-dir") {
      args.sourceResponseCacheDir = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--source-response-cache-identity") {
      args.sourceResponseCacheIdentity = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--cache-source-documents") {
      args.sourceResponseCacheDocuments = true;
    } else if (arg === "--source-only") {
      args.sourceOnly = true;
    } else if (arg === "--actor-label") {
      args.actorLabel = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--timeout-ms") {
      const raw = nextArg(argv, index, arg);
      if (!/^\d+$/u.test(raw) || Number.parseInt(raw, 10) <= 0) {
        throw new Error("--timeout-ms must be a positive integer");
      }
      args.timeoutMs = Number.parseInt(raw, 10);
      index += 1;
    } else if (arg === "--settle-ms") {
      const raw = nextArg(argv, index, arg);
      if (!/^\d+$/u.test(raw)) {
        throw new Error("--settle-ms must be a non-negative integer");
      }
      args.settleMs = Number.parseInt(raw, 10);
      index += 1;
    } else if (arg === "--wait-until") {
      args.waitUntil = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--visible-text-scope") {
      args.visibleTextScope = nextArg(argv, index, arg);
      if (!VISIBLE_TEXT_SCOPES.has(args.visibleTextScope)) {
        throw new Error("--visible-text-scope must be main-frame");
      }
      index += 1;
    } else if (arg === "--ignore-https-errors") {
      args.ignoreHttpsErrors = true;
    } else if (arg === "--no-screenshot") {
      args.screenshot = false;
    } else if (arg === "--json") {
      args.jsonOnly = true;
    } else if (arg === "--help") {
      return {help: true};
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.inventory) throw new Error("--inventory is required");
  if (!args.outputDir) throw new Error("--output-dir is required");
  if (args.sourceResponseCacheDir !== null && args.sourceResponseCacheIdentity === null) throw new Error("--source-response-cache-identity is required with --source-response-cache-dir");
  if (args.sourceResponseCacheDir === null && args.sourceResponseCacheIdentity !== null) throw new Error("--source-response-cache-dir is required with --source-response-cache-identity");
  if (args.sourceResponseCacheDir === null) {
    const defaults = defaultPublicEvidenceResponseCacheOptions();
    args.sourceResponseCacheDir = defaults.persistentDir;
    args.sourceResponseCacheIdentity = defaults.persistentIdentity;
    args.sourceResponseCacheDocuments = true;
  } else if (!args.sourceResponseCacheDocuments) {
    throw new Error("explicit browser evidence caches require --cache-source-documents so reruns do not reacquire source pages");
  }
  return args;
}

function printHelp() {
  console.log(`Usage: capture-browser-rendering.mjs --inventory FILE --output-dir DIR [--shard-manifest FILE --shard-id ID] [--fixture-id ID ...] [--limit N] [--browser-root framerail] [--browser-executable /usr/bin/google-chrome | --cdp-endpoint http://127.0.0.1:9222] [--storage-state FILE | --source-storage-state FILE --local-storage-state FILE] [--source-response-cache-dir DIR --source-response-cache-identity ID --cache-source-documents] [--actor-label LABEL] [--local-url-field local_https_url] [--source-only] [--timeout-ms 120000] [--settle-ms 1000] [--visible-text-scope main-frame] [--ignore-https-errors] [--no-screenshot] [--json]

Writes validator-compatible browser rendering evidence JSON plus DOM/screenshot artifacts for selected corpus inventory rows. The output directory should live under one of the render validator evidence roots, for example:

  $OUT/validation/browser-rendering/en-0001

Anonymous source responses use a persistent evidence-replay cache under XDG_CACHE_HOME (or ~/.cache) by default, including source documents. Reuse the default identity for retries. Supply all three source-response-cache options only when intentionally selecting a different acquisition identity.
`);
}

// The CLI used to log only AggregateError.message, hiding both independent
// reasons when capture and cleanup fail. Keep them separately attributable.
// Avoid exposing URL query strings or access tokens in diagnostics.
export function browserCaptureFailureDiagnostic(error) {
  const redact = (value) => String(value)
    .replace(/(https?:\/\/[^\s?#]+)(?:[?#][^\s]*)?/giu, "$1?[redacted]")
    .replace(/(token|password|secret|authorization|cookie)\s*[:=]\s*[^\s,;]+/giu, "$1=[redacted]");
  const message = (value) => redact(value instanceof Error ? value.message : value);
  if (!(error instanceof AggregateError)) return message(error);
  return [
    message(error),
    ...error.errors.map((cause, index) =>
      `${index === 0 ? "capture" : index === 1 ? "cleanup" : `cause ${index + 1}`}: ${message(cause)}`),
  ].join("\n");
}

export function browserCaptureFailure(captureError, cleanupError) {
  if (captureError !== null && cleanupError !== null) {
    return new AggregateError([captureError, cleanupError], "browser capture and cleanup both failed");
  }
  return captureError ?? cleanupError;
}

export {
  browserContextOptions,
  defaultBrowserRoot,
  openBrowser,
  resolveStorageStates,
};

async function collectVisibleText(page) {
  const frame = typeof page.mainFrame === "function" ? page.mainFrame() : page;
  try {
    return await frame.evaluate(() => document.body?.innerText ?? "");
  } catch (error) {
    void error;
    // Detached or inaccessible main frames should not abort page-level capture.
    return "";
  }
}

async function waitForLoadStateWithinBudget(page, state, timeoutMs, startedAt) {
  const remainingMs = timeoutMs - (Date.now() - startedAt);
  if (remainingMs <= 0) return;
  await page.waitForLoadState(state, {timeout: Math.min(POST_NAVIGATION_STATE_TIMEOUT_MS, remainingMs)}).catch(() => {});
}

export async function capturePage(page, url, {
  timeoutMs,
  waitUntil,
  settleMs = DEFAULT_SETTLE_MS,
  screenshotPath,
  visibleTextScope = "main-frame",
  captureStates = [],
  stateArtifactsDir = null,
  blockedRequestPrefixes = [],
  stateScreenshots = true,
}) {
  if (!VISIBLE_TEXT_SCOPES.has(visibleTextScope)) {
    throw new Error("visibleTextScope must be main-frame because cross-origin frame text is not captured");
  }
  validateBlockedRequestPrefixes(blockedRequestPrefixes);
  const consoleErrors = [];
  const failedRequests = [];
  const badResponses = [];
  const blockedRequests = [];
  let sawInitialMainFrameNavigationResponse = false;
  if (blockedRequestPrefixes.length) {
    await page.route("**/*", async (route) => {
      const requestUrl = route.request().url();
      if (blockedRequestPrefixes.some((prefix) => requestUrl.startsWith(prefix))) {
        blockedRequests.push({url: requestUrl, disposition: "task_scoped_source_asset_exclusion"});
        await route.abort("blockedbyclient");
        return;
      }
      await route.continue();
    });
  }
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });
  page.on("pageerror", (error) => {
    consoleErrors.push(error.message ?? String(error));
  });
  page.on("requestfailed", (request) => {
    failedRequests.push({
      url: request.url(),
      failure: request.failure()?.errorText ?? "unknown",
    });
  });
  page.on("response", (response) => {
    const request = response.request();
    let frame = null;
    try {
      frame = request.frame();
    } catch (error) {
      void error;
      // Some request kinds do not have a frame; keep their HTTP failure evidence.
    }
    const isMainFrameNavigation = request.isNavigationRequest() && frame === page.mainFrame();
    if (isMainFrameNavigation && !sawInitialMainFrameNavigationResponse) {
      sawInitialMainFrameNavigationResponse = true;
      return;
    }

    const status = response.status();
    if (status < 400) return;
    badResponses.push({
      url: response.url(),
      status,
      resourceType: request.resourceType(),
    });
  });

  let response = null;
  let navigationError = null;
  let visibleText = "";
  let html = "";
  let writtenScreenshotPath = null;
  const startedAt = Date.now();
  try {
    response = await page.goto(url, {timeout: timeoutMs, waitUntil});
  } catch (error) {
    navigationError = error;
  }

  try {
    await waitForLoadStateWithinBudget(page, "domcontentloaded", timeoutMs, startedAt);
    await waitForLoadStateWithinBudget(page, "load", timeoutMs, startedAt);
    if (settleMs > 0 && typeof page.waitForTimeout === "function") {
      await page.waitForTimeout(settleMs).catch(() => {});
    }
    visibleText = await collectVisibleText(page);
    html = await page.content();
  } catch (error) {
    if (!navigationError) navigationError = error;
  }

  if (screenshotPath && html) {
    try {
      const remainingMs = Math.max(1, timeoutMs - (Date.now() - startedAt));
      await page.screenshot({path: screenshotPath, fullPage: true, timeout: remainingMs});
      writtenScreenshotPath = screenshotPath;
    } catch (error) {
      if (!navigationError) navigationError = error;
    }
  }

  let sourceStates = [];
  if (captureStates.length) {
    if (!stateArtifactsDir) throw new Error("stateArtifactsDir is required for source state captures");
    await fs.mkdir(stateArtifactsDir, {recursive: true, mode: 0o700});
    sourceStates = await captureReadOnlyStates(page, url, captureStates, {
      timeoutMs,
      waitUntil,
      settleMs,
      screenshot: stateScreenshots,
      artifactsDir: stateArtifactsDir,
    });
  }

  if (!navigationError) {
    return {
      status: response?.status() ?? null,
      finalUrl: page.url(),
      visibleText,
      html,
      consoleErrors,
      failedRequests: [...failedRequests, ...badResponses],
      blockedRequests,
      states: sourceStates,
      screenshotPath: writtenScreenshotPath,
    };
  }

  return {
    status: response?.status() ?? null,
    finalUrl: page.url(),
    visibleText,
    html,
    consoleErrors,
    failedRequests: [...failedRequests, ...badResponses],
    blockedRequests,
    states: sourceStates,
    screenshotPath: writtenScreenshotPath,
    error: navigationError.message,
  };
}

const READ_ONLY_CAPTURE_CONTROLS = new Set([
  "#more-options-button",
  ".mobile-top-bar .open-menu a",
  ".yui-navset .yui-nav li a",
  ".yui-navset .yui-nav li:nth-child(2) a",
  ".collapsible-block .collapsible-block-link",
  "a[href=\"#u-credit-view\"]",
  "a[href=\"#u-credit-otherwise\"]",
]);
const READ_ONLY_CAPTURE_FOCUS_TARGETS = new Set([
  "#search-top-box-input",
  ".page-rate-widget-box a",
  ".mobile-top-bar .open-menu a",
  "#top-bar a",
  ".top-bar a",
]);
const READ_ONLY_CAPTURE_HOVER_TARGETS = new Set(["#top-bar", "#top-bar > ul > li:first-child", ".top-bar > ul > li:first-child"]);
const READ_ONLY_CAPTURE_HASHES = new Set(["#side-bar", "#u-credit-view", "#u-credit-otherwise"]);

export async function applyReadOnlyCaptureAction(page, action) {
  if (!action || typeof action !== "object" || typeof action.kind !== "string") throw new Error("source state action requires a kind");
  if (action.kind === "click") {
    if (!READ_ONLY_CAPTURE_CONTROLS.has(action.selector)) throw new Error(`source capture click is not an approved read-only control: ${action.selector}`);
    const matches = page.locator(action.selector);
    const count = await matches.count();
    for (let index = 0; index < count; index += 1) {
      const candidate = matches.nth(index);
      if (!await candidate.isVisible() || !await candidate.isEnabled()) continue;
      await candidate.click({timeout: 5_000, noWaitAfter: true});
      return {target_index: index};
    }
    throw new Error(`source capture found no visible, enabled control for ${action.selector}`);
  } else if (action.kind === "focus") {
    if (!READ_ONLY_CAPTURE_FOCUS_TARGETS.has(action.selector)) throw new Error(`source capture focus target is not approved: ${action.selector}`);
    await page.locator(action.selector).first().focus({timeout: 5_000});
  } else if (action.kind === "hover") {
    if (!READ_ONLY_CAPTURE_HOVER_TARGETS.has(action.selector)) throw new Error(`source capture hover target is not approved: ${action.selector}`);
    await page.locator(action.selector).first().hover({timeout: 5_000});
  } else if (action.kind === "fill-search") {
    if (action.selector !== "#search-top-box-input" || typeof action.value !== "string" || action.value.length > 256 || /[\r\n]/u.test(action.value)) {
      throw new Error("source capture fill is limited to a short, single-line search field value");
    }
    await page.evaluate(({selector, value}) => {
      const element = document.querySelector(selector);
      if (!(element instanceof HTMLInputElement)) throw new Error(`search input is missing: ${selector}`);
      element.focus();
      element.value = value;
      element.dispatchEvent(new Event("input", {bubbles: true}));
    }, {selector: action.selector, value: action.value});
  } else if (action.kind === "set-hash") {
    if (!READ_ONLY_CAPTURE_HASHES.has(action.value)) throw new Error(`source capture hash target is not approved: ${action.value}`);
    await page.evaluate((value) => { location.hash = value; }, action.value);
  } else if (action.kind === "scroll") {
    if (action.value !== "top" && action.value !== "bottom") throw new Error("source capture scroll value must be top or bottom");
    await page.evaluate((value) => window.scrollTo(0, value === "bottom" ? document.documentElement.scrollHeight : 0), action.value);
  } else {
    throw new Error(`unsupported read-only source state action: ${action.kind}`);
  }
}

function validateBlockedRequestPrefixes(prefixes) {
  if (!Array.isArray(prefixes)) throw new Error("blocked source request prefixes must be an array");
  for (const prefix of prefixes) {
    if (typeof prefix !== "string") throw new Error("blocked source request prefix must be a string");
    const url = new URL(prefix);
    if (!new Set(["http:", "https:"]).has(url.protocol) || url.username || url.password || url.search || url.hash || url.pathname === "/") {
      throw new Error(`blocked source request prefix must be an exact HTTP(S) host/path prefix: ${prefix}`);
    }
  }
}

async function captureReadOnlyStates(page, url, states, {timeoutMs, waitUntil, settleMs, screenshot, artifactsDir}) {
  if (!Array.isArray(states) || states.length === 0) return [];
  const records = [];
  for (const state of states) {
    if (!state || typeof state.state_id !== "string" || !/^[a-z0-9][a-z0-9._-]*$/u.test(state.state_id)) {
      throw new Error("source state requires a stable lowercase state_id");
    }
    if (!Array.isArray(state.actions)) throw new Error(`source state ${state.state_id} requires an actions array`);
    const startedAt = Date.now();
    const actionResults = [];
    let error = null;
    let response = null;
    try {
      if (state.viewport) {
        if (!Number.isSafeInteger(state.viewport.width) || !Number.isSafeInteger(state.viewport.height) || state.viewport.width < 200 || state.viewport.height < 200) {
          throw new Error(`source state ${state.state_id} has an invalid viewport`);
        }
        await page.setViewportSize(state.viewport);
      }
      response = await page.goto(url, {timeout: timeoutMs, waitUntil});
      await waitForLoadStateWithinBudget(page, "domcontentloaded", timeoutMs, startedAt);
      await waitForLoadStateWithinBudget(page, "load", timeoutMs, startedAt);
      for (const action of state.actions) {
        const observation = await applyReadOnlyCaptureAction(page, action);
        actionResults.push({kind: action.kind, selector: action.selector ?? null, value: action.value ?? null, status: "applied", ...(observation ?? {})});
      }
      if (settleMs > 0) await page.waitForTimeout(settleMs).catch(() => {});
    } catch (cause) {
      error = cause?.message ?? String(cause);
      actionResults.push({status: "failed", error});
    }

    let html = "";
    let visibleText = "";
    try {
      html = await page.content();
      visibleText = await collectVisibleText(page);
    } catch (cause) {
      error ??= cause?.message ?? String(cause);
    }
    const stateSlug = safePathSegment(state.state_id);
    const htmlPath = path.join(artifactsDir, `${stateSlug}.dom.html`);
    await fs.writeFile(htmlPath, html, {encoding: "utf8", flag: "wx", mode: 0o600});
    let screenshotPath = null;
    if (screenshot && html) {
      screenshotPath = path.join(artifactsDir, `${stateSlug}.png`);
      try {
        await page.screenshot({path: screenshotPath, fullPage: true, timeout: Math.max(1, timeoutMs - (Date.now() - startedAt))});
      } catch (cause) {
        error ??= cause?.message ?? String(cause);
        screenshotPath = null;
      }
    }
    records.push({
      state_id: state.state_id,
      viewport: state.viewport ?? null,
      source_status: response?.status() ?? null,
      source_final_url: page.url(),
      source_visible_text: compactVisibleText(visibleText),
      source_dom_artifact: htmlPath,
      source_dom_sha256: crypto.createHash("sha256").update(html, "utf8").digest("hex"),
      source_screenshot_artifact: screenshotPath,
      source_screenshot_sha256: screenshotPath ? await hashFile(screenshotPath) : null,
      actions: actionResults,
      capture_error: error,
    });
  }
  return records;
}

async function hashFile(filePath) {
  const bytes = await fs.readFile(filePath);
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

async function captureOptionalPage(context, url, missingMessage, options) {
  if (!url) {
    return {error: missingMessage, html: "", consoleErrors: [], failedRequests: []};
  }

  const page = await context.newPage();
  try {
    return await capturePage(page, url, options);
  } finally {
    await page.close();
  }
}

async function writeExclusiveJson(filePath, value) {
  const handle = await fs.open(filePath, "wx", 0o600);
  try {
    await handle.writeFile(`${JSON.stringify(value, null, 2)}\n`, "utf8");
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function run() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    printHelp();
    return 0;
  }
  if (args.shardManifest && !args.shardId) {
    throw new Error("--shard-id is required when --shard-manifest is provided");
  }
  const inventory = await readJson(args.inventory);
  const rows = inventoryRows(inventory);
  const shardManifest = args.shardManifest ? await readJson(args.shardManifest) : null;
  const selectedRows = selectInventoryRows({
    rows,
    fixtureIds: args.fixtureIds,
    shardManifest,
    shardId: args.shardId,
    limit: args.limit ?? null,
  });
  if (selectedRows.length === 0) {
    throw new Error("no inventory rows selected; check --fixture-id, --shard-id, and --limit inputs");
  }

  await fs.mkdir(args.outputDir, {recursive: true, mode: 0o700});
  await fs.chmod(args.outputDir, 0o700);
  if (args.cdpEndpoint) {
    throw new Error("--cdp-endpoint is disabled because capture egress cannot be pinned");
  }
  const localOrigins = [...new Set(selectedRows.flatMap((row) => {
    if (args.sourceOnly) return [];
    try {
      const value = rowLocalUrl(row, args.localUrlField);
      if (!value) return [];
      return localBrowserCaptureOrigins(value);
    } catch (error) {
      throw new Error(`invalid local capture URL for ${row.fixture_id}: ${error.message}`);
    }
  }))].sort();
  for (const row of selectedRows) {
    try {
      rowSourceUrl(row);
    } catch (error) {
      throw new Error(`invalid source capture URL for ${row.fixture_id}: ${error.message}`);
    }
  }
  const {chromium} = loadPlaywright(args.browserRoot);
  const runId = crypto.randomUUID();
  const captureLock = await acquireBrowserCaptureLock({runId});
  const requestGateConfigPath = path.join(args.outputDir, "request-gate-config.json");
  let sourceEgressProxy = null;
  let localEgressProxy = null;
  let browserSession = null;
  let requestGate = null;
  let requestGateReady = false;
  let gateStateConfirmed = false;
  let captureError = null;
  let captureExitCode;
  let cleanupError = null;
  try {
    requestGate = await createPersistentBrowserRequestGate({
      statePath: captureLock.statePath,
      intervalMs: DEFAULT_REQUEST_INTERVAL_MS,
    });
    requestGateReady = true;
    await writeExclusiveJson(requestGateConfigPath, {
      schema: "wikijump_full_parity.browser_request_gate_config.v1",
      status: "sealed_before_browser_request",
      run_id: runId,
      lock: {path: captureLock.path, owner: captureLock.owner},
      state_path: captureLock.statePath,
      interval_ms: DEFAULT_REQUEST_INTERVAL_MS,
      source_context_exempt_origins: [],
      local_context_exempt_origins: [...new Set(localOrigins)].sort(),
      source_only: args.sourceOnly,
      public_request_policy: "every HTTP(S) request except an exact local-context origin is admitted by the shared gate",
      source_response_cache: {
        persistent_dir: args.sourceResponseCacheDir,
        persistent_identity: args.sourceResponseCacheIdentity,
        cache_documents: args.sourceResponseCacheDocuments,
        evidence_replay: true,
      },
      service_workers: "block",
      web_sockets: "blocked_without_network_connection",
    });
    sourceEgressProxy = await startCaptureEgressProxy();
    localEgressProxy = await startCaptureEgressProxy({
      allowedLocalOrigins: localOrigins,
    });
    browserSession = await openBrowser({
      chromium,
      browserExecutable: args.browserExecutable,
      ignoreHttpsErrors: args.ignoreHttpsErrors,
      storageState: args.storageState,
      sourceStorageState: args.sourceStorageState,
      localStorageState: args.localStorageState,
      createInitialContexts: true,
      sourceProxyServer: sourceEgressProxy.url,
      localProxyServer: localEgressProxy.url,
      requestGate,
      localOrigins,
      sourceResponseCacheOptions: {
        ...defaultPublicEvidenceResponseCacheOptions(),
        persistentDir: args.sourceResponseCacheDir,
        persistentIdentity: args.sourceResponseCacheIdentity,
        cacheDocuments: args.sourceResponseCacheDocuments,
        evidenceReplay: true,
      },
    });
    const runContexts = {
      sourceContext: browserSession.sourceContext,
      localContext: browserSession.localContext,
    };
    if (!runContexts.sourceContext || !runContexts.localContext) throw new Error("browser run contexts were not initialized");
    const resolvedStorageStates = resolveStorageStates({
      storageState: args.storageState,
      sourceStorageState: args.sourceStorageState,
      localStorageState: args.localStorageState,
    });
    const records = [];
    for (const row of selectedRows) {
      const sourceUrl = rowSourceUrl(row);
      const localUrl = rowLocalUrl(row, args.localUrlField);
      const rowDir = path.join(args.outputDir, safePathSegment(row.fixture_id));
      await fs.mkdir(rowDir, {recursive: true});
      const artifacts = await writeEvidenceArtifacts({
        outputDir: args.outputDir,
        row,
        source: {},
        local: {},
        screenshot: args.screenshot,
      });
      const source = await captureOptionalPage(runContexts.sourceContext, sourceUrl, "missing source URL", {
        timeoutMs: args.timeoutMs,
        waitUntil: args.waitUntil,
        settleMs: args.settleMs,
        visibleTextScope: args.visibleTextScope,
        screenshotPath: artifacts.sourceScreenshot,
        captureStates: row.capture_states ?? [],
        stateArtifactsDir: path.join(rowDir, "states"),
        blockedRequestPrefixes: row.blocked_source_request_prefixes ?? [],
        stateScreenshots: args.screenshot,
      });
      const local = args.sourceOnly ? {html: "", consoleErrors: [], failedRequests: [], blockedRequests: [], states: [], screenshotPath: null} : await captureOptionalPage(runContexts.localContext, localUrl, "missing local URL", {
        timeoutMs: args.timeoutMs,
        waitUntil: args.waitUntil,
        settleMs: args.settleMs,
        visibleTextScope: args.visibleTextScope,
        screenshotPath: artifacts.localScreenshot,
      });

      await fs.writeFile(artifacts.sourceArtifact, source.html ?? "", "utf8");
      await fs.writeFile(artifacts.localArtifact, local.html ?? "", "utf8");
      const record = buildEvidenceRecord({
        row,
        source,
        local,
        sourceArtifact: artifacts.sourceArtifact,
        localArtifact: artifacts.localArtifact,
        sourceScreenshot: source.screenshotPath,
        localScreenshot: args.sourceOnly ? null : local.screenshotPath,
        localUrlField: args.localUrlField,
      });
      if (args.actorLabel) record.capture_actor = args.actorLabel;
      record.source_only = args.sourceOnly;
      record.source_states = source.states ?? [];
      record.source_blocked_requests = source.blockedRequests ?? [];
      record.source_blocked_request_prefixes = row.blocked_source_request_prefixes ?? [];
      record.source_storage_state = Boolean(resolvedStorageStates.sourceStorageState);
      record.local_storage_state = Boolean(resolvedStorageStates.localStorageState);
      records.push(record);
    }
    const result = {
    schema: "wikijump_full_parity.browser_rendering_evidence.v1",
    inventory: args.inventory,
    shard_manifest: args.shardManifest ?? null,
    shard_id: args.shardId ?? null,
    selected_count: selectedRows.length,
    evidence: records,
    capture: {
      timeout_ms: args.timeoutMs,
      settle_ms: args.settleMs,
      wait_until: args.waitUntil,
      visible_text_scope: args.visibleTextScope,
      ignore_https_errors: args.ignoreHttpsErrors,
      screenshot: args.screenshot,
      browser_executable: args.browserExecutable ?? null,
      cdp_endpoint: args.cdpEndpoint ?? null,
      actor_label: args.actorLabel ?? null,
      storage_state: Boolean(args.storageState),
      source_storage_state: Boolean(resolvedStorageStates.sourceStorageState),
      local_storage_state: Boolean(resolvedStorageStates.localStorageState),
      request_gate_config: requestGateConfigPath,
      request_gate: requestGate.snapshot(),
      browser_context_scope: "run",
      source_response_cache_mode: args.sourceResponseCacheDir === null ? "browser_context" : "persistent_identity_bound",
      source_response_cache: browserSession.sourceResponseCache?.snapshot() ?? null,
      browser_version: typeof browserSession.browser.version === "function"
        ? await browserSession.browser.version()
        : null,
      browser_executable_sha256: args.browserExecutable ? await hashFile(args.browserExecutable) : null,
      inventory_sha256: await hashFile(args.inventory),
      capture_script_sha256: await hashFile(SCRIPT_PATH),
      source_only: args.sourceOnly,
    },
    };
    const resultPath = path.join(args.outputDir, "records.json");
    await writeExclusiveJson(resultPath, result);
    await requestGate.flush();
    await captureLock.confirmState();
    gateStateConfirmed = true;
    if (!args.jsonOnly) {
      console.log(`wrote ${records.length} browser rendering records to ${resultPath}`);
    } else {
      console.log(JSON.stringify({result_path: resultPath, selected_count: selectedRows.length}));
    }

    const captureErrors = records.flatMap((record) => [
      ...(record.capture_errors ?? []),
      ...(record.source_states ?? []).filter((state) => state.capture_error).map((state) => ({side: "source_state", state_id: state.state_id, message: state.capture_error})),
    ]);
    captureExitCode = captureErrors.length === 0 ? 0 : 1;
  } catch (error) {
    captureError = error;
  } finally {
    cleanupError = requestGateReady ? null : new Error("browser request gate was not initialized; retaining the capture lock for operator review");
    try {
      await browserSession?.close();
    } catch (error) {
      cleanupError ??= error;
    }
    try {
      await Promise.all([sourceEgressProxy?.close(), localEgressProxy?.close()]);
    } catch (error) {
      cleanupError ??= error;
    }
    try {
      await requestGate?.flush();
    } catch (error) {
      cleanupError ??= error;
    }
    if (requestGateReady && !gateStateConfirmed && !requestGate?.snapshot().enforcement_failed) {
      try {
        await captureLock.confirmState();
        gateStateConfirmed = true;
      } catch (error) {
        cleanupError ??= error;
      }
    }
    if (gateStateConfirmed) {
      try {
        await captureLock.release();
      } catch (error) {
        cleanupError ??= error;
      }
    }
  }
  const failure = browserCaptureFailure(captureError, cleanupError);
  if (failure !== null) throw failure;
  return captureExitCode;
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  run().then((code) => {
    process.exitCode = code;
  }).catch((error) => {
    console.error(browserCaptureFailureDiagnostic(error));
    process.exitCode = 1;
  });
}
