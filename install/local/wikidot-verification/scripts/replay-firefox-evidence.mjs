#!/usr/bin/env node
// Maintained cache-only Firefox replay of retained Wikidot evidence.
//
// Firefox upgrades an insecure subresource through an internal redirect that
// Playwright route interception never observes, so a replay must launch
// Firefox with `firefoxEvidenceReplayLaunchOptions` (see
// `../src/browser-session.mjs`). This script is the repository entrypoint for
// that replay: it serves every request from a retained response cache,
// fails closed on any cache miss, and records a verdict about the replay
// itself. A `pass` verdict means the replay stayed sealed (no external
// network, no miss, no store); it is not a visual-parity claim.
//
// Evidence replay only: this script never fetches live Wikidot evidence.
import {createHash} from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import {
  createBrowserRequestGate,
  createBrowserResponseCache,
  defaultPublicEvidenceResponseCacheOptions,
  installBrowserRequestGate,
} from "../src/browser-request-gate.mjs";
import {
  defaultBrowserRoot,
  firefoxEvidenceReplayLaunchOptions,
  loadPlaywright,
} from "../src/browser-session.mjs";
import {runCliIfMain} from "../src/cli-entry.mjs";

const DEFAULT_VIEWPORT = {width: 1366, height: 900};
const DEFAULT_TIMEOUT_MS = 120_000;
const DEFAULT_SETTLE_MS = 1_500;

export const FIREFOX_REPLAY_SCHEMA = "wikijump.firefox_evidence_replay.v1";

// The launch prefs disable Firefox's internal http:// subresource upgrade so
// every request stays observable to the request gate. That is a deliberate
// harness normalization relative to default Firefox behavior, recorded in
// every verdict below.
export const FIREFOX_REPLAY_NORMALIZATIONS = [
  {
    kind: "firefox_mixed_content_upgrade_disabled",
    reason: "firefoxEvidenceReplayLaunchOptions sets dom.security.https_only_mode* and security.mixed_content.upgrade_display_content to false so http:// subresources are requested as written and stay serveable from retained evidence",
  },
];

export function replayNormalizations(excludePrefixes) {
  if (!Array.isArray(excludePrefixes) || excludePrefixes.length === 0) return [...FIREFOX_REPLAY_NORMALIZATIONS];
  return [
    ...FIREFOX_REPLAY_NORMALIZATIONS,
    {
      kind: "excluded_request_prefixes_aborted",
      reason: `requests whose URL starts with one of ${excludePrefixes.length} caller-supplied prefixes were aborted client-side before reaching the request gate and are counted as excluded_requests, not replayed evidence`,
    },
  ];
}

function nextValue(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`);
  return value;
}

function parseViewport(value) {
  const match = /^(\d+)x(\d+)$/u.exec(value);
  if (!match || Number(match[1]) < 1 || Number(match[2]) < 1) {
    throw new Error("--viewport must be WIDTHxHEIGHT");
  }
  return {width: Number(match[1]), height: Number(match[2])};
}

function positiveInteger(value, flag) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 1) {
    throw new Error(`${flag} must be a positive integer`);
  }
  return parsed;
}

function requireHttpUrl(value, flag) {
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    throw new Error(`${flag} must be an absolute http(s) URL`);
  }
  if (!new Set(["http:", "https:"]).has(parsed.protocol)) {
    throw new Error(`${flag} must be an absolute http(s) URL`);
  }
  return parsed.href;
}

export function parseArgs(argv) {
  const args = {
    cacheDir: null,
    cacheIdentity: null,
    url: null,
    outputDir: null,
    firefoxExecutable: null,
    browserRoot: defaultBrowserRoot(),
    viewport: {...DEFAULT_VIEWPORT},
    timeoutMs: DEFAULT_TIMEOUT_MS,
    settleMs: DEFAULT_SETTLE_MS,
    excludePrefixes: [],
  };
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (flag === "--cache-dir") args.cacheDir = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--cache-identity") args.cacheIdentity = nextValue(argv, index++, flag);
    else if (flag === "--url") args.url = requireHttpUrl(nextValue(argv, index++, flag), flag);
    else if (flag === "--output-dir") args.outputDir = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--firefox-executable") args.firefoxExecutable = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--browser-root") args.browserRoot = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--viewport") args.viewport = parseViewport(nextValue(argv, index++, flag));
    else if (flag === "--timeout-ms") args.timeoutMs = positiveInteger(nextValue(argv, index++, flag), flag);
    else if (flag === "--settle-ms") args.settleMs = positiveInteger(nextValue(argv, index++, flag), flag);
    else if (flag === "--exclude-prefix") args.excludePrefixes.push(nextValue(argv, index++, flag));
    else if (flag === "--help" || flag === "-h") args.help = true;
    else throw new Error(`unknown argument: ${flag}`);
  }
  if (args.help) return args;
  for (const [flag, value] of [["--cache-dir", args.cacheDir], ["--cache-identity", args.cacheIdentity], ["--url", args.url], ["--output-dir", args.outputDir]]) {
    if (!value) throw new Error(`${flag} is required`);
  }
  return args;
}

export function usage() {
  return `Usage: replay-firefox-evidence.mjs --cache-dir DIR --cache-identity ID --url URL --output-dir DIR [options]

Replay a retained Wikidot page in Firefox served entirely from a retained
response cache. Fails closed on any cache miss; never fetches live evidence.

Options:
  --cache-dir DIR            Retained response-cache directory (manifest.json)
  --cache-identity ID        Expected retained cache identity
  --url URL                  Retained page URL to replay (http or https)
  --output-dir DIR           Artifact directory (must not already exist)
  --firefox-executable FILE  Firefox executable (default: Playwright's Firefox)
  --browser-root DIR         Directory containing playwright
  --viewport WIDTHxHEIGHT    Capture viewport (default 1366x900)
  --timeout-ms N             Navigation timeout
  --settle-ms N              Settled-state delay after load
  --exclude-prefix PREFIX    Abort matching request URLs client-side (repeatable;
                             recorded in the verdict, never replayed evidence)
`;
}

async function sha256File(filePath) {
  return createHash("sha256").update(await fs.readFile(filePath)).digest("hex");
}

async function manifestSha256(cacheDir) {
  try {
    return await sha256File(path.join(cacheDir, "manifest.json"));
  } catch (error) {
    if (error?.code === "ENOENT") return null;
    throw error;
  }
}

function summarizeImages(rows) {
  const images = rows.length;
  const broken = rows.filter((row) => !row.complete || row.naturalWidth === 0).length;
  return {
    images,
    broken,
    incomplete_with_dimensions: rows.filter((row) => !row.complete && row.naturalWidth > 0).length,
    zero_natural: rows.filter((row) => row.naturalWidth === 0).length,
  };
}

export function buildReplayVerdict({
  url,
  cacheDir,
  cacheIdentity,
  manifestSha256Before,
  manifestSha256After,
  browser,
  navigationStatus,
  navigationError,
  images,
  requestCount,
  failedRequests,
  excludedRequestPrefixes = [],
  excludedRequests = 0,
  gateSnapshot,
  cacheSnapshot,
}) {
  const external = gateSnapshot.external_network_requests ?? null;
  const misses = cacheSnapshot.misses ?? null;
  const stores = cacheSnapshot.stores ?? null;
  const status = navigationError === null
    && typeof navigationStatus === "number"
    && navigationStatus >= 200
    && navigationStatus < 400
    && gateSnapshot.enforcement_failed === false
    && external === 0
    && misses === 0
    && stores === 0
    ? "pass"
    : "fail";
  return {
    schema: FIREFOX_REPLAY_SCHEMA,
    status,
    url,
    cache: {
      directory: cacheDir,
      identity: cacheIdentity,
      manifest_sha256_before: manifestSha256Before,
      manifest_sha256_after: manifestSha256After,
    },
    browser,
    navigation_status: navigationStatus,
    navigation_error: navigationError,
    images,
    requests: requestCount,
    failed_requests: failedRequests,
    excluded_request_prefixes: [...excludedRequestPrefixes],
    excluded_requests: excludedRequests,
    gate: {
      external_network_requests: external,
      enforcement_failed: gateSnapshot.enforcement_failed ?? null,
      unsupported_requests_blocked: gateSnapshot.unsupported_requests_blocked ?? null,
    },
    response_cache: {
      misses,
      stores,
    },
    harness_normalizations: replayNormalizations(excludedRequestPrefixes),
  };
}

export async function runFirefoxEvidenceReplay(args, {launchFirefox = null} = {}) {
  const manifestBefore = await manifestSha256(args.cacheDir);
  const gate = createBrowserRequestGate({intervalMs: 0});
  const responseCache = createBrowserResponseCache({
    ...defaultPublicEvidenceResponseCacheOptions(),
    persistentDir: args.cacheDir,
    persistentIdentity: args.cacheIdentity,
    cacheDocuments: true,
    evidenceReplay: true,
  });
  await responseCache.load();

  const launch = launchFirefox ?? (async () => {
    const {firefox} = loadPlaywright(args.browserRoot);
    const executable = args.firefoxExecutable ?? firefox.executablePath();
    return {
      browser: await firefox.launch(firefoxEvidenceReplayLaunchOptions(executable)),
      executable,
    };
  });
  const {browser, executable} = await launch();
  const executableSha256 = await sha256File(executable);
  const browserVersion = browser.version();

  let navigationStatus = null;
  let navigationError = null;
  let imageRows = [];
  let requestCount = 0;
  let failedRequests = 0;
  let excludedRequests = 0;
  const excludePrefixes = Array.isArray(args.excludePrefixes) ? args.excludePrefixes : [];
  const context = await browser.newContext({
    viewport: args.viewport,
    deviceScaleFactor: 1,
    ignoreHTTPSErrors: true,
    serviceWorkers: "block",
  });
  try {
    await installBrowserRequestGate(context, {
      gate,
      responseCache,
      cacheOnly: true,
    });
    const page = await context.newPage();
    if (excludePrefixes.length > 0) {
      await page.route("**/*", async (route) => {
        const requestUrl = route.request().url();
        if (excludePrefixes.some((prefix) => requestUrl.startsWith(prefix))) {
          excludedRequests += 1;
          await route.abort("blockedbyclient");
          return;
        }
        await route.fallback();
      });
    }
    page.on("request", () => {
      requestCount += 1;
    });
    page.on("requestfailed", () => {
      failedRequests += 1;
    });
    try {
      const response = await page.goto(args.url, {waitUntil: "domcontentloaded", timeout: args.timeoutMs});
      navigationStatus = response?.status() ?? null;
      await page.waitForTimeout(args.settleMs);
      imageRows = await page.evaluate(() => [...document.images].map((img) => ({
        complete: img.complete,
        naturalWidth: img.naturalWidth,
        naturalHeight: img.naturalHeight,
      })));
    } catch (error) {
      navigationError = String(error?.message ?? error).slice(0, 1000);
    }
    await context.close();
  } finally {
    await browser.close().catch(() => {});
  }

  let gateErrorMessage = null;
  try {
    await gate.flush();
  } catch (error) {
    gateErrorMessage = String(error?.message ?? error);
  }
  const verdict = buildReplayVerdict({
    url: args.url,
    cacheDir: args.cacheDir,
    cacheIdentity: args.cacheIdentity,
    manifestSha256Before: manifestBefore,
    manifestSha256After: await manifestSha256(args.cacheDir),
    browser: {
      engine: "firefox",
      version: browserVersion,
      executable_sha256: executableSha256,
    },
    navigationStatus,
    navigationError,
    images: summarizeImages(imageRows),
    requestCount,
    failedRequests,
    excludedRequestPrefixes: excludePrefixes,
    excludedRequests,
    gateSnapshot: gate.snapshot(),
    cacheSnapshot: responseCache.snapshot(),
  });
  if (gateErrorMessage !== null) {
    verdict.gate.flush_error = gateErrorMessage;
    verdict.status = "fail";
  }
  await fs.mkdir(args.outputDir, {recursive: true});
  await fs.writeFile(
    path.join(args.outputDir, "firefox-evidence-replay.json"),
    `${JSON.stringify(verdict, null, 2)}\n`,
    {flag: "wx"},
  );
  return {verdict, outputDir: args.outputDir};
}

export async function outputDirectory(requested) {
  if (requested) {
    await fs.mkdir(requested, {recursive: true});
    return requested;
  }
  return await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-firefox-evidence-replay-"));
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const {verdict, outputDir} = await runFirefoxEvidenceReplay({
    ...args,
    outputDir: await outputDirectory(args.outputDir),
  });
  console.log(JSON.stringify({status: verdict.status, output_dir: outputDir}));
  return verdict.status === "pass" ? 0 : 1;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error?.stack ?? error?.message ?? String(error));
    return 2;
  },
});
