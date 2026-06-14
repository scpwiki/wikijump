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
    baseUrl: process.env.WIKIDOT_VERIFY_BASE_URL || "https://scpwiki.localhost",
    outputDir: path.resolve(process.cwd(), "corpus-authoring-proof"),
    offset: 0,
    limit: 10,
    timeoutMs: 45000,
    slugPrefix: "ui-corpus-",
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
    } else if (arg === "--slug-prefix") {
      args.slugPrefix = argv[++index];
    } else if (arg === "--asset-file") {
      args.assetFile = path.resolve(argv[++index]);
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
  if (!Number.isFinite(args.limit) || args.limit < 0) args.limit = 10;
  if (!Number.isFinite(args.timeoutMs) || args.timeoutMs <= 0) args.timeoutMs = 45000;
  return args;
}

function printHelpAndExit() {
  console.log("Usage: node install/local/wikidot-verification/scripts/browser-authoring-corpus-workflow.mjs --input canary-pages.tsv --output-dir DIR [--base-url URL] [--offset 0] [--limit 10] [--timeout-ms 45000] [--slug-prefix ui-corpus-] [--asset-file FILE] [--headed]");
  process.exit(0);
}

function tsv(value) {
  if (Array.isArray(value)) value = value.join("|");
  if (value === null || value === undefined) return "";
  return String(value).replace(/\t/g, " ").replace(/\r?\n/g, "\\n");
}

function slugSegment(value) {
  return String(value || "source")
    .toLowerCase()
    .replace(/[^a-z0-9:_-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 100) || "source";
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

function splitPipe(value) {
  if (!value) return [];
  return value.split("|").map((part) => part.trim()).filter(Boolean);
}

async function readSource(row) {
  if (!row.source_path) throw new Error(`Missing source_path for ${row.slug || row.page_id}`);
  return fs.readFile(row.source_path, "utf8");
}

async function fillSourceForm(page, input) {
  await page.getByTestId("slug-input").fill(input.slug);
  await page.getByTestId("title-input").fill(input.title);
  await page.getByTestId("tags-input").fill(input.tags);
  await page.getByTestId("parent-input").fill(input.parent);
  await page.getByTestId("source-input").fill(input.source);
}

async function clickAndWait(page, testId, timeoutMs) {
  await Promise.all([
    page.waitForLoadState("domcontentloaded", { timeout: timeoutMs }).catch(() => {}),
    page.getByTestId(testId).click({ timeout: timeoutMs }),
  ]);
}

async function runWorkflow(browser, row, args, dirs, absoluteIndex) {
  const page = await browser.newPage({ viewport: { width: 1366, height: 900 } });
  page.setDefaultTimeout(args.timeoutMs);
  const source = await readSource(row);
  const slug = `${args.slugPrefix}${String(absoluteIndex).padStart(4, "0")}-${slugSegment(row.slug)}`;
  const title = row.title || row.slug || slug;
  const tags = ["v5-ui-proof", "real-corpus", ...splitPipe(row.tags).filter((tag) => !tag.startsWith("_")).slice(0, 6)].join(" ");
  const fileBase = `${String(absoluteIndex).padStart(4, "0")}-${slugSegment(row.slug)}`;
  const labUrl = `${args.baseUrl}/__local-wikidot-verify?slug=${encodeURIComponent(slug)}`;
  const stages = [];
  const record = (name, ok, detail = "") => stages.push({ name, ok, detail });
  const start = performance.now();
  let status = "failed-ui";
  let renderedUrl = `${args.baseUrl}/${slug}`;
  let error = "";
  let previewTextBytes = 0;
  let currentTextBytes = 0;
  let uploadedAsset = false;

  try {
    const response = await page.goto(labUrl, { waitUntil: "domcontentloaded", timeout: args.timeoutMs });
    record("open-lab", Boolean(response && response.ok()), response ? String(response.status()) : "no response");

    await fillSourceForm(page, {
      slug,
      title,
      tags,
      parent: "",
      source,
    });
    record("fill-source", true);

    await clickAndWait(page, "preview-button", args.timeoutMs);
    const previewHtml = await page.getByTestId("preview-html").innerText({ timeout: args.timeoutMs }).catch(() => "");
    previewTextBytes = Buffer.byteLength(previewHtml);
    record("preview", previewTextBytes > 0, `bytes:${previewTextBytes}`);

    await fillSourceForm(page, {
      slug,
      title,
      tags,
      parent: "",
      source: `${source}\n\n[[>]]\n//v5 UI proof edit ${absoluteIndex}//\n[[/<]]\n`,
    });
    await clickAndWait(page, "save-button", args.timeoutMs);
    const statusText = await page.getByTestId("lab-status").innerText({ timeout: args.timeoutMs }).catch(() => "");
    record("save", /savePage/.test(statusText), statusText);

    await page.getByTestId("tag-editor-input").fill(`${tags} edited`);
    await clickAndWait(page, "tag-save-button", args.timeoutMs);
    const tagText = await page.getByTestId("current-tags").innerText({ timeout: args.timeoutMs }).catch(() => "");
    record("tag-update", /edited/.test(tagText), tagText);

    if (args.assetFile) {
      await page.getByTestId("file-name-input").fill(path.basename(args.assetFile));
      await page.getByTestId("file-input").setInputFiles(args.assetFile);
      await clickAndWait(page, "file-upload-button", args.timeoutMs);
      const fileResult = await page.getByTestId("file-result").innerText({ timeout: args.timeoutMs }).catch(() => "");
      uploadedAsset = fileResult.length > 0;
      record("asset-upload", uploadedAsset, fileResult.slice(0, 300));
    }

    const renderedResponse = await page.goto(renderedUrl, { waitUntil: "domcontentloaded", timeout: args.timeoutMs });
    const bodyText = await page.locator("body").innerText({ timeout: args.timeoutMs }).catch(() => "");
    currentTextBytes = Buffer.byteLength(bodyText);
    record("open-rendered", Boolean(renderedResponse && renderedResponse.ok()) && currentTextBytes > 0, `${renderedResponse?.status() ?? "no response"} bytes:${currentTextBytes}`);
    status = stages.every((stage) => stage.ok) ? "pass" : "failed-ui";
  } catch (caught) {
    error = caught.stack || caught.message || String(caught);
    record("exception", false, error.slice(0, 500));
  } finally {
    await page.screenshot({ path: path.join(dirs.screenshots, `${fileBase}.png`), fullPage: true }).catch(() => {});
    await page.close().catch(() => {});
  }

  const detail = {
    index: absoluteIndex,
    sourceSlug: row.slug,
    sourcePath: row.source_path,
    proofSlug: slug,
    labUrl,
    renderedUrl,
    status,
    ok: status === "pass",
    stages,
    previewTextBytes,
    renderedTextBytes: currentTextBytes,
    uploadedAsset,
    durationMs: Math.round(performance.now() - start),
    error,
  };
  const detailPath = path.join(dirs.details, `${fileBase}.json`);
  await fs.writeFile(detailPath, JSON.stringify(detail, null, 2) + "\n");
  return { ...detail, detailPath };
}

function resultRow(result) {
  return [
    result.index,
    result.sourceSlug,
    result.proofSlug,
    result.status,
    result.ok ? "PASS" : "FAIL",
    result.previewTextBytes,
    result.renderedTextBytes,
    result.uploadedAsset,
    result.durationMs,
    result.detailPath,
    result.stages.filter((stage) => !stage.ok).map((stage) => `${stage.name}:${stage.detail}`).join("|"),
  ].map(tsv).join("\t");
}

async function main() {
  const args = parseArgs(process.argv);
  const rows = await readTsv(args.input);
  const selected = rows.slice(args.offset, args.offset + args.limit);
  const dirs = {
    details: path.join(args.outputDir, "details"),
    screenshots: path.join(args.outputDir, "screenshots"),
  };
  await fs.mkdir(dirs.details, { recursive: true });
  await fs.mkdir(dirs.screenshots, { recursive: true });

  const browser = await chromium.launch({ headless: !args.headed });
  const results = [];
  try {
    for (const [relativeIndex, row] of selected.entries()) {
      results.push(await runWorkflow(browser, row, args, dirs, args.offset + relativeIndex));
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
    assetFile: args.assetFile || null,
    pageCount: results.length,
    passed: results.filter((result) => result.ok).length,
    failed: results.filter((result) => !result.ok).length,
    statusCounts: Object.fromEntries([...new Set(results.map((result) => result.status))]
      .sort()
      .map((status) => [status, results.filter((result) => result.status === status).length])),
  };

  await fs.writeFile(path.join(args.outputDir, "authoring-results.tsv"), [
    "index\tsource_slug\tproof_slug\tstatus\tok\tpreview_text_bytes\trendered_text_bytes\tuploaded_asset\tduration_ms\tdetail_path\tfailed_stages",
    ...results.map(resultRow),
    "",
  ].join("\n"));
  await fs.writeFile(path.join(args.outputDir, "authoring-summary.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(JSON.stringify(summary, null, 2));
  if (summary.failed > 0) process.exit(1);
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
