#!/usr/bin/env node

import {createHash} from "node:crypto";
import fs from "node:fs/promises";
import {createRequire} from "node:module";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {
  DEFAULT_VISUAL_RMSE_THRESHOLD,
  compareCaptureToOfflineOracle,
  comparePngRmse,
  loadOfflineBrowserOracle,
  sha256File,
} from "../src/offline-compatibility-oracle.mjs";
import {browserReplayPayload, loadOfflineBrowserResponseFixture} from "../src/offline-browser-response-fixture.mjs";
import {captureBrowserParityObservation} from "../src/standing-browser-parity-observation.mjs";
import {
  DEFAULT_SETTLE_MS,
  DEFAULT_TIMEOUT_MS,
  DEFAULT_VIEWPORT,
  canaryForUrl,
} from "../src/standing-browser-canaries.mjs";
import {DEFAULT_PARITY_BROWSER_ROOT} from "../src/standing-browser-parity-browser-session.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const DEFAULT_ORACLE = path.resolve(
  HERE,
  "../fixtures/offline-compatibility/scp-9506-final-zero-oracle.json",
);
const DEFAULT_RESPONSE_FIXTURE = path.resolve(
  HERE,
  "../fixtures/offline-compatibility/scp-9506-browser-responses.json.gz",
);
const DEFAULT_LOCAL_ORIGIN = "https://scp-wiki.wikijump.localhost";

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

function localOrigin(value) {
  const parsed = new URL(value);
  if (
    parsed.protocol !== "https:" ||
    parsed.username ||
    parsed.password ||
    parsed.pathname !== "/" ||
    parsed.search ||
    parsed.hash ||
    !parsed.hostname.endsWith(".wikijump.localhost")
  ) {
    throw new Error("--local-origin must be an exact *.wikijump.localhost HTTPS origin");
  }
  return parsed.origin;
}

function positiveInteger(value, flag) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 1) {
    throw new Error(`${flag} must be a positive integer`);
  }
  return parsed;
}

export function parseArgs(argv) {
  const args = {
    oracle: DEFAULT_ORACLE,
    responseFixture: DEFAULT_RESPONSE_FIXTURE,
    outputDir: null,
    browserRoot: DEFAULT_PARITY_BROWSER_ROOT,
    browserExecutable: null,
    localOrigin: DEFAULT_LOCAL_ORIGIN,
    viewport: {...DEFAULT_VIEWPORT},
    timeoutMs: DEFAULT_TIMEOUT_MS,
    settleMs: DEFAULT_SETTLE_MS,
    visualRmseThreshold: DEFAULT_VISUAL_RMSE_THRESHOLD,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (flag === "--oracle") args.oracle = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--response-fixture") args.responseFixture = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--output-dir") args.outputDir = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--browser-root") args.browserRoot = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--browser-executable") args.browserExecutable = path.resolve(nextValue(argv, index++, flag));
    else if (flag === "--local-origin") args.localOrigin = localOrigin(nextValue(argv, index++, flag));
    else if (flag === "--viewport") args.viewport = parseViewport(nextValue(argv, index++, flag));
    else if (flag === "--timeout-ms") args.timeoutMs = positiveInteger(nextValue(argv, index++, flag), flag);
    else if (flag === "--settle-ms") args.settleMs = positiveInteger(nextValue(argv, index++, flag), flag);
    else if (flag === "--visual-rmse-threshold") {
      args.visualRmseThreshold = Number(nextValue(argv, index++, flag));
      if (!Number.isFinite(args.visualRmseThreshold) || args.visualRmseThreshold < 0 || args.visualRmseThreshold > 1) {
        throw new Error("--visual-rmse-threshold must be between 0 and 1");
      }
    } else if (flag === "--help" || flag === "-h") args.help = true;
    else throw new Error(`unknown argument: ${flag}`);
  }
  return args;
}

export function usage() {
  return `Usage: run-offline-browser-oracle.mjs [options]

Capture the local Wikijump SCP-9506 page and compare it with the repository-owned
frozen Wikidot observation plus the final-zero-accepted Wikijump full-page golden.
No live Wikidot acquisition is permitted.

Options:
  --oracle FILE                  Offline oracle fixture
  --response-fixture FILE        Frozen external-response fixture
  --output-dir DIR              Artifact directory (temporary by default)
  --local-origin ORIGIN         Local *.wikijump.localhost HTTPS origin
  --browser-root DIR            Directory containing @playwright/test
  --browser-executable FILE     Chromium executable override
  --viewport WIDTHxHEIGHT       Capture viewport (default 1366x900)
  --timeout-ms N                Browser capture timeout
  --settle-ms N                 Settled-state delay
  --visual-rmse-threshold N     Normalized full-page RMSE limit
`;
}

function requireNetworkIsolation() {
  if (process.env.WIKIJUMP_OFFLINE_BROWSER_NETNS_ACTIVE !== "1") {
    throw new Error(
      "offline browser oracle must run through scripts/run-offline-standing-browser-test.sh",
    );
  }
}

function requirePlaywright(browserRoot) {
  const requireFromRoot = createRequire(path.join(browserRoot, "package.json"));
  try {
    return requireFromRoot("@playwright/test");
  } catch (error) {
    throw new Error(
      `could not load @playwright/test from ${browserRoot}: ${error.message}`,
    );
  }
}

async function browserIdentity(executable, browser) {
  const bytes = await fs.readFile(executable);
  return {
    engine: "chromium",
    version: browser.version(),
    executable_sha256: createHash("sha256").update(bytes).digest("hex"),
  };
}

async function acceptedBrowserExecutable(chromium, requested, accepted) {
  const requiredSha256 = accepted.executable_sha256;
  const inspect = async (candidate) => {
    if (!candidate) return null;
    let real;
    try {
      real = await fs.realpath(candidate);
    } catch (error) {
      if (error?.code === "ENOENT") return null;
      throw error;
    }
    return (await sha256File(real)) === requiredSha256 ? real : null;
  };
  if (requested) {
    const match = await inspect(requested);
    if (!match) {
      throw new Error(
        `--browser-executable does not match final-zero Chromium ${accepted.version} (${requiredSha256})`,
      );
    }
    return match;
  }
  const current = await fs.realpath(chromium.executablePath());
  const currentMatch = await inspect(current);
  if (currentMatch) return currentMatch;

  const browserCache = path.dirname(path.dirname(path.dirname(current)));
  const directories = await fs.readdir(browserCache, {withFileTypes: true});
  for (const directory of directories
    .filter((entry) => entry.isDirectory() && entry.name.startsWith("chromium-"))
    .sort((left, right) => left.name.localeCompare(right.name))) {
    const match = await inspect(
      path.join(browserCache, directory.name, "chrome-linux64", "chrome"),
    );
    if (match) return match;
  }
  throw new Error(
    `final-zero Chromium ${accepted.version} (${requiredSha256}) is not installed; ` +
      "install that exact browser artifact instead of approving a new golden with a different browser",
  );
}

function isLoopbackHostname(hostname) {
  const normalized = hostname.toLowerCase();
  return (
    normalized === "localhost" ||
    normalized === "127.0.0.1" ||
    normalized === "::1" ||
    normalized.endsWith(".wikijump.localhost") ||
    normalized.endsWith(".wjfiles.localhost")
  );
}

export function requestIsOfflineSafe(rawUrl) {
  const url = new URL(rawUrl);
  if (new Set(["about:", "blob:", "data:"]).has(url.protocol)) return true;
  if (!new Set(["http:", "https:"]).has(url.protocol)) return false;
  return isLoopbackHostname(url.hostname);
}

async function outputDirectory(requested) {
  if (requested) {
    await fs.mkdir(requested, {recursive: true});
    return requested;
  }
  return await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-offline-browser-oracle-"));
}

export async function runOfflineBrowserOracle(args) {
  requireNetworkIsolation();
  const loaded = await loadOfflineBrowserOracle(args.oracle);
  const responseFixture = await loadOfflineBrowserResponseFixture(args.responseFixture);
  const outputDir = await outputDirectory(args.outputDir);
  const localUrl = new URL(new URL(loaded.oracle.pair.local_url).pathname, args.localOrigin).href;
  const contract = canaryForUrl(localUrl);
  if (!contract) throw new Error(`no standing canary contract for ${localUrl}`);

  const {chromium} = requirePlaywright(args.browserRoot);
  const executable = await acceptedBrowserExecutable(
    chromium,
    args.browserExecutable,
    loaded.oracle.provenance.accepted_browser,
  );
  const browser = await chromium.launch({
    executablePath: executable,
    headless: true,
    args: [
      "--disable-http-cache",
      "--disable-background-networking",
      "--disable-component-update",
      "--disable-domain-reliability",
      "--disable-sync",
      "--metrics-recording-only",
      "--no-first-run",
      "--disable-default-apps",
      "--disable-async-dns",
      "--disable-dns-probes",
      "--dns-prefetch-disable",
      "--disable-quic",
      "--disable-features=AsyncDns,DnsOverHttps,UseDnsHttpsSvcb,UseDnsHttpsSvcbAlpn,EncryptedClientHello,OptimizationHints,MediaRouter,DialMediaRouteProvider,Translate,NetworkTimeServiceQuerying",
      "--host-resolver-rules=MAP * 127.0.0.1",
    ],
  });
  const replayedRequests = [];
  const fixtureMisses = [];
  const harnessNormalizations = [];
  let capture;
  let identity;
  try {
    identity = await browserIdentity(executable, browser);
    const context = await browser.newContext({
      ignoreHTTPSErrors: true,
      viewport: args.viewport,
      deviceScaleFactor: 1,
    });
    await context.route("**/*", async (route) => {
      const request = route.request();
      const url = request.url();
      if (requestIsOfflineSafe(url)) {
        const parsed = new URL(url);
        if (
          request.method() === "GET" &&
          parsed.hostname.endsWith(".wikijump.localhost") &&
          parsed.pathname === "/local--favicon/favicon.gif"
        ) {
          harnessNormalizations.push({
            kind: "browser_internal_favicon_redirect_suppression",
            source_url: url,
          });
          await route.fulfill({
            status: 200,
            contentType: "image/gif",
            body: Buffer.from(
              "R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==",
              "base64",
            ),
          });
          return;
        }
        await route.continue();
        return;
      }
      const requestRecord = {
        url,
        method: request.method(),
        resource_type: request.resourceType(),
      };
      const retained =
        request.method() === "GET" ? responseFixture.lookup(url) : null;
      if (retained) {
        const replay = browserReplayPayload(retained);
        if (replay.rebased_urls > 0) {
          harnessNormalizations.push({
            kind: "redirected_stylesheet_relative_urls",
            source_url: url,
            terminal_url: retained.resolved_url,
            rewritten_count: replay.rebased_urls,
          });
        }
        replayedRequests.push(requestRecord);
        await route.fulfill({
          status: retained.status,
          headers: retained.headers,
          body: replay.body,
        });
        return;
      }
      fixtureMisses.push(requestRecord);
      await route.abort("blockedbyclient");
    });
    capture = await captureBrowserParityObservation({
      context,
      url: localUrl,
      label: "offline-local",
      index: 0,
      outputDir,
      contract,
      viewport: args.viewport,
      timeoutMs: args.timeoutMs,
      settleMs: args.settleMs,
    });
    await context.close();
  } finally {
    await browser.close();
  }

  const semantic = compareCaptureToOfflineOracle(capture, loaded.oracle);
  const actualFullPage = path.join(outputDir, capture.screenshot.path);
  const visual = await comparePngRmse(loaded.goldenPath, actualFullPage, {
    threshold: args.visualRmseThreshold,
  });
  const actualFullPageSha256 = await sha256File(actualFullPage);
  const localFailures = [
    ...(capture.failures ?? []),
    ...(capture.expected_failures ?? []),
    ...(capture.request_gate_aborts ?? []),
  ];
  const status =
    semantic.status === "pass" &&
    visual.status === "pass" &&
    fixtureMisses.length === 0 &&
    identity.executable_sha256 ===
      loaded.oracle.provenance.accepted_browser.executable_sha256
      ? "pass"
      : "fail";
  const verdict = {
    schema: "wikijump.offline_compatibility_browser_verdict.v1",
    status,
    fixture_id: loaded.oracle.fixture_id,
    local_url: localUrl,
    oracle_sha256: await sha256File(loaded.oraclePath),
    browser: identity,
    accepted_browser: loaded.oracle.provenance.accepted_browser,
    browser_identity_matches_final_zero:
      identity.executable_sha256 ===
      loaded.oracle.provenance.accepted_browser.executable_sha256,
    network: {
      guard: "scripts/run-offline-standing-browser-test.sh",
      response_fixture: {
        identity: responseFixture.identity,
        entries: responseFixture.entryCount,
        replayable_urls: responseFixture.responseCount,
      },
      replayed_external_requests: replayedRequests,
      fixture_misses: fixtureMisses,
      harness_normalizations: harnessNormalizations,
    },
    semantic,
    visual: {
      ...visual,
      expected_sha256: loaded.oracle.verified_local_golden.sha256,
      actual_sha256: actualFullPageSha256,
    },
    local_capture_failures: localFailures,
  };
  await fs.writeFile(
    path.join(outputDir, "offline-browser-oracle-verdict.json"),
    `${JSON.stringify(verdict, null, 2)}\n`,
  );
  return {verdict, outputDir};
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const {verdict, outputDir} = await runOfflineBrowserOracle(args);
  console.log(JSON.stringify({status: verdict.status, output_dir: outputDir}));
  return verdict.status === "pass" ? 0 : 1;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error?.stack ?? error?.message ?? String(error));
    return 2;
  },
});
