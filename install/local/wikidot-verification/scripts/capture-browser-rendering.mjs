#!/usr/bin/env node

import {createRequire} from "node:module";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";
import {
  buildEvidenceRecord,
  readJson,
  inventoryRows,
  rowLocalUrl,
  rowSourceUrl,
  safePathSegment,
  selectInventoryRows,
  writeEvidenceArtifacts,
} from "../src/browser-render-evidence.mjs";

const DEFAULT_TIMEOUT_MS = 30_000;
const DEFAULT_SETTLE_MS = 1_000;
const POST_NAVIGATION_STATE_TIMEOUT_MS = 2_000;
const VISIBLE_TEXT_SCOPES = new Set(["all-frames", "main-frame"]);
const SCRIPT_PATH = fileURLToPath(import.meta.url);
const SCRIPT_DIR = path.dirname(SCRIPT_PATH);

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
  };

  for (let index = 2; index < argv.length; index += 1) {
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
        throw new Error("--visible-text-scope must be all-frames or main-frame");
      }
      index += 1;
    } else if (arg === "--ignore-https-errors") {
      args.ignoreHttpsErrors = true;
    } else if (arg === "--no-screenshot") {
      args.screenshot = false;
    } else if (arg === "--json") {
      args.jsonOnly = true;
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.inventory) throw new Error("--inventory is required");
  if (!args.outputDir) throw new Error("--output-dir is required");
  return args;
}

function printHelpAndExit() {
  console.log(`Usage: capture-browser-rendering.mjs --inventory FILE --output-dir DIR [--shard-manifest FILE --shard-id ID] [--fixture-id ID ...] [--limit N] [--browser-root framerail] [--browser-executable /usr/bin/google-chrome | --cdp-endpoint http://127.0.0.1:9222] [--storage-state FILE | --source-storage-state FILE --local-storage-state FILE] [--actor-label LABEL] [--local-url-field local_https_url] [--timeout-ms 30000] [--settle-ms 1000] [--visible-text-scope all-frames|main-frame] [--ignore-https-errors] [--no-screenshot] [--json]

Writes validator-compatible browser rendering evidence JSON plus DOM/screenshot artifacts for selected corpus inventory rows. The output directory should live under one of the render validator evidence roots, for example:

  $OUT/validation/browser-rendering/en-0001
`);
  process.exit(0);
}

export function defaultBrowserRoot() {
  return path.resolve(SCRIPT_DIR, "../../../..", "framerail");
}

function requirePlaywright(browserRoot) {
  const root = browserRoot ?? defaultBrowserRoot();
  const requireFromRoot = createRequire(path.join(root, "package.json"));
  try {
    return requireFromRoot("playwright");
  } catch (error) {
    try {
      return requireFromRoot("@playwright/test");
    } catch (fallbackError) {
      throw new Error(`could not load playwright or @playwright/test from ${root}; pass --browser-root pointing at a package with Playwright installed (${error.message}; ${fallbackError.message})`);
    }
  }
}

export function resolveStorageStates({storageState = null, sourceStorageState = null, localStorageState = null}) {
  return {
    sourceStorageState: sourceStorageState ?? storageState ?? null,
    localStorageState: localStorageState ?? storageState ?? null,
  };
}

export function browserContextOptions({ignoreHttpsErrors, storageState = null}) {
  return {
    ignoreHTTPSErrors: ignoreHttpsErrors,
    ...(storageState ? {storageState} : {}),
  };
}

async function newContextPair({browser, ignoreHttpsErrors, sourceStorageState, localStorageState}) {
  let sourceContext = null;
  let localContext = null;
  try {
    sourceContext = await browser.newContext(
      browserContextOptions({ignoreHttpsErrors, storageState: sourceStorageState})
    );
    localContext = await browser.newContext(
      browserContextOptions({ignoreHttpsErrors, storageState: localStorageState})
    );
    return {sourceContext, localContext};
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
  if (localContext && localContext !== sourceContext) {
    await localContext.close().catch(() => {});
  }
  if (sourceContext) {
    await sourceContext.close().catch(() => {});
  }
}

function browserSession({browser, sourceContext, localContext, ignoreHttpsErrors, sourceStorageState, localStorageState}) {
  return {
    browser,
    context: sourceContext,
    sourceContext,
    localContext,
    async newContextPair() {
      return await newContextPair({
        browser,
        ignoreHttpsErrors,
        sourceStorageState,
        localStorageState,
      });
    },
    async close() {
      await closeContextPair({sourceContext, localContext});
      await browser.close().catch(() => {});
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
}) {
  const resolvedStates = resolveStorageStates({storageState, sourceStorageState, localStorageState});
  let browser = null;
  if (cdpEndpoint) {
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
        })
      : {sourceContext: null, localContext: null};
    return browserSession({
      browser,
      ...contexts,
      ignoreHttpsErrors,
      sourceStorageState: resolvedStates.sourceStorageState,
      localStorageState: resolvedStates.localStorageState,
    });
  } catch (error) {
    if (browser) {
      await browser.close().catch(() => {});
    }
    throw error;
  }
}

function frameOrigin(value) {
  try {
    return new URL(value).origin;
  } catch {
    return null;
  }
}

function frameUrl(frame) {
  if (typeof frame.url !== "function") return null;
  try {
    return frame.url();
  } catch {
    return null;
  }
}

async function collectVisibleText(page, visibleTextScope = "main-frame") {
  const frames =
    visibleTextScope === "main-frame"
      ? [typeof page.mainFrame === "function" ? page.mainFrame() : page]
      : typeof page.frames === "function"
        ? page.frames()
        : [page];
  const texts = [];
  const pageOrigin = frameOrigin(typeof page.url === "function" ? page.url() : "");
  for (const frame of frames) {
    try {
      const isMainFrame = typeof page.mainFrame === "function" && frame === page.mainFrame();
      const origin = frameOrigin(frameUrl(frame));
      if (!isMainFrame && (!pageOrigin || origin !== pageOrigin)) continue;
      if (!(await shouldCaptureFrameVisibleText(page, frame))) continue;
      const text = await frame.evaluate(() => document.body?.innerText ?? "");
      if (text) texts.push(text);
    } catch {
      // Detached or inaccessible frames should not abort page-level capture.
    }
  }
  return texts.join("\n");
}

async function shouldCaptureFrameVisibleText(page, frame) {
  if (typeof page.mainFrame === "function" && frame === page.mainFrame()) {
    return true;
  }
  if (typeof frame.frameElement !== "function") {
    return true;
  }

  let frameElement = null;
  try {
    frameElement = await frame.frameElement();
  } catch {
    return true;
  }
  if (!frameElement) {
    return true;
  }

  try {
    return await frameElement.evaluate((element) => {
      if (!(element instanceof HTMLElement)) return true;
      if (element.hidden) return false;
      const style = window.getComputedStyle(element);
      if (style.display === "none" || style.visibility === "hidden" || style.visibility === "collapse") {
        return false;
      }
      return true;
    });
  } catch {
    return true;
  }
}

async function waitForLoadStateWithinBudget(page, state, timeoutMs, startedAt) {
  const remainingMs = timeoutMs - (Date.now() - startedAt);
  if (remainingMs <= 0) return;
  await page.waitForLoadState(state, {timeout: Math.min(POST_NAVIGATION_STATE_TIMEOUT_MS, remainingMs)}).catch(() => {});
}

export async function capturePage(page, url, {timeoutMs, waitUntil, settleMs = DEFAULT_SETTLE_MS, screenshotPath, visibleTextScope = "main-frame"}) {
  const consoleErrors = [];
  const failedRequests = [];
  const badResponses = [];
  let sawInitialMainFrameNavigationResponse = false;
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
    } catch {
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
    visibleText = await collectVisibleText(page, visibleTextScope);
    html = await page.content();
  } catch (error) {
    if (!navigationError) navigationError = error;
  }

  if (screenshotPath && html) {
    try {
      await page.screenshot({path: screenshotPath, fullPage: true});
      writtenScreenshotPath = screenshotPath;
    } catch (error) {
      if (!navigationError) navigationError = error;
    }
  }

  if (!navigationError) {
    return {
      status: response?.status() ?? null,
      finalUrl: page.url(),
      visibleText,
      html,
      consoleErrors,
      failedRequests: [...failedRequests, ...badResponses],
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
    screenshotPath: writtenScreenshotPath,
    error: navigationError.message,
  };
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

async function run() {
  const args = parseArgs(process.argv);
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

  await fs.mkdir(args.outputDir, {recursive: true});
  const {chromium} = requirePlaywright(args.browserRoot);
  const browserSession = await openBrowser({
    chromium,
    cdpEndpoint: args.cdpEndpoint,
    browserExecutable: args.browserExecutable,
    ignoreHttpsErrors: args.ignoreHttpsErrors,
    storageState: args.storageState,
    sourceStorageState: args.sourceStorageState,
    localStorageState: args.localStorageState,
    createInitialContexts: false,
  });
  const resolvedStorageStates = resolveStorageStates({
    storageState: args.storageState,
    sourceStorageState: args.sourceStorageState,
    localStorageState: args.localStorageState,
  });
  const records = [];

  try {
    for (const row of selectedRows) {
      const rowContexts = await browserSession.newContextPair();
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
      try {
        const source = await captureOptionalPage(rowContexts.sourceContext, sourceUrl, "missing source URL", {
          timeoutMs: args.timeoutMs,
          waitUntil: args.waitUntil,
          settleMs: args.settleMs,
          visibleTextScope: args.visibleTextScope,
          screenshotPath: artifacts.sourceScreenshot,
        });
        const local = await captureOptionalPage(rowContexts.localContext, localUrl, "missing local URL", {
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
          localScreenshot: local.screenshotPath,
          localUrlField: args.localUrlField,
        });
        if (args.actorLabel) record.capture_actor = args.actorLabel;
        record.source_storage_state = Boolean(resolvedStorageStates.sourceStorageState);
        record.local_storage_state = Boolean(resolvedStorageStates.localStorageState);
        records.push(record);
      } finally {
        await closeContextPair(rowContexts);
      }
    }
  } finally {
    await browserSession.close();
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
    },
  };
  const resultPath = path.join(args.outputDir, "records.json");
  await fs.writeFile(resultPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
  if (!args.jsonOnly) {
    console.log(`wrote ${records.length} browser rendering records to ${resultPath}`);
  } else {
    console.log(JSON.stringify({result_path: resultPath, selected_count: selectedRows.length}));
  }

  const captureErrors = records.flatMap((record) => record.capture_errors ?? []);
  return captureErrors.length === 0 ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  run().then((code) => {
    process.exitCode = code;
  }).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
