import assert from 'node:assert/strict';
import test from 'node:test';

import { parsePrecreatedCategoryIds } from '../src/corpus-precreated-categories.mjs';

test('precreated category output round-trips delimiter, empty, and Unicode slugs', () => {
  const output = [
    `${Buffer.from('foo|bar').toString('hex')}|123`,
    '|456',
    `${Buffer.from('日本語').toString('hex')}|789`,
  ].join('\n');

  assert.deepEqual(
    [...parsePrecreatedCategoryIds(output)],
    [['foo|bar', 123], ['', 456], ['日本語', 789]],
  );
});

test('precreated category output rejects malformed hex and category IDs', () => {
  for (const output of [
    '0|1',
    'gg|1',
    '61|0',
    '61|-1',
    '61|1suffix',
    `61|${Number.MAX_SAFE_INTEGER + 1}`,
    '61|1|extra',
  ]) {
    assert.throws(
      () => parsePrecreatedCategoryIds(output),
      /invalid category precreate output/u,
      output,
    );
  }
});
