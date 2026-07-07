import assert from 'node:assert/strict';
import test from 'node:test';

import {
  createSqlExecutor,
  formatCaptureOutput,
} from '../src/corpus-import-sql.mjs';

test('formatCaptureOutput matches psql capture for multi-result rows', () => {
  const output = formatCaptureOutput([
    {
      fields: [{ name: 'first' }, { name: 'second' }],
      rows: [
        { first: 'alpha', second: null },
        { first: 'beta', second: 'gamma' },
      ],
    },
    {
      fields: [{ name: 'ignored' }],
      rows: [],
    },
    {
      fields: [{ name: 'last' }],
      rows: [{ last: 'omega ' }],
    },
  ]);

  assert.equal(output, 'alpha|\nbeta|gamma\nomega');
});

test('formatCaptureOutput handles single-column row arrays', () => {
  const output = formatCaptureOutput({
    fields: [{ name: 'value' }],
    rows: [['one'], ['two']],
  });

  assert.equal(output, 'one\ntwo');
});

test('createSqlExecutor selects pg mode when dbUrl is set without connecting', async () => {
  const executor = createSqlExecutor({ dbUrl: 'postgres://wikijump:wikijump@127.0.0.1:5432/wikijump', dbContainer: 'unused' });

  assert.equal(executor.mode, 'pg');
  await executor.close();
});

test('createSqlExecutor selects docker psql mode when dbUrl is unset', async () => {
  const executor = createSqlExecutor({ dbUrl: null, dbContainer: 'local-database-1' });

  assert.equal(executor.mode, 'docker-psql');
  await executor.close();
});
