#!/usr/bin/env node
// Read-only report of mirror source drift for a page's include closure.
//
// A mirrored page renders through every page it includes (themes,
// components), so a stale include source silently changes the page even when
// the page itself is current. This command walks the include closure of the
// selected root pages in the canonical corpus and compares each page's
// current local revision source with the corpus source.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import {runCliIfMain} from '../src/cli-entry.mjs';
import {createSqlExecutor} from '../src/corpus-import-sql.mjs';
import {sqlInt, sqlQuote} from '../src/corpus-import-sql-values.mjs';
import {
  includeTargetSlug,
  stripInactiveSourceSpans,
  INCLUDE_PATTERN,
} from '../src/corpus-include-closure.mjs';

const INCLUDE = INCLUDE_PATTERN;

export { includeTargetSlug };

export function parseArgs(argv) {
  const args = {
    corpusRoot: null,
    branch: 'en',
    sourceSite: 'scp-wiki',
    siteId: 6000005,
    dbContainer: 'local-database-1',
    dbUrl: process.env.DEEPWELL_VERIFY_DB_URL ?? null,
    roots: [],
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = () => {
      index += 1;
      if (index >= argv.length) throw new Error(`${arg} requires a value`);
      return argv[index];
    };
    if (arg === '--corpus-root') args.corpusRoot = next();
    else if (arg === '--branch') args.branch = next();
    else if (arg === '--source-site') args.sourceSite = next();
    else if (arg === '--site-id') args.siteId = Number.parseInt(next(), 10);
    else if (arg === '--db-container') args.dbContainer = next();
    else if (arg === '--db-url') args.dbUrl = next();
    else if (arg === '--root') args.roots.push(next());
    else if (arg === '--help' || arg === '-h') return {help: true};
    else throw new Error(`unknown argument: ${arg}`);
  }
  if (!args.corpusRoot) throw new Error('--corpus-root is required');
  if (args.roots.length === 0) throw new Error('at least one --root is required');
  if (!Number.isInteger(args.siteId)) throw new Error('--site-id must be an integer');
  return args;
}

export function usage() {
  return 'Usage: report-mirror-include-drift.mjs --corpus-root <path> --root <slug>... [--branch en] [--source-site scp-wiki] [--site-id <id>] [--db-container <name> | --db-url <url>]';
}

/** Breadth-first include closure, roots first, each slug once. */
export function includeClosure(roots, readSource, sourceSite) {
  const order = [];
  const seen = new Set();
  const queue = [...roots];
  while (queue.length) {
    const slug = queue.shift();
    if (seen.has(slug)) continue;
    seen.add(slug);
    const source = readSource(slug);
    order.push({slug, source});
    if (source === null) continue;
    // Raw spans and comments contain display-only documentation. Includes
    // inside code blocks remain active under Wikidot's include contract.
    const activeSource = stripInactiveSourceSpans(source.toString('utf8'));
    INCLUDE.lastIndex = 0;
    for (const match of activeSource.matchAll(INCLUDE)) {
      const target = includeTargetSlug(match[1], sourceSite);
      if (target && !seen.has(target)) queue.push(target);
    }
    INCLUDE.lastIndex = 0;
  }
  return order;
}

export function corpusSourceReader(corpusRoot, branch) {
  return (slug) => {
    const sourcePath = path.join(corpusRoot, branch, 'pages', slug, 'source.wikidot.txt');
    return fs.existsSync(sourcePath) ? fs.readFileSync(sourcePath) : null;
  };
}

export function buildDriftQuery(args, slugs) {
  const requested = slugs.map((slug) => `(${sqlQuote(slug)})`).join(',\n  ');
  return `WITH requested(slug) AS (VALUES\n  ${requested}\n)
SELECT requested.slug,
  COALESCE(encode(sha256(convert_to(source.contents, 'UTF8')), 'hex'), ''),
  COALESCE(revision.revision_number::text, ''),
  COALESCE(to_char(revision.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'), '')
FROM requested
LEFT JOIN page ON page.site_id = ${sqlInt(args.siteId)} AND page.slug = requested.slug AND page.deleted_at IS NULL
LEFT JOIN page_revision revision ON revision.revision_id = page.latest_revision_id
LEFT JOIN text source ON source.hash = revision.wikitext_hash
ORDER BY requested.slug;`;
}

const sha256 = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');

export function compareDriftRows(closure, output) {
  const local = new Map();
  for (const line of output.split('\n').filter(Boolean)) {
    const fields = line.split('|');
    if (fields.length !== 4) throw new Error('drift query returned a malformed row');
    if (local.has(fields[0])) throw new Error(`drift query returned duplicate page ${fields[0]}`);
    local.set(fields[0], fields);
  }
  const pages = closure.map(({slug, source}) => {
    const [, localSha = '', revisionNumber = '', createdAt = ''] = local.get(slug) ?? [];
    const corpusSha = source === null ? null : sha256(source);
    let status;
    if (corpusSha === null) status = 'corpus_missing';
    else if (!localSha) status = 'local_missing';
    else status = localSha === corpusSha ? 'current' : 'drift';
    return {
      slug,
      status,
      corpus_sha256: corpusSha,
      local_sha256: localSha || null,
      local_revision_number: revisionNumber === '' ? null : Number(revisionNumber),
      local_revision_created_at: createdAt || null,
    };
  });
  const stale = pages.filter((page) => page.status !== 'current');
  return {status: stale.length ? 'drift' : 'current', drifted: stale.map((page) => page.slug), pages};
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) { console.log(usage()); return 0; }
  const closure = includeClosure(args.roots, corpusSourceReader(args.corpusRoot, args.branch), args.sourceSite);
  const sqlExecutor = createSqlExecutor({dbUrl: args.dbUrl, dbContainer: args.dbContainer});
  try {
    const output = await sqlExecutor.runSql(buildDriftQuery(args, closure.map((page) => page.slug)), {capture: true});
    const report = compareDriftRows(closure, output);
    console.log(JSON.stringify({site_id: args.siteId, branch: args.branch, roots: args.roots, ...report}, null, 2));
    return report.status === 'current' ? 0 : 1;
  } finally {
    await sqlExecutor.close();
  }
}

await runCliIfMain(import.meta.url, main);
