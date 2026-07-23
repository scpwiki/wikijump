import assert from 'node:assert/strict';
import test from 'node:test';

import {
  ensureCorpusImportRun,
  finishCorpusImportRun,
  recordCorpusImportItemSql,
} from '../src/corpus-import-run-state.mjs';

test('corpus import run state writes a single-source manifest lifecycle', async () => {
  const queries = [];
  const sqlExecutor = {
    async runSql(sql) {
      queries.push(sql);
      return queries.length === 1 ? '9' : '';
    },
  };
  const args = { siteId: 6000005, sourceSite: null, sourceBranch: null };
  const rows = [{ source_site: 'scp-wiki', source_branch: 'en' }];

  assert.equal(await ensureCorpusImportRun(args, sqlExecutor, 'fixture', rows, rows, true), 9);
  assert.match(queries[0], /source_branch/);
  assert.match(queries[0], /'scp-wiki'/);
  assert.match(recordCorpusImportItemSql({ source_entity_id: '00000000-0000-0000-0000-000000000001', fullname: 'scp-173', source_sha256: 'a'.repeat(64), meta_sha256: 'b'.repeat(64) }, 173, 9, 'done'), /ON CONFLICT/);
  await finishCorpusImportRun(args, sqlExecutor, 9, { created: 1 });
  assert.match(queries[1], /SET state = 'done'/);
});
