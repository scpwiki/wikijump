import crypto from "node:crypto";
import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";

import { readCorpusFile } from "./corpus-file-reader.mjs";

export const LISTPAGES_CAMPAIGN_SCHEMA =
  "wikijump_listpages_compat.campaign_inventory.v1";

const execFileAsync = promisify(execFile);

const DEFAULT_DOCS_ROOT =
  "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages";
const DEFAULT_CORPUS_ROOT =
  "/home/roku/src/Rokurolize/scp-wiki-translation/corpus";

const LEGACY_MODULE_FULLNAMES = new Set([
  "doc-modules:pages-module",
  "doc-modules:pagesbytag-module",
  "doc-modules:childpages-module",
  "doc-modules:countpages-module",
  "doc-modules:pagecalendar-module",
  "doc-modules:nextpreviouspage-module",
  "doc-modules:tagcloud-module",
  "doc-modules:categories-module",
  "doc-modules:pagetree-module",
  "doc-modules:backlinks-module",
  "doc-modules:orphanedpages-module",
  "doc-modules:wantedpages-module",
]);

const DOC_INDICATORS = [
  "ListPages",
  "[[module ListPages",
  "CountPages",
  "@URL",
  "/p/2",
  "[[head]]",
  "[[body]]",
  "[[foot]]",
  "%%title_linked%%",
  "%%total%%",
  "separate",
  "wrapper",
  "rssOnly",
  "rssDescription",
  "data form",
  "data-form",
  "created_at",
  "updated_at",
  "Hash Magic",
  "#_history",
  "PagesByTag",
  "ChildPages",
  "PageCalendar",
  "TagCloud",
  "NextPage",
  "PreviousPage",
];

const URL_ARGUMENT_NAMES = new Set([
  "category",
  "created_at",
  "date",
  "limit",
  "offset",
  "order",
  "p",
  "tag",
  "tags",
  "urlAttrPrefix",
]);

const DOC_LINK_PREFIXES = [
  "doc",
  "doc:",
  "doc-",
  "doc-include:",
  "doc-modules:",
  "doc-data-forms:",
  "doc-wiki-syntax:",
  "community:",
  "community-sites:",
  "howto:",
];

const SKIPPED_CORPUS_ROOTS = new Set(["www", "discord"]);

function sha256(text) {
  return crypto.createHash("sha256").update(text).digest("hex");
}

function normalizeReferenceFullname(rawTarget) {
  let target = rawTarget.trim();
  if (!target || /^[a-z][a-z0-9+.-]*:\/\//i.test(target)) return null;
  target = target.replace(/\|[\s\S]*$/u, "");
  target = target.replace(/#[\s\S]*$/u, "");
  target = target.replace(/^\//u, "");
  target = target.trim();
  if (!target || target.startsWith("#")) return null;
  if (target.startsWith(":")) return null;
  return target.replace(/\s+/gu, "-").toLowerCase();
}

function shouldFollowDocLink(fullname) {
  return DOC_LINK_PREFIXES.some((prefix) => fullname.startsWith(prefix));
}

function splitLines(text) {
  return text.split(/\r?\n/u);
}

function lineStarts(text) {
  const starts = [0];
  for (let index = 0; index < text.length; index += 1) {
    if (text[index] === "\n") starts.push(index + 1);
  }
  return starts;
}

function lineNumberForOffset(starts, offset) {
  let low = 0;
  let high = starts.length - 1;
  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (starts[mid] <= offset) low = mid + 1;
    else high = mid - 1;
  }
  return high + 1;
}

function sourceReference(fullname, sourcePath, lineStart, lineEnd = lineStart) {
  return {
    corpus_area: "wikidot-docs",
    page_fullname: fullname,
    path: sourcePath,
    line_start: lineStart,
    line_end: lineEnd,
  };
}

function tableCells(line) {
  const trimmed = line.trim();
  if (!trimmed.startsWith("||")) return null;
  return trimmed
    .split("||")
    .slice(1, -1)
    .map((cell) => cell.replace(/^~/u, "").trim());
}

function quotedValues(text) {
  return [...text.matchAll(/"([^"]*)"|'([^']*)'|\{\{([^{}]+)\}\}/gu)]
    .map((match) => match[1] ?? match[2] ?? match[3])
    .filter((value, index, values) => values.indexOf(value) === index);
}

function variableNames(text) {
  return [...text.matchAll(/%%([^%\r\n]+)%%/gu)]
    .map((match) => match[1].trim())
    .filter((value, index, values) => value && values.indexOf(value) === index);
}

function argumentNames(text) {
  const names = [];
  for (const match of text.matchAll(/\b([A-Za-z_][A-Za-z0-9_-]*)\s*=/gu)) {
    names.push(match[1]);
  }
  for (const match of text.matchAll(/\b(urlAttrPrefix|perPage|prependLine|appendLine|rssDescription|rssHome|rssLimit|rssOnly|rssTitle|skipCurrent|tagTarget)\b/gu)) {
    names.push(match[1]);
  }
  return [...new Set(names)];
}

function defaultFromText(text) {
  const match = text.match(/\bdefault(?:s| is)?\b[:\s-]*(.+)$/iu);
  return match ? match[1].trim() : null;
}

function claimKind(line, cells, sectionPath) {
  const lower = line.toLowerCase();
  if (variableNames(line).length > 0) return "template-variable";
  if (cells?.length && /argument/i.test(sectionPath.join(" "))) return "argument";
  if (argumentNames(line).length > 0) return "argument";
  if (lower.includes("@url") || /\/[a-z_]+\/\S+/iu.test(line)) return "url-behavior";
  if (lower.includes("pagination") || lower.includes("perpage")) return "pagination";
  if (lower.includes("rss")) return "rss";
  if (lower.includes("default")) return "default";
  if (lower.includes("deprecated") || lower.includes("instead of this")) return "deprecated-alias";
  if (lower.includes("not allowed") || lower.includes("cannot") || lower.includes("caution")) {
    return "limitation";
  }
  if (cells) return "table-row";
  if (/^\s*\*/u.test(line)) return "bullet-claim";
  if (/\[\[(?:module|include|head|body|foot|\/module)/iu.test(line)) return "syntax-example";
  return "text-claim";
}

function isRelevantClaimLine(line) {
  const trimmed = line.trim();
  if (!trimmed) return false;
  if (trimmed === "[[code]]" || trimmed === "[[/code]]") return false;
  if (trimmed === "[[module CSS]]" || trimmed === "[[/module]]") return false;
  return true;
}

function extractDocReferences(fullname, source, sourcePath) {
  const references = [];
  const starts = lineStarts(source);

  function add(kind, rawTarget, offset, raw) {
    const target = normalizeReferenceFullname(rawTarget);
    if (!target) return;
    references.push({
      kind,
      raw,
      raw_target: rawTarget,
      target_fullname: target,
      source: sourceReference(fullname, sourcePath, lineNumberForOffset(starts, offset)),
    });
  }

  for (const match of source.matchAll(/\[\[include\s+([^\]\s|]+)[^\]]*\]\]/giu)) {
    add("include", match[1], match.index, match[0]);
  }
  for (const match of source.matchAll(/\[\[\[([^\]\r\n]+?)\]\]\]/gu)) {
    add("link", match[1], match.index, match[0]);
  }
  for (const match of source.matchAll(/\[\/([^\]\s]+)(?:\s+[^\]]*)?\]/gu)) {
    add("short-link", match[1], match.index, match[0]);
  }
  return references;
}

function extractDocClaims(fullname, source, sourcePath, reasons) {
  const claims = [];
  const lines = splitLines(source);
  const sectionPath = [];
  let inCodeBlock = false;

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
    const lineNo = lineIndex + 1;
    const line = lines[lineIndex];
    const trimmed = line.trim();
    if (/^\+{1,6}\s+/u.test(trimmed)) {
      const level = trimmed.match(/^\++/u)[0].length;
      sectionPath.length = Math.max(0, level - 1);
      sectionPath[level - 1] = trimmed.replace(/^\++\s*/u, "");
    }
    if (trimmed === "[[code]]") inCodeBlock = true;
    if (!isRelevantClaimLine(line)) {
      if (trimmed === "[[/code]]") inCodeBlock = false;
      continue;
    }

    const cells = tableCells(line);
    const variables = variableNames(line);
    const args = argumentNames(line);
    const kind = inCodeBlock ? "syntax-example" : claimKind(line, cells, sectionPath);
    const item = {
      id: `${fullname}:L${lineNo}`,
      kind,
      claim: trimmed,
      section_path: [...sectionPath],
      argument_names: args,
      aliases: [],
      stated_default: defaultFromText(line),
      accepted_values: quotedValues(line),
      rejected_values: /not allowed|cannot|must not/iu.test(line) ? quotedValues(line) : [],
      empty_values: /\bempty\b/iu.test(line) ? quotedValues(line) : [],
      omitted_values: /\bomitted|not passed|not set|does not have\b/iu.test(line)
        ? quotedValues(line)
        : [],
      interactions: /with|when|if|conflict|combined|inherits/iu.test(line) ? [trimmed] : [],
      output_structure: /container|div|link|rss|feed|wrapper|header|footer|head|body|foot/iu.test(line)
        ? [trimmed]
        : [],
      template_variables: variables,
      url_pagination_implications: /@URL|\/p\/|pagination|urlAttrPrefix|URL/iu.test(line)
        ? [trimmed]
        : [],
      provenance_reasons: reasons,
      source: sourceReference(fullname, sourcePath, lineNo),
      live_verification: { status: "unverified" },
      notes: [],
    };

    if (cells?.length === 2 && /instead of this/i.test(sectionPath.join(" "))) {
      item.aliases.push({ old: cells[0], replacement: cells[1] });
    }
    claims.push(item);
    if (trimmed === "[[/code]]") inCodeBlock = false;
  }
  return claims;
}

async function readDocPage(docsRoot, fullname) {
  const sourcePath = path.join(docsRoot, fullname, "source.wikidot.txt");
  const metaPath = path.join(docsRoot, fullname, "meta.json");
  const [source, metaText] = await Promise.all([
    readCorpusFile(docsRoot, sourcePath, { optional: true, maxBytes: 5 * 1024 * 1024 }),
    readCorpusFile(docsRoot, metaPath, { optional: true, maxBytes: 512 * 1024 }),
  ]);
  if (source === null) return null;
  return {
    fullname,
    source_path: sourcePath,
    source_sha256: sha256(source),
    source,
    meta: metaText ? JSON.parse(metaText) : null,
  };
}

async function listDocFullnames(docsRoot) {
  const entries = await fs.readdir(docsRoot, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort((left, right) => left.localeCompare(right));
}

function docHitReasons(source, fullname) {
  const lower = source.toLowerCase();
  const reasons = [];
  for (const indicator of DOC_INDICATORS) {
    if (lower.includes(indicator.toLowerCase())) reasons.push(`indicator:${indicator}`);
  }
  if (LEGACY_MODULE_FULLNAMES.has(fullname)) reasons.push("legacy-listing-module");
  if (fullname.startsWith("doc-data-forms:")) reasons.push("data-form-docs");
  return [...new Set(reasons)];
}

async function seedDocQueue(docsRoot, allFullnames) {
  const reasonsByFullname = new Map();

  function add(fullname, reason) {
    if (!allFullnames.has(fullname)) return false;
    const reasons = reasonsByFullname.get(fullname) ?? [];
    if (!reasons.includes(reason)) reasons.push(reason);
    reasonsByFullname.set(fullname, reasons);
    return true;
  }

  add("doc-modules:start", "seed:module-index");
  add("doc-modules:listpages-module", "seed:listpages-doc");
  for (const fullname of allFullnames) {
    if (fullname.startsWith("doc-modules:")) add(fullname, "module-index-full-enumeration");
    if (LEGACY_MODULE_FULLNAMES.has(fullname)) add(fullname, "legacy-listing-module");
  }

  for (const fullname of allFullnames) {
    const sourcePath = path.join(docsRoot, fullname, "source.wikidot.txt");
    const source = await readCorpusFile(docsRoot, sourcePath, {
      optional: true,
      maxBytes: 5 * 1024 * 1024,
    });
    if (source === null) continue;
    for (const reason of docHitReasons(source, fullname)) add(fullname, reason);
  }

  return reasonsByFullname;
}

export async function buildDocumentationInventory({
  docsRoot = DEFAULT_DOCS_ROOT,
} = {}) {
  const allFullnames = new Set(await listDocFullnames(docsRoot));
  const reasonsByFullname = await seedDocQueue(docsRoot, allFullnames);
  const queue = [...reasonsByFullname.keys()].sort();
  const documents = [];
  const claims = [];
  const references = [];
  const missingReferences = [];
  const inspected = new Set();

  while (queue.length > 0) {
    const fullname = queue.shift();
    if (inspected.has(fullname)) continue;
    inspected.add(fullname);
    const page = await readDocPage(docsRoot, fullname);
    if (page === null) continue;
    const reasons = reasonsByFullname.get(fullname) ?? [];
    const pageReferences = extractDocReferences(fullname, page.source, page.source_path);
    references.push(...pageReferences);
    claims.push(...extractDocClaims(fullname, page.source, page.source_path, reasons));
    documents.push({
      fullname,
      title: page.meta?.title_shown ?? page.meta?.title ?? page.meta?.fullname ?? fullname,
      source_path: page.source_path,
      source_sha256: page.source_sha256,
      line_count: splitLines(page.source).length,
      relevance_reasons: reasons,
      reference_count: pageReferences.length,
    });

    for (const ref of pageReferences) {
      if (allFullnames.has(ref.target_fullname)) {
        if (ref.kind === "include" || shouldFollowDocLink(ref.target_fullname)) {
          const reason = `${ref.kind}:${fullname}`;
          const existing = reasonsByFullname.get(ref.target_fullname) ?? [];
          if (!existing.includes(reason)) {
            existing.push(reason);
            reasonsByFullname.set(ref.target_fullname, existing);
            queue.push(ref.target_fullname);
            queue.sort((left, right) => left.localeCompare(right));
          }
        }
      } else if (ref.kind === "include" || shouldFollowDocLink(ref.target_fullname)) {
        missingReferences.push(ref);
      }
    }
  }

  return {
    schema: `${LISTPAGES_CAMPAIGN_SCHEMA}.docs`,
    generated_at: new Date().toISOString(),
    docs_root: docsRoot,
    seed_pages: ["doc-modules:start", "doc-modules:listpages-module"],
    documents: documents.sort((left, right) => left.fullname.localeCompare(right.fullname)),
    claims: claims.sort((left, right) => left.id.localeCompare(right.id)),
    references,
    missing_references: missingReferences,
    summary: {
      all_doc_page_count: allFullnames.size,
      inspected_document_count: documents.length,
      claim_count: claims.length,
      reference_count: references.length,
      missing_reference_count: missingReferences.length,
    },
  };
}

function isHeadTerminator(source, index, quote) {
  return source[index] === "]" && source[index + 1] === "]" && quote === null;
}

function findModuleHeadEnd(source, openStart) {
  let quote = null;
  for (let index = openStart + 2; index < source.length - 1; index += 1) {
    const char = source[index];
    if ((char === '"' || char === "'") && source[index - 1] !== "\\") {
      quote = quote === char ? null : quote ?? char;
    }
    if (isHeadTerminator(source, index, quote)) return index + 2;
  }
  return null;
}

function parseModuleHead(head) {
  const open = head.match(/^\[\[\s*(module654|module)_?\s+([^\s\]]+)/iu);
  if (!open) return null;
  const moduleName = open[2];
  const attributes = [];
  for (const match of head.matchAll(/\b([A-Za-z_][A-Za-z0-9_-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s\]]+))/gu)) {
    attributes.push({
      name: match[1],
      value: match[2] ?? match[3] ?? match[4] ?? "",
      quote: match[2] !== undefined ? '"' : match[3] !== undefined ? "'" : null,
      raw: match[0],
    });
  }
  const duplicateAttributes = [
    ...new Set(
      attributes
        .map((attr) => attr.name.toLowerCase())
        .filter((name, index, names) => names.indexOf(name) !== index),
    ),
  ];
  return {
    wrapper_name: open[1],
    module_name: moduleName,
    attributes,
    duplicate_attributes: duplicateAttributes,
  };
}

function nextModuleEvent(source, start) {
  const openMatch = /\[\[\s*(?:module654|module)_?\s+[^\s\]]+/giu;
  const closeMatch = /\[\[\s*\/module\s*\]\]/giu;
  openMatch.lastIndex = start;
  closeMatch.lastIndex = start;
  const open = openMatch.exec(source);
  const close = closeMatch.exec(source);
  if (!open && !close) return null;
  if (open && (!close || open.index < close.index)) {
    const end = findModuleHeadEnd(source, open.index);
    if (end === null) return { kind: "malformed-open", start: open.index, end: source.length };
    return { kind: "open", start: open.index, end };
  }
  return { kind: "close", start: close.index, end: closeMatch.lastIndex };
}

function findBalancedModuleEnd(source, bodyStart) {
  let depth = 1;
  let cursor = bodyStart;
  while (cursor < source.length) {
    const event = nextModuleEvent(source, cursor);
    if (event === null) return null;
    if (event.kind === "open") depth += 1;
    else if (event.kind === "close") depth -= 1;
    cursor = event.end;
    if (depth === 0) {
      return { moduleEnd: event.end, bodyEnd: event.start };
    }
  }
  return null;
}

function contextLines(lines, lineStart, lineEnd, radius = 2) {
  const start = Math.max(1, lineStart - radius);
  const end = Math.min(lines.length, lineEnd + radius);
  return lines.slice(start - 1, end).map((text, offset) => ({
    line: start + offset,
    text,
  }));
}

function attributeSignature(attributes) {
  return attributes
    .map((attr) => `${attr.name.toLowerCase()}=${attr.value}`)
    .sort((left, right) => left.localeCompare(right));
}

function usageClusterKey(attributes, body) {
  const attrs = attributeSignature(attributes).join("\u001f");
  const vars = variableNames(body).sort().join("\u001f");
  const sections = ["head", "body", "foot"]
    .filter((name) => new RegExp(`\\[\\[${name}\\]\\]`, "iu").test(body))
    .join(",");
  return sha256(`${attrs}\n${vars}\n${sections}`);
}

export function extractListPagesInvocationsFromSource({
  branch,
  pageFullname,
  sourcePath,
  source,
}) {
  const starts = lineStarts(source);
  const lines = splitLines(source);
  const invocations = [];
  const openRegex = /\[\[\s*(?:module654|module)_?\s+listpages\b/giu;
  let match;
  while ((match = openRegex.exec(source)) !== null) {
    const openStart = match.index;
    const headEnd = findModuleHeadEnd(source, openStart);
    const lineStart = lineNumberForOffset(starts, openStart);
    if (headEnd === null) {
      invocations.push({
        id: `${branch}:${pageFullname}:L${lineStart}:B${openStart}`,
        branch,
        page_fullname: pageFullname,
        source_path: sourcePath,
        line_start: lineStart,
        line_end: lineStart,
        byte_start: openStart,
        byte_end: source.length,
        balanced: false,
        malformed_reason: "unclosed-module-head",
        head: source.slice(openStart),
        body: "",
        attributes: [],
        duplicate_attributes: [],
        url_driven_attributes: [],
        template_variables: [],
        body_sections: [],
        source_sha256: sha256(source.slice(openStart)),
        semantic_cluster_key: sha256(source.slice(openStart)),
        context_lines: contextLines(lines, lineStart, lineStart),
      });
      break;
    }

    const head = source.slice(openStart, headEnd);
    const parsedHead = parseModuleHead(head);
    const balanced = findBalancedModuleEnd(source, headEnd);
    const bodyEnd = balanced?.bodyEnd ?? source.length;
    const moduleEnd = balanced?.moduleEnd ?? source.length;
    const body = source.slice(headEnd, bodyEnd);
    const lineEnd = lineNumberForOffset(starts, moduleEnd);
    const attributes = parsedHead?.attributes ?? [];
    const urlDriven = attributes
      .filter((attr) => attr.value.startsWith("@URL"))
      .map((attr) => attr.name);
    const sections = ["head", "body", "foot"].filter((name) =>
      new RegExp(`\\[\\[${name}\\]\\]`, "iu").test(body),
    );

    invocations.push({
      id: `${branch}:${pageFullname}:L${lineStart}:B${openStart}`,
      branch,
      page_fullname: pageFullname,
      source_path: sourcePath,
      line_start: lineStart,
      line_end: lineEnd,
      byte_start: openStart,
      byte_end: moduleEnd,
      balanced: balanced !== null,
      malformed_reason: balanced === null ? "missing-module-close" : null,
      wrapper_name: parsedHead?.wrapper_name ?? null,
      module_name: parsedHead?.module_name ?? "ListPages",
      head,
      body,
      attributes,
      duplicate_attributes: parsedHead?.duplicate_attributes ?? [],
      url_driven_attributes: urlDriven,
      recognized_url_argument_names: urlDriven.filter((name) =>
        URL_ARGUMENT_NAMES.has(name),
      ),
      template_variables: variableNames(body),
      body_sections: sections,
      source_sha256: sha256(source.slice(openStart, moduleEnd)),
      semantic_cluster_key: usageClusterKey(attributes, body),
      context_lines: contextLines(lines, lineStart, lineEnd),
    });
    openRegex.lastIndex = Math.max(moduleEnd, headEnd);
  }
  return invocations;
}

async function listFirstLevelCorpusRoots(corpusRoot) {
  const entries = await fs.readdir(corpusRoot, { withFileTypes: true });
  const roots = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    if (entry.name.startsWith("_") || SKIPPED_CORPUS_ROOTS.has(entry.name)) continue;
    const pagesRoot = path.join(corpusRoot, entry.name, "pages");
    try {
      const stats = await fs.stat(pagesRoot);
      if (stats.isDirectory()) roots.push(entry.name);
    } catch {
      // No pages directory.
    }
  }
  return roots.sort((left, right) => left.localeCompare(right));
}

async function listPageSourcePaths(root) {
  const pagesRoot = path.join(root, "pages");
  const entries = await fs.readdir(pagesRoot, { withFileTypes: true });
  const output = [];
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    if (!entry.isDirectory()) continue;
    output.push({
      pageFullname: entry.name,
      sourcePath: path.join(pagesRoot, entry.name, "source.wikidot.txt"),
    });
  }
  return output;
}

async function rgListPagesSourcePaths(branchRoot) {
  const pagesRoot = path.join(branchRoot, "pages");
  try {
    const { stdout } = await execFileAsync(
      "rg",
      [
        "-U",
        "--files-with-matches",
        "--ignore-case",
        "--glob",
        "source.wikidot.txt",
        String.raw`\[\[\s*(?:module654|module)_?\s+listpages\b`,
        pagesRoot,
      ],
      { maxBuffer: 64 * 1024 * 1024 },
    );
    return stdout
      .split(/\r?\n/u)
      .filter(Boolean)
      .sort((left, right) => left.localeCompare(right))
      .map((sourcePath) => ({
        pageFullname: path.basename(path.dirname(sourcePath)),
        sourcePath,
      }));
  } catch (error) {
    if (error.code === 1) return [];
    if (error.code !== "ENOENT") throw error;
  }

  const allSources = await listPageSourcePaths(branchRoot);
  const candidates = [];
  for (const candidate of allSources) {
    const source = await readCorpusFile(branchRoot, candidate.sourcePath, {
      optional: true,
      maxBytes: 10 * 1024 * 1024,
    });
    if (source !== null && /\[\[\s*(?:module654|module)_?\s+listpages\b/iu.test(source)) {
      candidates.push(candidate);
    }
  }
  return candidates;
}

export async function buildCorpusListPagesInventory({
  corpusRoot = DEFAULT_CORPUS_ROOT,
  branches = null,
  onProgress = null,
} = {}) {
  const branchNames = branches ?? (await listFirstLevelCorpusRoots(corpusRoot));
  const invocations = [];
  const branchSummaries = [];

  for (const branch of branchNames) {
    const branchRoot = path.join(corpusRoot, branch);
    const allSources = await listPageSourcePaths(branchRoot);
    const sources = await rgListPagesSourcePaths(branchRoot);
    onProgress?.({
      phase: "corpus-branch-candidates",
      branch,
      source_page_count: allSources.length,
      candidate_source_count: sources.length,
    });
    let pagesWithListPages = 0;
    for (const { pageFullname, sourcePath } of sources) {
      const source = await readCorpusFile(branchRoot, sourcePath, {
        optional: true,
        maxBytes: 10 * 1024 * 1024,
      });
      if (source === null || !/\[\[\s*(?:module654|module)_?\s+listpages\b/iu.test(source)) {
        continue;
      }
      const rows = extractListPagesInvocationsFromSource({
        corpusRoot: branchRoot,
        branch,
        pageFullname,
        sourcePath,
        source,
      });
      if (rows.length > 0) pagesWithListPages += 1;
      invocations.push(...rows);
    }
    branchSummaries.push({
      branch,
      source_page_count: allSources.length,
      candidate_source_count: sources.length,
      pages_with_listpages: pagesWithListPages,
      invocation_count: invocations.filter((row) => row.branch === branch).length,
    });
  }

  const clusters = new Map();
  for (const invocation of invocations) {
    const cluster = clusters.get(invocation.semantic_cluster_key) ?? {
      semantic_cluster_key: invocation.semantic_cluster_key,
      count: 0,
      exact_source_hashes: new Set(),
      argument_signature: attributeSignature(invocation.attributes),
      template_variables: invocation.template_variables,
      body_sections: invocation.body_sections,
      first_provenance: {
        branch: invocation.branch,
        page_fullname: invocation.page_fullname,
        source_path: invocation.source_path,
        line_start: invocation.line_start,
      },
    };
    cluster.count += 1;
    cluster.exact_source_hashes.add(invocation.source_sha256);
    clusters.set(invocation.semantic_cluster_key, cluster);
  }

  return {
    schema: `${LISTPAGES_CAMPAIGN_SCHEMA}.corpus_usages`,
    generated_at: new Date().toISOString(),
    corpus_root: corpusRoot,
    branches: branchSummaries,
    invocations,
    clusters: [...clusters.values()]
      .map((cluster) => ({
        ...cluster,
        exact_source_hashes: [...cluster.exact_source_hashes].sort(),
      }))
      .sort((left, right) => right.count - left.count || left.semantic_cluster_key.localeCompare(right.semantic_cluster_key)),
    summary: {
      branch_count: branchNames.length,
      invocation_count: invocations.length,
      page_count_with_invocations: new Set(
        invocations.map((row) => `${row.branch}:${row.page_fullname}`),
      ).size,
      cluster_count: clusters.size,
      malformed_count: invocations.filter((row) => !row.balanced).length,
    },
  };
}

export async function buildListPagesCampaignInventory(options = {}) {
  const [docs, corpus] = await Promise.all([
    buildDocumentationInventory(options),
    buildCorpusListPagesInventory(options),
  ]);
  return {
    schema: LISTPAGES_CAMPAIGN_SCHEMA,
    generated_at: new Date().toISOString(),
    docs,
    corpus,
    summary: {
      docs: docs.summary,
      corpus: corpus.summary,
    },
  };
}
