import assert from 'node:assert/strict';
import test from 'node:test';

import {
  buildParentLinkParentPagesSql,
  buildParentLinkSql,
  manifestRowsWithParents,
  parseParentLinkParentPages,
  parseParentLinkSummary,
  shouldProcessParentLinks,
} from '../src/corpus-import-parent-links.mjs';

test('attachment-only imports do not alter or rerender parent links', () => {
  assert.equal(shouldProcessParentLinks({ attachmentsOnlyExisting: true }), false);
  assert.equal(shouldProcessParentLinks({ attachmentsOnlyExisting: false }), true);
});

test('corpus import builds parent links from manifest parent_fullname metadata', () => {
  const rows = [
    { fullname: 'fragment:scp-7243-0', parent_fullname: 'scp-7243' },
    { fullname: 'scp-7243', parent_fullname: null },
  ];

  assert.deepEqual(manifestRowsWithParents(rows), [rows[0]]);
  const sql = buildParentLinkSql({ siteId: 6000006 }, rows);

  assert.match(sql, /INSERT INTO page_parent/);
  assert.match(sql, /fragment:scp-7243-0/);
  assert.match(sql, /scp-7243/);
  assert.match(sql, /child\.site_id = 6000006/);
  assert.match(sql, /parent\.site_id = 6000006/);
  assert.match(sql, /ON CONFLICT DO NOTHING/);

  const parentSql = buildParentLinkParentPagesSql({ siteId: 6000006 }, rows);
  assert.match(parentSql, /SELECT DISTINCT parent\.page_id::text/);
  assert.match(parentSql, /parent\.slug = requested\.parent_slug/);
});

test('corpus import parent link summary is parsed fail-closed', () => {
  assert.deepEqual(parseParentLinkSummary('2|1|1|1|0'), {
    parent_link_requested: 2,
    parent_link_ready: 1,
    parent_link_inserted: 1,
    parent_link_missing_parent: 1,
    parent_link_missing_child: 0,
  });

  assert.throws(() => parseParentLinkSummary('not-a-summary'), /invalid parent link summary/);
  assert.throws(() => parseParentLinkSummary('2|1|1|1|0\nNOTICE'), /invalid parent link summary/);
  assert.throws(() => parseParentLinkSummary('2|1|1|1|0|extra'), /invalid parent link summary/);
  assert.throws(() => parseParentLinkSummary('2|1|1|-1|0'), /invalid parent link summary/);
});

test('corpus import parses parent pages to rerender after link insertion', () => {
  assert.deepEqual(parseParentLinkParentPages('3000087504|7000001\n'), [
    { page_id: 3000087504, page_category_id: 7000001 },
  ]);
  assert.deepEqual(parseParentLinkParentPages('\n'), []);
  assert.throws(() => parseParentLinkParentPages('bad-row'), /invalid parent page link row/);
  assert.throws(() => parseParentLinkParentPages('3000087504abc|7000001'), /invalid parent page link row/);
  assert.throws(() => parseParentLinkParentPages('3000087504|7000001|extra'), /invalid parent page link row/);
});
