#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";

const ADMIN_USER_ID = -1;
const IP_ADDRESS = "127.0.0.1";
const RAW_SYNTAX_PATTERNS = [
  /\[\[include\b/gi,
  /\[\[module\s+ListPages\b/gi,
  /\[\[image\b/gi,
  /\[\[collapsible\b/gi,
  /\[\[tabview\b/gi,
  /\[\[\/module\]\]/gi,
  /%%content%%/gi,
];

function parseArgs(argv) {
  const args = {
    outputDir: path.resolve(process.cwd(), "corpus-render-batch"),
    offset: 0,
    batchSize: 250,
    slugPrefix: "",
    maxDependencies: 500,
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--manifest") {
      args.manifest = path.resolve(argv[++index]);
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(argv[++index]);
    } else if (arg === "--batch-size") {
      args.batchSize = Number.parseInt(argv[++index], 10);
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
    } else if (arg === "--preload-dependencies") {
      args.preloadDependencies = true;
    } else if (arg === "--max-dependencies") {
      args.maxDependencies = Number.parseInt(argv[++index], 10);
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.manifest) throw new Error("--manifest is required");
  if (!Number.isFinite(args.batchSize) || args.batchSize <= 0) args.batchSize = 250;
  if (!Number.isFinite(args.offset) || args.offset < 0) args.offset = 0;
  if (args.limit !== undefined && (!Number.isFinite(args.limit) || args.limit < 0)) {
    throw new Error("--limit must be a non-negative integer");
  }
  if (!Number.isFinite(args.maxDependencies) || args.maxDependencies < 0) args.maxDependencies = 500;
  return args;
}

function printHelpAndExit() {
  console.log(`Usage: node install/local/wikidot-verification/scripts/corpus-render-batch.mjs --manifest FILE --output-dir DIR [--offset 0] [--limit N] [--batch-size 250] [--rpc-url URL] [--site scp-wiki] [--slug-prefix PREFIX] [--preload-dependencies] [--max-dependencies 500]`);
  process.exit(0);
}

function tsv(value) {
  if (Array.isArray(value)) value = value.join("|");
  if (value === null || value === undefined) return "";
  return String(value).replace(/\t/g, " ").replace(/\r?\n/g, "\\n");
}

function splitPipe(value) {
  if (!value) return [];
  return value.split("|").map((part) => part.trim()).filter(Boolean);
}

function sha256Text(text) {
  return crypto.createHash("sha256").update(text).digest("hex");
}

function normalizeTags(value) {
  return splitPipe(value).filter((tag) => !tag.startsWith("_")).sort();
}

function sameTags(left = [], right = []) {
  const a = [...new Set(left)].sort();
  const b = [...new Set(right)].sort();
  return a.length === b.length && a.every((value, index) => value === b[index]);
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

class DeepwellClient {
  constructor(rpcUrl) {
    this.rpcUrl = rpcUrl;
    this.nextId = 1;
  }

  async call(method, params = {}, context = {}) {
    const headers = { "content-type": "application/json" };
    if (context.sessionToken) headers["X-Deepwell-Session-Token"] = context.sessionToken;
    if (context.siteId) headers["X-Deepwell-Site-Id"] = String(context.siteId);
    if (context.page) headers["X-Deepwell-Page"] = String(context.page);

    const response = await fetch(this.rpcUrl, {
      method: "POST",
      headers,
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: this.nextId++,
        method,
        params,
      }),
    });

    const bodyText = await response.text();
    let body;
    try {
      body = JSON.parse(bodyText);
    } catch {
      throw new Error(`Invalid JSON-RPC response for ${method}: HTTP ${response.status} ${bodyText.slice(0, 300)}`);
    }

    if (!response.ok || body.error) {
      const message = body.error ? JSON.stringify(body.error) : bodyText;
      throw new Error(`JSON-RPC ${method} failed: HTTP ${response.status} ${message}`);
    }

    return body.result;
  }
}

async function maybeGetPage(client, siteId, slug) {
  try {
    return await client.call("page_get", {
      site_id: siteId,
      page: slug,
      details: {
        wikitext: true,
        compiled: true,
      },
    });
  } catch (error) {
    if (String(error.message).includes("PageMissing") || String(error.message).includes("not found")) {
      return null;
    }
    throw error;
  }
}

async function createOrUpdatePage(client, siteId, sessionToken, row, source, importSlug) {
  const title = row.title || row.slug;
  const tags = normalizeTags(row.tags);
  const existing = await maybeGetPage(client, siteId, importSlug);
  let parserErrors = [];
  let action = "unchanged";

  if (!existing) {
    const created = await client.call("page_create", {
      site_id: siteId,
      wikitext: source,
      title,
      alt_title: null,
      slug: importSlug,
      layout: "wikidot",
      revision_comments: "v5 corpus render batch create",
      user_id: ADMIN_USER_ID,
      ip_address: IP_ADDRESS,
    });
    parserErrors = created.parser_errors ?? [];
    action = "created";
  } else if (existing.wikitext !== source || existing.title !== title || !sameTags(existing.tags, tags)) {
    const edited = await client.call("page_edit", {
      site_id: siteId,
      page: existing.page_id,
      last_revision_id: existing.revision_id,
      revision_comments: "v5 corpus render batch update",
      user_id: ADMIN_USER_ID,
      ip_address: IP_ADDRESS,
      wikitext: source,
      title,
      tags,
    }, {
      sessionToken,
      siteId,
      page: importSlug,
    });
    parserErrors = edited?.parser_errors ?? [];
    action = "edited";
  }

  const page = await maybeGetPage(client, siteId, importSlug);
  if (!page) throw new Error(`Page missing after import: ${importSlug}`);

  await client.call("page_rerender", {
    site_id: siteId,
    category_id: page.page_category_id,
    page_id: page.page_id,
  });

  const rendered = await maybeGetPage(client, siteId, importSlug);
  if (!rendered) throw new Error(`Page missing after rerender: ${importSlug}`);
  return { action, parserErrors, page: rendered };
}

function findRawSyntaxLeaks(html) {
  const leaks = [];
  for (const pattern of RAW_SYNTAX_PATTERNS) {
    for (const match of html.matchAll(pattern)) {
      const start = Math.max(0, match.index - 60);
      const end = Math.min(html.length, match.index + match[0].length + 60);
      leaks.push({
        pattern: pattern.source,
        text: match[0],
        context: html.slice(start, end).replace(/\s+/g, " ").trim(),
      });
      if (leaks.length >= 20) return leaks;
    }
  }
  return leaks;
}

function classifyResult({ parserErrors, html, dependencyHints, assetHints, importError }) {
  if (importError) {
    return {
      status: "failed-import",
      severity: "S4",
      category: "wikijump-import",
      warnings: [],
      errors: [String(importError.message || importError)],
      rawSyntaxLeaks: [],
      missingIncludes: [],
      missingAssets: [],
    };
  }

  if (!html) {
    return {
      status: "failed-renderer",
      severity: "S4",
      category: "ftml-renderer",
      warnings: [],
      errors: ["compiled_body_html was empty"],
      rawSyntaxLeaks: [],
      missingIncludes: [],
      missingAssets: [],
    };
  }

  const rawSyntaxLeaks = findRawSyntaxLeaks(html);
  const missingIncludes = /No such page|no such page|Missing include/i.test(html)
    ? dependencyHints.filter((hint) => hint.startsWith("include:"))
    : [];
  const missingAssets = assetHints.filter((asset) => asset && !html.includes(asset) && !asset.startsWith("http"));
  const warnings = [];
  const errors = [];

  if (parserErrors.length) warnings.push(`${parserErrors.length} parser warning(s)`);
  if (assetHints.some((asset) => /^https?:\/\//i.test(asset))) warnings.push("external asset reference(s)");
  if (rawSyntaxLeaks.length) errors.push(`${rawSyntaxLeaks.length} raw syntax leak(s)`);
  if (missingIncludes.length) errors.push(`${missingIncludes.length} missing include hint(s)`);
  if (missingAssets.length) warnings.push(`${missingAssets.length} unresolved local asset hint(s)`);

  if (rawSyntaxLeaks.length || missingIncludes.length) {
    return {
      status: "failed-renderer",
      severity: "S3",
      category: rawSyntaxLeaks.length ? "ftml-renderer" : "wikijump-include-fragment",
      warnings,
      errors,
      rawSyntaxLeaks,
      missingIncludes,
      missingAssets,
    };
  }

  if (missingAssets.length) {
    return {
      status: "unsupported-known",
      severity: "S2",
      category: "wikijump-assets-files",
      warnings,
      errors,
      rawSyntaxLeaks,
      missingIncludes,
      missingAssets,
    };
  }

  if (warnings.length) {
    return {
      status: "pass-with-warnings",
      severity: "S1",
      category: "ftml-parser",
      warnings,
      errors,
      rawSyntaxLeaks,
      missingIncludes,
      missingAssets,
    };
  }

  return {
    status: "pass",
    severity: "S0",
    category: "ftml-renderer",
    warnings,
    errors,
    rawSyntaxLeaks,
    missingIncludes,
    missingAssets,
  };
}

async function writeJson(filePath, data) {
  await fs.mkdir(path.dirname(filePath), { recursive: true });
  await fs.writeFile(filePath, JSON.stringify(data, null, 2) + "\n");
}

function compatibilityRow(result) {
  return [
    result.pageId,
    result.slug,
    result.batchId,
    result.parseStatus,
    result.renderStatus,
    result.importStatus,
    result.browserStatus,
    result.severity,
    result.category,
    result.warnings.join("|"),
    result.errors.join("|"),
    result.rawSyntaxLeaks.map((leak) => leak.text).join("|"),
    result.missingIncludes.join("|"),
    result.missingAssets.join("|"),
    result.durationMs,
    result.artifact,
    result.notes,
  ].map(tsv).join("\t");
}

function normalizeIncludeSlug(hint) {
  if (!hint.startsWith("include:")) return "";
  let value = hint.slice("include:".length).trim();
  if (!value) return "";
  value = value.replace(/^\|+/, "");
  if (value.startsWith(":scp-wiki:")) value = value.slice(":scp-wiki:".length);
  if (value.startsWith("scp-wiki:")) value = value.slice("scp-wiki:".length);
  return value.toLowerCase();
}

async function preloadDependencies({
  client,
  siteId,
  sessionToken,
  manifestBySlug,
  selected,
  slugPrefix,
  maxDependencies,
  outputDir,
}) {
  const pending = [];
  const queued = new Set();
  const selectedSlugs = new Set(selected.map((row) => row.slug));

  for (const row of selected) {
    for (const hint of splitPipe(row.dependency_hints)) {
      const slug = normalizeIncludeSlug(hint);
      if (!slug || selectedSlugs.has(slug) || queued.has(slug)) continue;
      if (!manifestBySlug.has(slug)) continue;
      queued.add(slug);
      pending.push(slug);
      if (pending.length >= maxDependencies) break;
    }
    if (pending.length >= maxDependencies) break;
  }

  const results = [];
  for (const slug of pending) {
    const row = manifestBySlug.get(slug);
    const importSlug = `${slugPrefix}${row.slug}`;
    const start = performance.now();
    try {
      const source = await fs.readFile(row.source_path, "utf8");
      const imported = await createOrUpdatePage(client, siteId, sessionToken, row, source, importSlug);
      results.push({
        slug,
        importSlug,
        status: "pass",
        action: imported.action,
        durationMs: Math.round(performance.now() - start),
        parserErrorCount: imported.parserErrors.length,
      });
    } catch (error) {
      results.push({
        slug,
        importSlug,
        status: "failed",
        action: "failed",
        durationMs: Math.round(performance.now() - start),
        error: String(error.message || error),
      });
    }
  }

  await writeJson(path.join(outputDir, "dependency-preload.json"), {
    generatedAt: new Date().toISOString(),
    requested: pending.length,
    maxDependencies,
    passed: results.filter((result) => result.status === "pass").length,
    failed: results.filter((result) => result.status === "failed").length,
    results,
  });
  return results;
}

function batchLedgerRow(summary) {
  return [
    summary.batchId,
    summary.startIndex,
    summary.endIndex,
    summary.pageCount,
    summary.command,
    summary.artifactDir,
    summary.pass,
    summary.warning,
    summary.unsupported,
    summary.failed,
    summary.skipped,
    summary.status,
    summary.next,
  ].map(tsv).join("\t");
}

async function main() {
  const args = parseArgs(process.argv);
  await fs.mkdir(args.outputDir, { recursive: true });
  await fs.mkdir(path.join(args.outputDir, "diagnostics"), { recursive: true });
  await fs.mkdir(path.join(args.outputDir, "html"), { recursive: true });

  const manifest = await readTsv(args.manifest);
  const selected = manifest.slice(args.offset, args.limit === undefined ? args.offset + args.batchSize : args.offset + args.limit);
  const manifestBySlug = new Map(manifest.map((row) => [row.slug.toLowerCase(), row]));
  const batchId = `v5-corpus-${args.offset}-${args.offset + selected.length - 1}`;
  const rpcUrl = args.rpcUrl || process.env.WIKIDOT_VERIFY_RPC_URL || "http://127.0.0.1:2747/jsonrpc";
  const siteSlug = args.siteSlug || process.env.WIKIDOT_VERIFY_SITE_SLUG || "scp-wiki";
  const adminEmail = process.env.WIKIDOT_VERIFY_ADMIN_EMAIL || "admin@wikijump";
  const adminPassword = process.env.WIKIDOT_VERIFY_ADMIN_PASS || "wikijumpadmin1";
  const client = new DeepwellClient(rpcUrl);

  await client.call("ping", {});
  const site = await client.call("site_get", { site: siteSlug });
  const login = await client.call("login", {
    name_or_email: adminEmail,
    password: adminPassword,
    ip_address: IP_ADDRESS,
    user_agent: "wikidot-corpus-render-batch/0.1",
  });

  let dependencyPreload = [];
  if (args.preloadDependencies) {
    dependencyPreload = await preloadDependencies({
      client,
      siteId: site.site_id,
      sessionToken: login.session_token,
      manifestBySlug,
      selected,
      slugPrefix: args.slugPrefix,
      maxDependencies: args.maxDependencies,
      outputDir: args.outputDir,
    });
  }

  const results = [];
  for (const [relativeIndex, row] of selected.entries()) {
    const manifestIndex = args.offset + relativeIndex;
    const start = performance.now();
    const dependencyHints = splitPipe(row.dependency_hints);
    const assetHints = splitPipe(row.asset_paths);
    const importSlug = `${args.slugPrefix}${row.slug}`;
    const source = await fs.readFile(row.source_path, "utf8");
    let imported = null;
    let classification;
    let sourceSha256 = sha256Text(source);

    try {
      imported = await createOrUpdatePage(client, site.site_id, login.session_token, row, source, importSlug);
      const html = imported.page.compiled_body_html || "";
      classification = classifyResult({
        parserErrors: imported.parserErrors,
        html,
        dependencyHints,
        assetHints,
      });

      if (html) {
        await fs.writeFile(path.join(args.outputDir, "html", `${encodeURIComponent(importSlug)}.html`), html);
      }
    } catch (error) {
      classification = classifyResult({
        parserErrors: [],
        html: "",
        dependencyHints,
        assetHints,
        importError: error,
      });
    }

    const durationMs = Math.round(performance.now() - start);
    const diagnostic = {
      pageId: row.page_id,
      slug: row.slug,
      importSlug,
      sourcePath: row.source_path,
      sourceSha256,
      manifestIndex,
      parse: {
        status: classification.warnings.some((warning) => warning.includes("parser")) ? "warning" : "pass",
        warnings: imported?.parserErrors ?? [],
        errors: [],
      },
      render: {
        status: classification.severity === "S4" || classification.severity === "S3" ? "failed" : "pass",
        htmlPath: imported?.page?.compiled_body_html ? path.join(args.outputDir, "html", `${encodeURIComponent(importSlug)}.html`) : "",
        rawSyntaxLeaks: classification.rawSyntaxLeaks,
      },
      wikijump: {
        importStatus: classification.status === "failed-import" ? "failed" : "pass",
        url: `/${importSlug}`,
        httpStatus: null,
        action: imported?.action ?? "failed",
        pageId: imported?.page?.page_id,
        revisionNumber: imported?.page?.revision_number,
      },
      dependencies: {
        includes: dependencyHints.filter((hint) => hint.startsWith("include:")),
        missingIncludes: classification.missingIncludes,
        assets: assetHints,
        missingAssets: classification.missingAssets,
      },
      constructs: splitPipe(row.construct_hints),
      severity: classification.severity,
      category: classification.category,
      durationMs,
      warnings: classification.warnings,
      errors: classification.errors,
    };
    const diagnosticPath = path.join(args.outputDir, "diagnostics", `${encodeURIComponent(importSlug)}.json`);
    await writeJson(diagnosticPath, diagnostic);

    results.push({
      pageId: row.page_id,
      slug: row.slug,
      batchId,
      parseStatus: diagnostic.parse.status,
      renderStatus: diagnostic.render.status,
      importStatus: diagnostic.wikijump.importStatus,
      browserStatus: "not-run",
      severity: classification.severity,
      category: classification.category,
      warnings: classification.warnings,
      errors: classification.errors,
      rawSyntaxLeaks: classification.rawSyntaxLeaks,
      missingIncludes: classification.missingIncludes,
      missingAssets: classification.missingAssets,
      durationMs,
      artifact: diagnosticPath,
      notes: `manifest_index:${manifestIndex};import_slug:${importSlug}`,
      status: classification.status,
    });
  }

  const counts = {
    pass: results.filter((result) => result.status === "pass").length,
    warning: results.filter((result) => result.status === "pass-with-warnings").length,
    unsupported: results.filter((result) => result.status === "unsupported-known").length,
    failed: results.filter((result) => result.status.startsWith("failed")).length,
    skipped: results.filter((result) => result.status.startsWith("skipped")).length,
  };
  const summary = {
    generatedAt: new Date().toISOString(),
    batchId,
    rpcUrl,
    siteSlug,
    offset: args.offset,
    limit: selected.length,
    batchSize: args.batchSize,
    slugPrefix: args.slugPrefix,
    dependencyPreload: {
      enabled: Boolean(args.preloadDependencies),
      requested: dependencyPreload.length,
      passed: dependencyPreload.filter((result) => result.status === "pass").length,
      failed: dependencyPreload.filter((result) => result.status === "failed").length,
    },
    manifest: args.manifest,
    outputDir: args.outputDir,
    pageCount: selected.length,
    counts,
    severityCounts: Object.fromEntries(["S0", "S1", "S2", "S3", "S4"].map((severity) => [
      severity,
      results.filter((result) => result.severity === severity).length,
    ])),
    categoryCounts: results.reduce((acc, result) => {
      acc[result.category] = (acc[result.category] || 0) + 1;
      return acc;
    }, {}),
  };

  await writeJson(path.join(args.outputDir, "batch-summary.json"), summary);
  await fs.writeFile(path.join(args.outputDir, "compatibility-results.tsv"), [
    "page_id\tslug\tbatch_id\tparse_status\trender_status\timport_status\tbrowser_status\tseverity\tcategory\twarnings\terrors\traw_syntax_leaks\tmissing_includes\tmissing_assets\tduration_ms\tartifact\tnotes",
    ...results.map(compatibilityRow),
    "",
  ].join("\n"));

  const ledger = {
    batchId,
    startIndex: args.offset,
    endIndex: args.offset + selected.length - 1,
    pageCount: selected.length,
    command: process.argv.map((part) => part.includes(" ") ? JSON.stringify(part) : part).join(" "),
    artifactDir: args.outputDir,
    ...counts,
    status: counts.failed > 0 ? "complete-with-failures" : "complete",
    next: counts.failed > 0 ? "Review S3/S4 diagnostics and fix highest-impact categories." : "Continue next deterministic batch.",
  };
  await fs.writeFile(path.join(args.outputDir, "corpus-batch-ledger.tsv"), [
    "batch_id\tstart_index\tend_index\tpage_count\tcommand\tartifact_dir\tpass\twarning\tunsupported\tfailed\tskipped\tstatus\tnext",
    batchLedgerRow(ledger),
    "",
  ].join("\n"));

  console.log(JSON.stringify(summary, null, 2));
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
