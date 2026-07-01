#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

import { canReuseExistingPageForDbImport } from '../src/corpus-import-apply-policy.mjs';
import {
  buildParentLinkParentPagesSql,
  buildParentLinkSql,
  parseParentLinkParentPages,
  parseParentLinkSummary,
} from '../src/corpus-import-parent-links.mjs';

const DEFAULT_API_URL = 'http://localhost:2747/jsonrpc';
const DEFAULT_DB_CONTAINER = 'local-database-1';
const DEFAULT_SITE_ID = 6000005;
const DEFAULT_USER_ID = -1;
const DEFAULT_IP_ADDRESS = '127.0.0.1';
const SHELL_COMPILED_GENERATOR = 'corpus db import';
const SHELL_IMPORT_MARKER = 'corpus-shell-import';
const SHELL_IMPORT_MESSAGE = 'Content not rendered yet for local Wikidot corpus snapshot import';
const FATAL_UTF8_DECODER = new TextDecoder('utf-8', { fatal: true });

function parseArgs(argv) {
  const args = {
    manifest: null,
    migration: path.resolve('deepwell/migrations/20260625104500_wikidot_corpus_import.sql'),
    applyMigration: false,
    apiUrl: DEFAULT_API_URL,
    dbContainer: DEFAULT_DB_CONTAINER,
    siteId: DEFAULT_SITE_ID,
    userId: DEFAULT_USER_ID,
    ipAddress: DEFAULT_IP_ADDRESS,
    rpcTimeoutMs: 120_000,
    textHashCommand: process.env.DEEPWELL_TEXT_HASH_COMMAND ?? null,
    slug: [],
    slugFile: null,
    limit: null,
    adoptExisting: false,
    skipExistingDone: false,
    skipRerender: false,
    rerenderAfterDbCreate: false,
    replaceExisting: false,
    createMode: 'rpc',
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
    else if (arg === '--db-container') args.dbContainer = next();
    else if (arg === '--site-id') args.siteId = Number.parseInt(next(), 10);
    else if (arg === '--user-id') args.userId = Number.parseInt(next(), 10);
    else if (arg === '--ip-address') args.ipAddress = next();
    else if (arg === '--rpc-timeout-ms') args.rpcTimeoutMs = Number.parseInt(next(), 10);
    else if (arg === '--text-hash-command') args.textHashCommand = next();
    else if (arg === '--slug') args.slug.push(next());
    else if (arg === '--slug-file') args.slugFile = next();
    else if (arg === '--limit') args.limit = Number.parseInt(next(), 10);
    else if (arg === '--adopt-existing') args.adoptExisting = true;
    else if (arg === '--skip-existing-done') args.skipExistingDone = true;
    else if (arg === '--skip-rerender') args.skipRerender = true;
    else if (arg === '--rerender-after-db-create') args.rerenderAfterDbCreate = true;
    else if (arg === '--replace-existing') args.replaceExisting = true;
    else if (arg === '--create-mode') {
      args.createMode = next();
    }
    else if (arg === '--dry-run') args.dryRun = true;
    else if (arg === '--source-site') args.sourceSite = next();
    else if (arg === '--source-branch') args.sourceBranch = next();
    else if (arg === '--help' || arg === '-h') {
      console.log(`Usage: apply-corpus-import-manifest.mjs --manifest <manifest.jsonl> [--apply-migration] [--slug <slug>...] [--adopt-existing] [--replace-existing] [--skip-existing-done] [--skip-rerender] [--create-mode rpc|db] [--rerender-after-db-create] [--dry-run]

Imports current corpus snapshot pages into a local Wikijump mirror. This is an operator-only local tool: it uses Deepwell JSON-RPC for page create/rerender and direct Postgres SQL for corpus snapshot metadata, timestamps, and tags.`);
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (!args.manifest) throw new Error('--manifest is required');
  if (!Number.isInteger(args.siteId)) throw new Error('--site-id must be an integer');
  if (!Number.isInteger(args.userId)) throw new Error('--user-id must be an integer');
  if (!Number.isInteger(args.rpcTimeoutMs) || args.rpcTimeoutMs <= 0) throw new Error('--rpc-timeout-ms must be a positive integer');
  if (!['rpc', 'db'].includes(args.createMode)) throw new Error('--create-mode must be rpc or db');
  if (args.rerenderAfterDbCreate && args.createMode !== 'db') {
    throw new Error('--rerender-after-db-create requires --create-mode db');
  }
  if (args.rerenderAfterDbCreate && args.skipRerender) {
    throw new Error('--rerender-after-db-create cannot be combined with --skip-rerender');
  }
  if (args.createMode === 'db' && !args.rerenderAfterDbCreate) args.skipRerender = true;
  if (args.createMode === 'db' && !args.dryRun && !args.textHashCommand) {
    throw new Error('--create-mode db requires --text-hash-command or DEEPWELL_TEXT_HASH_COMMAND');
  }
  if (args.replaceExisting && args.createMode !== 'db') {
    throw new Error('--replace-existing requires --create-mode db');
  }
  if (args.limit !== null && (!Number.isInteger(args.limit) || args.limit < 0)) {
    throw new Error('--limit must be a non-negative integer');
  }
  return args;
}

function sqlQuote(value) {
  if (value === null || value === undefined) return 'NULL';
  return `'${String(value).replaceAll("'", "''")}'`;
}

function sqlTimestamp(value) {
  if (value === null || value === undefined || value === '') return 'NULL';
  return `TIMESTAMPTZ ${sqlQuote(value)}`;
}

function sqlInt(value) {
  if (!Number.isInteger(value)) throw new Error(`expected integer, got ${value}`);
  return String(value);
}

function sqlByteaFromHex(hex) {
  if (!/^[0-9a-f]{64}$/iu.test(hex)) throw new Error(`expected sha256 hex, got ${hex}`);
  return `decode(${sqlQuote(hex.toLowerCase())}, 'hex')`;
}

function sqlTextHash(hex) {
  if (!/^[0-9a-f]{32}$/iu.test(hex)) throw new Error(`expected 16-byte text hash hex, got ${hex}`);
  return `decode(${sqlQuote(hex.toLowerCase())}, 'hex')`;
}

function textHashHex(args, contents) {
  const result = spawnSync(args.textHashCommand, {
    input: contents,
    shell: true,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });
  if (result.status !== 0) {
    throw new Error(`text hash command failed (${result.status})\nSTDOUT:\n${result.stdout}\nSTDERR:\n${result.stderr}`);
  }
  const hash = result.stdout.trim();
  if (!/^[0-9a-f]{32}$/iu.test(hash)) {
    throw new Error(`text hash command returned invalid 16-byte hex hash: ${hash}`);
  }
  return hash.toLowerCase();
}

function sqlTextFromBase64(value) {
  return `convert_from(decode(${sqlQuote(Buffer.from(value, 'utf8').toString('base64'))}, 'base64'), 'UTF8')`;
}

function sqlTextArray(values) {
  return `ARRAY[${values.map((value) => sqlQuote(value)).join(',')}]::text[]`;
}

function runPsql(args, sql, { capture = false } = {}) {
  const result = spawnSync(
    'docker',
    ['exec', '-i', '-e', 'PGPASSWORD=wikijump', args.dbContainer, 'psql', '-h', 'localhost', '-U', 'wikijump', '-d', 'wikijump', '-1', '-v', 'ON_ERROR_STOP=1', '-q', '-t', '-A'],
    { input: sql, encoding: 'utf8' },
  );
  if (result.status !== 0) {
    const sqlPreview = sql.length > 2000 ? `${sql.slice(0, 2000)}\n... <truncated ${sql.length - 2000} bytes>` : sql;
    throw new Error(`psql failed (${result.status})\nSTDOUT:\n${result.stdout}\nSTDERR:\n${result.stderr}\nSQL preview:\n${sqlPreview}`);
  }
  return capture ? result.stdout.trim() : null;
}

function applyMigration(args) {
  const migrationSql = fs.readFileSync(args.migration, 'utf8');
  runPsql(args, migrationSql);
}

let rpcSequence = 0;
async function rpc(args, method, params) {
  rpcSequence += 1;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), args.rpcTimeoutMs);
  let response;
  try {
    response = await fetch(args.apiUrl, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
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
  return readManifestFile(row, 'source_path', 'source_sha256');
}

function metaJsonText(row) {
  return readManifestFile(row, 'meta_path', 'meta_sha256');
}

function ensureImportRun(args, manifestText, manifestRows, selectedRows, completeInventory) {
  const manifestSha = crypto.createHash('sha256').update(manifestText).digest('hex');
  const sourceSites = new Set(manifestRows.map((row) => row.source_site));
  const sourceBranches = new Set(manifestRows.map((row) => row.source_branch));
  if (sourceSites.size > 1 || sourceBranches.size > 1) {
    throw new Error('manifest must contain a single source_site/source_branch');
  }
  const [sourceSite = args.sourceSite] = sourceSites;
  const [sourceBranch = args.sourceBranch] = sourceBranches;
  const summary = JSON.stringify({ selected_row_count: selectedRows.length, complete_inventory: completeInventory });
  const sql = `
INSERT INTO wikidot_corpus_import_run (
  site_id,
  source_branch,
  source_site,
  manifest_sha256,
  manifest_row_count,
  complete_inventory,
  state,
  summary
) VALUES (
  ${sqlInt(args.siteId)},
  ${sqlQuote(sourceBranch)},
  ${sqlQuote(sourceSite)},
  ${sqlByteaFromHex(manifestSha)},
  ${sqlInt(manifestRows.length)},
  ${completeInventory ? 'true' : 'false'},
  'running',
  ${sqlQuote(summary)}::jsonb
)
RETURNING import_run_id;
`;
  return Number.parseInt(runPsql(args, sql, { capture: true }), 10);
}

async function getPage(args, slug) {
  return await rpc(args, 'page_get', {
    site_id: args.siteId,
    page: slug,
    details: { wikitext: false, compiled: false },
  });
}

async function createPage(args, row) {
  return await rpc(args, 'page_create', {
    site_id: args.siteId,
    wikitext: sourceText(row),
    title: fallbackTitle(row),
    alt_title: null,
    slug: row.fullname,
    layout: 'wikidot',
    revision_comments: 'local scp-wiki mirror import from scp-wiki-translation corpus',
    user_id: args.userId,
    bypass_filter: true,
    ip_address: args.ipAddress,
  });
}

function categoryName(slug) {
  const index = slug.lastIndexOf(':');
  return index === -1 ? '_default' : slug.slice(0, index);
}

function shellCreatePage(args, row, { replaceExistingRevision = false } = {}) {
  const wikitext = sourceText(row);
  const bodyHtml = `<div class="wj-proof-stub ${SHELL_IMPORT_MARKER}">${SHELL_IMPORT_MESSAGE}.</div>`;
  const wikitextHash = textHashHex(args, wikitext);
  const bodyHash = textHashHex(args, bodyHtml);
  const title = fallbackTitle(row);
  const category = categoryName(row.fullname);
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

WITH inserted_wikitext AS (
  INSERT INTO text (hash, contents)
  VALUES (${sqlTextHash(wikitextHash)}, ${sqlTextFromBase64(wikitext)})
  ON CONFLICT (hash) DO NOTHING
  RETURNING 1
), inserted_body AS (
  INSERT INTO text (hash, contents)
  VALUES (${sqlTextHash(bodyHash)}, ${sqlTextFromBase64(bodyHtml)})
  ON CONFLICT (hash) DO NOTHING
  RETURNING 1
), category AS (
  INSERT INTO page_category (site_id, slug)
  VALUES (${sqlInt(args.siteId)}, ${sqlQuote(category)})
  ON CONFLICT (site_id, slug) DO UPDATE SET slug = EXCLUDED.slug
  RETURNING category_id
), target_page AS (
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
  const output = runPsql(args, sql, { capture: true });
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

async function rerenderPage(args, pageId, categoryId) {
  return await rpc(args, 'page_rerender', {
    site_id: args.siteId,
    category_id: categoryId,
    page_id: pageId,
  });
}

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

function existingSnapshotPageStatus(args, row) {
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
  const output = runPsql(args, sql, { capture: true });
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

function existingActivePage(args, slug) {
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
  const output = runPsql(args, sql, { capture: true });
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

function pageSnapshotStatus(args, row, pageId) {
  const sql = `
SELECT encode(source_sha256, 'hex') || '|' || encode(meta_sha256, 'hex')
FROM wikidot_page_snapshot
WHERE page_id = ${sqlInt(pageId)}
LIMIT 1;
`;
  const output = runPsql(args, sql, { capture: true });
  if (!output) return 'absent';
  const [sourceSha, metaSha] = output.split('|');
  return sourceSha === row.source_sha256 && metaSha === row.meta_sha256 ? 'matching' : 'mismatched';
}

function existingPageSourceStatus(args, row, revisionId) {
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
  const output = runPsql(args, sql, { capture: true });
  return output === 'matching' ? 'matching' : 'mismatched';
}

function recordItemSql(row, pageId, importRunId, state, error = null) {
  return `
INSERT INTO wikidot_corpus_import_item (
  import_run_id,
  source_entity_id,
  source_fullname,
  page_id,
  source_sha256,
  meta_sha256,
  state,
  error
) VALUES (
  ${sqlInt(importRunId)},
  ${sqlQuote(row.source_entity_id)},
  ${sqlQuote(row.fullname)},
  ${pageId === null ? 'NULL' : sqlInt(pageId)},
  ${sqlByteaFromHex(row.source_sha256)},
  ${sqlByteaFromHex(row.meta_sha256)},
  ${sqlQuote(state)},
  ${error === null ? 'NULL' : `${sqlQuote(JSON.stringify(error))}::jsonb`}
)
ON CONFLICT (import_run_id, source_entity_id) DO UPDATE SET
  page_id = EXCLUDED.page_id,
  state = EXCLUDED.state,
  error = EXCLUDED.error,
  updated_at = NOW();
`;
}

async function importRow(args, row, importRunId) {
  if (args.skipExistingDone) {
    const snapshotStatus = existingSnapshotPageStatus(args, row);
    if (snapshotStatus !== null && snapshotStatus.render_complete) {
      if (!args.dryRun) runPsql(args, recordItemSql(row, snapshotStatus.page_id, importRunId, 'done'));
      return { slug: row.fullname, action: args.dryRun ? 'would_skip_existing_done' : 'skipped_existing_done', page_id: snapshotStatus.page_id };
    }
    if (snapshotStatus !== null && args.skipRerender) {
      if (!args.dryRun) {
        runPsql(args, recordItemSql(row, snapshotStatus.page_id, importRunId, 'render_pending', { render: 'matching_snapshot_still_shell_or_pending' }));
      }
      return { slug: row.fullname, action: args.dryRun ? 'would_keep_existing_render_pending' : 'kept_existing_render_pending', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id };
    }
    if (snapshotStatus !== null) {
      if (args.dryRun) {
        return { slug: row.fullname, action: 'would_rerender_existing_pending', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id };
      }
      runPsql(args, recordItemSql(row, snapshotStatus.page_id, importRunId, 'render_pending', { render: 'matching_snapshot_rerender_requested' }));
      try {
        await rerenderPage(args, snapshotStatus.page_id, snapshotStatus.page_category_id);
      } catch (error) {
        runPsql(args, recordItemSql(row, snapshotStatus.page_id, importRunId, 'render_failed', { message: error.message }));
        return { slug: row.fullname, action: 'render_failed', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id, error: error.message };
      }
      runPsql(args, recordItemSql(row, snapshotStatus.page_id, importRunId, 'done'));
      return { slug: row.fullname, action: 'rerendered_existing_pending', page_id: snapshotStatus.page_id, revision_id: snapshotStatus.revision_id };
    }
  }

  let pageId;
  let revisionId;
  let categoryId;
  let action;

  if (args.createMode === 'db') {
    const existing = existingActivePage(args, row.fullname);
    let replaceExistingRevision = false;
    if (existing !== null) {
      const existingSnapshotStatus = pageSnapshotStatus(args, row, existing.page_id);
      const existingSourceStatus = existingPageSourceStatus(args, row, existing.revision_id);
      const contentMatches = existingSourceStatus === 'matching';
      const needsReplacement = existingSnapshotStatus !== 'matching' || !contentMatches;

      if (needsReplacement && args.replaceExisting) {
        replaceExistingRevision = true;
      } else if (existingSnapshotStatus === 'absent' && !args.adoptExisting) {
        if (!args.dryRun) runPsql(args, recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_requires_adopt_or_replace', source_status: existingSourceStatus }));
        return { slug: row.fullname, action: 'collision_existing_page', page_id: existing.page_id, source_status: existingSourceStatus };
      } else if (existingSnapshotStatus === 'mismatched') {
        if (!args.dryRun) runPsql(args, recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_snapshot_mismatch_requires_replace', source_status: existingSourceStatus }));
        return { slug: row.fullname, action: 'collision_existing_snapshot_mismatch', page_id: existing.page_id, source_status: existingSourceStatus };
      } else if (!contentMatches) {
        if (!args.dryRun) runPsql(args, recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_content_mismatch_requires_replace', snapshot_status: existingSnapshotStatus }));
        return { slug: row.fullname, action: 'collision_existing_content_mismatch', page_id: existing.page_id, snapshot_status: existingSnapshotStatus };
      }
      if (args.dryRun) {
        return { slug: row.fullname, action: replaceExistingRevision ? 'would_replace_existing' : 'would_adopt', page_id: existing.page_id, snapshot_status: existingSnapshotStatus, source_status: existingSourceStatus };
      }
    }
    if (args.dryRun) return { slug: row.fullname, action: 'would_db_create' };
    const created = shellCreatePage(args, row, { replaceExistingRevision });
    pageId = created.page_id;
    revisionId = created.revision_id;
    categoryId = created.page_category_id;
    const snapshotStatus = pageSnapshotStatus(args, row, pageId);
    if (!created.created_page && snapshotStatus === 'absent' && !args.adoptExisting && !replaceExistingRevision) {
      runPsql(args, recordItemSql(row, pageId, importRunId, 'failed', { collision: 'existing_page_requires_adopt_or_replace' }));
      return { slug: row.fullname, action: 'collision_existing_page', page_id: pageId };
    }
    if (snapshotStatus === 'mismatched' && !replaceExistingRevision) {
      runPsql(args, recordItemSql(row, pageId, importRunId, 'failed', { collision: 'existing_page_snapshot_mismatch_requires_replace' }));
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
        if (!args.dryRun) runPsql(args, recordItemSql(row, existing.page_id ?? null, importRunId, 'failed', { collision: 'existing_page_requires_adopt' }));
        return { slug: row.fullname, action: 'collision_existing_page', page_id: existing.page_id };
      }
      if (args.dryRun) return { slug: row.fullname, action: 'would_adopt', page_id: existing.page_id };
      const snapshotStatus = pageSnapshotStatus(args, row, existing.page_id);
      if (snapshotStatus === 'mismatched') {
        runPsql(args, recordItemSql(row, existing.page_id, importRunId, 'failed', { collision: 'existing_page_snapshot_mismatch_update_not_implemented' }));
        return { slug: row.fullname, action: 'collision_existing_snapshot_mismatch', page_id: existing.page_id };
      }
      pageId = existing.page_id;
      revisionId = existing.revision_id;
      categoryId = existing.page_category_id;
      action = 'adopted';
    }
  }

  runPsql(args, upsertSnapshotSql(args, row, pageId, revisionId, importRunId));
  if (args.skipRerender) {
    runPsql(args, recordItemSql(row, pageId, importRunId, 'render_pending'));
    return { slug: row.fullname, action: `${action}_snapshot_ready`, page_id: pageId, revision_id: revisionId, rating: row.rating, tags: row.tags.length };
  }

  runPsql(args, recordItemSql(row, pageId, importRunId, 'render_pending'));
  try {
    await rerenderPage(args, pageId, categoryId);
  } catch (error) {
    runPsql(args, recordItemSql(row, pageId, importRunId, 'render_failed', { message: error.message }));
    return { slug: row.fullname, action: 'render_failed', page_id: pageId, revision_id: revisionId, error: error.message };
  }
  runPsql(args, recordItemSql(row, pageId, importRunId, 'done'));
  return { slug: row.fullname, action, page_id: pageId, revision_id: revisionId, rating: row.rating, tags: row.tags.length };
}

function finishRun(args, importRunId, summary, state = 'done') {
  runPsql(args, `
UPDATE wikidot_corpus_import_run
SET state = ${sqlQuote(state)}, finished_at = NOW(), summary = ${sqlQuote(JSON.stringify(summary))}::jsonb
WHERE import_run_id = ${sqlInt(importRunId)};
`);
}

function upsertParentLinks(args, selectedRows) {
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
  return parseParentLinkSummary(runPsql(args, sql, { capture: true }));
}

async function rerenderParentLinkPages(args, selectedRows) {
  if (args.skipRerender) return { parent_link_parent_rerendered: 0 };
  const sql = buildParentLinkParentPagesSql(args, selectedRows);
  if (sql === null) return { parent_link_parent_rerendered: 0 };
  const pages = parseParentLinkParentPages(runPsql(args, sql, { capture: true }));
  let rerendered = 0;
  for (const page of pages) {
    await rerenderPage(args, page.page_id, page.page_category_id);
    rerendered += 1;
  }
  return { parent_link_parent_rerendered: rerendered };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const manifestText = fs.readFileSync(args.manifest, 'utf8');
  const allRows = parseRows(manifestText);
  const selectedRows = filterRows(args, allRows);
  const completeInventory = selectedRows.length === allRows.length && args.limit === null && args.slug.length === 0 && args.slugFile === null;

  if (args.applyMigration && !args.dryRun) applyMigration(args);
  if (args.dryRun) {
    console.log(JSON.stringify({ dry_run: true, selected_rows: selectedRows.length, complete_inventory: completeInventory }, null, 2));
    return;
  }

  const importRunId = ensureImportRun(args, manifestText, allRows, selectedRows, completeInventory);
  const results = [];
  const summary = { created: 0, created_db_snapshot_ready: 0, adopted: 0, created_snapshot_ready: 0, adopted_snapshot_ready: 0, skipped_existing_done: 0, collision_existing_page: 0, collision_existing_snapshot_mismatch: 0, failed: 0, import_run_id: importRunId };
  let finalState = 'failed';

  try {
    for (const row of selectedRows) {
      try {
        const result = await importRow(args, row, importRunId);
        results.push(result);
        summary[result.action] = (summary[result.action] ?? 0) + 1;
        if (result.action === 'render_failed') summary.failed += 1;
        console.log(JSON.stringify(result));
      } catch (error) {
        summary.failed += 1;
        runPsql(args, recordItemSql(row, null, importRunId, 'failed', { message: error.message }));
        console.error(JSON.stringify({ slug: row.fullname, action: 'failed', error: error.message }));
      }
    }
    Object.assign(summary, upsertParentLinks(args, selectedRows));
    Object.assign(summary, await rerenderParentLinkPages(args, selectedRows));
    finalState = summary.failed > 0 ? 'failed' : 'done';
  } finally {
    finishRun(args, importRunId, summary, finalState);
  }
  console.log(JSON.stringify({ summary }, null, 2));
  if (finalState === 'failed') process.exitCode = 1;
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
