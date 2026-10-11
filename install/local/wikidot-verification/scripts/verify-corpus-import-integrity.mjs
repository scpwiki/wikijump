#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import {runCliIfMain} from '../src/cli-entry.mjs';
import {createSqlExecutor} from '../src/corpus-import-sql.mjs';
import {sqlInt, sqlQuote} from '../src/corpus-import-sql-values.mjs';

const UTF8 = new TextDecoder('utf-8', {fatal: true});

export function parseArgs(argv) {
  const args = {manifest: null, siteId: 6000005, dbContainer: 'local-database-1', dbUrl: process.env.DEEPWELL_VERIFY_DB_URL ?? null, slugs: [], renderedBodySlugs: []};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = () => {
      index += 1;
      if (index >= argv.length) throw new Error(`${arg} requires a value`);
      return argv[index];
    };
    if (arg === '--manifest') args.manifest = next();
    else if (arg === '--site-id') args.siteId = Number.parseInt(next(), 10);
    else if (arg === '--db-container') args.dbContainer = next();
    else if (arg === '--db-url') args.dbUrl = next();
    else if (arg === '--slug') args.slugs.push(next());
    else if (arg === '--require-rendered-body') args.renderedBodySlugs.push(next());
    else if (arg === '--help' || arg === '-h') return {help: true};
    else throw new Error(`unknown argument: ${arg}`);
  }
  if (!args.manifest) throw new Error('--manifest is required');
  if (!Number.isInteger(args.siteId)) throw new Error('--site-id must be an integer');
  return args;
}

export function usage() {
  return 'Usage: verify-corpus-import-integrity.mjs --manifest <manifest.jsonl> [--site-id <id>] [--db-container <name> | --db-url <url>] [--slug <slug>...] [--require-rendered-body <slug>...]';
}

export function readIntegrityManifest(manifestText, selectedSlugs = []) {
  const selected = new Set(selectedSlugs);
  const rows = manifestText.split('\n').filter((line) => line.trim()).map((line) => JSON.parse(line));
  const unique = new Set();
  const available = new Set(rows.map((row) => row.fullname));
  const unknown = [...selected].filter((slug) => !available.has(slug));
  if (unknown.length) throw new Error(`manifest does not contain selected page ${unknown[0]}`);
  const chosen = selected.size ? rows.filter((row) => selected.has(row.fullname)) : rows;
  if (chosen.length === 0) throw new Error('manifest selection contains no pages');
  for (const row of chosen) {
    if (unique.has(row.fullname)) throw new Error(`manifest contains duplicate page ${row.fullname}`);
    unique.add(row.fullname);
    if (typeof row.fullname !== 'string' || !row.fullname || typeof row.source_path !== 'string' || !row.source_path || !/^[0-9a-f]{64}$/iu.test(row.source_sha256 ?? '')) {
      throw new Error(`manifest row has invalid source identity for ${row.fullname ?? '<unknown>'}`);
    }
    const bytes = fs.readFileSync(row.source_path);
    const actualSha = crypto.createHash('sha256').update(bytes).digest('hex');
    if (actualSha !== row.source_sha256.toLowerCase()) throw new Error(`source_path hash mismatch for ${row.fullname}`);
    if (row.source_bytes !== undefined && row.source_bytes !== null && row.source_bytes !== bytes.length) throw new Error(`source_path size mismatch for ${row.fullname}`);
    try { UTF8.decode(bytes); } catch (error) { throw new Error(`source_path is invalid UTF-8 for ${row.fullname}: ${error.message}`); }
  }
  return chosen;
}

export function buildIntegrityQuery(args, rows) {
  const requested = rows.map((row) => `(${sqlQuote(row.fullname)})`).join(',\n  ');
  return `WITH requested(slug) AS (VALUES\n  ${requested}\n)
SELECT requested.slug,
  COALESCE(page.page_id::text, ''),
  COALESCE(revision.revision_id::text, ''),
  COALESCE(translate(encode(convert_to(source.contents, 'UTF8'), 'base64'), E'\\n', ''), ''),
  COALESCE(encode(snapshot.source_sha256, 'hex'), ''),
  COALESCE(length(body.contents)::text, '-1'),
  COALESCE(latest_item.state, '')
FROM requested
LEFT JOIN page ON page.site_id = ${sqlInt(args.siteId)} AND page.slug = requested.slug AND page.deleted_at IS NULL
LEFT JOIN page_revision revision ON revision.revision_id = page.latest_revision_id
LEFT JOIN text source ON source.hash = revision.wikitext_hash
LEFT JOIN text body ON body.hash = revision.compiled_body_html_hash
LEFT JOIN wikidot_page_snapshot snapshot ON snapshot.page_id = page.page_id
LEFT JOIN LATERAL (
  SELECT item.state
  FROM wikidot_corpus_import_item item
  WHERE item.page_id = page.page_id
  ORDER BY item.import_run_id DESC
  LIMIT 1
) latest_item ON true
ORDER BY requested.slug;`;
}

export function verifyIntegrityRows(rows, output, {renderedBodySlugs = []} = {}) {
  const actualBySlug = new Map();
  for (const line of output.split('\n').filter(Boolean)) {
    const fields = line.split('|');
    if (fields.length !== 7) throw new Error('integrity query returned a malformed row');
    if (actualBySlug.has(fields[0])) throw new Error(`integrity query returned duplicate page ${fields[0]}`);
    actualBySlug.set(fields[0], fields);
  }
  const requireBody = new Set(renderedBodySlugs);
  const pages = [];
  const failures = [];
  for (const row of rows) {
    const result = {slug: row.fullname, current_source_matches_manifest: false, provenance_matches_manifest: false, rendered_body_characters: 0, import_state: null};
    const fields = actualBySlug.get(row.fullname);
    if (!fields || !fields[1]) {
      failures.push({slug: row.fullname, reason: 'missing_page'});
      pages.push(result);
      continue;
    }
    const [, pageId, revisionId, sourceBase64, provenanceSha, bodyLengthText, importState] = fields;
    const sourceText = Buffer.from(sourceBase64, 'base64');
    const expectedSha = row.source_sha256.toLowerCase();
    const parsedPageId = Number(pageId);
    const parsedRevisionId = Number(revisionId);
    const parsedBodyLength = Number(bodyLengthText);
    if (!Number.isSafeInteger(parsedPageId) || parsedPageId <= 0 || !Number.isSafeInteger(parsedRevisionId) || parsedRevisionId <= 0 || !Number.isInteger(parsedBodyLength) || parsedBodyLength < -1) {
      throw new Error(`integrity query returned invalid numeric fields for ${row.fullname}`);
    }
    result.page_id = parsedPageId;
    result.revision_id = parsedRevisionId;
    result.current_source_matches_manifest = crypto.createHash('sha256').update(sourceText).digest('hex') === expectedSha;
    result.provenance_matches_manifest = provenanceSha === expectedSha;
    result.rendered_body_characters = parsedBodyLength;
    result.import_state = importState || null;
    if (!result.current_source_matches_manifest) failures.push({slug: row.fullname, reason: 'current_source_mismatch'});
    if (!result.provenance_matches_manifest) failures.push({slug: row.fullname, reason: 'missing_or_mismatched_provenance'});
    if (result.import_state !== 'done') failures.push({slug: row.fullname, reason: `import_not_done:${result.import_state ?? 'missing'}`});
    if (requireBody.has(row.fullname) && result.rendered_body_characters <= 0) failures.push({slug: row.fullname, reason: 'rendered_body_empty'});
    pages.push(result);
  }
  return {status: failures.length ? 'fail' : 'pass', pages, failures};
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) { console.log(usage()); return 0; }
  const rows = readIntegrityManifest(fs.readFileSync(args.manifest, 'utf8'), args.slugs);
  const selected = new Set(rows.map((row) => row.fullname));
  const invalidRenderedSlug = args.renderedBodySlugs.find((slug) => !selected.has(slug));
  if (invalidRenderedSlug) throw new Error(`rendered-body check page is outside the manifest selection: ${invalidRenderedSlug}`);
  const sqlExecutor = createSqlExecutor({dbUrl: args.dbUrl, dbContainer: args.dbContainer});
  try {
    const output = await sqlExecutor.runSql(buildIntegrityQuery(args, rows), {capture: true});
    const report = verifyIntegrityRows(rows, output, {renderedBodySlugs: args.renderedBodySlugs});
    console.log(JSON.stringify(report, null, 2));
    return report.status === 'pass' ? 0 : 1;
  } finally {
    await sqlExecutor.close();
  }
}

await runCliIfMain(import.meta.url, main);
