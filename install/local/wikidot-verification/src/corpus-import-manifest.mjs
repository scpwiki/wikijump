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
const FATAL_UTF8_DECODER = new TextDecoder('utf-8', { fatal: true });
const SOURCE_BUNDLE_FALLBACK_TIMESTAMP = '1970-01-01T00:00:00+00:00';
const SOURCE_BROWSER_VISIBILITIES = new Set([
  'browser_visible',
  'actor_required',
  'source_only',
  'fail_closed',
]);

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

function readUtf8File(filePath) {
  const buffer = fs.readFileSync(filePath);
  try {
    return {
      buffer,
      text: FATAL_UTF8_DECODER.decode(buffer),
    };
  } catch (error) {
    throw new Error(`${filePath}: invalid UTF-8: ${error.message}`);
  }
}

function readText(filePath) {
  return readUtf8File(filePath).text;
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

function assertSha256(value, field, rowPath) {
  if (typeof value !== 'string' || !/^[0-9a-f]{64}$/iu.test(value)) {
    throw new Error(`${rowPath}: meta.${field} must be a sha256 hex string`);
  }
}

function coerceInteger(value, field, rowPath) {
  if (Number.isInteger(value)) return value;
  if (typeof value === 'string' && /^-?[0-9]+$/u.test(value)) {
    return Number.parseInt(value, 10);
  }
  throw new Error(`${rowPath}: meta.${field} must be an integer`);
}

function coerceNonNegativeInteger(value, field, rowPath) {
  const integer = coerceInteger(value, field, rowPath);
  if (integer < 0) {
    throw new Error(`${rowPath}: meta.${field} must be a non-negative integer`);
  }
  return integer;
}

function deterministicUuid(input) {
  const bytes = crypto.createHash('sha256').update(input).digest().subarray(0, 16);
  bytes[6] = (bytes[6] & 0x0f) | 0x50;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = bytes.toString('hex');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function readSourceBundleManifest(sourceBundleRoot) {
  const manifestPath = path.join(sourceBundleRoot, 'corpus-manifest.tsv');
  if (!fs.existsSync(manifestPath)) return { exists: false, rows: new Map() };
  const manifestText = readText(manifestPath);
  const [headerLine, ...lines] = manifestText.split(/\r?\n/u).filter((line) => line.length > 0);
  const headers = headerLine.split('\t');
  const indexByHeader = new Map(headers.map((header, index) => [header, index]));
  for (const required of ['site', 'fullname', 'source_bytes', 'source_sha256']) {
    if (!indexByHeader.has(required)) {
      throw new Error(`${manifestPath}: missing required column ${required}`);
    }
  }
  const rows = new Map();
  for (const line of lines) {
    const columns = line.split('\t');
    const fullname = columns[indexByHeader.get('fullname')];
    if (!fullname) throw new Error(`${manifestPath}: manifest row missing fullname`);
    if (rows.has(fullname)) throw new Error(`${manifestPath}: duplicate fullname ${fullname}`);
    rows.set(fullname, Object.fromEntries(headers.map((header, index) => [header, columns[index] ?? ''])));
  }
  return { exists: true, rows };
}

function normalizeSourceBundleMeta(meta, rowPath) {
  assertNonEmptyString(meta.fullname, 'fullname', rowPath);
  assertString(meta.title, 'title', rowPath);
  assertNullableString(meta.title_shown ?? null, 'title_shown', rowPath);
  assertNullableString(meta.parent_fullname ?? null, 'parent_fullname', rowPath);
  assertNullableString(meta.parent_title ?? null, 'parent_title', rowPath);
  assertNullableString(meta.created_by ?? null, 'created_by', rowPath);
  assertNullableString(meta.updated_by ?? null, 'updated_by', rowPath);
  assertNullableString(meta.commented_at ?? null, 'commented_at', rowPath);
  assertNullableString(meta.commented_by ?? null, 'commented_by', rowPath);
  if (!Array.isArray(meta.tags) || meta.tags.some((tag) => typeof tag !== 'string')) {
    throw new Error(`${rowPath}: meta.tags must be an array of strings`);
  }
  if (Object.hasOwn(meta, 'source_sha256')) assertSha256(meta.source_sha256, 'source_sha256', rowPath);

  const createdAt = meta.created_at ?? meta.updated_at ?? SOURCE_BUNDLE_FALLBACK_TIMESTAMP;
  const updatedAt = meta.updated_at ?? meta.created_at ?? SOURCE_BUNDLE_FALLBACK_TIMESTAMP;
  assertNonEmptyString(createdAt, 'created_at', rowPath);
  assertNonEmptyString(updatedAt, 'updated_at', rowPath);

  return {
    fullname: meta.fullname,
    title: meta.title,
    title_shown: meta.title_shown ?? meta.title,
    parent_fullname: meta.parent_fullname ?? null,
    parent_title: meta.parent_title ?? null,
    created_at: createdAt,
    created_by: meta.created_by ?? null,
    updated_at: updatedAt,
    updated_by: meta.updated_by ?? null,
    commented_at: meta.commented_at ?? null,
    commented_by: meta.commented_by ?? null,
    children: coerceNonNegativeInteger(meta.children ?? meta.children_count ?? 0, 'children_count', rowPath),
    comments: coerceNonNegativeInteger(meta.comments ?? meta.comments_count ?? 0, 'comments_count', rowPath),
    rating: Object.hasOwn(meta, 'rating') ? coerceInteger(meta.rating, 'rating', rowPath) : 0,
    revisions: coerceNonNegativeInteger(meta.revisions ?? meta.revisions_count ?? 0, 'revisions_count', rowPath),
    tags: meta.tags,
    source_bytes: Object.hasOwn(meta, 'source_bytes') ? coerceNonNegativeInteger(meta.source_bytes, 'source_bytes', rowPath) : null,
    source_sha256: meta.source_sha256 ?? null,
    capture_method: meta.capture_method ?? null,
    source_browser_visibility: firstNonEmptyValue(meta.source_browser_visibility, meta.browser_visibility),
    source_browser_status: firstNonEmptyValue(meta.source_browser_status, meta.browser_status),
    source_required_actor: firstNonEmptyValue(meta.source_required_actor, meta.required_actor),
  };
}

function firstNonEmptyValue(...values) {
  for (const value of values) {
    if (typeof value === 'string') {
      const trimmed = value.trim();
      if (trimmed.length > 0) return trimmed;
    } else if (value !== null && value !== undefined) {
      return value;
    }
  }
  return null;
}

function optionalNonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0 ? value.trim() : null;
}

function optionalInteger(value, field, rowPath) {
  if (value === null || value === undefined || value === '') return null;
  return coerceInteger(value, field, rowPath);
}

function normalizeSourceBrowserVisibility(value, rowPath) {
  if (value !== null && value !== undefined && typeof value !== 'string') {
    throw new Error(`${rowPath}: source browser visibility must be a string`);
  }
  const normalized = optionalNonEmptyString(value)?.toLowerCase().replaceAll('-', '_') ?? null;
  if (normalized === null) return null;
  if (normalized === 'visible' || normalized === 'public' || normalized === 'browser_required') {
    return 'browser_visible';
  }
  if (normalized === 'actor' || normalized === 'auth_required' || normalized === 'authenticated') {
    return 'actor_required';
  }
  if (!SOURCE_BROWSER_VISIBILITIES.has(normalized)) {
    throw new Error(`${rowPath}: source browser visibility must be one of ${[...SOURCE_BROWSER_VISIBILITIES].join(', ')}`);
  }
  return normalized;
}

function classifySourceBrowserEvidence({ meta, manifestRow, rowPath }) {
  const visibility = normalizeSourceBrowserVisibility(
    firstNonEmptyValue(
      manifestRow?.browser_visibility,
      manifestRow?.source_browser_visibility,
      meta.source_browser_visibility,
    ),
    rowPath,
  );
  const sourceRequiredActor = optionalNonEmptyString(
    firstNonEmptyValue(
      manifestRow?.required_actor,
      manifestRow?.source_required_actor,
      meta.source_required_actor,
    ),
  );
  const sourceBrowserStatus = optionalInteger(
    firstNonEmptyValue(
      manifestRow?.source_browser_status,
      manifestRow?.browser_status,
      meta.source_browser_status,
    ),
    'source_browser_status',
    rowPath,
  );
  const explicitReason = optionalNonEmptyString(
    firstNonEmptyValue(
      manifestRow?.source_visibility_reason,
      manifestRow?.browser_visibility_reason,
      meta.source_visibility_reason,
    ),
  );

  if (visibility === 'actor_required' && sourceRequiredActor === null) {
    throw new Error(`${rowPath}: actor_required source browser visibility requires required_actor`);
  }

  const classifiedVisibility = visibility ?? 'source_only';
  const sourceVisibilityReason = explicitReason
    ?? (visibility === null && meta.revisions === 0
      ? 'zero_revisions_without_browser_visibility_proof'
      : visibility === null
        ? 'missing_browser_visibility_proof'
        : 'declared_source_browser_visibility');

  return {
    source_capture_method: meta.capture_method,
    source_browser_visibility: classifiedVisibility,
    source_browser_status: sourceBrowserStatus,
    source_required_actor: classifiedVisibility === 'actor_required' ? sourceRequiredActor : null,
    required_browser: classifiedVisibility === 'browser_visible' || classifiedVisibility === 'actor_required',
    source_visibility_reason: sourceVisibilityReason,
  };
}

function rowFromRecord({
  sourceSite,
  sourceBranch,
  sourceEntityId,
  sourcePath,
  sourceFile,
  metaPath,
  metaFile,
  entityIdPath,
  meta,
  sourceBrowser = null,
}) {
  const row = {
    source_site: sourceSite,
    source_branch: sourceBranch,
    source_entity_id: sourceEntityId,
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
    source_sha256: sha256Hex(sourceFile.buffer),
    meta_sha256: sha256Hex(metaFile.buffer),
    source_bytes: sourceFile.buffer.length,
    meta_bytes: metaFile.buffer.length,
    source_path: sourcePath,
    meta_path: metaPath,
    entity_id_path: entityIdPath,
  };
  if (sourceBrowser !== null) {
    Object.assign(row, sourceBrowser);
  }
  return row;
}

export function buildCorpusImportManifest({ corpusRoot = null, sourceBundleRoot = null, branch, sourceSite = null, sourceBranch = null }) {
  if ((corpusRoot === null) === (sourceBundleRoot === null)) {
    throw new Error('exactly one of corpusRoot or sourceBundleRoot is required');
  }
  if (sourceBundleRoot !== null) {
    const sourceBundleInput = { sourceBundleRoot, sourceSite };
    if (sourceBranch !== null) sourceBundleInput.sourceBranch = sourceBranch;
    return buildSourceBundleImportManifest(sourceBundleInput);
  }

  const pagesRoot = path.join(corpusRoot, branch, 'pages');
  const effectiveSourceSite = sourceSite ?? branch;
  const effectiveSourceBranch = sourceBranch ?? branch;
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

    const sourceFile = readUtf8File(sourcePath);
    const metaFile = readUtf8File(metaPath);
    const meta = JSON.parse(metaFile.text);
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

    rows.push(rowFromRecord({
      sourceSite: effectiveSourceSite,
      sourceBranch: effectiveSourceBranch,
      sourceEntityId: entityId,
      sourcePath,
      sourceFile,
      metaPath,
      metaFile,
      entityIdPath,
      meta,
    }));
  }

  return rows;
}

export function buildSourceBundleImportManifest({ sourceBundleRoot, sourceSite = null, sourceBranch = 'source-bundle' }) {
  const pagesRoot = path.join(sourceBundleRoot, 'pages');
  const bundleManifest = readSourceBundleManifest(sourceBundleRoot);
  const bundleManifestRows = bundleManifest.rows;
  const entries = fs.readdirSync(pagesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort(codePointCompare);

  const rows = [];
  const entityIds = new Map();
  const fullnames = new Map();
  const unconsumedManifestFullnames = bundleManifest.exists ? new Set(bundleManifestRows.keys()) : new Set();

  for (const directoryName of entries) {
    const pageDir = path.join(pagesRoot, directoryName);
    const sourcePath = path.join(pageDir, 'source.wikidot.txt');
    const metaPath = path.join(pageDir, 'meta.json');
    const entityIdPath = path.join(pageDir, 'entity_id.txt');

    for (const requiredPath of [sourcePath, metaPath]) {
      if (!fs.existsSync(requiredPath)) {
        throw new Error(`${pageDir}: missing ${path.basename(requiredPath)}`);
      }
    }

    const sourceFile = readUtf8File(sourcePath);
    const metaFile = readUtf8File(metaPath);
    const meta = normalizeSourceBundleMeta(JSON.parse(metaFile.text), pageDir);
    if (meta.fullname !== directoryName) {
      throw new Error(`${pageDir}: meta.fullname ${meta.fullname} does not match directory name ${directoryName}`);
    }
    const manifestRow = bundleManifestRows.get(meta.fullname);
    if (bundleManifest.exists && manifestRow === undefined) {
      throw new Error(`${pageDir}: missing row in corpus-manifest.tsv for ${meta.fullname}`);
    }
    unconsumedManifestFullnames.delete(meta.fullname);
    const explicitSourceSite = optionalNonEmptyString(sourceSite);
    const manifestSourceSite = optionalNonEmptyString(manifestRow?.site);
    const rowSourceSite = explicitSourceSite ?? manifestSourceSite ?? sourceBranch;
    const sourceSha256 = sha256Hex(sourceFile.buffer);
    const sourceEntityId = fs.existsSync(entityIdPath)
      ? readText(entityIdPath).trim()
      : deterministicUuid(`wikidot-source-bundle:${rowSourceSite}:${sourceBranch}:${meta.fullname}`);

    validateEntityId(sourceEntityId, pageDir);
    if (manifestRow !== undefined) {
      const manifestBytes = coerceNonNegativeInteger(manifestRow.source_bytes, 'source_bytes', pageDir);
      if (manifestBytes !== sourceFile.buffer.length) {
        throw new Error(`${pageDir}: source byte count mismatch with corpus-manifest.tsv`);
      }
      if (manifestRow.source_sha256.toLowerCase() !== sourceSha256) {
        throw new Error(`${pageDir}: source sha256 mismatch with corpus-manifest.tsv`);
      }
    }
    if (meta.source_bytes !== null && meta.source_bytes !== sourceFile.buffer.length) {
      throw new Error(`${pageDir}: meta.source_bytes does not match source.wikidot.txt`);
    }
    if (meta.source_sha256 !== null && meta.source_sha256.toLowerCase() !== sourceSha256) {
      throw new Error(`${pageDir}: meta.source_sha256 does not match source.wikidot.txt`);
    }
    if (entityIds.has(sourceEntityId)) {
      throw new Error(`${pageDir}: duplicate source_entity_id also used by ${entityIds.get(sourceEntityId)}`);
    }
    if (fullnames.has(meta.fullname)) {
      throw new Error(`${pageDir}: duplicate fullname also used by ${fullnames.get(meta.fullname)}`);
    }
    entityIds.set(sourceEntityId, pageDir);
    fullnames.set(meta.fullname, pageDir);

    const sourceBrowser = classifySourceBrowserEvidence({ meta, manifestRow, rowPath: pageDir });

    rows.push(rowFromRecord({
      sourceSite: rowSourceSite,
      sourceBranch,
      sourceEntityId,
      sourcePath,
      sourceFile,
      metaPath,
      metaFile,
      entityIdPath: fs.existsSync(entityIdPath) ? entityIdPath : null,
      meta,
      sourceBrowser,
    }));
  }

  if (unconsumedManifestFullnames.size > 0) {
    throw new Error(`${sourceBundleRoot}: corpus-manifest.tsv rows have no matching page directories: ${[...unconsumedManifestFullnames].sort(codePointCompare).slice(0, 20).join(', ')}`);
  }

  return rows;
}

export function formatJsonl(rows) {
  return `${rows.map((row) => stableStringify(row)).join('\n')}\n`;
}

export function buildManifestSummary(rows, jsonl) {
  const parentCount = rows.filter((row) => row.parent_fullname !== null).length;
  const sourceBrowserVisibilityCounts = {};
  for (const row of rows) {
    if (row.source_browser_visibility !== undefined) {
      sourceBrowserVisibilityCounts[row.source_browser_visibility] = (sourceBrowserVisibilityCounts[row.source_browser_visibility] ?? 0) + 1;
    }
  }
  return {
    row_count: rows.length,
    manifest_sha256: sha256Hex(jsonl),
    parent_count: parentCount,
    source_sites: [...new Set(rows.map((row) => row.source_site))].sort(codePointCompare),
    source_branches: [...new Set(rows.map((row) => row.source_branch))].sort(codePointCompare),
    required_browser_count: rows.filter((row) => row.required_browser === true).length,
    source_required_actor_count: rows.filter((row) => row.source_required_actor !== undefined && row.source_required_actor !== null).length,
    source_browser_visibility_counts: sourceBrowserVisibilityCounts,
    first_fullname: rows[0]?.fullname ?? null,
    last_fullname: rows.at(-1)?.fullname ?? null,
  };
}
