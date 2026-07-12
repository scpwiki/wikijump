import { strict as assert } from 'node:assert';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  applyThreshold,
  buildImportHealthVerdict,
  classifyFailure,
  parseImportLog,
  IMPORT_HEALTH_SCHEMA,
} from '../src/import-health.mjs';

test('classifyFailure maps known error shapes', () => {
  assert.equal(
    classifyFailure('page_create failed: {"code":3106,"message":"User does not have permission"}'),
    'auth-context-missing',
  );
  assert.equal(
    classifyFailure('page_create failed: ... include expansion exceeded maximum depth 8 ...'),
    'include-depth-exceeded',
  );
  assert.equal(classifyFailure('page_create timed out after 120000ms'), 'render-timeout');
  assert.equal(
    classifyFailure('created page not found after page_create: art:re:goi-arts'),
    'page-missing-after-create',
  );
  assert.equal(classifyFailure('page_create failed: something novel'), 'rpc-error');
  assert.equal(classifyFailure('totally unknown breakage'), null);
});

test('parseImportLog reads JSONL rows and multi-line summary', () => {
  const log = [
    '{"slug":"a","action":"created","page_id":1}',
    '{"slug":"b","action":"failed","error":"page_create timed out after 120000ms"}',
    '{',
    ' "summary": { "created": 1, "failed": 1 }',
    '}',
  ].join('\n');
  const { rows, summary } = parseImportLog(log);
  assert.equal(rows.length, 2);
  assert.deepEqual(summary, { created: 1, failed: 1 });
});

test('parseImportLog retains incomplete action rows for fail-closed classification', () => {
  const { rows, summary } = parseImportLog([
    'null',
    '{"slug":"missing-action"}',
    '{"action":"created"}',
    '{"summary":null}',
  ].join('\n'));

  assert.deepEqual(rows, [{ slug: 'missing-action' }, { action: 'created' }]);
  assert.equal(summary, null);
  const { verdict, exitCode } = buildImportHealthVerdict({
    runId: 'r',
    family: 'EN',
    rows,
  });
  assert.equal(exitCode, 2);
  assert.equal(verdict.aggregate.unclassified, 2);
  assert.equal(verdict.aggregate.rows_done, 0);
});

test('verdict counts done/failed and classifies failures', () => {
  const rows = [
    { slug: 'a', action: 'created' },
    { slug: 'b', action: 'skipped_existing_done' },
    { slug: 'c', action: 'collision_existing_page' },
    { slug: 'd', action: 'failed', error: 'include expansion exceeded maximum depth 8' },
  ];
  const { verdict, exitCode } = buildImportHealthVerdict({ runId: 'r', family: 'EN', rows });
  assert.equal(verdict.schema, IMPORT_HEALTH_SCHEMA);
  assert.equal(verdict.aggregate.rows_total, 4);
  assert.equal(verdict.aggregate.rows_done, 3);
  assert.equal(verdict.aggregate.import_rate, 0.75);
  assert.equal(verdict.aggregate.failure_counts['include-depth-exceeded'], 1);
  assert.equal(verdict.aggregate.unclassified, 0);
  assert.equal(exitCode, 0);
});

test('unclassified failure forces exit 2', () => {
  const rows = [{ slug: 'x', action: 'failed', error: 'mystery' }];
  const { verdict, exitCode } = buildImportHealthVerdict({ runId: 'r', family: 'EN', rows });
  assert.equal(exitCode, 2);
  assert.equal(verdict.failures[0].code, 'unclassified');
});

test('snapshot-mismatch collision is a classified failure, not done', () => {
  const rows = [{ slug: 'x', action: 'collision_existing_snapshot_mismatch' }];
  const { verdict, exitCode } = buildImportHealthVerdict({ runId: 'r', family: 'EN', rows });
  assert.equal(verdict.aggregate.rows_done, 0);
  assert.equal(verdict.aggregate.failure_counts['collision-snapshot-mismatch'], 1);
  assert.equal(exitCode, 0);
});

test('prototype-named actions remain unclassified', () => {
  for (const action of ['__proto__', 'constructor', 'toString', 'prototype', 'hasOwnProperty']) {
    const { verdict, exitCode } = buildImportHealthVerdict({
      runId: 'r',
      family: 'EN',
      rows: [{ slug: `x-${action}`, action }],
    });
    assert.equal(exitCode, 2);
    assert.equal(verdict.aggregate.unclassified, 1);
    assert.equal(verdict.aggregate.failure_counts.unclassified, 1);
    assert.deepEqual(verdict.failures, [
      { slug: `x-${action}`, code: 'unclassified', detail: `unknown action ${action}` },
    ]);
  }
});

test('CLI reports prototype-named, null, and non-string actions as unclassified', () => {
  const temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), 'import-health-actions-'));
  const logPath = path.join(temporaryDirectory, 'import.jsonl');
  const outputPath = path.join(temporaryDirectory, 'verdict.json');
  const scriptPath = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    '../scripts/import-health-report.mjs',
  );
  const actions = [
    '__proto__',
    'constructor',
    'toString',
    'prototype',
    null,
    17,
    { name: 'created' },
    { toString: null },
    { valueOf: null, toString: null },
  ];
  fs.writeFileSync(
    logPath,
    actions.map((action, index) => JSON.stringify({ slug: `row-${index}`, action })).join('\n'),
  );

  try {
    const result = spawnSync(
      process.execPath,
      [scriptPath, '--log', logPath, '--output', outputPath, '--run-id', 'readback'],
      { encoding: 'utf8' },
    );
    assert.equal(result.status, 2, result.stderr);

    const verdict = JSON.parse(fs.readFileSync(outputPath, 'utf8'));
    assert.equal(verdict.aggregate.rows_total, actions.length);
    assert.equal(verdict.aggregate.rows_done, 0);
    assert.equal(verdict.aggregate.unclassified, actions.length);
    assert.equal(verdict.aggregate.failure_counts.unclassified, actions.length);
    assert.deepEqual(
      verdict.failures.map(({ slug, code }) => ({ slug, code })),
      actions.map((_, index) => ({ slug: `row-${index}`, code: 'unclassified' })),
    );
  } finally {
    fs.rmSync(temporaryDirectory, { recursive: true, force: true });
  }
});

test('threshold gate', () => {
  const rows = [
    { slug: 'a', action: 'created' },
    { slug: 'b', action: 'failed', error: 'page_create timed out after 1ms' },
  ];
  const { verdict } = buildImportHealthVerdict({ runId: 'r', family: 'EN', rows });
  assert.equal(applyThreshold(verdict, 0.9), 1);
  assert.equal(applyThreshold(verdict, 0.5), 0);
  assert.equal(applyThreshold(verdict, null), 0);
});
