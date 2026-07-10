#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";

const DEFAULT_CORPUS = "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/en";
const CANARY_COUNT = 100;

function parseArgs(argv) {
  const args = {
    corpus: DEFAULT_CORPUS,
    outputDir: path.resolve(process.cwd(), "corpus-discovery"),
    canaryCount: CANARY_COUNT,
  };

  function nextValue(index, optionName) {
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`Missing value for ${optionName}`);
    }
    return value;
  }

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--corpus") {
      args.corpus = path.resolve(nextValue(index, arg));
      index += 1;
    } else if (arg === "--output-dir") {
      args.outputDir = path.resolve(nextValue(index, arg));
      index += 1;
    } else if (arg === "--canary-count") {
      const raw = nextValue(index, arg);
      if (!/^[1-9]\d*$/.test(raw)) {
        throw new Error("--canary-count must be a positive integer");
      }
      args.canaryCount = Number(raw);
      index += 1;
    } else if (arg === "--help") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!Number.isInteger(args.canaryCount) || args.canaryCount <= 0) {
    throw new Error("--canary-count must be a positive integer");
  }
  return args;
}

function printHelpAndExit() {
  console.log(`Usage: node install/local/wikidot-verification/scripts/corpus-discover.mjs --corpus DIR --output-dir DIR [--canary-count 100]`);
  process.exit(0);
}

async function pathExists(filePath) {
  try {
    await fs.lstat(filePath);
    return true;
  } catch {
    return false;
  }
}

function isPathInside(root, filePath) {
  const relativePath = path.relative(root, filePath);
  return relativePath === "" || (!relativePath.startsWith("..") && !path.isAbsolute(relativePath));
}

async function readCorpusFile(corpusRoot, filePath, { optional = false, maxBytes = 1024 * 1024 } = {}) {
  let stats;
  try {
    stats = await fs.lstat(filePath);
  } catch (error) {
    if (optional && error.code === "ENOENT") return null;
    throw error;
  }

  if (!stats.isFile()) {
    throw new Error(`Corpus path must be a regular file: ${filePath}`);
  }
  if (stats.size > maxBytes) {
    throw new Error(`Corpus file exceeds ${maxBytes} byte limit: ${filePath}`);
  }

  const [realCorpusRoot, realFilePath] = await Promise.all([
    fs.realpath(corpusRoot),
    fs.realpath(filePath),
  ]);
  if (!isPathInside(realCorpusRoot, realFilePath)) {
    throw new Error(`Corpus file escapes corpus root: ${filePath}`);
  }

  return fs.readFile(filePath, "utf8");
}

async function readJsonIfPresent(corpusRoot, filePath) {
  const text = await readCorpusFile(corpusRoot, filePath, { optional: true });
  return text === null ? null : JSON.parse(text);
}

async function readTextIfPresent(corpusRoot, filePath, options) {
  return (await readCorpusFile(corpusRoot, filePath, { optional: true, ...options })) ?? "";
}

function validateEntityId(entityId, sourceLabel) {
  if (!entityId) return "";
  if (!/^[A-Za-z0-9_-]{1,128}$/.test(entityId)) {
    throw new Error(`Invalid entity id value in ${sourceLabel}`);
  }
  return entityId;
}

function entityIdFromText(text, filePath) {
  return validateEntityId(text.trim(), filePath);
}

function entityIdFromMeta(meta, filePath) {
  return validateEntityId(String(meta?.entity_id ?? "").trim(), filePath);
}

async function walkFiles(root) {
  const output = [];
  const stack = [root];

  while (stack.length) {
    const current = stack.pop();
    const entries = await fs.readdir(current, { withFileTypes: true });
    entries.sort((left, right) => left.name.localeCompare(right.name));

    for (const entry of entries) {
      const fullPath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(fullPath);
      } else if (entry.isFile()) {
        output.push(fullPath);
      }
    }
  }

  return output.sort();
}

function relativeUnix(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join("/");
}

function tsv(value) {
  if (Array.isArray(value)) value = value.join("|");
  if (value === null || value === undefined) return "";
  return String(value).replace(/\t/g, " ").replace(/\r?\n/g, "\\n");
}

function extensionFor(relativePath) {
  if (relativePath.endsWith(".wikidot.txt")) return ".wikidot.txt";
  return path.extname(relativePath) || "";
}

function classifyFile(relativePath) {
  if (relativePath === "index.json") return "metadata";
  if (/^pages\/[^/]+\/source\.wikidot\.txt$/.test(relativePath)) return "page-source";
  if (/^pages\/[^/]+\/meta\.json$/.test(relativePath)) return "metadata";
  if (/^pages\/[^/]+\/entity_id\.txt$/.test(relativePath)) return "metadata";
  if (/^by-uuid\/[^/]+\/observations\.ndjson$/.test(relativePath)) return "metadata";
  if (/^posts\/[^/]+\/state\.json$/.test(relativePath)) return "metadata";
  if (/^posts\/[^/]+\/comments\.ndjson$/.test(relativePath)) return "metadata";
  if (/\.(png|jpe?g|gif|svg|webp|ico)$/i.test(relativePath)) return "image";
  if (/\.(css|less|scss)$/i.test(relativePath)) return "style";
  if (/\.(woff2?|ttf|otf|eot)$/i.test(relativePath)) return "asset";
  if (/\.(json|ndjson|txt|md|html|xml|csv|tsv)$/i.test(relativePath)) return "unknown-text";
  return "unknown-binary";
}

function pageSlugGuess(relativePath) {
  const match = relativePath.match(/^pages\/([^/]+)\//);
  return match ? match[1] : "";
}

function hashNumber(value) {
  const hash = crypto.createHash("sha256").update(value).digest("hex").slice(0, 12);
  return Number.parseInt(hash, 16);
}

function uniqueSorted(values) {
  return [...new Set(values.filter(Boolean))].sort();
}

function collectRegex(text, regex, mapper = (match) => match[1]) {
  const values = [];
  for (const match of text.matchAll(regex)) values.push(mapper(match));
  return uniqueSorted(values);
}

function detectConstructs(source, slug, meta) {
  const hints = [];
  const lowerSlug = slug.toLowerCase();

  if (/\[\[include\b/i.test(source)) hints.push("include");
  if (/\[\[module\s+ListPages\b/i.test(source)) hints.push("module-listpages");
  if (/\[\[module\s+(?!ListPages\b)[^\]]+/i.test(source)) hints.push("module-other");
  if (/\[\[image\b/i.test(source) || /<img\b/i.test(source)) hints.push("image");
  if (/\[\[module\s+CSS\b/i.test(source)) hints.push("css-module");
  if (/<style\b/i.test(source) || /@import\s+url/i.test(source)) hints.push("style-block");
  if (/\[\[collapsible\b/i.test(source)) hints.push("collapsible");
  if (/\[\[tabview\b|\[\[tab\b/i.test(source)) hints.push("tabs");
  if (/\|\|/.test(source)) hints.push("table");
  if (/@@|``|<code\b|\[\[code\b/i.test(source)) hints.push("code-block");
  if (/\[\[math\b|\[math\]|\$\$/.test(source)) hints.push("math");
  if (/\[\[footnote\b/i.test(source)) hints.push("footnote");
  if (/<iframe\b|<embed\b|<object\b|<html\b|\[\[html\b/i.test(source)) hints.push("iframe/html/embed");
  if (/^(fragment|component):/.test(lowerSlug) || /\[\[include\s+[^]]*(fragment|component):/i.test(source)) hints.push("fragment/component");
  if (/\[\[[^\]]+\]\]|https?:\/\//i.test(source)) hints.push("links");
  if (slug.includes(":") || slug.includes("/")) hints.push("category/path");
  if (Array.isArray(meta?.tags) && meta.tags.length) hints.push("metadata/tags");
  if (/^(nav|theme):/i.test(slug)) hints.push("navigation");

  return uniqueSorted(hints);
}

function extractDependencyHints(source) {
  const includes = collectRegex(source, /\[\[include\s+([^\]\s|]+)[^\]]*\]\]/gi);
  const moduleRefs = collectRegex(source, /\[\[module\s+([A-Za-z0-9_-]+)/g, (match) => `module:${match[1]}`);
  return uniqueSorted([...includes.map((value) => `include:${value}`), ...moduleRefs]);
}

function extractAssetHints(source) {
  const imageRefs = collectRegex(source, /\[\[image\s+([^\]\s|]+)[^\]]*\]\]/gi);
  const htmlImages = collectRegex(source, /<img[^>]+src=["']([^"']+)["']/gi);
  const cssUrls = collectRegex(source, /url\(["']?([^"')]+)["']?\)/gi);
  return uniqueSorted([...imageRefs, ...htmlImages, ...cssUrls]);
}

function inferTitle(slug, meta) {
  return meta?.title_shown || meta?.title || meta?.fullname || slug;
}

function statusNotes(meta, entityId) {
  const notes = [];
  if (meta?.status) notes.push(`status:${meta.status}`);
  if (meta?.parent_fullname) notes.push(`parent:${meta.parent_fullname}`);
  if (!entityId) notes.push("missing-entity-id");
  return notes;
}

async function buildFileInventory(corpusRoot, files) {
  const rows = [];
  const counts = new Map();

  for (const filePath of files) {
    const stats = await fs.stat(filePath);
    const relativePath = relativeUnix(corpusRoot, filePath);
    const candidateType = classifyFile(relativePath);
    counts.set(candidateType, (counts.get(candidateType) || 0) + 1);
    rows.push({
      path: filePath,
      size_bytes: stats.size,
      extension: extensionFor(relativePath),
      candidate_type: candidateType,
      page_slug_guess: pageSlugGuess(relativePath),
      asset_guess: ["asset", "image", "style"].includes(candidateType) ? filePath : "",
      metadata_guess: candidateType === "metadata" ? filePath : "",
      notes: relativePath,
    });
  }

  return { rows, counts };
}

async function buildManifest(corpusRoot) {
  const pagesRoot = path.join(corpusRoot, "pages");
  if (!(await pathExists(pagesRoot))) return [];
  const entries = await fs.readdir(pagesRoot, { withFileTypes: true });
  const pageDirs = entries.filter((entry) => entry.isDirectory()).map((entry) => entry.name).sort();
  const rows = [];

  for (const slug of pageDirs) {
    const pageDir = path.join(pagesRoot, slug);
    const sourcePath = path.join(pageDir, "source.wikidot.txt");
    if (!(await pathExists(sourcePath))) continue;

    const metadataPath = path.join(pageDir, "meta.json");
    const entityIdPath = path.join(pageDir, "entity_id.txt");
    const [source, meta, entityIdText] = await Promise.all([
      readCorpusFile(corpusRoot, sourcePath, { maxBytes: 10 * 1024 * 1024 }),
      readJsonIfPresent(corpusRoot, metadataPath),
      readTextIfPresent(corpusRoot, entityIdPath, { maxBytes: 256 }),
    ]);

    const pageId = entityIdFromText(entityIdText, entityIdPath) || entityIdFromMeta(meta, metadataPath);
    const bytes = Buffer.byteLength(source);
    const lineCount = source.length ? source.split(/\r?\n/).length : 0;
    const tags = uniqueSorted(Array.isArray(meta?.tags) ? meta.tags : []);
    const constructHints = detectConstructs(source, slug, meta);
    const dependencyHints = extractDependencyHints(source);
    const assetPaths = extractAssetHints(source);
    const notes = statusNotes(meta, pageId);

    rows.push({
      page_id: pageId,
      slug,
      title: inferTitle(slug, meta),
      source_path: sourcePath,
      metadata_path: (await pathExists(metadataPath)) ? metadataPath : "",
      tags,
      asset_paths: assetPaths,
      dependency_hints: dependencyHints,
      construct_hints: constructHints,
      bytes,
      line_count: lineCount,
      status: "discovered",
      notes,
    });
  }

  return rows;
}

function chooseCanaries(manifestRows, targetCount) {
  const selected = new Map();
  const byConstruct = new Map();

  for (const row of manifestRows) {
    for (const construct of row.construct_hints) {
      if (!byConstruct.has(construct)) byConstruct.set(construct, []);
      byConstruct.get(construct).push(row);
    }
  }

  function add(row, reason, priority) {
    if (!row || selected.has(row.page_id || row.slug)) return;
    selected.set(row.page_id || row.slug, { row, reason, priority });
  }

  for (const construct of [...byConstruct.keys()].sort()) {
    const candidates = byConstruct.get(construct).sort((left, right) => right.bytes - left.bytes);
    for (const row of candidates.slice(0, 4)) add(row, `construct:${construct}`, "high");
  }

  for (const row of [...manifestRows].sort((left, right) => right.bytes - left.bytes).slice(0, 25)) {
    add(row, "large-page", "high");
  }

  for (const row of manifestRows.filter((row) => /^scp-\d+$/i.test(row.slug)).slice(0, 20)) {
    add(row, "scp-sample", "medium");
  }

  const randomOrder = [...manifestRows].sort((left, right) => hashNumber(left.slug) - hashNumber(right.slug));
  for (const row of randomOrder) {
    if (selected.size >= Math.min(targetCount, manifestRows.length)) break;
    add(row, "deterministic-random", "medium");
  }

  return [...selected.values()].slice(0, Math.min(targetCount, manifestRows.length));
}

function writeTsv(rows, columns) {
  return [
    columns.join("\t"),
    ...rows.map((row) => columns.map((column) => tsv(row[column])).join("\t")),
  ].join("\n") + "\n";
}

function summarizeCounts(rows, key) {
  const counts = new Map();
  for (const row of rows) {
    const value = row[key] || "";
    counts.set(value, (counts.get(value) || 0) + 1);
  }
  return [...counts.entries()].sort((left, right) => left[0].localeCompare(right[0]));
}

function summarizeConstructs(manifestRows) {
  const counts = new Map();
  for (const row of manifestRows) {
    for (const construct of row.construct_hints) {
      counts.set(construct, (counts.get(construct) || 0) + 1);
    }
  }
  return [...counts.entries()].sort((left, right) => left[0].localeCompare(right[0]));
}

function summaryMarkdown(args, fileInventory, manifestRows, canaries) {
  const candidateCounts = summarizeCounts(fileInventory.rows, "candidate_type")
    .map(([type, count]) => `* ${type}: ${count}`)
    .join("\n");
  const extensionCounts = summarizeCounts(fileInventory.rows, "extension")
    .map(([extension, count]) => `* ${extension || "(none)"}: ${count}`)
    .join("\n");
  const constructCounts = summarizeConstructs(manifestRows)
    .map(([construct, count]) => `* ${construct}: ${count}`)
    .join("\n");

  return `# v5 corpus discovery summary

Corpus root:

\`\`\`text
${args.corpus}
\`\`\`

Output directory:

\`\`\`text
${args.outputDir}
\`\`\`

Discovery results:

* files inventoried: ${fileInventory.rows.length}
* page-source candidates: ${manifestRows.length}
* manifest rows: ${manifestRows.length}
* canary rows: ${canaries.length}

Candidate type counts:

${candidateCounts}

Extension counts:

${extensionCounts}

Construct hint counts:

${constructCounts}

Notes:

* Page sources are pages/<slug>/source.wikidot.txt.
* Metadata joins use pages/<slug>/meta.json and pages/<slug>/entity_id.txt.
* by-uuid observations and posts comment state are inventoried as metadata, not page-source rows.
* Canary selection is deterministic and combines construct coverage, large pages, SCP samples, and hash-ordered random pages.
`;
}

async function main() {
  const args = parseArgs(process.argv);
  const outputInsideCorpus = isPathInside(args.corpus, args.outputDir);
  if (outputInsideCorpus) {
    throw new Error("--output-dir must be outside --corpus to avoid self-inventory");
  }
  await fs.mkdir(args.outputDir, { recursive: true });

  const files = await walkFiles(args.corpus);
  const fileInventory = await buildFileInventory(args.corpus, files);
  const manifestRows = await buildManifest(args.corpus);
  const canaries = chooseCanaries(manifestRows, args.canaryCount);

  await fs.writeFile(path.join(args.outputDir, "corpus-file-inventory.tsv"), writeTsv(fileInventory.rows, [
    "path",
    "size_bytes",
    "extension",
    "candidate_type",
    "page_slug_guess",
    "asset_guess",
    "metadata_guess",
    "notes",
  ]));

  await fs.writeFile(path.join(args.outputDir, "corpus-manifest.tsv"), writeTsv(manifestRows, [
    "page_id",
    "slug",
    "title",
    "source_path",
    "metadata_path",
    "tags",
    "asset_paths",
    "dependency_hints",
    "construct_hints",
    "bytes",
    "line_count",
    "status",
    "notes",
  ]));

  await fs.writeFile(path.join(args.outputDir, "canary-pages.tsv"), writeTsv(canaries.map(({ row, reason, priority }) => ({
    page_id: row.page_id,
    slug: row.slug,
    source_path: row.source_path,
    reason,
    constructs: row.construct_hints,
    priority,
  })), [
    "page_id",
    "slug",
    "source_path",
    "reason",
    "constructs",
    "priority",
  ]));

  await fs.writeFile(path.join(args.outputDir, "corpus-discovery-summary.md"), summaryMarkdown(args, fileInventory, manifestRows, canaries));

  const summary = {
    corpus: args.corpus,
    outputDir: args.outputDir,
    filesInventoried: fileInventory.rows.length,
    pageSourceCandidates: manifestRows.length,
    canaryRows: canaries.length,
    candidateTypeCounts: Object.fromEntries(summarizeCounts(fileInventory.rows, "candidate_type")),
    extensionCounts: Object.fromEntries(summarizeCounts(fileInventory.rows, "extension")),
    constructCounts: Object.fromEntries(summarizeConstructs(manifestRows)),
  };

  await fs.writeFile(path.join(args.outputDir, "corpus-discovery-summary.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(JSON.stringify(summary, null, 2));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
