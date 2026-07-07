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

test('granular visible-text ledger accepts exact source/local text edits', () => {
  const ledger = [
    {
      category: 'visible_text_difference',
      scope: 'EN:scp-9506-navside-source-freshness-20260708',
      policy_reason: 'frozen live capture predates current corpus nav:side',
      fixture_ids: ['EN:scp-9506'],
      source_text: 'X OTHER SCP',
      local_text: 'X Lost Series OTHER SCP',
    },
    {
      category: 'visible_text_difference',
      scope: 'EN:scp-9506-rating-source-freshness-20260708',
      policy_reason: 'frozen live rating differs from imported corpus snapshot',
      fixture_ids: ['EN:scp-9506'],
      source_text: 'RATING: +401 Rate ( +401 ) Discuss (94)',
      local_text: 'RATING: +371 Rate ( +371 ) Discuss (79)',
    },
  ];
  const pair = comparePair({
    fixtureId: 'EN:scp-9506',
    sourceVisibleText: 'SCP BY SERIES IX | X OTHER SCP Explained RATING: +401 Rate ( +401 ) Discuss (94)',
    localVisibleText: 'SCP BY SERIES IX | X Lost Series OTHER SCP Explained RATING: +371 Rate ( +371 ) Discuss (79)',
    ledger,
  });

  assert.equal(pair.verdict, 'accepted-diff');
  assert.deepEqual(pair.ledger_refs, [
    'visible_text_difference:EN:scp-9506-navside-source-freshness-20260708',
    'visible_text_difference:EN:scp-9506-rating-source-freshness-20260708',
  ]);
  const finding = pair.findings.find((f) => f.category === 'visible_text_difference');
  assert.equal(finding.accepted_by_ledger.length, 2);
});

test('granular visible-text ledger does not accept unlisted differences', () => {
  const ledger = [
    {
      category: 'visible_text_difference',
      scope: 'EN:scp-9506-navside-source-freshness-20260708',
      policy_reason: 'frozen live capture predates current corpus nav:side',
      fixture_ids: ['EN:scp-9506'],
      source_text: 'X OTHER SCP',
      local_text: 'X Lost Series OTHER SCP',
    },
  ];
  const pair = comparePair({
    fixtureId: 'EN:scp-9506',
    sourceVisibleText: 'SCP BY SERIES IX | X OTHER SCP Explained RATING: +401',
    localVisibleText: 'SCP BY SERIES IX | X Lost Series OTHER SCP Explained RATING: +371',
    ledger,
  });

  assert.equal(pair.verdict, 'regression');
  assert.deepEqual(pair.ledger_refs, []);
  const finding = pair.findings.find((f) => f.category === 'visible_text_difference');
  assert.equal(finding.accepted_by_ledger.length, 1);
  assert.ok(finding.remaining.source.includes('+401'));
  assert.ok(finding.remaining.local.includes('+371'));
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
