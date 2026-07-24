#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";

import {loadPlaywright, openBrowser} from "../src/browser-session.mjs";
import {runCliIfMain} from "../src/cli-entry.mjs";
import {
  DEFAULT_COMPUTED_STYLE_WHITELIST,
  DEFAULT_SCP9506_DESCRIPTORS,
  buildDiagnosticsRecord,
  collectDocumentMetrics,
  collectElementDiagnostics,
  collectLayoutShifts,
  collectTimingDiagnostics,
  installLayoutShiftObserver,
  installTimingObserver,
  parseViewport,
} from "../src/layout-diagnostics.mjs";

const DEFAULT_TIMEOUT_MS = 30_000;
const DEFAULT_SETTLE_MS = 1_000;

function nextArg(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

export function parseArgs(argv) {
  const args = {
    fixtureId: "EN:scp-9506",
    viewports: [],
    timeoutMs: DEFAULT_TIMEOUT_MS,
    settleMs: DEFAULT_SETTLE_MS,
    ignoreHttpsErrors: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--url") {
      args.url = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--fixture-id") {
      args.fixtureId = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(nextArg(argv, index, arg));
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
    } else if (arg === "--viewport") {
      args.viewports.push(parseViewport(nextArg(argv, index, arg)));
      index += 1;
    } else if (arg === "--timeout-ms") {
      args.timeoutMs = positiveInteger(nextArg(argv, index, arg), arg);
      index += 1;
    } else if (arg === "--settle-ms") {
      args.settleMs = nonNegativeInteger(nextArg(argv, index, arg), arg);
      index += 1;
    } else if (arg === "--ignore-https-errors") {
      args.ignoreHttpsErrors = true;
    } else if (arg === "--json") {
      args.jsonOnly = true;
    } else if (arg === "--help" || arg === "-h") {
      return {help: true};
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.url) throw new Error("--url is required");
  if (!args.outputDir) throw new Error("--output-dir is required");
  if (args.viewports.length === 0) {
    args.viewports.push({width: 1366, height: 900});
  }
  return args;
}

function positiveInteger(value, flag) {
  const number = nonNegativeInteger(value, flag);
  if (number <= 0) throw new Error(`${flag} must be a positive integer`);
  return number;
}

function nonNegativeInteger(value, flag) {
  if (!/^\d+$/u.test(String(value))) {
    throw new Error(`${flag} must be a non-negative integer`);
  }
  return Number.parseInt(value, 10);
}

export function usage() {
  return `Usage: layout-diagnostics.mjs --url URL --output-dir DIR [--fixture-id EN:scp-9506] [--viewport 1366x900 ...] [--browser-root framerail] [--browser-executable /usr/bin/google-chrome | --cdp-endpoint http://127.0.0.1:9222] [--timeout-ms 30000] [--settle-ms 1000] [--ignore-https-errors] [--json]

Writes local-only layout diagnostic JSON for a page. This is adjunct evidence for layout triage, not a V2/V3 fidelity verdict.
`;
}

async function captureViewport({browser, args, viewport}) {
  const context = await browser.newContext({
    ignoreHTTPSErrors: args.ignoreHttpsErrors,
    viewport,
  });
  const page = await context.newPage();
  const failedRequests = [];
  const consoleErrors = [];

  page.on("requestfailed", (request) => {
    failedRequests.push({
      url: request.url(),
      method: request.method(),
      failure: request.failure()?.errorText ?? "request failed",
    });
  });
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });

  try {
    await installTimingObserver(page);
    await installLayoutShiftObserver(page);
    const response = await page.goto(args.url, {waitUntil: "domcontentloaded", timeout: args.timeoutMs});
    if (args.settleMs > 0) {
      await page.waitForTimeout(args.settleMs);
    }
    const document = await collectDocumentMetrics(page);
    const elements = await collectElementDiagnostics(
      page,
      DEFAULT_SCP9506_DESCRIPTORS,
      DEFAULT_COMPUTED_STYLE_WHITELIST,
    );
    const layoutShifts = await collectLayoutShifts(page);
    const timing = await collectTimingDiagnostics(page, layoutShifts);
    return buildDiagnosticsRecord({
      fixtureId: args.fixtureId,
      url: args.url,
      viewport,
      status: response?.status() ?? 0,
      finalUrl: page.url(),
      failedRequests,
      consoleErrors,
      document,
      elements,
      layoutShifts,
      timing,
    });
  } finally {
    await page.close().catch(() => {});
    await context.close().catch(() => {});
  }
}

export async function run(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const {chromium} = loadPlaywright(args.browserRoot);
  const session = await openBrowser({
    chromium,
    cdpEndpoint: args.cdpEndpoint,
    browserExecutable: args.browserExecutable,
    ignoreHttpsErrors: args.ignoreHttpsErrors,
    createInitialContexts: false,
  });
  try {
    const records = [];
    for (const viewport of args.viewports) {
      records.push(await captureViewport({browser: session.browser, args, viewport}));
    }
    const aggregate = {
      schema: "wikijump_local_lab.layout_diagnostics_run.v1",
      generated_at: new Date().toISOString(),
      fixture_id: args.fixtureId,
      url: args.url,
      records,
      summary: {
        viewports_total: records.length,
        failed_viewports: records.filter((record) => record.verdict.summary.status === "fail").length,
        anomalies_total: records.reduce((sum, record) => sum + record.verdict.anomalies.length, 0),
      },
    };
    await fs.mkdir(args.outputDir, {recursive: true});
    const resultPath = path.join(args.outputDir, "layout-diagnostics.json");
    await fs.writeFile(resultPath, `${JSON.stringify(aggregate, null, 2)}\n`, "utf8");
    if (args.jsonOnly) {
      console.log(JSON.stringify({result_path: resultPath, summary: aggregate.summary}));
    } else {
      console.log(`wrote layout diagnostics to ${resultPath}`);
    }
    return aggregate.summary.failed_viewports === 0 ? 0 : 1;
  } finally {
    await session.close();
  }
}

await runCliIfMain(import.meta.url, run);
