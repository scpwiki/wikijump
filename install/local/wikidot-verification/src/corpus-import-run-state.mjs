import crypto from 'node:crypto';

import {
  sqlByteaFromHex,
  sqlInt,
  sqlQuote,
} from './corpus-import-sql-values.mjs';

export async function ensureCorpusImportRun(args, sqlExecutor, manifestText, manifestRows, selectedRows, completeInventory) {
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
  site_id, source_branch, source_site, manifest_sha256, manifest_row_count, complete_inventory, state, summary
) VALUES (
  ${sqlInt(args.siteId)}, ${sqlQuote(sourceBranch)}, ${sqlQuote(sourceSite)}, ${sqlByteaFromHex(manifestSha)},
  ${sqlInt(manifestRows.length)}, ${completeInventory ? 'true' : 'false'}, 'running', ${sqlQuote(summary)}::jsonb
)
RETURNING import_run_id;
`;
  return Number.parseInt(await sqlExecutor.runSql(sql, { capture: true }), 10);
}

export function recordCorpusImportItemSql(row, pageId, importRunId, state, error = null) {
  return `
INSERT INTO wikidot_corpus_import_item (
  import_run_id, source_entity_id, source_fullname, page_id, source_sha256, meta_sha256, state, error
) VALUES (
  ${sqlInt(importRunId)}, ${sqlQuote(row.source_entity_id)}, ${sqlQuote(row.fullname)},
  ${pageId === null ? 'NULL' : sqlInt(pageId)}, ${sqlByteaFromHex(row.source_sha256)}, ${sqlByteaFromHex(row.meta_sha256)},
  ${sqlQuote(state)}, ${error === null ? 'NULL' : `${sqlQuote(JSON.stringify(error))}::jsonb`}
)
ON CONFLICT (import_run_id, source_entity_id) DO UPDATE SET
  page_id = EXCLUDED.page_id, state = EXCLUDED.state, error = EXCLUDED.error, updated_at = NOW();
`;
}

export async function finishCorpusImportRun(args, sqlExecutor, importRunId, summary, state = 'done') {
  await sqlExecutor.runSql(`
UPDATE wikidot_corpus_import_run
SET state = ${sqlQuote(state)}, finished_at = NOW(), summary = ${sqlQuote(JSON.stringify(summary))}::jsonb
WHERE import_run_id = ${sqlInt(importRunId)};
`);
}
