import assert from 'node:assert/strict';
import test from 'node:test';

import { exactDataRecord } from '../src/wikidot-xmlrpc-exact-data-record.mjs';

test('exactDataRecord snapshots enumerable own data fields only', () => {
  const input = Object.create(null);
  input.first = 'one';
  input.second = 'two';
  const result = exactDataRecord(input, ['first', 'second'], 'fixture');

  assert.deepEqual(result, { first: 'one', second: 'two' });
  assert.equal(Object.isFrozen(result), true);
  assert.throws(() => exactDataRecord(['one'], ['0'], 'fixture'), /data object/);
  assert.throws(() => exactDataRecord({ first: 'one' }, ['first', 'second'], 'fixture'), /unexpected fields/);
});
