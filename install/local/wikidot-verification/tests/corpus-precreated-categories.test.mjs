import assert from 'node:assert/strict';
import test from 'node:test';

import { parsePrecreatedCategoryIds } from '../src/corpus-precreated-categories.mjs';

test('precreated category JSON preserves empty, delimiter-bearing, and Unicode slugs', () => {
  const output = JSON.stringify([
    { slug: '', category_id: '11' },
    { slug: 'foo|bar', category_id: '12' },
    { slug: 'line\nbreak 雪', category_id: '13' },
  ]);

  assert.deepEqual(
    [...parsePrecreatedCategoryIds(output)],
    [['', 11], ['foo|bar', 12], ['line\nbreak 雪', 13]],
  );
});

test('precreated category JSON fails closed on missing or malformed output', () => {
  for (const output of [
    '',
    'not JSON',
    '{}',
    '[null]',
    '[{"slug":"x","category_id":1}]',
    '[{"slug":"x","category_id":"0"}]',
    '[{"slug":"x","category_id":"9007199254740992"}]',
    '[{"slug":"x","category_id":"1","extra":true}]',
    '[{"slug":"x","category_id":"1"},{"slug":"x","category_id":"2"}]',
  ]) {
    assert.throws(
      () => parsePrecreatedCategoryIds(output),
      /invalid category precreate output/u,
      output,
    );
  }
});

test('precreated category JSON accepts an explicit empty result array', () => {
  assert.deepEqual([...parsePrecreatedCategoryIds('[]')], []);
});
