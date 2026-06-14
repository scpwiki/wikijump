#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

function parseArgs(argv) {
  const args = {
    outputDir: path.resolve(process.cwd(), "preview-batch-output"),
    offset: 0,
    limit: 100,
    slugPrefix: "preview-batch-",
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--input") {
      args.input = path.resolve(argv[++index]);
    } else if (arg === "--manifest") {
      args.manifest = path.resolve(argv[++index]);
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(argv[++index]);
    } else if (arg === "--offset") {
      args.offset = Number.parseInt(argv[++index], 10);
    } else if (arg === "--limit") {
      args.limit = Number.parseInt(argv[++index], 10);
    } else if (arg === "--rpc-url") {
      args.rpcUrl = argv[++index];
    } else if (arg === "--site") {
      args.siteSlug = argv[++index];
    } else if (arg === "--slug-prefix") {
      args.slugPrefix = argv[++index];
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.input) throw new Error("--input is required");
  if (!args.manifest) throw new Error("--manifest is required");
  if (!Number.isFinite(args.offset) || args.offset < 0) args.offset = 0;
  if (!Number.isFinite(args.limit) || args.limit < 0) args.limit = 100;
  return args;
}

function printHelpAndExit() {
  console.log(`Usage: node install/local/wikidot-verification/scripts/preview-batch.mjs --input canary-pages.tsv --manifest corpus-manifest.tsv --output-dir DIR [--offset 0] [--limit 100] [--rpc-url URL] [--site scp-wiki] [--slug-prefix preview-batch-]`);
  process.exit(0);
}

function tsv(value) {
  if (Array.isArray(value)) value = value.join("|");
  if (value === null || value === undefined) return "";
  return String(value).replace(/\t/g, " ").replace(/\r?\n/g, "\\n");
}

function slugSegment(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9:_-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 140) || "source";
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

function runPreview(commandArgs, cwd) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, commandArgs, {
      cwd,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("close", (code) => {
      resolve({ code, stdout, stderr });
    });
  });
}

function percentile(values, pct) {
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.ceil((pct / 100) * sorted.length) - 1;
  return sorted[Math.max(0, Math.min(sorted.length - 1, index))];
}

function resultRow(result) {
  return [
    result.index,
    result.pageId,
    result.slug,
    result.sourcePath,
    result.status,
    result.severity,
    result.category,
    result.htmlBytes,
    result.totalMs,
    result.importMs,
    result.connectMs,
    result.previewSlug,
    result.resultPath,
    result.htmlPath,
    result.warningCount,
    result.errorCount,
    result.missingIncludeCount,
    result.missingAssetCount,
    result.rawSyntaxLeakCount,
    result.notes,
  ].map(tsv).join("\t");
}

async function main() {
  const args = parseArgs(process.argv);
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const repoRoot = path.resolve(scriptDir, "../../../..");
  const previewScript = path.join(repoRoot, "install/local/wikidot-verification/scripts/preview-source.mjs");
  await fs.mkdir(args.outputDir, { recursive: true });
  await fs.mkdir(path.join(args.outputDir, "pages"), { recursive: true });

  const rows = await readTsv(args.input);
  const selected = rows.slice(args.offset, args.offset + args.limit);
  const rpcUrl = args.rpcUrl || process.env.WIKIDOT_VERIFY_RPC_URL || "http://127.0.0.1:2747/jsonrpc";
  const siteSlug = args.siteSlug || process.env.WIKIDOT_VERIFY_SITE_SLUG || "scp-wiki";
  const results = [];

  for (const [relativeIndex, row] of selected.entries()) {
    const index = args.offset + relativeIndex;
    const pageOutputDir = path.join(args.outputDir, "pages", `${String(index).padStart(4, "0")}-${slugSegment(row.slug)}`);
    const commandArgs = [
      previewScript,
      "--source", row.source_path,
      "--manifest", args.manifest,
      "--output-dir", pageOutputDir,
      "--rpc-url", rpcUrl,
      "--site", siteSlug,
      "--slug-prefix", args.slugPrefix,
      "--json",
    ];
    const run = await runPreview(commandArgs, repoRoot);
    const resultPath = path.join(pageOutputDir, "preview-result.json");
    let preview = null;
    let notes = "";

    if (run.code === 0) {
      try {
        preview = JSON.parse(await fs.readFile(resultPath, "utf8"));
      } catch (error) {
        notes = `preview-result read failed: ${error.message}`;
      }
    } else {
      notes = `preview-source exit ${run.code}: ${run.stderr.slice(0, 500)}`;
    }

    results.push({
      index,
      pageId: row.page_id,
      slug: row.slug,
      sourcePath: row.source_path,
      status: preview?.diagnostics?.status ?? "failed-cli",
      severity: preview?.diagnostics?.severity ?? "S4",
      category: preview?.diagnostics?.category ?? "preview-cli",
      htmlBytes: preview?.html?.bytes ?? 0,
      totalMs: preview?.timing?.totalMs ?? null,
      importMs: preview?.timing?.importMs ?? null,
      connectMs: preview?.timing?.connectMs ?? null,
      previewSlug: preview?.request?.previewSlug ?? "",
      resultPath,
      htmlPath: preview?.html?.path ?? "",
      warningCount: preview?.diagnostics?.warnings?.length ?? 0,
      errorCount: preview?.diagnostics?.errors?.length ?? 1,
      missingIncludeCount: preview?.dependencies?.missingIncludes?.length ?? 0,
      missingAssetCount: preview?.assets?.missingAssets?.length ?? 0,
      rawSyntaxLeakCount: preview?.diagnostics?.rawSyntaxLeaks?.length ?? 0,
      notes,
    });
  }

  const timingValues = results.map((result) => result.totalMs).filter((value) => Number.isFinite(value));
  const severityCounts = Object.fromEntries(["S0", "S1", "S2", "S3", "S4"].map((severity) => [
    severity,
    results.filter((result) => result.severity === severity).length,
  ]));
  const statusCounts = Object.fromEntries([...new Set(results.map((result) => result.status))]
    .sort()
    .map((status) => [status, results.filter((result) => result.status === status).length]));

  const header = [
    "index",
    "page_id",
    "slug",
    "source_path",
    "status",
    "severity",
    "category",
    "html_bytes",
    "total_ms",
    "import_ms",
    "connect_ms",
    "preview_slug",
    "result_path",
    "html_path",
    "warning_count",
    "error_count",
    "missing_include_count",
    "missing_asset_count",
    "raw_syntax_leak_count",
    "notes",
  ].join("\t");
  await fs.writeFile(path.join(args.outputDir, "preview-results.tsv"), `${header}\n${results.map(resultRow).join("\n")}\n`);

  const summary = {
    generatedAt: new Date().toISOString(),
    input: args.input,
    manifest: args.manifest,
    outputDir: args.outputDir,
    rpcUrl,
    siteSlug,
    offset: args.offset,
    limit: args.limit,
    pageCount: results.length,
    statusCounts,
    severityCounts,
    timing: {
      count: timingValues.length,
      p50Ms: percentile(timingValues, 50),
      p95Ms: percentile(timingValues, 95),
      minMs: timingValues.length ? Math.min(...timingValues) : null,
      maxMs: timingValues.length ? Math.max(...timingValues) : null,
    },
  };
  await fs.writeFile(path.join(args.outputDir, "preview-summary.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(JSON.stringify(summary, null, 2));
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
