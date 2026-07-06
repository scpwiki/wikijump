import { strict as assert } from 'node:assert';
import test from 'node:test';

import {
  aggregateCompareVerdict,
  comparePair,
  hasRawMarker,
  matchLedgerEntry,
  normalizeText,
  RENDER_COMPARE_SCHEMA,
} from '../src/render-compare.mjs';

test('identical texts match', () => {
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'SCP-173 is a statue.',
    localVisibleText: 'SCP-173 is a statue.',
  });
  assert.equal(pair.verdict, 'match');
  assert.deepEqual(pair.findings, []);
});

test('hostname differences are explained by hostname_map channel', () => {
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'See https://scp-wiki.wikidot.com/scp-002 now',
    localVisibleText: 'See https://scp-wiki.wikijump.localhost/scp-002 now',
  });
  assert.equal(pair.verdict, 'match');
  const info = pair.findings.find((f) => f.category === 'normalized_difference');
  assert.ok(info?.informational);
});

test('relative timestamps normalize', () => {
  const { text } = normalizeText('edited 827 days ago by someone');
  assert.ok(text.includes('{{relative-time}}'));
});

test('real text difference is a regression', () => {
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'The object is safe.',
    localVisibleText: 'The object is euclid.',
  });
  assert.equal(pair.verdict, 'regression');
  const finding = pair.findings.find((f) => f.category === 'visible_text_difference');
  assert.ok(finding.source.includes('safe'));
  assert.ok(finding.local.includes('euclid'));
});

test('raw marker in local text is flagged', () => {
  assert.ok(hasRawMarker('leftover [[module ListPages]] text'));
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'clean',
    localVisibleText: 'clean [[include :scp-wiki:x]]',
  });
  assert.ok(pair.findings.some((f) => f.category === 'raw_marker_visible' && f.side === 'local'));
  assert.equal(pair.verdict, 'regression');
});

test('self comparison guard', () => {
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'x',
    localVisibleText: 'x',
    sourceUrl: 'https://same/x',
    localUrl: 'https://same/x',
  });
  assert.ok(pair.findings.some((f) => f.category === 'self_comparison'));
  assert.equal(pair.verdict, 'regression');
});

test('whitespace_collapse hiding a difference triggers the guard', () => {
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'alpha  beta',
    localVisibleText: 'alpha beta',
    channels: { whitespace_collapse: true },
  });
  assert.ok(
    pair.findings.some((f) => f.category === 'normalization_hides_visible_text_difference'),
  );
  assert.equal(pair.verdict, 'regression');
});

test('whitespace difference without collapse channel is a plain difference', () => {
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'alpha  beta',
    localVisibleText: 'alpha beta',
  });
  assert.ok(pair.findings.some((f) => f.category === 'visible_text_difference'));
});

test('ledger entry converts regression to accepted-diff', () => {
  const ledger = [
    {
      category: 'visible_text_difference',
      scope: 'EN:chrome-words',
      policy_reason: 'local chrome uses Wikijump labels',
      fixture_ids: ['EN:a'],
    },
  ];
  const pair = comparePair({
    fixtureId: 'EN:a',
    sourceVisibleText: 'Wikidot footer',
    localVisibleText: 'Wikijump footer',
    ledger,
  });
  assert.equal(pair.verdict, 'accepted-diff');
  assert.deepEqual(pair.ledger_refs, ['visible_text_difference:EN:chrome-words']);
});

test('ledger does not apply to other fixtures', () => {
  const ledger = [
    { category: 'visible_text_difference', scope: 's', fixture_ids: ['EN:other'] },
  ];
  assert.equal(matchLedgerEntry(ledger, 'EN:a', 'visible_text_difference'), null);
});

test('missing visible text pair blocks', () => {
  const pair = comparePair({ fixtureId: 'EN:a', sourceVisibleText: '', localVisibleText: 'x' });
  assert.ok(pair.findings.some((f) => f.category === 'visible_text_pair_missing'));
  assert.equal(pair.verdict, 'regression');
});

test('aggregate exit code 1 on any regression', () => {
  const pairs = [
    comparePair({ fixtureId: 'a', sourceVisibleText: 'x', localVisibleText: 'x' }),
    comparePair({ fixtureId: 'b', sourceVisibleText: 'x', localVisibleText: 'y' }),
  ];
  const { verdict, exitCode } = aggregateCompareVerdict({ runId: 'r', pairs });
  assert.equal(verdict.schema, RENDER_COMPARE_SCHEMA);
  assert.equal(verdict.aggregate.counts.regression, 1);
  assert.deepEqual(verdict.aggregate.regressions, ['b']);
  assert.equal(exitCode, 1);
});

test('normalization channel list is recorded on every pair', () => {
  const pair = comparePair({ fixtureId: 'a', sourceVisibleText: 'x', localVisibleText: 'x' });
  assert.ok(pair.normalization_channels.includes('hostname_map'));
  assert.ok(!pair.normalization_channels.includes('whitespace_collapse'));
});
