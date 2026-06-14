#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "../../../..");
const chromium = loadPlaywrightChromium();

function loadPlaywrightChromium() {
  const candidates = [
    { base: path.join(repoRoot, "framerail", "package.json"), names: ["@playwright/test", "playwright"] },
    { base: "/home/roku/.npm-global/lib/node_modules/playwright/package.json", names: ["playwright"] },
    { base: path.join(process.env.HOME || "/home/roku", ".npm-global/lib/node_modules/playwright/package.json"), names: ["playwright"] },
  ];

  const failures = [];
  for (const candidate of candidates) {
    try {
      const candidateRequire = createRequire(candidate.base);
      for (const name of candidate.names) {
        try {
          const mod = candidateRequire(name);
          if (mod.chromium) return mod.chromium;
        } catch (error) {
          failures.push(`${candidate.base} -> ${name}: ${error.code || error.message}`);
        }
      }
    } catch (error) {
      failures.push(`${candidate.base}: ${error.code || error.message}`);
    }
  }

  throw new Error(`Unable to load Playwright chromium. Tried:\n${failures.join("\n")}`);
}

function parseArgs(argv) {
  const args = {
    baseUrl: process.env.WIKIDOT_VERIFY_BASE_URL || "http://scp-wiki.wikijump.localhost:18443",
    outputDir: path.resolve(process.cwd(), "corpus-browser-proof"),
    offset: 0,
    limit: 100,
    timeoutMs: 45000,
    slugColumn: null,
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--input") {
      args.input = path.resolve(argv[++index]);
    } else if (arg === "--base-url") {
      args.baseUrl = argv[++index].replace(/\/$/, "");
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(argv[++index]);
    } else if (arg === "--offset") {
      args.offset = Number.parseInt(argv[++index], 10);
    } else if (arg === "--limit") {
      args.limit = Number.parseInt(argv[++index], 10);
    } else if (arg === "--timeout-ms") {
      args.timeoutMs = Number.parseInt(argv[++index], 10);
    } else if (arg === "--slug-column") {
      args.slugColumn = argv[++index];
    } else if (arg === "--headed") {
      args.headed = true;
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.input) throw new Error("--input is required");
  if (!Number.isFinite(args.offset) || args.offset < 0) args.offset = 0;
  if (!Number.isFinite(args.limit) || args.limit < 0) args.limit = 100;
  if (!Number.isFinite(args.timeoutMs) || args.timeoutMs <= 0) args.timeoutMs = 45000;
  return args;
}

function printHelpAndExit() {
  console.log("Usage: node install/local/wikidot-verification/scripts/browser-proof-corpus-pages.mjs --input preview-results.tsv --output-dir DIR [--base-url URL] [--offset 0] [--limit 100] [--timeout-ms 45000] [--slug-column preview_slug] [--headed]");
  process.exit(0);
}

function tsv(value) {
  if (Array.isArray(value)) value = value.join("|");
  if (value === null || value === undefined) return "";
  return String(value).replace(/\t/g, " ").replace(/\r?\n/g, "\\n");
}

function safeName(value) {
  return String(value || "page").replace(/[^a-zA-Z0-9._:-]+/g, "_").slice(0, 180);
}

async function readTsv(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  const lines = text.split(/\r?\n/).filter(Boolean);
  const header = lines.shift()?.split("\t") ?? [];
  return lines.map((line) => {
    const cells = line.split("\t");
    const row = {};
    header.forEach((key, index) => {
      row[key] = cells[index] ?? "";
    });
    return row;
  });
}

function slugFromRow(row, slugColumn) {
  if (slugColumn && row[slugColumn]) return row[slugColumn];
  if (row.preview_slug) return row.preview_slug;
  if (row.import_slug) return row.import_slug;
  if (row.notes) {
    const match = row.notes.match(/(?:^|;)import_slug:([^;]+)/);
    if (match) return match[1];
  }
  return row.slug || "";
}

async function runBrowserProof(browser, row, args, dirs, absoluteIndex) {
  const renderedSlug = slugFromRow(row, args.slugColumn);
  const url = renderedSlug ? `${args.baseUrl}/${renderedSlug}` : "";
  const fileBase = `${String(absoluteIndex).padStart(4, "0")}-${safeName(renderedSlug || row.slug || "missing-slug")}`;
  const page = await browser.newPage({ viewport: { width: 1366, height: 900 } });
  const network = {
    requests: [],
    failedRequests: [],
    badResponses: [],
  };
  const consoleMessages = [];

  page.on("request", (request) => {
    network.requests.push({ method: request.method(), url: request.url(), resourceType: request.resourceType() });
  });
  page.on("requestfailed", (request) => {
    network.failedRequests.push({
      method: request.method(),
      url: request.url(),
      resourceType: request.resourceType(),
      failure: request.failure()?.errorText || "unknown",
    });
  });
  page.on("response", (response) => {
    if (response.status() >= 400) {
      network.badResponses.push({ status: response.status(), url: response.url() });
    }
  });
  page.on("console", (message) => {
    if (["error", "warning"].includes(message.type())) {
      consoleMessages.push({ type: message.type(), text: message.text() });
    }
  });

  const start = performance.now();
  let status = "failed-browser";
  let ok = false;
  let httpStatus = null;
  let bodyText = "";
  let title = "";
  let error = "";
  try {
    if (!renderedSlug) throw new Error("No rendered slug found; provide --slug-column or preview/import rows.");
    const response = await page.goto(url, { waitUntil: "domcontentloaded", timeout: args.timeoutMs });
    httpStatus = response?.status() ?? null;
    title = await page.title().catch(() => "");
    bodyText = await page.locator("body").innerText({ timeout: Math.min(10000, args.timeoutMs) }).catch(() => "");
    await page.screenshot({ path: path.join(dirs.screenshots, `${fileBase}.png`), fullPage: true });

    const hasErrorPage = /Internal Server Error|Application error|Not Found|Page not found/i.test(bodyText);
    ok = Boolean(response && response.ok()) &&
      bodyText.trim().length > 0 &&
      !hasErrorPage &&
      network.failedRequests.length === 0 &&
      network.badResponses.length === 0;
    status = ok ? "pass" : "failed-browser";
  } catch (caught) {
    error = caught.stack || caught.message || String(caught);
    await page.screenshot({ path: path.join(dirs.screenshots, `${fileBase}-error.png`), fullPage: true }).catch(() => {});
  } finally {
    await page.close().catch(() => {});
  }

  const durationMs = Math.round(performance.now() - start);
  const detail = {
    index: absoluteIndex,
    sourceSlug: row.slug || "",
    renderedSlug,
    url,
    status,
    ok,
    httpStatus,
    title,
    bodyTextBytes: Buffer.byteLength(bodyText),
    bodyTextSample: bodyText.slice(0, 2000),
    durationMs,
    failedRequests: network.failedRequests,
    badResponses: network.badResponses,
    requestCount: network.requests.length,
    consoleMessages,
    error,
  };
  const detailPath = path.join(dirs.network, `${fileBase}.json`);
  await fs.writeFile(detailPath, JSON.stringify(detail, null, 2) + "\n");

  return {
    ...detail,
    detailPath,
    failedRequestCount: network.failedRequests.length,
    badResponseCount: network.badResponses.length,
    consoleMessageCount: consoleMessages.length,
  };
}

function resultRow(result) {
  return [
    result.index,
    result.sourceSlug,
    result.renderedSlug,
    result.url,
    result.status,
    result.ok ? "PASS" : "FAIL",
    result.httpStatus,
    result.bodyTextBytes,
    result.durationMs,
    result.failedRequestCount,
    result.badResponseCount,
    result.consoleMessageCount,
    result.detailPath,
    result.error,
  ].map(tsv).join("\t");
}

async function main() {
  const args = parseArgs(process.argv);
  const rows = await readTsv(args.input);
  const selected = rows.slice(args.offset, args.offset + args.limit);
  const dirs = {
    screenshots: path.join(args.outputDir, "screenshots"),
    network: path.join(args.outputDir, "network"),
  };
  await fs.mkdir(dirs.screenshots, { recursive: true });
  await fs.mkdir(dirs.network, { recursive: true });

  const browser = await chromium.launch({ headless: !args.headed });
  const results = [];
  try {
    for (const [relativeIndex, row] of selected.entries()) {
      results.push(await runBrowserProof(browser, row, args, dirs, args.offset + relativeIndex));
    }
  } finally {
    await browser.close();
  }

  const summary = {
    generatedAt: new Date().toISOString(),
    input: args.input,
    outputDir: args.outputDir,
    baseUrl: args.baseUrl,
    offset: args.offset,
    limit: args.limit,
    timeoutMs: args.timeoutMs,
    pageCount: results.length,
    passed: results.filter((result) => result.ok).length,
    failed: results.filter((result) => !result.ok).length,
    statusCounts: Object.fromEntries([...new Set(results.map((result) => result.status))]
      .sort()
      .map((status) => [status, results.filter((result) => result.status === status).length])),
  };

  await fs.writeFile(path.join(args.outputDir, "browser-results.tsv"), [
    "index\tsource_slug\trendered_slug\turl\tstatus\tok\thttp_status\tbody_text_bytes\tduration_ms\tfailed_requests\tbad_responses\tconsole_messages\tdetail_path\terror",
    ...results.map(resultRow),
    "",
  ].join("\n"));
  await fs.writeFile(path.join(args.outputDir, "browser-summary.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(JSON.stringify(summary, null, 2));
  if (summary.failed > 0) process.exit(1);
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
