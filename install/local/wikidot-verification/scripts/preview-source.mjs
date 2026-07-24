#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import {findRawSyntaxLeaks} from "../src/render-health.mjs";

const ADMIN_USER_ID = -1;
const IP_ADDRESS = "127.0.0.1";
const DEFAULT_RPC_TIMEOUT_MS = 10_000;

function parseArgs(argv) {
  const args = {
    outputDir: path.resolve(process.cwd(), "preview-source-output"),
    rpcTimeoutMs: DEFAULT_RPC_TIMEOUT_MS,
    slugPrefix: "preview-",
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    const nextValue = (flag) => {
      const value = argv[index + 1];
      if (!value || value.startsWith("--")) {
        throw new Error(`${flag} requires a value`);
      }
      index += 1;
      return value;
    };

    if (arg === "--source") {
      args.source = path.resolve(nextValue(arg));
    } else if (arg === "--manifest") {
      args.manifest = path.resolve(nextValue(arg));
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(nextValue(arg));
    } else if (arg === "--rpc-url") {
      args.rpcUrl = nextValue(arg);
    } else if (arg === "--rpc-timeout-ms") {
      const rawValue = nextValue(arg);
      if (!/^\d+$/u.test(rawValue)) {
        throw new Error("--rpc-timeout-ms must be a positive integer");
      }
      const value = Number.parseInt(rawValue, 10);
      if (!Number.isSafeInteger(value) || value <= 0) {
        throw new Error("--rpc-timeout-ms must be a positive integer");
      }
      args.rpcTimeoutMs = value;
    } else if (arg === "--site") {
      args.siteSlug = nextValue(arg);
    } else if (arg === "--slug") {
      args.slug = nextValue(arg);
    } else if (arg === "--title") {
      args.title = nextValue(arg);
    } else if (arg === "--slug-prefix") {
      args.slugPrefix = nextValue(arg);
    } else if (arg === "--json") {
      args.jsonOnly = true;
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.source) throw new Error("--source is required");
  return args;
}

function printHelpAndExit() {
  console.log(`Usage: node install/local/wikidot-verification/scripts/preview-source.mjs --source FILE [--manifest corpus-manifest.tsv] [--output-dir DIR] [--rpc-url URL] [--rpc-timeout-ms MS] [--site scp-wiki] [--slug SLUG] [--title TITLE] [--slug-prefix preview-] [--json]`);
  process.exit(0);
}

function sha256Text(text) {
  return crypto.createHash("sha256").update(text).digest("hex");
}

function splitPipe(value) {
  if (!value) return [];
  return value.split("|").map((part) => part.trim()).filter(Boolean);
}

function normalizeTags(value) {
  return splitPipe(value).filter((tag) => !tag.startsWith("_")).sort();
}

function sameTags(left = [], right = []) {
  const a = [...new Set(left)].sort();
  const b = [...new Set(right)].sort();
  return a.length === b.length && a.every((value, index) => value === b[index]);
}

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9:_-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 120) || "source";
}

function rowForSource(manifest, sourcePath) {
  if (!manifest.length) return null;
  const normalized = path.resolve(sourcePath);
  return manifest.find((row) => path.resolve(row.source_path) === normalized) ?? null;
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
  constructor(rpcUrl, timeoutMs = DEFAULT_RPC_TIMEOUT_MS) {
    this.rpcUrl = rpcUrl;
    this.timeoutMs = timeoutMs;
    this.nextId = 1;
  }

  async call(method, params = {}, context = {}) {
    const headers = { "content-type": "application/json" };
    if (context.sessionToken) headers["X-Deepwell-Session-Token"] = context.sessionToken;
    if (context.siteId) headers["X-Deepwell-Site-Id"] = String(context.siteId);
    if (context.page) headers["X-Deepwell-Page"] = String(context.page);

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), this.timeoutMs);
    let response;
    try {
      response = await fetch(this.rpcUrl, {
        method: "POST",
        headers,
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: this.nextId++,
          method,
          params,
        }),
        signal: controller.signal,
      });
    } catch (error) {
      if (error.name === "AbortError") {
        throw new Error(`JSON-RPC ${method} timed out after ${this.timeoutMs}ms for ${this.rpcUrl}`);
      }
      const cause = error.cause?.message ? ` cause=${error.cause.message}` : "";
      throw new Error(`JSON-RPC ${method} fetch failed for ${this.rpcUrl}: ${error.message}${cause}`);
    } finally {
      clearTimeout(timeout);
    }

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

async function createOrUpdatePreviewPage(client, siteId, sessionToken, preview, source) {
  const existing = await maybeGetPage(client, siteId, preview.slug);
  let parserErrors = [];
  let action = "unchanged";

  if (!existing) {
    const created = await client.call("page_create", {
      site_id: siteId,
      wikitext: source,
      title: preview.title,
      alt_title: null,
      slug: preview.slug,
      layout: "wikidot",
      revision_comments: "v5 preview-source create",
      user_id: ADMIN_USER_ID,
      ip_address: IP_ADDRESS,
      tags: preview.tags,
    });
    parserErrors = created.parser_errors ?? [];
    action = "created";
  } else if (existing.wikitext !== source || existing.title !== preview.title || !sameTags(existing.tags, preview.tags)) {
    const edited = await client.call("page_edit", {
      site_id: siteId,
      page: existing.page_id,
      last_revision_id: existing.revision_id,
      revision_comments: "v5 preview-source update",
      user_id: ADMIN_USER_ID,
      ip_address: IP_ADDRESS,
      wikitext: source,
      title: preview.title,
      tags: preview.tags,
    }, {
      sessionToken,
      siteId,
      page: preview.slug,
    });
    parserErrors = edited?.parser_errors ?? [];
    action = "edited";
  }

  const page = await maybeGetPage(client, siteId, preview.slug);
  if (!page) throw new Error(`Page missing after preview import: ${preview.slug}`);

  await client.call("page_rerender", {
    site_id: siteId,
    category_id: page.page_category_id,
    page_id: page.page_id,
  });

  const rendered = await maybeGetPage(client, siteId, preview.slug);
  if (!rendered) throw new Error(`Page missing after preview rerender: ${preview.slug}`);
  return { action, parserErrors, page: rendered };
}

function inferIncludes(source) {
  const includes = [];
  const pattern = /\[\[include\s+([^\]|\n]+)(?:[^\]]*)\]\]/gi;
  for (const match of source.matchAll(pattern)) {
    includes.push(`include:${match[1].trim().replace(/\s+/g, "")}`);
  }
  return [...new Set(includes)];
}

function inferAssets(source) {
  const assets = [];
  const urlPattern = /https?:\/\/[^\s"'\])<]+/gi;
  for (const match of source.matchAll(urlPattern)) assets.push(match[0]);
  const imagePattern = /\[\[image\s+([^\]\s]+)(?:[^\]]*)\]\]/gi;
  for (const match of source.matchAll(imagePattern)) assets.push(match[1]);
  return [...new Set(assets)];
}

function classifyPreview({ html, source, parserErrors, includes, assets, importError }) {
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

  const rawSyntaxLeaks = findRawSyntaxLeaks({html, source});
  const missingIncludes = /No such page|no such page|Missing include/i.test(html)
    ? includes
    : [];
  const missingAssets = assets.filter((asset) => asset && !html.includes(asset) && !asset.startsWith("http"));
  const warnings = [];
  const errors = [];

  if (parserErrors.length) warnings.push(`${parserErrors.length} parser warning(s)`);
  if (assets.some((asset) => /^https?:\/\//i.test(asset))) warnings.push("external asset reference(s)");
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

async function main() {
  const args = parseArgs(process.argv);
  const totalStart = performance.now();
  await fs.mkdir(args.outputDir, { recursive: true });
  await fs.mkdir(path.join(args.outputDir, "html"), { recursive: true });

  const source = await fs.readFile(args.source, "utf8");
  const sourceSha256 = sha256Text(source);
  const manifest = args.manifest ? await readTsv(args.manifest) : [];
  const manifestRow = rowForSource(manifest, args.source);
  const sourceSlug = args.slug || manifestRow?.slug || slugify(path.basename(path.dirname(args.source)) || path.basename(args.source));
  const previewSlug = `${args.slugPrefix}${slugify(sourceSlug)}`;
  const title = args.title || manifestRow?.title || sourceSlug;
  const tags = normalizeTags(manifestRow?.tags || "");
  const includes = manifestRow ? splitPipe(manifestRow.dependency_hints).filter((hint) => hint.startsWith("include:")) : inferIncludes(source);
  const assets = manifestRow ? splitPipe(manifestRow.asset_paths) : inferAssets(source);
  const rpcUrl = args.rpcUrl || process.env.WIKIDOT_VERIFY_RPC_URL || "http://127.0.0.1:2747/jsonrpc";
  const siteSlug = args.siteSlug || process.env.WIKIDOT_VERIFY_SITE_SLUG || "scp-wiki";
  const adminEmail = process.env.WIKIDOT_VERIFY_ADMIN_EMAIL || "admin@wikijump";
  const adminPassword = process.env.WIKIDOT_VERIFY_ADMIN_PASS || "wikijumpadmin1";
  const client = new DeepwellClient(rpcUrl, args.rpcTimeoutMs);
  const timings = {};
  let imported = null;
  let classification;

  try {
    const connectStart = performance.now();
    await client.call("ping", {});
    const site = await client.call("site_get", { site: siteSlug });
    const login = await client.call("login", {
      name_or_email: adminEmail,
      password: adminPassword,
      ip_address: IP_ADDRESS,
      user_agent: "wikidot-preview-source/0.1",
    });
    timings.connectMs = Math.round(performance.now() - connectStart);

    const importStart = performance.now();
    imported = await createOrUpdatePreviewPage(client, site.site_id, login.session_token, {
      slug: previewSlug,
      title,
      tags,
    }, source);
    timings.importMs = Math.round(performance.now() - importStart);

    classification = classifyPreview({
      html: imported.page.compiled_body_html || "",
      source,
      parserErrors: imported.parserErrors,
      includes,
      assets,
    });
  } catch (error) {
    classification = classifyPreview({
      html: "",
      source,
      parserErrors: [],
      includes,
      assets,
      importError: error,
    });
  }

  const html = imported?.page?.compiled_body_html || "";
  const htmlPath = html ? path.join(args.outputDir, "html", `${encodeURIComponent(previewSlug)}.html`) : "";
  if (html) await fs.writeFile(htmlPath, html);

  timings.totalMs = Math.round(performance.now() - totalStart);
  const output = {
    generatedAt: new Date().toISOString(),
    source: {
      path: args.source,
      sha256: sourceSha256,
      bytes: Buffer.byteLength(source),
      manifestMatched: Boolean(manifestRow),
      manifestIndex: manifestRow ? manifest.indexOf(manifestRow) : null,
      corpusSlug: manifestRow?.slug || null,
    },
    request: {
      rpcUrl,
      site: siteSlug,
      title,
      previewSlug,
      slugPrefix: args.slugPrefix,
      rpcTimeoutMs: args.rpcTimeoutMs,
    },
    html: {
      path: htmlPath,
      bytes: Buffer.byteLength(html),
    },
    diagnostics: {
      status: classification.status,
      severity: classification.severity,
      category: classification.category,
      warnings: classification.warnings,
      errors: classification.errors,
      parserErrors: imported?.parserErrors ?? [],
      rawSyntaxLeaks: classification.rawSyntaxLeaks,
    },
    dependencies: {
      includes,
      missingIncludes: classification.missingIncludes,
    },
    assets: {
      references: assets,
      missingAssets: classification.missingAssets,
    },
    wikijump: {
      action: imported?.action ?? "failed",
      pageId: imported?.page?.page_id ?? null,
      revisionId: imported?.page?.revision_id ?? null,
      revisionNumber: imported?.page?.revision_number ?? null,
      url: `/${previewSlug}`,
    },
    timing: timings,
  };

  const outputPath = path.join(args.outputDir, "preview-result.json");
  await writeJson(outputPath, output);
  if (!args.jsonOnly) console.log(JSON.stringify(output, null, 2));
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
