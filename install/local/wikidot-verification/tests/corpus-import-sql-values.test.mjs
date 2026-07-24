import assert from 'node:assert/strict';
import test from 'node:test';

import {
  sqlByteaFromHex,
  sqlInt,
  sqlQuote,
  sqlTextArray,
  sqlTextFromBase64,
  sqlTextHash,
  sqlTimestamp,
} from '../src/corpus-import-sql-values.mjs';

test('corpus import SQL value helpers quote and validate typed values', () => {
  assert.equal(sqlQuote("O'Brien"), "'O''Brien'");
  assert.equal(sqlQuote(null), 'NULL');
  assert.equal(sqlTimestamp('2026-07-23T00:00:00Z'), "TIMESTAMPTZ '2026-07-23T00:00:00Z'");
  assert.equal(sqlTimestamp(''), 'NULL');
  assert.equal(sqlInt(42), '42');
  assert.throws(() => sqlInt(2.5), /expected integer/);
  assert.equal(sqlByteaFromHex('a'.repeat(64)), "decode('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'hex')");
  assert.equal(sqlTextHash('b'.repeat(32)), "decode('bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', 'hex')");
  assert.equal(sqlTextArray(['one', "two's"]), "ARRAY['one','two''s']::text[]");
  assert.match(sqlTextFromBase64('snowman ☃'), /^convert_from\(decode\('/);
});
