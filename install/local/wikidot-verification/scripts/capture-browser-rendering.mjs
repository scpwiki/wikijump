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
  readJson,
  inventoryRows,
  rowLocalUrl,
  rowSourceUrl,
  safePathSegment,
  selectInventoryRows,
  writeEvidenceArtifacts,
} from "../src/browser-render-evidence.mjs";

const DEFAULT_TIMEOUT_MS = 900_000;
const DEFAULT_SETTLE_MS = 1_000;
const POST_NAVIGATION_STATE_TIMEOUT_MS = 2_000;
const VISIBLE_TEXT_SCOPES = new Set(["all-frames", "main-frame"]);
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
    visibleTextScope: "all-frames",
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
      return {help: true};
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.inventory) throw new Error("--inventory is required");
  if (!args.outputDir) throw new Error("--output-dir is required");
  return args;
}

function printHelp() {
  console.log(`Usage: capture-browser-rendering.mjs --inventory FILE --output-dir DIR [--shard-manifest FILE --shard-id ID] [--fixture-id ID ...] [--limit N] [--browser-root framerail] [--browser-executable /usr/bin/google-chrome | --cdp-endpoint http://127.0.0.1:9222] [--storage-state FILE | --source-storage-state FILE --local-storage-state FILE] [--actor-label LABEL] [--local-url-field local_https_url] [--timeout-ms 900000] [--settle-ms 1000] [--visible-text-scope all-frames|main-frame] [--ignore-https-errors] [--no-screenshot] [--json]

Writes validator-compatible browser rendering evidence JSON plus DOM/screenshot artifacts for selected corpus inventory rows. The output directory should live under one of the render validator evidence roots, for example:

  $OUT/validation/browser-rendering/en-0001
`);
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

async function collectVisibleText(page, visibleTextScope = "all-frames") {
  const frames =
    visibleTextScope === "main-frame"
      ? [typeof page.mainFrame === "function" ? page.mainFrame() : page]
      : typeof page.frames === "function"
        ? page.frames()
        : [page];
  const texts = [];
  for (const frame of frames) {
    try {
      if (!(await shouldCaptureFrameVisibleText(page, frame))) continue;
      const text = await frame.evaluate(() => document.body?.innerText ?? "");
      if (text) texts.push(text);
    } catch (error) {
      void error;
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

export async function capturePage(page, url, {timeoutMs, waitUntil, settleMs = DEFAULT_SETTLE_MS, screenshotPath, visibleTextScope = "all-frames"}) {
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
    visibleText = await collectVisibleText(page, visibleTextScope);
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
    const value = rowLocalUrl(row, args.localUrlField);
    if (!value) return [];
    try {
      return localBrowserCaptureOrigins(value);
    } catch (error) {
      throw new Error(`invalid local capture URL for ${row.fixture_id}: ${error.message}`);
    }
  }))].sort();
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
      public_request_policy: "every HTTP(S) request except an exact local-context origin is admitted by the shared gate",
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
      });
      const local = await captureOptionalPage(runContexts.localContext, localUrl, "missing local URL", {
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
      source_response_cache: browserSession.sourceResponseCache?.snapshot() ?? null,
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

    const captureErrors = records.flatMap((record) => record.capture_errors ?? []);
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
    console.error(error.message);
    process.exitCode = 1;
  });
}
