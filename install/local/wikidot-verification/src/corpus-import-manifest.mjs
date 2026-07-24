import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

import { codePointCompare, sha256Hex, stableStringify } from './canonical-json.mjs';
import { isWikidotResourceHost } from './resource-manifest.mjs';

export { sha256Hex, stableStringify } from './canonical-json.mjs';

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
const ATTACHMENT_MANIFEST_FILENAME = 'files.json';
const ATTACHMENT_STATE_MANIFEST_FILENAME = '_state.json';
const ATTACHMENT_FILES_DIRECTORY = 'files';
const SHA256_RE = /^[0-9a-f]{64}$/iu;

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
  if (typeof value !== 'string' || !SHA256_RE.test(value)) {
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

function toPosixPath(filePath) {
  return filePath.split(path.sep).join(path.posix.sep);
}

function assertAttachmentSha256(value, field, rowPath) {
  if (typeof value !== 'string' || !SHA256_RE.test(value)) {
    throw new Error(`${rowPath}: attachment ${field} must be a sha256 hex string`);
  }
}

function assertSafeAttachmentRelativePath(relativePath, rowPath) {
  if (typeof relativePath !== 'string' || relativePath.length === 0) {
    throw new Error(`${rowPath}: attachment path must be a non-empty string`);
  }
  if (
    path.posix.isAbsolute(relativePath) ||
    relativePath.includes('\\') ||
    relativePath.includes('\0')
  ) {
    throw new Error(`${rowPath}: attachment path must be a safe relative POSIX path`);
  }
  const segments = relativePath.split('/');
  if (
    segments[0] !== ATTACHMENT_FILES_DIRECTORY ||
    segments.some((segment) => segment.length === 0 || segment === '.' || segment === '..')
  ) {
    throw new Error(`${rowPath}: attachment path must remain below ${ATTACHMENT_FILES_DIRECTORY}/`);
  }
}

function assertPathWithin(rootPath, candidatePath, field, rowPath) {
  const relative = path.relative(rootPath, candidatePath);
  if (
    relative === '..' ||
    relative.startsWith(`..${path.sep}`) ||
    path.isAbsolute(relative)
  ) {
    throw new Error(`${rowPath}: attachment ${field} escapes its page directory`);
  }
}

function decodedFilenameFromWikidotPath(wikidotPath, rowPath) {
  const encoded = wikidotPath.split('/').at(-1) ?? '';
  try {
    const decoded = decodeURIComponent(encoded);
    if (
      decoded.length === 0 ||
      decoded === '.' ||
      decoded === '..' ||
      decoded.includes('/') ||
      decoded.includes('\\') ||
      decoded.includes('\0')
    ) {
      throw new Error('unsafe decoded filename');
    }
    return decoded;
  } catch (error) {
    throw new Error(`${rowPath}: attachment wikidot_path has invalid filename encoding: ${encoded}`, { cause: error });
  }
}

function validateAttachmentEntry(entry, index, rowPath) {
  const entryPath = `${rowPath}/${ATTACHMENT_MANIFEST_FILENAME}[${index}]`;
  if (entry === null || typeof entry !== 'object' || Array.isArray(entry)) {
    throw new Error(`${entryPath}: attachment entry must be an object`);
  }
  for (const field of ['filename', 'original_url', 'wikidot_path', 'path', 'sha256', 'mime', 'size']) {
    if (!Object.hasOwn(entry, field)) {
      throw new Error(`${entryPath}: missing attachment ${field}`);
    }
  }
  if (typeof entry.filename !== 'string' || entry.filename.length === 0) {
    throw new Error(`${entryPath}: attachment filename must be a non-empty string`);
  }
  if (entry.filename.includes('/') || entry.filename.includes('\\') || entry.filename.includes('\0')) {
    throw new Error(`${entryPath}: attachment filename contains an unsafe character`);
  }
  if (typeof entry.wikidot_path !== 'string' || !entry.wikidot_path.startsWith('/local--files/')) {
    throw new Error(`${entryPath}: attachment wikidot_path must start with /local--files/`);
  }
  if (decodedFilenameFromWikidotPath(entry.wikidot_path, entryPath) !== entry.filename) {
    throw new Error(`${entryPath}: attachment filename does not match wikidot_path`);
  }
  if (typeof entry.original_url !== 'string' || entry.original_url.length === 0) {
    throw new Error(`${entryPath}: attachment original_url must be a non-empty string`);
  }
  let parsed;
  try {
    parsed = new URL(entry.original_url);
  } catch (error) {
    throw new Error(`${entryPath}: attachment original_url is invalid: ${entry.original_url}`, { cause: error });
  }
  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    throw new Error(`${entryPath}: attachment original_url must use http or https`);
  }
  if (parsed.username || parsed.password || parsed.hash) {
    throw new Error(`${entryPath}: attachment original_url must not contain credentials or a fragment`);
  }
  if (!isWikidotResourceHost(parsed.hostname)) {
    throw new Error(`${entryPath}: attachment original_url host is out of scope: ${parsed.hostname}`);
  }
  if (parsed.pathname !== entry.wikidot_path) {
    throw new Error(`${entryPath}: attachment original_url pathname does not match wikidot_path`);
  }
  const canonicalUrl = `${parsed.origin}${parsed.pathname}${parsed.search}`;
  if (canonicalUrl !== entry.original_url) {
    throw new Error(`${entryPath}: attachment original_url is not canonical`);
  }
  assertSafeAttachmentRelativePath(entry.path, entryPath);
  assertAttachmentSha256(entry.sha256, 'sha256', entryPath);
  if (entry.sha256 !== entry.sha256.toLowerCase()) {
    throw new Error(`${entryPath}: attachment sha256 must be lowercase`);
  }
  if (typeof entry.mime !== 'string' || entry.mime.length === 0) {
    throw new Error(`${entryPath}: attachment mime must be a non-empty string`);
  }
  if (!Number.isSafeInteger(entry.size) || entry.size < 0) {
    throw new Error(`${entryPath}: attachment size must be a non-negative safe integer`);
  }
}

// Convert a capture-state attachment manifest (files/_state.json, the
// source-bundle spec format: {"files": {"<filename>": {download_url,
// mime_type, sha256, size, ...}}}) into files.json-shaped entries so both
// formats flow through the same validation below.
function attachmentEntriesFromStateManifest(statePath) {
  const state = readJson(statePath);
  if (state === null || typeof state !== 'object' || Array.isArray(state)) {
    throw new Error(`${statePath}: state manifest must be an object`);
  }
  const files = state.files ?? {};
  if (files === null || typeof files !== 'object' || Array.isArray(files)) {
    throw new Error(`${statePath}: state manifest "files" must be an object`);
  }
  return Object.entries(files).map(([filename, record]) => {
    if (record === null || typeof record !== 'object' || Array.isArray(record)) {
      throw new Error(`${statePath}: state entry for ${filename} must be an object`);
    }
    const downloadUrl = record.download_url;
    if (typeof downloadUrl !== 'string' || downloadUrl.length === 0) {
      throw new Error(`${statePath}: state entry for ${filename} is missing download_url`);
    }
    let wikidotPath;
    try {
      wikidotPath = new URL(downloadUrl).pathname;
    } catch (error) {
      throw new Error(`${statePath}: state entry for ${filename} has invalid download_url`, { cause: error });
    }
    const sha256 = typeof record.sha256 === 'string'
      ? record.sha256.replace(/^sha256:/, '')
      : record.sha256;
    return {
      filename,
      original_url: downloadUrl,
      wikidot_path: wikidotPath,
      path: `files/${filename}`,
      sha256,
      mime: record.mime_type,
      size: record.size,
    };
  });
}

function readAttachmentManifest({ pageDir, manifestRoot }) {
  let manifestPath = path.join(pageDir, ATTACHMENT_MANIFEST_FILENAME);
  let manifest;
  if (fs.existsSync(manifestPath)) {
    manifest = readJson(manifestPath);
  } else {
    const statePath = path.join(pageDir, 'files', ATTACHMENT_STATE_MANIFEST_FILENAME);
    if (!fs.existsSync(statePath)) return [];
    manifestPath = statePath;
    manifest = attachmentEntriesFromStateManifest(statePath);
  }
  if (!Array.isArray(manifest)) {
    throw new Error(`${manifestPath}: attachment manifest must be an array`);
  }

  const pageRoot = path.resolve(pageDir);
  const rootPath = path.resolve(manifestRoot);
  const seenFilenames = new Set();
  const seenPaths = new Set();
  const attachments = manifest.map((entry, index) => {
    validateAttachmentEntry(entry, index, pageDir);
    if (seenFilenames.has(entry.filename)) {
      throw new Error(`${pageDir}: duplicate attachment filename ${entry.filename}`);
    }
    if (seenPaths.has(entry.wikidot_path)) {
      throw new Error(`${pageDir}: duplicate attachment wikidot_path ${entry.wikidot_path}`);
    }
    seenFilenames.add(entry.filename);
    seenPaths.add(entry.wikidot_path);

    const filePath = path.resolve(pageRoot, ...entry.path.split('/'));
    assertPathWithin(pageRoot, filePath, 'path', pageDir);
    if (!fs.existsSync(filePath)) {
      throw new Error(`${pageDir}: missing attachment file ${entry.path}`);
    }
    const bytes = fs.readFileSync(filePath);
    const actualSha = sha256Hex(bytes);
    if (actualSha !== entry.sha256) {
      throw new Error(`${pageDir}: attachment ${entry.filename} sha256 mismatch: expected ${entry.sha256}, got ${actualSha}`);
    }
    if (bytes.length !== entry.size) {
      throw new Error(`${pageDir}: attachment ${entry.filename} size mismatch: expected ${entry.size}, got ${bytes.length}`);
    }

    return {
      filename: entry.filename,
      original_url: entry.original_url,
      wikidot_path: entry.wikidot_path,
      sha256: entry.sha256,
      size: entry.size,
      mime: entry.mime,
      file_path: filePath,
      corpus_path: toPosixPath(path.relative(rootPath, filePath)),
      metadata_path: manifestPath,
    };
  });

  attachments.sort((left, right) =>
    `${left.wikidot_path}\u0000${left.filename}`.localeCompare(`${right.wikidot_path}\u0000${right.filename}`),
  );
  return attachments;
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
  attachments = [],
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
  if (attachments.length > 0) {
    row.attachments = attachments;
  }
  if (sourceBrowser !== null) {
    Object.assign(row, sourceBrowser);
  }
  return row;
}

export function buildCorpusImportManifest({ corpusRoot = null, sourceBundleRoot = null, branch, sourceSite = null, sourceBranch = null, fullnames: fullnameFilter = null }) {
  if ((corpusRoot === null) === (sourceBundleRoot === null)) {
    throw new Error('exactly one of corpusRoot or sourceBundleRoot is required');
  }
  if (sourceBundleRoot !== null) {
    if (fullnameFilter !== null) {
      throw new Error('fullnames filtering is only supported with corpusRoot');
    }
    const sourceBundleInput = { sourceBundleRoot, sourceSite };
    if (sourceBranch !== null) sourceBundleInput.sourceBranch = sourceBranch;
    return buildSourceBundleImportManifest(sourceBundleInput);
  }

  const pagesRoot = path.join(corpusRoot, branch, 'pages');
  const effectiveSourceSite = sourceSite ?? branch;
  const effectiveSourceBranch = sourceBranch ?? branch;
  const requestedFullnames = fullnameFilter === null ? null : new Set(fullnameFilter);
  if (requestedFullnames !== null && requestedFullnames.size === 0) {
    throw new Error('fullnames filter must name at least one page');
  }
  const entries = fs.readdirSync(pagesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .filter((entry) => requestedFullnames === null || requestedFullnames.has(entry.name))
    .map((entry) => entry.name)
    .sort(codePointCompare);
  // Requested pages absent from the corpus must fail closed, not silently
  // shrink the manifest.
  if (requestedFullnames !== null && entries.length !== requestedFullnames.size) {
    const found = new Set(entries);
    const missing = [...requestedFullnames].filter((name) => !found.has(name)).sort(codePointCompare);
    throw new Error(`fullnames not found in corpus ${pagesRoot}: ${missing.join(', ')}`);
  }

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

    const attachments = readAttachmentManifest({
      pageDir,
      manifestRoot: corpusRoot,
    });

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
      attachments,
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

    const attachments = readAttachmentManifest({
      pageDir,
      manifestRoot: sourceBundleRoot,
    });

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
      attachments,
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
    attachment_count: rows.reduce((count, row) => count + (row.attachments?.length ?? 0), 0),
    attachment_page_count: rows.filter((row) => (row.attachments?.length ?? 0) > 0).length,
    source_sites: [...new Set(rows.map((row) => row.source_site))].sort(codePointCompare),
    source_branches: [...new Set(rows.map((row) => row.source_branch))].sort(codePointCompare),
    required_browser_count: rows.filter((row) => row.required_browser === true).length,
    source_required_actor_count: rows.filter((row) => row.source_required_actor !== undefined && row.source_required_actor !== null).length,
    source_browser_visibility_counts: sourceBrowserVisibilityCounts,
    first_fullname: rows[0]?.fullname ?? null,
    last_fullname: rows.at(-1)?.fullname ?? null,
  };
}
