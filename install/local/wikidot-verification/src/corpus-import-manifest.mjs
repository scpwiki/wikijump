import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

const REQUIRED_META_KEYS = [
  'children',
  'commented_at',
  'commented_by',
  'comments',
  'created_at',
  'created_by',
  'fullname',
  'parent_fullname',
  'parent_title',
  'rating',
  'revisions',
  'tags',
  'title',
  'title_shown',
  'updated_at',
  'updated_by',
];

export function sha256Hex(bufferOrString) {
  return crypto.createHash('sha256').update(bufferOrString).digest('hex');
}

function codePointCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function stableStringify(value) {
  if (value === null || typeof value !== 'object') {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableStringify(item)).join(',')}]`;
  }
  const entries = Object.entries(value).sort(([left], [right]) => codePointCompare(left, right));
  return `{${entries.map(([key, entryValue]) => `${JSON.stringify(key)}:${stableStringify(entryValue)}`).join(',')}}`;
}

function readText(filePath) {
  return fs.readFileSync(filePath, 'utf8');
}

function readJson(filePath) {
  return JSON.parse(readText(filePath));
}

function assertString(value, field, rowPath) {
  if (typeof value !== 'string') {
    throw new Error(`${rowPath}: meta.${field} must be a string`);
  }
}

function assertNonEmptyString(value, field, rowPath) {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${rowPath}: meta.${field} must be a non-empty string`);
  }
}

function assertNullableString(value, field, rowPath) {
  if (value !== null && typeof value !== 'string') {
    throw new Error(`${rowPath}: meta.${field} must be null or a string`);
  }
}

function assertInteger(value, field, rowPath) {
  if (!Number.isInteger(value)) {
    throw new Error(`${rowPath}: meta.${field} must be an integer`);
  }
}

function assertNonNegativeInteger(value, field, rowPath) {
  if (!Number.isInteger(value) || value < 0) {
    throw new Error(`${rowPath}: meta.${field} must be a non-negative integer`);
  }
}

function validateMeta(meta, rowPath) {
  for (const key of REQUIRED_META_KEYS) {
    if (!Object.hasOwn(meta, key)) {
      throw new Error(`${rowPath}: missing required meta key ${key}`);
    }
  }

  assertNonEmptyString(meta.fullname, 'fullname', rowPath);
  assertString(meta.title, 'title', rowPath);
  assertNonEmptyString(meta.created_at, 'created_at', rowPath);
  assertNullableString(meta.created_by, 'created_by', rowPath);
  assertNonEmptyString(meta.updated_at, 'updated_at', rowPath);
  assertNullableString(meta.updated_by, 'updated_by', rowPath);
  assertNullableString(meta.parent_fullname, 'parent_fullname', rowPath);
  assertNullableString(meta.parent_title, 'parent_title', rowPath);
  assertNullableString(meta.title_shown, 'title_shown', rowPath);
  assertNullableString(meta.commented_at, 'commented_at', rowPath);
  assertNullableString(meta.commented_by, 'commented_by', rowPath);
  assertNonNegativeInteger(meta.children, 'children', rowPath);
  assertNonNegativeInteger(meta.comments, 'comments', rowPath);
  assertInteger(meta.rating, 'rating', rowPath);
  assertNonNegativeInteger(meta.revisions, 'revisions', rowPath);

  if (!Array.isArray(meta.tags) || meta.tags.some((tag) => typeof tag !== 'string')) {
    throw new Error(`${rowPath}: meta.tags must be an array of strings`);
  }
}

function validateEntityId(entityId, rowPath) {
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/iu.test(entityId)) {
    throw new Error(`${rowPath}: entity_id.txt does not contain a UUID`);
  }
}

export function buildCorpusImportManifest({ corpusRoot, branch, sourceSite = branch, sourceBranch = branch }) {
  const pagesRoot = path.join(corpusRoot, branch, 'pages');
  const entries = fs.readdirSync(pagesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort(codePointCompare);

  const rows = [];
  const entityIds = new Map();
  const fullnames = new Map();

  for (const directoryName of entries) {
    const pageDir = path.join(pagesRoot, directoryName);
    const sourcePath = path.join(pageDir, 'source.wikidot.txt');
    const metaPath = path.join(pageDir, 'meta.json');
    const entityIdPath = path.join(pageDir, 'entity_id.txt');

    for (const requiredPath of [sourcePath, metaPath, entityIdPath]) {
      if (!fs.existsSync(requiredPath)) {
        throw new Error(`${pageDir}: missing ${path.basename(requiredPath)}`);
      }
    }

    const source = readText(sourcePath);
    const metaRaw = readText(metaPath);
    const meta = JSON.parse(metaRaw);
    const entityId = readText(entityIdPath).trim();
    validateMeta(meta, pageDir);
    validateEntityId(entityId, pageDir);

    if (meta.fullname !== directoryName) {
      throw new Error(`${pageDir}: meta.fullname ${meta.fullname} does not match directory name ${directoryName}`);
    }
    if (entityIds.has(entityId)) {
      throw new Error(`${pageDir}: duplicate source_entity_id also used by ${entityIds.get(entityId)}`);
    }
    if (fullnames.has(meta.fullname)) {
      throw new Error(`${pageDir}: duplicate fullname also used by ${fullnames.get(meta.fullname)}`);
    }
    entityIds.set(entityId, pageDir);
    fullnames.set(meta.fullname, pageDir);

    rows.push({
      source_site: sourceSite,
      source_branch: sourceBranch,
      source_entity_id: entityId,
      fullname: meta.fullname,
      title: meta.title,
      title_shown: meta.title_shown,
      parent_fullname: meta.parent_fullname,
      parent_title: meta.parent_title,
      created_at: meta.created_at,
      created_by: meta.created_by,
      updated_at: meta.updated_at,
      updated_by: meta.updated_by,
      commented_at: meta.commented_at,
      commented_by: meta.commented_by,
      children: meta.children,
      comments: meta.comments,
      rating: meta.rating,
      revisions: meta.revisions,
      tags: [...meta.tags].sort(codePointCompare),
      source_sha256: sha256Hex(source),
      meta_sha256: sha256Hex(metaRaw),
      source_bytes: Buffer.byteLength(source),
      meta_bytes: Buffer.byteLength(metaRaw),
      source_path: sourcePath,
      meta_path: metaPath,
      entity_id_path: entityIdPath,
    });
  }

  return rows;
}

export function formatJsonl(rows) {
  return `${rows.map((row) => stableStringify(row)).join('\n')}\n`;
}

export function buildManifestSummary(rows, jsonl) {
  const parentCount = rows.filter((row) => row.parent_fullname !== null).length;
  return {
    row_count: rows.length,
    manifest_sha256: sha256Hex(jsonl),
    parent_count: parentCount,
    source_sites: [...new Set(rows.map((row) => row.source_site))].sort(codePointCompare),
    source_branches: [...new Set(rows.map((row) => row.source_branch))].sort(codePointCompare),
    first_fullname: rows[0]?.fullname ?? null,
    last_fullname: rows.at(-1)?.fullname ?? null,
  };
}
