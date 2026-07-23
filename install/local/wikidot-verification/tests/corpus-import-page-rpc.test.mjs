import assert from 'node:assert/strict';
import test from 'node:test';

import {
  corpusImportCategoryName,
  createCorpusImportPage,
  getCorpusImportFile,
  getCorpusImportPage,
  rerenderCorpusImportPage,
} from '../src/corpus-import-page-rpc.mjs';

test('corpus import page RPC operations preserve their request contracts', async () => {
  const calls = [];
  const rpc = async (...parameters) => {
    calls.push(parameters);
    return null;
  };
  const args = { siteId: 6000005, userId: 17, ipAddress: '127.0.0.1' };
  const row = { fullname: 'component:page', title_shown: 'Shown title' };

  await getCorpusImportPage(args, rpc, row.fullname);
  await getCorpusImportFile(args, rpc, 173, 'fixture.txt');
  await createCorpusImportPage(args, rpc, row, '[[module]]');
  await rerenderCorpusImportPage(args, rpc, 173, 44);

  assert.deepEqual(calls[0][1], 'page_get');
  assert.deepEqual(calls[1][3], { siteId: 6000005, pageRef: 173 });
  assert.equal(calls[2][2].title, 'Shown title');
  assert.equal(calls[2][2].wikitext, '[[module]]');
  assert.deepEqual(calls[3][2], { site_id: 6000005, category_id: 44, page_id: 173 });
  assert.equal(corpusImportCategoryName('component:page'), 'component');
  assert.equal(corpusImportCategoryName('page'), '_default');
});
