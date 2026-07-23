#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

import { canReuseExistingPageForDbImport } from '../src/corpus-import-apply-policy.mjs';
import { assertEmptyDbImportTarget } from '../src/corpus-import-empty-target.mjs';
import {
  DEFAULT_IMPORT_USER_ID,
  validateAttachmentActorArgs,
} from '../src/corpus-attachment-policy.mjs';
import { planDirectAttachmentMaterialization } from '../src/corpus-attachment-direct.mjs';
import {
  commitDirectCorpusAttachmentStaging,
  materializeCorpusRowAttachments,
  summarizeCorpusAttachmentUpload,
  uploadDirectCorpusAttachmentBlobs,
} from '../src/corpus-import-attachments.mjs';
import {
  buildParentLinkParentPagesSql,
  buildParentLinkSql,
  parseParentLinkParentPages,
  parseParentLinkSummary,
  shouldProcessParentLinks,
} from '../src/corpus-import-parent-links.mjs';
import { createSqlExecutor } from '../src/corpus-import-sql.mjs';
import {
  sqlByteaFromHex,
  sqlInt,
  sqlQuote,
  sqlTextArray,
  sqlTextFromBase64,
  sqlTextHash,
  sqlTimestamp,
} from '../src/corpus-import-sql-values.mjs';
import {
  ensureCorpusImportRun,
  finishCorpusImportRun,
  recordCorpusImportItemSql,
} from '../src/corpus-import-run-state.mjs';
import { parsePrecreatedCategoryIds } from '../src/corpus-precreated-categories.mjs';
import {
  corpusImportCategoryName,
  createCorpusImportPage,
  getCorpusImportFile,
  getCorpusImportPage,
  rerenderCorpusImportPage,
} from '../src/corpus-import-page-rpc.mjs';

const DEFAULT_API_URL = 'http://localhost:2747/jsonrpc';
const DEFAULT_DB_CONTAINER = 'local-database-1';
const DEFAULT_SITE_ID = 6000005;
const DEFAULT_USER_ID = DEFAULT_IMPORT_USER_ID;
const DEFAULT_IP_ADDRESS = '127.0.0.1';
const DEFAULT_SESSION_TOKEN = process.env.DEEPWELL_SESSION_TOKEN ?? null;
const SHELL_COMPILED_GENERATOR = 'corpus db import';
const SHELL_IMPORT_MARKER = 'corpus-shell-import';
const SHELL_IMPORT_MESSAGE = 'Content not rendered yet for local Wikidot corpus snapshot import';
const SHELL_BODY_HTML = `<div class="wj-proof-stub ${SHELL_IMPORT_MARKER}">${SHELL_IMPORT_MESSAGE}.</div>`;
const FATAL_UTF8_DECODER = new TextDecoder('utf-8', { fatal: true });
let shellBodyHash = null;
let shellBodyTextPrecreated = false;
const precomputedTextHashes = new Map();
const precomputedSourceTexts = new Map();
const precreatedSourceTextHashes = new Set();
const precreatedCategoryIds = new Map();
const SOURCE_TEXT_PRECREATE_MAX_ROWS = 200;
const SOURCE_TEXT_PRECREATE_MAX_BASE64_BYTES = 4 * 1024 * 1024;
const DB_SHELL_BATCH_MAX_ROWS = 200;
const ensureImportRun = ensureCorpusImportRun;
const recordItemSql = recordCorpusImportItemSql;
const finishRun = finishCorpusImportRun;

function monotonicMsSince(start) {
  return Number(process.hrtime.bigint() - start) / 1_000_000;
}

function recordPhaseTiming(timings, name, startedAt) {
  timings[name] = Math.round(monotonicMsSince(startedAt) * 1000) / 1000;
}

function timePhaseSync(timings, name, callback) {
  const startedAt = process.hrtime.bigint();
  try {
    return callback();
  } finally {
    recordPhaseTiming(timings, name, startedAt);
  }
}

async function timePhase(timings, name, callback) {
  const startedAt = process.hrtime.bigint();
  try {
    return await callback();
  } finally {
    recordPhaseTiming(timings, name, startedAt);
  }
}

function envString(name) {
  const value = process.env[name];
  return value === undefined || value === '' ? null : value;
}

function parseBooleanString(value, label) {
  if (value === null || value === undefined || value === '') return null;
  if (value === 'true') return true;
  if (value === 'false') return false;
  throw new Error(`${label} must be true or false`);
}

function parseArgs(argv) {
  const args = {
    manifest: null,
    migration: path.resolve('deepwell/migrations/20260625104500_wikidot_corpus_import.sql'),
    applyMigration: false,
    apiUrl: DEFAULT_API_URL,
    dbUrl: process.env.DEEPWELL_VERIFY_DB_URL ?? null,
    dbContainer: DEFAULT_DB_CONTAINER,
    siteId: DEFAULT_SITE_ID,
    userId: DEFAULT_USER_ID,
    attachmentUserId: null,
    ipAddress: DEFAULT_IP_ADDRESS,
    sessionToken: DEFAULT_SESSION_TOKEN,
    presignHostAlias: process.env.DEEPWELL_PRESIGN_HOST_ALIAS ? [process.env.DEEPWELL_PRESIGN_HOST_ALIAS] : [],
    rpcTimeoutMs: 120_000,
    textHashCommand: process.env.DEEPWELL_TEXT_HASH_COMMAND ?? null,
    textHashBatchCommand: process.env.DEEPWELL_TEXT_HASH_BATCH_COMMAND ?? null,
    slug: [],
    slugFile: null,
    limit: null,
    adoptExisting: false,
    skipExistingDone: false,
    skipRerender: false,
    rerenderAfterDbCreate: false,
    replaceExisting: false,
    assumeEmptyDbImport: false,
    attachmentsOnlyExisting: false,
    skipAttachments: false,
    createMode: 'rpc',
    attachmentCreateMode: 'rpc',
    attachmentS3Endpoint: envString('S3_CUSTOM_ENDPOINT'),
    attachmentS3Bucket: envString('S3_FILES_BUCKET'),
    attachmentS3AccessKeyId: envString('S3_ACCESS_KEY_ID'),
    attachmentS3SecretAccessKey: envString('S3_SECRET_ACCESS_KEY'),
    attachmentS3Region: envString('S3_REGION_NAME'),
    attachmentS3PathStyle: parseBooleanString(process.env.S3_PATH_STYLE, 'S3_PATH_STYLE'),
    dryRun: false,
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = () => {
      index += 1;
      if (index >= argv.length) throw new Error(`${arg} requires a value`);
      return argv[index];
    };
    if (arg === '--manifest') args.manifest = next();
    else if (arg === '--migration') args.migration = next();
    else if (arg === '--apply-migration') args.applyMigration = true;
    else if (arg === '--api-url') args.apiUrl = next();
    else if (arg === '--db-url') args.dbUrl = next();
    else if (arg === '--db-container') args.dbContainer = next();
    else if (arg === '--site-id') args.siteId = Number.parseInt(next(), 10);
    else if (arg === '--user-id') args.userId = Number.parseInt(next(), 10);
    else if (arg === '--attachment-user-id') args.attachmentUserId = Number.parseInt(next(), 10);
    else if (arg === '--ip-address') args.ipAddress = next();
    else if (arg === '--session-token') args.sessionToken = next();
    else if (arg === '--presign-host-alias') args.presignHostAlias.push(next());
    else if (arg === '--rpc-timeout-ms') args.rpcTimeoutMs = Number.parseInt(next(), 10);
    else if (arg === '--text-hash-command') args.textHashCommand = next();
    else if (arg === '--text-hash-batch-command') args.textHashBatchCommand = next();
    else if (arg === '--slug') args.slug.push(next());
    else if (arg === '--slug-file') args.slugFile = next();
    else if (arg === '--limit') args.limit = Number.parseInt(next(), 10);
    else if (arg === '--adopt-existing') args.adoptExisting = true;
    else if (arg === '--skip-existing-done') args.skipExistingDone = true;
    else if (arg === '--skip-rerender') args.skipRerender = true;
    else if (arg === '--rerender-after-db-create') args.rerenderAfterDbCreate = true;
    else if (arg === '--replace-existing') args.replaceExisting = true;
    else if (arg === '--assume-empty-db-import') args.assumeEmptyDbImport = true;
    else if (arg === '--attachments-only-existing') args.attachmentsOnlyExisting = true;
    else if (arg === '--skip-attachments') args.skipAttachments = true;
    else if (arg === '--create-mode') {
      args.createMode = next();
    }
    else if (arg === '--attachment-create-mode') {
      args.attachmentCreateMode = next();
    }
    else if (arg === '--attachment-s3-endpoint') args.attachmentS3Endpoint = next();
    else if (arg === '--attachment-s3-bucket') args.attachmentS3Bucket = next();
    else if (arg === '--attachment-s3-access-key-id') args.attachmentS3AccessKeyId = next();
    else if (arg === '--attachment-s3-secret-access-key') args.attachmentS3SecretAccessKey = next();
    else if (arg === '--attachment-s3-region') args.attachmentS3Region = next();
    else if (arg === '--attachment-s3-path-style') args.attachmentS3PathStyle = parseBooleanString(next(), '--attachment-s3-path-style');
    else if (arg === '--dry-run') args.dryRun = true;
    else if (arg === '--source-site') args.sourceSite = next();
    else if (arg === '--source-branch') args.sourceBranch = next();
    else if (arg === '--help' || arg === '-h') {
      console.log(`Usage: apply-corpus-import-manifest.mjs --manifest <manifest.jsonl> [--apply-migration] [--slug <slug>...] [--adopt-existing] [--replace-existing] [--assume-empty-db-import] [--skip-existing-done] [--skip-rerender] [--attachments-only-existing] [--skip-attachments] [--create-mode rpc|db] [--attachment-create-mode rpc|direct] [--attachment-s3-endpoint <url>] [--attachment-s3-bucket <bucket>] [--attachment-s3-access-key-id <key>] [--attachment-s3-secret-access-key <secret>] [--attachment-s3-region <region>] [--attachment-s3-path-style true|false] [--rerender-after-db-create] [--db-url postgres://wikijump:wikijump@127.0.0.1:5432/wikijump] [--text-hash-command <cmd>] [--text-hash-batch-command <cmd>] [--session-token <token>] [--attachment-user-id <id>] [--presign-host-alias files=127.0.0.1] [--dry-run]

Imports current corpus snapshot pages into a local Wikijump mirror. This is an operator-only local tool: it uses Deepwell JSON-RPC for page create/rerender and corpus-backed file attachment materialization, and direct Postgres SQL for corpus snapshot metadata, timestamps, and tags. Set --db-url or DEEPWELL_VERIFY_DB_URL to use a persistent Postgres client instead of docker exec psql. Non-dry-run --assume-empty-db-import is disabled until its site-level empty-page guard and DB shell writes are atomic; dry-run accepts the flag for planning without probing or changing the target. RPC attachment materialization requires --session-token or DEEPWELL_SESSION_TOKEN so Deepwell file_create has an authenticated request context. Direct attachment materialization requires --db-url plus --attachment-user-id or a non-default --user-id, and uploads blobs with S3 config from --attachment-s3-* options or S3_CUSTOM_ENDPOINT, S3_FILES_BUCKET, S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY, S3_REGION_NAME, and S3_PATH_STYLE. Pass --attachment-user-id, or --user-id if page and attachment attribution should be the same authenticated user. Use --attachments-only-existing to materialize attachments for already-imported pages without replacing page source snapshots. Use --skip-attachments to defer attachment materialization without requiring a session token. Use --presign-host-alias only when Deepwell returns a Docker-internal file-service host that the local operator process cannot resolve.`);
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (!args.manifest) throw new Error('--manifest is required');
  if (!Number.isInteger(args.siteId)) throw new Error('--site-id must be an integer');
  if (!Number.isInteger(args.userId)) throw new Error('--user-id must be an integer');
  if (args.attachmentUserId !== null && !Number.isInteger(args.attachmentUserId)) {
    throw new Error('--attachment-user-id must be an integer');
  }
  if (!Number.isInteger(args.rpcTimeoutMs) || args.rpcTimeoutMs <= 0) throw new Error('--rpc-timeout-ms must be a positive integer');
  if (!['rpc', 'db'].includes(args.createMode)) throw new Error('--create-mode must be rpc or db');
  if (!['rpc', 'direct'].includes(args.attachmentCreateMode)) throw new Error('--attachment-create-mode must be rpc or direct');
  if (args.skipAttachments && args.attachmentCreateMode === 'direct') {
    throw new Error('--skip-attachments cannot be combined with --attachment-create-mode direct');
  }
  if (args.attachmentCreateMode === 'direct' && !args.dryRun) {
    if (!args.dbUrl) throw new Error('--attachment-create-mode direct requires --db-url or DEEPWELL_VERIFY_DB_URL');
    if ((args.attachmentUserId ?? null) === null && args.userId === DEFAULT_USER_ID) {
      throw new Error('--attachment-create-mode direct requires --attachment-user-id or non-default --user-id');
    }
  }
  if (args.rerenderAfterDbCreate && args.createMode !== 'db') {
    throw new Error('--rerender-after-db-create requires --create-mode db');
  }
  if (args.rerenderAfterDbCreate && args.skipRerender) {
    throw new Error('--rerender-after-db-create cannot be combined with --skip-rerender');
  }
  if (args.createMode === 'db' && !args.rerenderAfterDbCreate) args.skipRerender = true;
  if (args.createMode === 'db' && !args.dryRun && !args.textHashCommand && !args.textHashBatchCommand) {
    throw new Error('--create-mode db requires --text-hash-command/DEEPWELL_TEXT_HASH_COMMAND or --text-hash-batch-command/DEEPWELL_TEXT_HASH_BATCH_COMMAND');
  }
  if (args.replaceExisting && args.createMode !== 'db') {
    throw new Error('--replace-existing requires --create-mode db');
  }
  if (args.assumeEmptyDbImport && args.createMode !== 'db') {
    throw new Error('--assume-empty-db-import requires --create-mode db');
  }
  if (args.assumeEmptyDbImport && (args.adoptExisting || args.replaceExisting)) {
    throw new Error('--assume-empty-db-import cannot be combined with --adopt-existing or --replace-existing');
  }
  if (args.assumeEmptyDbImport && !args.dryRun) {
    throw new Error('--assume-empty-db-import is disabled until its empty-target guard and DB shell writes are atomic');
  }
  if (args.attachmentsOnlyExisting && args.replaceExisting) {
    throw new Error('--attachments-only-existing cannot be combined with --replace-existing');
  }
  if (args.skipAttachments && args.attachmentsOnlyExisting) {
    throw new Error('--skip-attachments cannot be combined with --attachments-only-existing');
  }
  if (args.attachmentsOnlyExisting && args.rerenderAfterDbCreate) {
    throw new Error('--attachments-only-existing cannot be combined with --rerender-after-db-create');
  }
  if (args.limit !== null && (!Number.isInteger(args.limit) || args.limit < 0)) {
    throw new Error('--limit must be a non-negative integer');
  }
  args.presignHostAliases = parsePresignHostAliases(args.presignHostAlias);
  return args;
}

function parsePresignHostAliases(values) {
  const aliases = new Map();
  for (const value of values) {
    const index = value.indexOf('=');
    if (index <= 0 || index === value.length - 1) {
      throw new Error('--presign-host-alias must be formatted as hostname=address');
    }
    const hostname = value.slice(0, index).trim().toLowerCase();
    const address = value.slice(index + 1).trim();
    if (!hostname || hostname.includes('/') || hostname.includes(':')) {
      throw new Error('--presign-host-alias hostname must be a bare hostname');
    }
    if (!address) throw new Error('--presign-host-alias address must be non-empty');
    aliases.set(hostname, address);
  }
  return aliases;
}

function validateTextHash(hash) {
  if (!/^[0-9a-f]{32}$/iu.test(hash)) {
    throw new Error(`text hash command returned invalid 16-byte hex hash: ${hash}`);
  }
  return hash.toLowerCase();
}

function textHashHex(args, contents, cacheKey = null) {
  if (cacheKey !== null && precomputedTextHashes.has(cacheKey)) {
    return precomputedTextHashes.get(cacheKey);
  }
  if (!args.textHashCommand) {
    throw new Error(`missing precomputed text hash for ${cacheKey ?? '<uncached text>'}; set --text-hash-command or use --text-hash-batch-command before DB import`);
  }
  const result = spawnSync(args.textHashCommand, {
    input: contents,
    shell: true,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });
  if (result.status !== 0) {
    throw new Error(`text hash command failed (${result.status})\nSTDOUT:\n${result.stdout}\nSTDERR:\n${result.stderr}`);
  }
  return validateTextHash(result.stdout.trim());
}

function shellBodyHashHex(args) {
  if (shellBodyHash === null) shellBodyHash = textHashHex(args, SHELL_BODY_HTML, '__shell_body__');
  return shellBodyHash;
}

function batchTextHashesHex(args, items) {
  const input = items
    .map(({ id, contents }) => `${id}\t${Buffer.from(contents, 'utf8').toString('base64')}`)
    .join('\n') + '\n';
  const result = spawnSync(args.textHashBatchCommand, {
    input,
    shell: true,
    encoding: 'utf8',
    maxBuffer: Math.max(1024 * 1024, items.length * 80),
  });
  if (result.status !== 0) {
    throw new Error(`text hash batch command failed (${result.status})\nSTDOUT:\n${result.stdout}\nSTDERR:\n${result.stderr}`);
  }
  const hashes = new Map();
  for (const line of result.stdout.split('\n')) {
    if (!line.trim()) continue;
    const [id, hash, extra] = line.split('\t');
    if (!id || extra !== undefined) throw new Error(`invalid text hash batch output line: ${line}`);
    hashes.set(id, validateTextHash(hash));
  }
  if (hashes.size !== items.length) {
    throw new Error(`text hash batch command returned ${hashes.size} hashes for ${items.length} inputs`);
  }
  return hashes;
}

function precomputeDbTextHashes(args, selectedRows) {
  if (args.createMode !== 'db' || !args.textHashBatchCommand) return;
  const items = [{ id: '__shell_body__', contents: SHELL_BODY_HTML }];
  for (let index = 0; index < selectedRows.length; index += 1) {
    const contents = sourceText(selectedRows[index]);
    precomputedSourceTexts.set(selectedRows[index].fullname, contents);
    items.push({ id: `page:${index}`, contents });
  }
  const hashes = batchTextHashesHex(args, items);
  shellBodyHash = hashes.get('__shell_body__');
  for (let index = 0; index < selectedRows.length; index += 1) {
    precomputedTextHashes.set(selectedRows[index].fullname, hashes.get(`page:${index}`));
  }
}

async function precreateDbShellBodyText(args, sqlExecutor, selectedRows) {
  if (args.createMode !== 'db' || selectedRows.length === 0) return;
  const sql = `
INSERT INTO text (hash, contents)
VALUES (${sqlTextHash(shellBodyHashHex(args))}, ${sqlTextFromBase64(SHELL_BODY_HTML)})
ON CONFLICT (hash) DO NOTHING;
`;
  await sqlExecutor.runSql(sql);
  shellBodyTextPrecreated = true;
}

async function precreateDbSourceTexts(args, sqlExecutor, selectedRows) {
  if (args.createMode !== 'db' || !args.textHashBatchCommand || selectedRows.length === 0) return;
  let batch = [];
  let batchBytes = 0;

  async function flush() {
    if (batch.length === 0) return;
    const sql = `
INSERT INTO text (hash, contents)
VALUES
${batch.map((item) => item.valueSql).join(',\n')}
ON CONFLICT (hash) DO NOTHING;
`;
    await sqlExecutor.runSql(sql);
    for (const item of batch) {
      precreatedSourceTextHashes.add(item.fullname);
    }
    batch = [];
    batchBytes = 0;
  }

  for (const row of selectedRows) {
    const contents = sourceText(row);
    const hash = textHashHex(args, contents, row.fullname);
    const encodedBytes = Buffer.byteLength(contents, 'utf8') * 4 / 3;
    if (batch.length > 0 && (batch.length >= SOURCE_TEXT_PRECREATE_MAX_ROWS || batchBytes + encodedBytes > SOURCE_TEXT_PRECREATE_MAX_BASE64_BYTES)) {
      await flush();
    }
    batch.push({
      fullname: row.fullname,
      valueSql: `(${sqlTextHash(hash)}, ${sqlTextFromBase64(contents)})`,
    });
    batchBytes += encodedBytes;
  }
  await flush();
}

function buildPrecreateDbShellCategoriesSql(args, selectedRows) {
  const categories = [...new Set(selectedRows.map((row) => categoryName(row.fullname)))].sort();
  if (args.createMode !== 'db' || categories.length === 0) return null;
  const values = categories.map((category) => `(${sqlInt(args.siteId)}, ${sqlQuote(category)})`).join(',\n  ');
  return `
WITH requested(site_id, slug) AS (
  VALUES
  ${values}
), inserted AS (
  INSERT INTO page_category (site_id, slug)
  SELECT site_id, slug
  FROM requested
  ON CONFLICT (site_id, slug) DO UPDATE SET slug = EXCLUDED.slug
  RETURNING category_id, slug
)
SELECT COALESCE(
  json_agg(
    json_build_object('slug', slug, 'category_id', category_id::text)
    ORDER BY slug
  ),
  '[]'::json
)::text
FROM inserted;
`;
}

async function precreateDbShellCategories(args, sqlExecutor, selectedRows) {
  const sql = buildPrecreateDbShellCategoriesSql(args, selectedRows);
  if (sql === null) return;
  const ids = parsePrecreatedCategoryIds(await sqlExecutor.runSql(sql, { capture: true }));
  const expected = new Set(selectedRows.map((row) => categoryName(row.fullname)));
  if (ids.size !== expected.size || [...expected].some((slug) => !ids.has(slug))) {
    throw new Error('category precreate output did not contain every requested category');
  }
  for (const [slug, categoryId] of ids) {
    precreatedCategoryIds.set(slug, categoryId);
  }
}

async function applyMigration(args, sqlExecutor) {
  const migrationSql = fs.readFileSync(args.migration, 'utf8');
  await sqlExecutor.runSql(migrationSql);
}

let rpcSequence = 0;
async function rpc(args, method, params, requestContext = {}) {
  rpcSequence += 1;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), args.rpcTimeoutMs);
  const headers = { 'content-type': 'application/json' };
  const sessionToken = requestContext.sessionToken ?? args.sessionToken;
  const siteId = requestContext.siteId ?? null;
  const pageRef = requestContext.pageRef ?? null;
  if (sessionToken) headers['X-Deepwell-Session-Token'] = sessionToken;
  if (siteId !== null) headers['X-Deepwell-Site-Id'] = String(siteId);
  if (pageRef !== null) headers['X-Deepwell-Page'] = String(pageRef);
  let response;
  try {
    response = await fetch(args.apiUrl, {
      method: 'POST',
      redirect: 'error',
      headers,
      body: JSON.stringify({ jsonrpc: '2.0', id: rpcSequence, method, params }),
      signal: controller.signal,
    });
  } catch (error) {
    if (error.name === 'AbortError') {
      throw new Error(`${method} timed out after ${args.rpcTimeoutMs}ms`);
    }
    throw error;
  } finally {
    clearTimeout(timeout);
  }
  const data = await response.json();
  if (data.error) {
    throw new Error(`${method} failed: ${JSON.stringify(data.error)}`);
  }
  return data.result;
}

function parseRows(manifestText) {
  return manifestText
    .split('\n')
    .filter((line) => line.trim().length > 0)
    .map((line) => JSON.parse(line));
}

function readSlugSet(args) {
  const slugs = new Set(args.slug);
  if (args.slugFile) {
    for (const line of fs.readFileSync(args.slugFile, 'utf8').split('\n')) {
      const slug = line.trim();
      if (slug && !slug.startsWith('#')) slugs.add(slug);
    }
  }
  return slugs;
}

function filterRows(args, rows) {
  const slugSet = readSlugSet(args);
  let filtered = rows;
  if (slugSet.size > 0) filtered = filtered.filter((row) => slugSet.has(row.fullname));
  if (args.limit !== null) filtered = filtered.slice(0, args.limit);
  return filtered;
}

function fallbackTitle(row) {
  return row.title || row.title_shown || row.fullname;
}

function readManifestFile(row, pathKey, shaKey) {
  const filePath = row[pathKey];
  const expectedSha = String(row[shaKey]).toLowerCase();
  const contents = fs.readFileSync(filePath);
  const actualSha = crypto.createHash('sha256').update(contents).digest('hex');
  if (actualSha !== expectedSha) {
    throw new Error(`${pathKey} hash mismatch for ${row.fullname}: expected ${expectedSha}, got ${actualSha}`);
  }
  try {
    return FATAL_UTF8_DECODER.decode(contents);
  } catch (error) {
    throw new Error(`${pathKey} invalid UTF-8 for ${row.fullname}: ${error.message}`);
  }
}

function sourceText(row) {
  const cached = precomputedSourceTexts.get(row.fullname);
  if (cached !== undefined) return cached;
  return readManifestFile(row, 'source_path', 'source_sha256');
}

function metaJsonText(row) {
  return readManifestFile(row, 'meta_path', 'meta_sha256');
}

const getPage = (args, slug) => getCorpusImportPage(args, rpc, slug);
const getFile = (args, pageId, filename) => getCorpusImportFile(args, rpc, pageId, filename);

function materializeRowAttachments(args, row, pageId) {
  return materializeCorpusRowAttachments({ args, row, pageId, getFile, rpc });
}

const createPage = (args, row) => createCorpusImportPage(args, rpc, row, sourceText(row));
const categoryName = corpusImportCategoryName;

async function shellCreatePage(args, sqlExecutor, row, { replaceExistingRevision = false } = {}) {
  const sourceTextPrecreated = precreatedSourceTextHashes.has(row.fullname);
  const wikitext = sourceTextPrecreated ? '' : sourceText(row);
  const wikitextHash = textHashHex(args, wikitext, row.fullname);
  const bodyHash = shellBodyHashHex(args);
  const title = fallbackTitle(row);
  const category = categoryName(row.fullname);
  const precreatedCategoryId = precreatedCategoryIds.get(category);
  const categorySql = precreatedCategoryId === undefined
    ? `category AS (
  INSERT INTO page_category (site_id, slug)
  VALUES (${sqlInt(args.siteId)}, ${sqlQuote(category)})
  ON CONFLICT (site_id, slug) DO UPDATE SET slug = EXCLUDED.slug
  RETURNING category_id
)`
    : `category AS (
  SELECT ${sqlInt(precreatedCategoryId)}::bigint AS category_id
)`;
  const bodyTextSql = shellBodyTextPrecreated
    ? ''
    : `, inserted_body AS (
  INSERT INTO text (hash, contents)
  VALUES (${sqlTextHash(bodyHash)}, ${sqlTextFromBase64(SHELL_BODY_HTML)})
  ON CONFLICT (hash) DO NOTHING
  RETURNING 1
)`;
  const wikitextSql = sourceTextPrecreated
    ? `prefetched_wikitext AS (
  SELECT 1
)`
    : `inserted_wikitext AS (
  INSERT INTO text (hash, contents)
  VALUES (${sqlTextHash(wikitextHash)}, ${sqlTextFromBase64(wikitext)})
  ON CONFLICT (hash) DO NOTHING
  RETURNING 1
)`;
  const canUseExisting = canReuseExistingPageForDbImport(args, { replaceExistingRevision });
  const sql = `
CREATE TEMP TABLE corpus_shell_import_result (
  page_id BIGINT NOT NULL,
  page_category_id BIGINT NOT NULL,
  latest_revision_id BIGINT,
  existed BOOLEAN NOT NULL,
  created_page BOOLEAN NOT NULL,
  created_revision BOOLEAN NOT NULL DEFAULT false
) ON COMMIT DROP;

WITH ${wikitextSql}${bodyTextSql}, ${categorySql}, target_page AS (
  SELECT
    p.page_id,
    p.page_category_id,
    COALESCE(p.latest_revision_id, latest_revision.revision_id) AS latest_revision_id,
    true AS existed
  FROM page p
  LEFT JOIN LATERAL (
    SELECT revision_id
    FROM page_revision pr
    WHERE pr.page_id = p.page_id
    ORDER BY pr.revision_number DESC, pr.revision_id DESC
    LIMIT 1
  ) latest_revision ON true
  WHERE p.site_id = ${sqlInt(args.siteId)}
    AND p.slug = ${sqlQuote(row.fullname)}
    AND p.deleted_at IS NULL
    AND ${canUseExisting ? 'TRUE' : 'FALSE'}
  ORDER BY p.page_id
  LIMIT 1
  FOR UPDATE OF p
), inserted_page AS (
  INSERT INTO page (created_at, updated_at, from_wikidot, site_id, page_category_id, slug)
  SELECT ${sqlTimestamp(row.created_at)}, ${sqlTimestamp(row.updated_at)}, true, ${sqlInt(args.siteId)}, category_id, ${sqlQuote(row.fullname)}
  FROM category
  WHERE NOT EXISTS (SELECT 1 FROM target_page)
  RETURNING page_id, page_category_id, latest_revision_id, false AS existed
), page_row AS (
  SELECT page_id, page_category_id, latest_revision_id, existed
  FROM target_page
  UNION ALL
  SELECT page_id, page_category_id, latest_revision_id, existed
  FROM inserted_page
)
INSERT INTO corpus_shell_import_result (page_id, page_category_id, latest_revision_id, existed, created_page)
SELECT page_id, page_category_id, latest_revision_id, existed, NOT existed
FROM page_row;

WITH new_revision AS (
  INSERT INTO page_revision (
    revision_type,
    created_at,
    revision_number,
    page_id,
    site_id,
    user_id,
    from_wikidot,
    changes,
    wikitext_hash,
    compiled_body_html_hash,
    compiled_top_bar_html_hash,
    compiled_side_bar_html_hash,
    compiled_at,
    compiled_generator,
    comments,
    hidden,
    title,
    alt_title,
    slug,
    tags
  )
  SELECT
    CASE
      WHEN latest_revision_id IS NULL THEN 'create'
      ELSE 'regular'
    END,
    ${sqlTimestamp(row.updated_at)},
    CASE
      WHEN latest_revision_id IS NULL THEN 0
      ELSE COALESCE((
        SELECT MAX(pr.revision_number) + 1
        FROM page_revision pr
        WHERE pr.page_id = corpus_shell_import_result.page_id
      ), 0)
    END,
    page_id,
    ${sqlInt(args.siteId)},
    ${sqlInt(args.userId)},
    true,
    ARRAY['wikitext', 'title', 'alt_title', 'slug', 'tags']::text[],
    ${sqlTextHash(wikitextHash)},
    ${sqlTextHash(bodyHash)},
    NULL,
    NULL,
    NOW(),
    ${sqlQuote(SHELL_COMPILED_GENERATOR)},
    'local scp-wiki mirror DB import from scp-wiki-translation corpus',
    ARRAY[]::text[],
    ${sqlQuote(title)},
    NULL,
    ${sqlQuote(row.fullname)},
    ${sqlTextArray(row.tags)}
  FROM corpus_shell_import_result
  WHERE latest_revision_id IS NULL
     OR (${replaceExistingRevision ? 'TRUE' : 'FALSE'} AND existed)
  RETURNING revision_id
), updated_page AS (
  UPDATE page
  SET
    latest_revision_id = COALESCE((SELECT revision_id FROM new_revision), corpus_shell_import_result.latest_revision_id),
    created_at = ${sqlTimestamp(row.created_at)},
    updated_at = ${sqlTimestamp(row.updated_at)},
    from_wikidot = true,
    page_category_id = corpus_shell_import_result.page_category_id
  FROM corpus_shell_import_result
  WHERE page.page_id = corpus_shell_import_result.page_id
  RETURNING page.page_id, page.page_category_id, page.latest_revision_id,
    (SELECT revision_id FROM new_revision) IS NOT NULL AS created_revision
)
UPDATE corpus_shell_import_result
SET
  latest_revision_id = updated_page.latest_revision_id,
  created_revision = updated_page.created_revision
FROM updated_page
WHERE corpus_shell_import_result.page_id = updated_page.page_id;

SELECT page_id || '|' || page_category_id || '|' || COALESCE(latest_revision_id::text, '') || '|' || existed::text || '|' || created_revision::text
FROM corpus_shell_import_result;
`;
  const output = await sqlExecutor.runSql(sql, { capture: true });
  const [pageIdText, categoryIdText, revisionIdText = '', pageExistedText = '', createdRevisionText = ''] = output.split('|');
  const pageId = Number.parseInt(pageIdText, 10);
  const categoryId = Number.parseInt(categoryIdText, 10);
  const revisionId = revisionIdText === '' ? null : Number.parseInt(revisionIdText, 10);
  const pageExisted = pageExistedText === 'true';
  const createdRevision = createdRevisionText === 'true';
  if (!Number.isInteger(pageId) || !Number.isInteger(categoryId) || (revisionId !== null && !Number.isInteger(revisionId)) || !['true', 'false'].includes(pageExistedText) || !['true', 'false'].includes(createdRevisionText)) {
    throw new Error(`invalid DB shell import output: ${output}`);
  }
  if (revisionId === null) {
    throw new Error(`DB shell import did not create or find a latest revision: ${output}`);
  }
  return { page_id: pageId, page_category_id: categoryId, revision_id: revisionId, created_page: !pageExisted, created_revision: createdRevision };
}

const rerenderPage = (args, pageId, categoryId) => rerenderCorpusImportPage(args, rpc, pageId, categoryId);

function upsertSnapshotSql(args, row, pageId, revisionId, importRunId) {
  const metaText = metaJsonText(row);
  const title = fallbackTitle(row);
  return `
UPDATE page
SET
  created_at = ${sqlTimestamp(row.created_at)},
  updated_at = ${sqlTimestamp(row.updated_at)},
  from_wikidot = true
WHERE page_id = ${sqlInt(pageId)};

UPDATE page_revision
SET
  created_at = ${sqlTimestamp(row.updated_at)},
  from_wikidot = true,
  title = ${sqlQuote(title)},
  tags = ${sqlTextArray(row.tags)}
WHERE revision_id = ${sqlInt(revisionId)};

INSERT INTO wikidot_page_snapshot (
  page_id,
  source_branch,
  source_site,
  source_entity_id,
  source_fullname,
  source_created_at,
  source_updated_at,
  source_revision_count,
  imported_rating,
  created_by_name,
  updated_by_name,
  title_shown,
  parent_fullname,
  comments,
  commented_at,
  commented_by_name,
  source_sha256,
  meta_sha256,
  meta_json,
  last_import_run_id
) VALUES (
  ${sqlInt(pageId)},
  ${sqlQuote(row.source_branch)},
  ${sqlQuote(row.source_site)},
  ${sqlQuote(row.source_entity_id)},
  ${sqlQuote(row.fullname)},
  ${sqlTimestamp(row.created_at)},
  ${sqlTimestamp(row.updated_at)},
  ${sqlInt(row.revisions)},
  ${sqlInt(row.rating)},
  ${sqlQuote(row.created_by)},
  ${sqlQuote(row.updated_by)},
  ${sqlQuote(row.title_shown)},
  ${sqlQuote(row.parent_fullname)},
  ${sqlInt(row.comments)},
  ${sqlTimestamp(row.commented_at)},
  ${sqlQuote(row.commented_by)},
  ${sqlByteaFromHex(row.source_sha256)},
  ${sqlByteaFromHex(row.meta_sha256)},
  ${sqlQuote(metaText)}::jsonb,
  ${sqlInt(importRunId)}
)
ON CONFLICT (page_id) DO UPDATE SET
  source_branch = EXCLUDED.source_branch,
  source_site = EXCLUDED.source_site,
  source_entity_id = EXCLUDED.source_entity_id,
  source_fullname = EXCLUDED.source_fullname,
  source_created_at = EXCLUDED.source_created_at,
  source_updated_at = EXCLUDED.source_updated_at,
  source_revision_count = EXCLUDED.source_revision_count,
  imported_rating = EXCLUDED.imported_rating,
  created_by_name = EXCLUDED.created_by_name,
  updated_by_name = EXCLUDED.updated_by_name,
  title_shown = EXCLUDED.title_shown,
  parent_fullname = EXCLUDED.parent_fullname,
  comments = EXCLUDED.comments,
  commented_at = EXCLUDED.commented_at,
  commented_by_name = EXCLUDED.commented_by_name,
  source_sha256 = EXCLUDED.source_sha256,
  meta_sha256 = EXCLUDED.meta_sha256,
  meta_json = EXCLUDED.meta_json,
  last_import_run_id = EXCLUDED.last_import_run_id,
  imported_at = NOW();
`;
}

async function existingSnapshotPageStatus(args, sqlExecutor, row) {
  const sql = `
WITH matching_snapshot AS (
  SELECT snapshot.page_id
  FROM wikidot_page_snapshot snapshot
  JOIN page p ON p.page_id = snapshot.page_id
  WHERE snapshot.source_branch = ${sqlQuote(row.source_branch)}
    AND snapshot.source_site = ${sqlQuote(row.source_site)}
    AND snapshot.source_entity_id = ${sqlQuote(row.source_entity_id)}
    AND snapshot.source_fullname = ${sqlQuote(row.fullname)}
    AND encode(snapshot.source_sha256, 'hex') = ${sqlQuote(row.source_sha256)}
    AND encode(snapshot.meta_sha256, 'hex') = ${sqlQuote(row.meta_sha256)}
    AND p.site_id = ${sqlInt(args.siteId)}
    AND p.deleted_at IS NULL
  LIMIT 1
), active_page AS (
  SELECT p.page_id, p.page_category_id
  FROM page p
  JOIN matching_snapshot snapshot ON snapshot.page_id = p.page_id
  LIMIT 1
), latest_revision AS (
  SELECT
    pr.revision_id,
    pr.compiled_generator,
    body.contents AS compiled_body
  FROM page_revision pr
  JOIN active_page p ON p.page_id = pr.page_id
  LEFT JOIN text body ON body.hash = pr.compiled_body_html_hash
  ORDER BY pr.revision_number DESC, pr.revision_id DESC
  LIMIT 1
)
SELECT
  p.page_id || '|' ||
  p.page_category_id || '|' ||
  COALESCE((SELECT revision_id::text FROM latest_revision), '') || '|' ||
  CASE
    WHEN (SELECT revision_id FROM latest_revision) IS NOT NULL
      AND COALESCE((SELECT compiled_generator FROM latest_revision), '') <> ${sqlQuote(SHELL_COMPILED_GENERATOR)}
      AND POSITION(${sqlQuote(SHELL_IMPORT_MARKER)} IN COALESCE((SELECT compiled_body FROM latest_revision), '')) = 0
      AND POSITION(${sqlQuote(SHELL_IMPORT_MESSAGE)} IN COALESCE((SELECT compiled_body FROM latest_revision), '')) = 0
    THEN 'true'
    ELSE 'false'
  END
FROM active_page p;
`;
  const output = await sqlExecutor.runSql(sql, { capture: true });
  if (!output) return null;
  const [pageIdText, categoryIdText, revisionIdText = '', renderCompleteText = ''] = output.split('|');
  const pageId = Number.parseInt(pageIdText, 10);
  const categoryId = Number.parseInt(categoryIdText, 10);
  const revisionId = revisionIdText === '' ? null : Number.parseInt(revisionIdText, 10);
  const renderComplete = renderCompleteText === 'true';
  if (!Number.isInteger(pageId) || !Number.isInteger(categoryId) || (revisionId !== null && !Number.isInteger(revisionId)) || !['true', 'false'].includes(renderCompleteText)) {
    throw new Error(`invalid matching snapshot page status output: ${output}`);
  }
  return { page_id: pageId, page_category_id: categoryId, revision_id: revisionId, render_complete: renderComplete };
}

async function existingActivePage(args, sqlExecutor, slug) {
  const sql = `
WITH target_page AS (
  SELECT
    p.page_id,
    p.page_category_id,
    p.latest_revision_id,
    latest_revision.revision_id AS latest_revision_id_by_history
  FROM page p
  LEFT JOIN LATERAL (
    SELECT revision_id
    FROM page_revision pr
    WHERE pr.page_id = p.page_id
    ORDER BY pr.revision_number DESC, pr.revision_id DESC
    LIMIT 1
  ) latest_revision ON true
  WHERE p.site_id = ${sqlInt(args.siteId)}
    AND p.slug = ${sqlQuote(slug)}
    AND p.deleted_at IS NULL
  ORDER BY p.page_id
  LIMIT 1
  FOR UPDATE OF p
), repaired_page AS (
  UPDATE page
  SET latest_revision_id = target_page.latest_revision_id_by_history
  FROM target_page
  WHERE page.page_id = target_page.page_id
    AND ${args.dryRun ? 'FALSE' : 'TRUE'}
    AND page.latest_revision_id IS NULL
    AND target_page.latest_revision_id IS NULL
    AND target_page.latest_revision_id_by_history IS NOT NULL
  RETURNING page.page_id, page.page_category_id, page.latest_revision_id
)
SELECT page_id || '|' || page_category_id || '|' || COALESCE(latest_revision_id::text, '')
FROM repaired_page
UNION ALL
SELECT
  page_id || '|' || page_category_id || '|' || COALESCE(COALESCE(latest_revision_id, latest_revision_id_by_history)::text, '')
FROM target_page
WHERE NOT EXISTS (SELECT 1 FROM repaired_page);
`;
  const output = await sqlExecutor.runSql(sql, { capture: true });
  if (!output) return null;
  const [pageIdText, categoryIdText, revisionIdText = ''] = output.split('|');
  const pageId = Number.parseInt(pageIdText, 10);
  const categoryId = Number.parseInt(categoryIdText, 10);
  const revisionId = revisionIdText === '' ? null : Number.parseInt(revisionIdText, 10);
  if (!Number.isInteger(pageId) || !Number.isInteger(categoryId) || (revisionId !== null && !Number.isInteger(revisionId))) {
    throw new Error(`invalid existing page output: ${output}`);
  }
  return { page_id: pageId, page_category_id: categoryId, revision_id: revisionId };
}

async function pageSnapshotStatus(args, sqlExecutor, row, pageId) {
  const sql = `
SELECT encode(source_sha256, 'hex') || '|' || encode(meta_sha256, 'hex')
FROM wikidot_page_snapshot
WHERE page_id = ${sqlInt(pageId)}
LIMIT 1;
`;
  const output = await sqlExecutor.runSql(sql, { capture: true });
  if (!output) return 'absent';
  const [sourceSha, metaSha] = output.split('|');
  return sourceSha === row.source_sha256 && metaSha === row.meta_sha256 ? 'matching' : 'mismatched';
}

async function existingPageSourceStatus(args, sqlExecutor, row, revisionId) {
  if (revisionId === null) return 'missing_revision';
  const sql = `
SELECT CASE
  WHEN text.contents = ${sqlTextFromBase64(sourceText(row))} THEN 'matching'
  ELSE 'mismatched'
END
FROM page_revision revision
LEFT JOIN text ON text.hash = revision.wikitext_hash
WHERE revision.revision_id = ${sqlInt(revisionId)}
LIMIT 1;
`;
  const output = await sqlExecutor.runSql(sql, { capture: true });
  return output === 'matching' ? 'matching' : 'mismatched';
}

function canCombineSnapshotReadyRecord(args) {
  return args.skipRerender && (args.skipAttachments || args.attachmentCreateMode === 'direct');
}

function canBatchCreateDbShellPages(args) {
  return args.createMode === 'db'
    && args.assumeEmptyDbImport
    && args.skipRerender
    && !args.skipExistingDone
    && !args.adoptExisting
    && !args.replaceExisting
    && !args.rerenderAfterDbCreate
    && !args.attachmentsOnlyExisting
    && !!args.textHashBatchCommand
    && (args.skipAttachments || args.attachmentCreateMode === 'direct');
}

function recordImportResult(results, summary, result) {
  results.push(result);
  summary[result.action] = (summary[result.action] ?? 0) + 1;
  summary.attachments_requested += result.attachments_requested ?? 0;
  summary.attachments_uploaded += result.attachments_uploaded ?? 0;
  summary.attachments_skipped_existing += result.attachments_skipped_existing ?? 0;
  summary.attachments_deferred += result.attachments_deferred ?? 0;
  if (result.action === 'render_failed') summary.failed += 1;
  console.log(JSON.stringify(result));
}

function batchShellCreatePageValues(args, rows) {
  const bodyHash = shellBodyHashHex(args);
  return rows.map((row, index) => {
    const categoryId = precreatedCategoryIds.get(categoryName(row.fullname));
    if (categoryId === undefined) {
      throw new Error(`missing precreated category id for ${row.fullname}`);
    }
    const sourceTextPrecreated = precreatedSourceTextHashes.has(row.fullname);
    if (!sourceTextPrecreated) {
      throw new Error(`missing precreated source text for ${row.fullname}`);
    }
    const wikitextHash = textHashHex(args, '', row.fullname);
    const title = fallbackTitle(row);
    const metaText = metaJsonText(row);
    return `(
      ${sqlInt(index)},
      ${sqlQuote(row.source_entity_id)}::uuid,
      ${sqlQuote(row.source_branch)},
      ${sqlQuote(row.source_site)},
      ${sqlQuote(row.fullname)},
      ${sqlInt(categoryId)},
      ${sqlTimestamp(row.created_at)},
      ${sqlTimestamp(row.updated_at)},
      ${sqlInt(row.revisions)},
      ${sqlInt(row.rating)},
      ${sqlQuote(row.created_by)},
      ${sqlQuote(row.updated_by)},
      ${sqlQuote(row.title_shown)},
      ${sqlQuote(row.parent_fullname)},
      ${sqlInt(row.comments)},
      ${sqlTimestamp(row.commented_at)},
      ${sqlQuote(row.commented_by)},
      ${sqlByteaFromHex(row.source_sha256)},
      ${sqlByteaFromHex(row.meta_sha256)},
      ${sqlQuote(metaText)}::jsonb,
      ${sqlTextHash(wikitextHash)},
      ${sqlTextHash(bodyHash)},
      ${sqlQuote(title)},
      ${sqlTextArray(row.tags)}
    )`;
  }).join(',\n');
}

async function batchShellCreatePages(args, sqlExecutor, rows, importRunId) {
  if (rows.length === 0) return [];
  const duplicateFullnames = rows
    .map((row) => row.fullname)
    .filter((fullname, index, fullnames) => fullnames.indexOf(fullname) !== index);
  if (duplicateFullnames.length > 0) {
    throw new Error(`batched DB shell import requires unique fullnames; duplicate ${duplicateFullnames[0]}`);
  }
  const values = batchShellCreatePageValues(args, rows);
  const sql = `
CREATE TEMP TABLE corpus_shell_batch_result (
  row_index INTEGER NOT NULL,
  page_id BIGINT NOT NULL,
  revision_id BIGINT NOT NULL
) ON COMMIT DROP;

WITH input_rows (
  row_index,
  source_entity_id,
  source_branch,
  source_site,
  fullname,
  page_category_id,
  created_at,
  updated_at,
  source_revision_count,
  imported_rating,
  created_by_name,
  updated_by_name,
  title_shown,
  parent_fullname,
  comments,
  commented_at,
  commented_by_name,
  source_sha256,
  meta_sha256,
  meta_json,
  wikitext_hash,
  body_hash,
  title,
  tags
) AS (
  VALUES
${values}
), inserted_pages AS (
  INSERT INTO page (created_at, updated_at, from_wikidot, site_id, page_category_id, slug)
  SELECT created_at, updated_at, true, ${sqlInt(args.siteId)}, page_category_id, fullname
  FROM input_rows
  ORDER BY row_index
  RETURNING page_id, slug
), inserted_revisions AS (
  INSERT INTO page_revision (
    revision_type,
    created_at,
    revision_number,
    page_id,
    site_id,
    user_id,
    from_wikidot,
    changes,
    wikitext_hash,
    compiled_body_html_hash,
    compiled_top_bar_html_hash,
    compiled_side_bar_html_hash,
    compiled_at,
    compiled_generator,
    comments,
    hidden,
    title,
    alt_title,
    slug,
    tags
  )
  SELECT
    'create',
    input_rows.updated_at,
    0,
    inserted_pages.page_id,
    ${sqlInt(args.siteId)},
    ${sqlInt(args.userId)},
    true,
    ARRAY['wikitext', 'title', 'alt_title', 'slug', 'tags']::text[],
    input_rows.wikitext_hash,
    input_rows.body_hash,
    NULL,
    NULL,
    NOW(),
    ${sqlQuote(SHELL_COMPILED_GENERATOR)},
    'local scp-wiki mirror DB import from scp-wiki-translation corpus',
    ARRAY[]::text[],
    input_rows.title,
    NULL,
    input_rows.fullname,
    input_rows.tags
  FROM input_rows
  JOIN inserted_pages ON inserted_pages.slug = input_rows.fullname
  RETURNING revision_id, page_id, slug
), inserted_snapshots AS (
  INSERT INTO wikidot_page_snapshot (
    page_id,
    source_branch,
    source_site,
    source_entity_id,
    source_fullname,
    source_created_at,
    source_updated_at,
    source_revision_count,
    imported_rating,
    created_by_name,
    updated_by_name,
    title_shown,
    parent_fullname,
    comments,
    commented_at,
    commented_by_name,
    source_sha256,
    meta_sha256,
    meta_json,
    last_import_run_id
  )
  SELECT
    inserted_revisions.page_id,
    input_rows.source_branch,
    input_rows.source_site,
    input_rows.source_entity_id,
    input_rows.fullname,
    input_rows.created_at,
    input_rows.updated_at,
    input_rows.source_revision_count,
    input_rows.imported_rating,
    input_rows.created_by_name,
    input_rows.updated_by_name,
    input_rows.title_shown,
    input_rows.parent_fullname,
    input_rows.comments,
    input_rows.commented_at,
    input_rows.commented_by_name,
    input_rows.source_sha256,
    input_rows.meta_sha256,
    input_rows.meta_json,
    ${sqlInt(importRunId)}
  FROM input_rows
  JOIN inserted_revisions ON inserted_revisions.slug = input_rows.fullname
  RETURNING page_id
), inserted_items AS (
  INSERT INTO wikidot_corpus_import_item (
    import_run_id,
    source_entity_id,
    source_fullname,
    page_id,
    source_sha256,
    meta_sha256,
    state
  )
  SELECT
    ${sqlInt(importRunId)},
    input_rows.source_entity_id,
    input_rows.fullname,
    inserted_revisions.page_id,
    input_rows.source_sha256,
    input_rows.meta_sha256,
    'render_pending'
  FROM input_rows
  JOIN inserted_revisions ON inserted_revisions.slug = input_rows.fullname
  RETURNING page_id
)
INSERT INTO corpus_shell_batch_result (row_index, page_id, revision_id)
SELECT
  input_rows.row_index,
  inserted_revisions.page_id,
  inserted_revisions.revision_id
FROM input_rows
JOIN inserted_revisions ON inserted_revisions.slug = input_rows.fullname
JOIN inserted_snapshots ON inserted_snapshots.page_id = inserted_revisions.page_id
JOIN inserted_items ON inserted_items.page_id = inserted_revisions.page_id;

UPDATE page
SET
  latest_revision_id = corpus_shell_batch_result.revision_id,
  from_wikidot = true
FROM corpus_shell_batch_result
WHERE page.page_id = corpus_shell_batch_result.page_id;

SELECT
  row_index::text || '|' ||
  page_id::text || '|' ||
  revision_id::text
FROM corpus_shell_batch_result
ORDER BY row_index;
`;
  const output = await sqlExecutor.runSql(sql, { capture: true });
  const parsed = new Map();
  if (output.trim()) {
    for (const line of output.split('\n')) {
      const [indexText, pageIdText, revisionIdText, extra] = line.split('|');
      const index = Number.parseInt(indexText, 10);
      const pageId = Number.parseInt(pageIdText, 10);
      const revisionId = Number.parseInt(revisionIdText, 10);
      if (extra !== undefined || !Number.isInteger(index) || !Number.isInteger(pageId) || !Number.isInteger(revisionId)) {
        throw new Error(`invalid batched DB shell import output: ${line}`);
      }
      parsed.set(index, { page_id: pageId, revision_id: revisionId });
    }
  }
  if (parsed.size !== rows.length) {
    throw new Error(`batched DB shell import returned ${parsed.size} rows for ${rows.length} inputs`);
  }
  const results = [];
  for (let index = 0; index < rows.length; index += 1) {
    const row = rows[index];
    const parsedRow = parsed.get(index);
    const attachmentSummary = await materializeRowAttachments(args, row, parsedRow.page_id);
    results.push({
      slug: row.fullname,
      action: 'created_db_snapshot_ready',
      page_id: parsedRow.page_id,
      revision_id: parsedRow.revision_id,
      rating: row.rating,
      tags: row.tags.length,
      ...attachmentSummary,
    });
  }
  return results;
}

async function importRow(args, sqlExecutor, row, importRunId) {
  if (!shouldProcessParentLinks(args)) {
    const existing = await getPage(args, row.fullname);
    if (existing === null) {
      if (!args.dryRun) {
        await sqlExecutor.runSql(recordItemSql(row, null, importRunId, 'failed', { collision: 'missing_existing_page_for_attachments' }));
      }
      return { slug: row.fullname, action: 'missing_existing_page_for_attachments' };
    }
    if (args.dryRun) {
      return {
        slug: row.fullname,
        action: 'would_materialize_existing_attachments',
        page_id: existing.page_id,
        attachments_requested: Array.isArray(row.attachments) ? row.attachments.length : 0,
        attachments_uploaded: 0,
        attachments_skipped_existing: 0,
      };
    }
    const attachmentSummary = await materializeRowAttachments(args, row, existing.page_id);
    await sqlExecutor.runSql(recordItemSql(row, existing.page_id, importRunId, 'done'));
    return {
      slug: row.fullname,
      action: 'materialized_existing_attachments',
      page_id: existing.page_id,
      ...attachmentSummary,
    };
  }

  if (args.skipExistingDone) {
    const snapshotStatus = await existingSnapshotPageStatus(args, sqlExecutor, row);
    if (snapshotStatus !== null && snapshotStatus.render_complete) {
      const attachmentSummary = await materializeRowAttachments(args, row, snapshotStatus.page_id);
      if (!args.dryRun) await sqlExecutor.runSql(recordItemSql(row, snapshotStatus.page_id, importRunId, 'done'));
      return { slug: row.fullname, action: args.dryRun ? 'would_skip_existing_done' : 'skipped_existing_done', page_id: snapshotStatus.page_id, ...attachmentSummary };
    }
    if (snapshotStatus !== null && args.skipRerender) {
      const attachmentSummary = await materializeRowAttachments(args, row, snapshotStatus.page_id);
      if (!args.dryRun) {
        await sqlExecutor.runSql(recordItemSql(row, snapshotStatus.page_id, importRunId, 'render_pending', { render: 'matching_snapshot_still_shell_or_pending' }));
      }
      return { slug: row.fullname, action: args.dryRun ? 'would_keep_existing_render_pending' : 'kept_existing_render_pending', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id, ...attachmentSummary };
    }
    if (snapshotStatus !== null) {
      if (args.dryRun) {
        return { slug: row.fullname, action: 'would_rerender_existing_pending', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id };
      }
      const attachmentSummary = await materializeRowAttachments(args, row, snapshotStatus.page_id);
      await sqlExecutor.runSql(recordItemSql(row, snapshotStatus.page_id, importRunId, 'render_pending', { render: 'matching_snapshot_rerender_requested' }));
      try {
        await rerenderPage(args, snapshotStatus.page_id, snapshotStatus.page_category_id);
      } catch (error) {
        await sqlExecutor.runSql(recordItemSql(row, snapshotStatus.page_id, importRunId, 'render_failed', { message: error.message }));
        return { slug: row.fullname, action: 'render_failed', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id, error: error.message };
      }
      await sqlExecutor.runSql(recordItemSql(row, snapshotStatus.page_id, importRunId, 'done'));
      return { slug: row.fullname, action: 'rerendered_existing_pending', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id, ...attachmentSummary };
    }
  }

  let pageId;
  let revisionId;
  let categoryId;
  let action;

  if (args.createMode === 'db') {
    const existing = args.assumeEmptyDbImport ? null : await existingActivePage(args, sqlExecutor, row.fullname);
    let replaceExistingRevision = false;
    if (existing !== null) {
      const existingSnapshotStatus = await pageSnapshotStatus(args, sqlExecutor, row, existing.page_id);
      const existingSourceStatus = await existingPageSourceStatus(args, sqlExecutor, row, existing.revision_id);
      const contentMatches = existingSourceStatus === 'matching';
      const needsReplacement = existingSnapshotStatus !== 'matching' || !contentMatches;

      if (needsReplacement && args.replaceExisting) {
        replaceExistingRevision = true;
      } else if (existingSnapshotStatus === 'absent' && !args.adoptExisting) {
        if (!args.dryRun) await sqlExecutor.runSql(recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_requires_adopt_or_replace', source_status: existingSourceStatus }));
        return { slug: row.fullname, action: 'collision_existing_page', page_id: existing.page_id, source_status: existingSourceStatus };
      } else if (existingSnapshotStatus === 'mismatched') {
        if (!args.dryRun) await sqlExecutor.runSql(recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_snapshot_mismatch_requires_replace', source_status: existingSourceStatus }));
        return { slug: row.fullname, action: 'collision_existing_snapshot_mismatch', page_id: existing.page_id, source_status: existingSourceStatus };
      } else if (!contentMatches) {
        if (!args.dryRun) await sqlExecutor.runSql(recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_content_mismatch_requires_replace', snapshot_status: existingSnapshotStatus }));
        return { slug: row.fullname, action: 'collision_existing_content_mismatch', page_id: existing.page_id, snapshot_status: existingSnapshotStatus };
      }
      if (args.dryRun) {
        return { slug: row.fullname, action: replaceExistingRevision ? 'would_replace_existing' : 'would_adopt', page_id: existing.page_id, snapshot_status: existingSnapshotStatus, source_status: existingSourceStatus };
      }
    }
    if (args.dryRun) return { slug: row.fullname, action: 'would_db_create' };
    const created = await shellCreatePage(args, sqlExecutor, row, { replaceExistingRevision });
    pageId = created.page_id;
    revisionId = created.revision_id;
    categoryId = created.page_category_id;
    const snapshotStatus = created.created_page ? 'absent' : await pageSnapshotStatus(args, sqlExecutor, row, pageId);
    if (!created.created_page && snapshotStatus === 'absent' && !args.adoptExisting && !replaceExistingRevision) {
      await sqlExecutor.runSql(recordItemSql(row, pageId, importRunId, 'failed', { collision: 'existing_page_requires_adopt_or_replace' }));
      return { slug: row.fullname, action: 'collision_existing_page', page_id: pageId };
    }
    if (snapshotStatus === 'mismatched' && !replaceExistingRevision) {
      await sqlExecutor.runSql(recordItemSql(row, pageId, importRunId, 'failed', { collision: 'existing_page_snapshot_mismatch_requires_replace' }));
      return { slug: row.fullname, action: 'collision_existing_snapshot_mismatch', page_id: pageId };
    }
    action = created.created_page ? 'created_db' : created.created_revision ? 'replaced_db' : 'adopted';
  } else {
    const existing = await getPage(args, row.fullname);

    if (existing === null) {
      if (args.dryRun) return { slug: row.fullname, action: 'would_create' };
      const created = await createPage(args, row);
      const pageAfterCreate = await getPage(args, row.fullname);
      if (pageAfterCreate === null) throw new Error(`created page not found after page_create: ${row.fullname}`);
      pageId = pageAfterCreate.page_id;
      revisionId = created.revision_id;
      categoryId = pageAfterCreate.page_category_id;
      action = 'created';
    } else {
      if (!args.adoptExisting) {
        if (!args.dryRun) await sqlExecutor.runSql(recordItemSql(row, existing.page_id ?? null, importRunId, 'failed', { collision: 'existing_page_requires_adopt' }));
        return { slug: row.fullname, action: 'collision_existing_page', page_id: existing.page_id };
      }
      if (args.dryRun) return { slug: row.fullname, action: 'would_adopt', page_id: existing.page_id };
      const snapshotStatus = await pageSnapshotStatus(args, sqlExecutor, row, existing.page_id);
      if (snapshotStatus === 'mismatched') {
        await sqlExecutor.runSql(recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_snapshot_mismatch_update_not_implemented' }));
        return { slug: row.fullname, action: 'collision_existing_snapshot_mismatch', page_id: existing.page_id };
      }
      pageId = existing.page_id;
      revisionId = existing.revision_id;
      categoryId = existing.page_category_id;
      action = 'adopted';
    }
  }

  if (args.skipRerender) {
    const snapshotSql = upsertSnapshotSql(args, row, pageId, revisionId, importRunId);
    const renderPendingSql = recordItemSql(row, pageId, importRunId, 'render_pending');
    let attachmentSummary;
    if (canCombineSnapshotReadyRecord(args)) {
      await sqlExecutor.runSql(`${snapshotSql}\n${renderPendingSql}`);
      attachmentSummary = await materializeRowAttachments(args, row, pageId);
    } else {
      await sqlExecutor.runSql(snapshotSql);
      attachmentSummary = await materializeRowAttachments(args, row, pageId);
      await sqlExecutor.runSql(renderPendingSql);
    }
    return { slug: row.fullname, action: `${action}_snapshot_ready`, page_id: pageId, revision_id: revisionId, rating: row.rating, tags: row.tags.length, ...attachmentSummary };
  }

  await sqlExecutor.runSql(upsertSnapshotSql(args, row, pageId, revisionId, importRunId));
  const attachmentSummary = await materializeRowAttachments(args, row, pageId);
  await sqlExecutor.runSql(recordItemSql(row, pageId, importRunId, 'render_pending'));
  try {
    await rerenderPage(args, pageId, categoryId);
  } catch (error) {
    await sqlExecutor.runSql(recordItemSql(row, pageId, importRunId, 'render_failed', { message: error.message }));
    return { slug: row.fullname, action: 'render_failed', page_id: pageId, revision_id: revisionId, error: error.message };
  }
  await sqlExecutor.runSql(recordItemSql(row, pageId, importRunId, 'done'));
  return { slug: row.fullname, action, page_id: pageId, revision_id: revisionId, rating: row.rating, tags: row.tags.length, ...attachmentSummary };
}

async function upsertParentLinks(args, sqlExecutor, selectedRows) {
  // Attachment-only imports do not change page snapshots or page topology. The
  // selected rows can include existing pages whose include graph is outside
  // this run, so parent-link work must not rerender those pages after the file
  // rows have already been committed.
  if (args.attachmentsOnlyExisting) {
    return {
      parent_link_requested: 0,
      parent_link_ready: 0,
      parent_link_inserted: 0,
      parent_link_missing_parent: 0,
      parent_link_missing_child: 0,
    };
  }
  const sql = buildParentLinkSql(args, selectedRows);
  if (sql === null) {
    return {
      parent_link_requested: 0,
      parent_link_ready: 0,
      parent_link_inserted: 0,
      parent_link_missing_parent: 0,
      parent_link_missing_child: 0,
    };
  }
  return parseParentLinkSummary(await sqlExecutor.runSql(sql, { capture: true }));
}

async function rerenderParentLinkPages(args, sqlExecutor, selectedRows) {
  if (args.skipRerender || !shouldProcessParentLinks(args)) return { parent_link_parent_rerendered: 0 };
  const sql = buildParentLinkParentPagesSql(args, selectedRows);
  if (sql === null) return { parent_link_parent_rerendered: 0 };
  const pages = parseParentLinkParentPages(await sqlExecutor.runSql(sql, { capture: true }));
  let rerendered = 0;
  for (const page of pages) {
    await rerenderPage(args, page.page_id, page.page_category_id);
    rerendered += 1;
  }
  return { parent_link_parent_rerendered: rerendered };
}

async function main() {
  const totalStartedAt = process.hrtime.bigint();
  const phaseTimingsMs = {};
  const args = parseArgs(process.argv.slice(2));
  const sqlExecutor = createSqlExecutor({ dbUrl: args.dbUrl, dbContainer: args.dbContainer });
  try {
    const manifestText = timePhaseSync(phaseTimingsMs, 'read_manifest', () => fs.readFileSync(args.manifest, 'utf8'));
    const allRows = timePhaseSync(phaseTimingsMs, 'parse_manifest', () => parseRows(manifestText));
    const selectedRows = timePhaseSync(phaseTimingsMs, 'filter_rows', () => filterRows(args, allRows));
    const completeInventory = selectedRows.length === allRows.length && args.limit === null && args.slug.length === 0 && args.slugFile === null;
    const directAttachmentPlan = timePhaseSync(phaseTimingsMs, 'plan_direct_attachments', () => (
      args.attachmentCreateMode === 'direct' ? planDirectAttachmentMaterialization(selectedRows) : null
    ));
    if (!args.dryRun) validateAttachmentActorArgs(args, selectedRows);

    if (args.applyMigration && !args.dryRun) await timePhase(phaseTimingsMs, 'apply_migration', () => applyMigration(args, sqlExecutor));
    if (args.dryRun) {
      const output = { dry_run: true, selected_rows: selectedRows.length, complete_inventory: completeInventory };
      if (args.attachmentCreateMode === 'direct') {
        output.attachment_direct_plan = directAttachmentPlan.attachment_direct_plan;
      }
      console.log(JSON.stringify(output, null, 2));
      return;
    }

    timePhaseSync(phaseTimingsMs, 'precompute_db_text_hashes', () => precomputeDbTextHashes(args, selectedRows));
    await timePhase(phaseTimingsMs, 'verify_empty_db_import_target', () => assertEmptyDbImportTarget(args, sqlExecutor));
    const importRunId = await timePhase(phaseTimingsMs, 'ensure_import_run', () => ensureImportRun(args, sqlExecutor, manifestText, allRows, selectedRows, completeInventory));
    await timePhase(phaseTimingsMs, 'precreate_db_shell_body_text', () => precreateDbShellBodyText(args, sqlExecutor, selectedRows));
    await timePhase(phaseTimingsMs, 'precreate_db_source_texts', () => precreateDbSourceTexts(args, sqlExecutor, selectedRows));
    await timePhase(phaseTimingsMs, 'precreate_db_shell_categories', () => precreateDbShellCategories(args, sqlExecutor, selectedRows));
    const results = [];
    const summary = { created: 0, created_db_snapshot_ready: 0, adopted: 0, created_snapshot_ready: 0, adopted_snapshot_ready: 0, skipped_existing_done: 0, collision_existing_page: 0, collision_existing_snapshot_mismatch: 0, failed: 0, attachments_requested: 0, attachments_uploaded: 0, attachments_skipped_existing: 0, attachments_deferred: 0, import_run_id: importRunId, phase_timings_ms: phaseTimingsMs };
    let finalState = 'failed';
    let directAttachmentUpload = null;

    try {
      if (directAttachmentPlan !== null) {
        summary.attachment_direct_plan = directAttachmentPlan.attachment_direct_plan;
        directAttachmentUpload = await timePhase(phaseTimingsMs, 'upload_direct_attachment_blobs', () => uploadDirectCorpusAttachmentBlobs(args, directAttachmentPlan));
        summary.attachment_direct_upload = summarizeCorpusAttachmentUpload(directAttachmentUpload);
      }
      if (canBatchCreateDbShellPages(args)) {
        let shellBatchCount = 0;
        await timePhase(phaseTimingsMs, 'import_rows_batched', async () => {
          for (let index = 0; index < selectedRows.length; index += DB_SHELL_BATCH_MAX_ROWS) {
            const batch = selectedRows.slice(index, index + DB_SHELL_BATCH_MAX_ROWS);
            shellBatchCount += 1;
            try {
              for (const result of await batchShellCreatePages(args, sqlExecutor, batch, importRunId)) {
                recordImportResult(results, summary, result);
              }
            } catch (error) {
              summary.failed += batch.length;
              for (const row of batch) {
                await sqlExecutor.runSql(recordItemSql(row, null, importRunId, 'failed', { message: error.message }));
                console.error(JSON.stringify({ slug: row.fullname, action: 'failed', error: error.message }));
              }
            }
          }
        });
        summary.db_shell_batch_size = DB_SHELL_BATCH_MAX_ROWS;
        summary.db_shell_batches = shellBatchCount;
      } else {
        await timePhase(phaseTimingsMs, 'import_rows_serial', async () => {
          for (const row of selectedRows) {
            try {
              const result = await importRow(args, sqlExecutor, row, importRunId);
              recordImportResult(results, summary, result);
            } catch (error) {
              summary.failed += 1;
              await sqlExecutor.runSql(recordItemSql(row, null, importRunId, 'failed', { message: error.message }));
              console.error(JSON.stringify({ slug: row.fullname, action: 'failed', error: error.message }));
            }
          }
        });
      }
      if (directAttachmentPlan !== null) {
        let attachmentStaging = { summary: { total: 0, insert: 0, skip_existing: 0, fail_closed: 0 }, rows: [] };
        if (directAttachmentUpload.failed === 0) {
          attachmentStaging = await timePhase(phaseTimingsMs, 'commit_direct_attachment_staging', () => commitDirectCorpusAttachmentStaging(args, sqlExecutor, directAttachmentPlan));
          summary.attachments_uploaded += attachmentStaging.summary.insert;
          summary.attachments_skipped_existing += attachmentStaging.summary.skip_existing;
        }
        summary.attachment_direct_staging = attachmentStaging.summary;
        console.log(JSON.stringify({
          action: 'direct_attachment_materialization',
          attachment_direct_upload: summary.attachment_direct_upload,
          attachment_direct_staging: summary.attachment_direct_staging,
        }));
        if (directAttachmentUpload.failed > 0 || attachmentStaging.summary.fail_closed > 0) summary.failed += 1;
      }
      Object.assign(summary, await timePhase(phaseTimingsMs, 'upsert_parent_links', () => upsertParentLinks(args, sqlExecutor, selectedRows)));
      Object.assign(summary, await timePhase(phaseTimingsMs, 'rerender_parent_link_pages', () => rerenderParentLinkPages(args, sqlExecutor, selectedRows)));
      finalState = summary.failed > 0 ? 'failed' : 'done';
    } finally {
      recordPhaseTiming(phaseTimingsMs, 'total_before_finish_run', totalStartedAt);
      await finishRun(args, sqlExecutor, importRunId, summary, finalState);
    }
    console.log(JSON.stringify({ summary }, null, 2));
    if (finalState === 'failed') process.exitCode = 1;
  } finally {
    await sqlExecutor.close();
  }
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
