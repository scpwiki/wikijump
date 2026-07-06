import { strict as assert } from 'node:assert';
import test from 'node:test';

import {
  buildMergeReadiness,
  parseDeviationLog,
  MERGE_READINESS_SCHEMA,
} from '../src/deviation-log.mjs';

const entry = (overrides = {}) =>
  JSON.stringify({
    id: 'dev-001',
    date: '2026-07-06',
    summary: 'whitespace_collapse default off',
    reason: 'spec open question',
    review_state: 'approved',
    ...overrides,
  });

test('parseDeviationLog accepts valid entries and reports errors', () => {
  const good = parseDeviationLog(`${entry()}\n${entry({ id: 'dev-002', review_state: 'pending' })}\n`);
  assert.equal(good.entries.length, 2);
  assert.deepEqual(good.errors, []);

  const bad = parseDeviationLog(`not json\n${entry({ review_state: 'maybe' })}\n${entry({ summary: '' })}`);
  assert.equal(bad.errors.length, 3);
});

test('merge readiness blocks on failing validators and pending deviations', () => {
  const { entries } = parseDeviationLog(`${entry({ id: 'dev-002', review_state: 'pending' })}`);
  const report = buildMergeReadiness({
    runId: 'r',
    branch: 'b',
    validators: [
      { name: 'v1-import-health', exitCode: 0 },
      { name: 'v2-render-health', exitCode: 1 },
    ],
    deviations: entries,
  });
  assert.equal(report.schema, MERGE_READINESS_SCHEMA);
  assert.equal(report.merge_ready, false);
  assert.deepEqual(
    report.blockers.map((b) => b.kind).sort(),
    ['deviation-unreviewed', 'validator-failing'],
  );
});

test('merge ready when validators pass and deviations reviewed', () => {
  const { entries } = parseDeviationLog(entry());
  const report = buildMergeReadiness({
    runId: 'r',
    branch: 'b',
    validators: [{ name: 'v1', exitCode: 0 }],
    deviations: entries,
  });
  assert.equal(report.merge_ready, true);
  assert.deepEqual(report.blockers, []);
});

test('rejected deviations and invalid log lines block', () => {
  const parsed = parseDeviationLog(`garbage\n${entry({ review_state: 'rejected' })}`);
  const report = buildMergeReadiness({
    runId: 'r',
    branch: 'b',
    validators: [],
    deviations: parsed.entries,
    logErrors: parsed.errors,
  });
  assert.equal(report.merge_ready, false);
  assert.ok(report.blockers.some((b) => b.kind === 'deviation-rejected'));
  assert.ok(report.blockers.some((b) => b.kind === 'deviation-log-invalid'));
});
