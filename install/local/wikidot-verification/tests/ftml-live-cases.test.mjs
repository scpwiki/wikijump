import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {
  buildFailedPreviewRetryPlans,
  buildSavedPagePlans,
  classifyFixture,
  collectFtmlFixtureCases,
  collectFtmlRecordedCases,
  isolateBatchInteractions,
} from '../src/ftml-live-cases.mjs';
import {extractMarkedFragments} from '../scripts/verify-ftml-live-pages.mjs';
import {compareFragment} from '../scripts/compare-wikidot-live-pages.mjs';

test('fixture classification separates pack-safe, isolated, runtime, and FTML-only cases', () => {
  assert.deepEqual(
    classifyFixture('test/bold/basic/input.ftml', '**alpha**'),
    {
      execution_class: 'saved-page-batch',
      page_scope: 'batch-safe',
      reasons: ['conservative-static-pack-safe'],
    },
  );
  assert.equal(
    classifyFixture('test/code/basic/input.ftml', '[[code]]\nalpha\n[[/code]]').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/misc/symbols/input.ftml', 'alpha\\\n').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/misc/char/input.ftml', '[[char copy]]').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/link/internal/input.ftml', '[[[target-page|Label]]]').execution_class,
    'wikijump-runtime',
  );
  assert.equal(
    classifyFixture('test/email/basic/input.ftml', 'abc@example.com').execution_class,
    'wikijump-runtime',
  );
  assert.equal(
    classifyFixture('test/misc/email/input.ftml', 'abc@example.com').execution_class,
    'wikijump-runtime',
  );
  assert.equal(
    classifyFixture('test/tabs/basic/input.ftml', '[[tabview]]').execution_class,
    'wikijump-runtime',
  );
  assert.equal(
    classifyFixture('record/includes/input.ftml', '[[include').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('record/image/input.ftml', 'UNCLOSED [[image landscape.jpg').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/misc/guillemet/input.ftml', 'left << guillemets >> right').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/monospace/basic/input.ftml', '[[tt]]text[[/tt]]').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/radio/basic/input.ftml', '[[radio fruit]] Apple').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/include/wikidot/input.ftml', '[[include component:x]]').execution_class,
    'wikijump-runtime',
  );
  assert.equal(
    classifyFixture('test/misc/variable/input.ftml', 'A {$variable}').execution_class,
    'wikijump-runtime',
  );
  assert.equal(
    classifyFixture('test/module/css/input.ftml', '[[module CSS]]').execution_class,
    'page-preview-isolated',
  );
  assert.equal(
    classifyFixture('test/module/rate/input.ftml', '[[module Rate]]').execution_class,
    'wikijump-runtime',
  );
  assert.equal(
    classifyFixture('record/toc/input.ftml', '[[toc]]\n+ Heading\n[[[target-page]]]').page_scope,
    'isolated',
  );
  assert.equal(
    classifyFixture('record/footnote/input.ftml', '[[footnote]][[module Rate]][[/footnote]]').page_scope,
    'isolated',
  );
  assert.equal(
    classifyFixture('record/comment/input.ftml', '[!--[[include]]--]visible').page_scope,
    'isolated',
  );
  assert.equal(
    classifyFixture('test/module/rate/input.ftml', '[[module Rate]]').page_scope,
    'batch-safe',
  );
  assert.equal(
    classifyFixture('test/include/elements/input.ftml', '[[include-elements x]]').execution_class,
    'not-applicable',
  );
});

test('fixture collector reads both FTML fixture roots with source identities', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'ftml-live-cases-'));
  await fs.mkdir(path.join(root, 'test', 'bold', 'basic'), {recursive: true});
  await fs.mkdir(path.join(root, 'tests', 'fixtures', 'article'), {recursive: true});
  await fs.writeFile(path.join(root, 'test', 'bold', 'basic', 'input.ftml'), '**alpha**');
  await fs.writeFile(path.join(root, 'tests', 'fixtures', 'article', 'source.ftml'), 'beta');
  const cases = collectFtmlFixtureCases(root);
  assert.equal(cases.length, 2);
  assert.deepEqual(cases.map((value) => value.case_id), ['test--bold--basic', 'tests--fixtures--article--source']);
  assert.ok(cases.every((value) => value.source_sha256.length === 64));
});

test('record collector splits only on LF and deduplicates runtime sources', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'ftml-recorded-cases-'));
  const records = path.join(root, 'records.jsonl');
  const source = 'alpha\u2028beta';
  const values = [
    {schema: 'ftml.test_source_record.v1', stage: 'tokenize', test_name: 'one', caller: {}, source},
    {schema: 'ftml.test_source_record.v1', stage: 'preprocess-input', test_name: 'two', caller: {}, source},
  ];
  await fs.writeFile(records, `${values.map(JSON.stringify).join('\n')}\n`);
  const cases = collectFtmlRecordedCases([records]);
  assert.equal(cases.length, 1);
  assert.equal(cases[0].source, source);
  assert.equal(cases[0].record_origins.length, 2);
});

test('record collector isolates sources larger than the observed safe batch size', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'ftml-recorded-large-'));
  const records = path.join(root, 'records.jsonl');
  const value = {
    schema: 'ftml.test_source_record.v1',
    stage: 'tokenize',
    test_name: 'large',
    caller: {},
    source: 'x'.repeat(7_501),
  };
  await fs.writeFile(records, `${JSON.stringify(value)}\n`);
  const [recordedCase] = collectFtmlRecordedCases([records]);
  assert.equal(recordedCase.execution_class, 'page-preview-isolated');
  assert.deepEqual(recordedCase.reasons, ['exceeds-observed-safe-batch-size']);
});

test('record collector keeps oversized runtime cases in the runtime lane', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'ftml-recorded-large-runtime-'));
  const records = path.join(root, 'records.jsonl');
  const value = {
    schema: 'ftml.test_source_record.v1',
    stage: 'tokenize',
    test_name: 'large-runtime',
    caller: {},
    source: `[[module Rate]]\n${'x'.repeat(7_501)}`,
  };
  await fs.writeFile(records, `${JSON.stringify(value)}\n`);
  const [recordedCase] = collectFtmlRecordedCases([records]);
  assert.equal(recordedCase.execution_class, 'wikijump-runtime');
  assert.equal(recordedCase.page_scope, 'isolated');
  assert.deepEqual(recordedCase.reasons, [
    'page-or-site-runtime',
    'exceeds-observed-safe-batch-size',
  ]);
});

test('measured batch interactions move to isolated preview execution', () => {
  const cases = [
    {case_id: 'safe', execution_class: 'saved-page-batch', reasons: ['initial']},
    {case_id: 'interacting', execution_class: 'saved-page-batch', reasons: ['initial']},
  ];
  const verification = {
    comparisons: [
      {case_id: 'safe', status: 'match'},
      {case_id: 'interacting', status: 'batch-context-interaction'},
    ],
  };
  const reclassified = isolateBatchInteractions(cases, verification);
  assert.equal(reclassified[0].execution_class, 'saved-page-batch');
  assert.equal(reclassified[1].execution_class, 'page-preview-isolated');
  assert.deepEqual(reclassified[1].reasons, ['measured-batch-context-interaction']);
});

test('page builder uses deterministic markers and splits before its target', () => {
  const cases = Array.from({length: 3}, (_, index) => ({
    case_id: `case-${index}`,
    source: `alpha-${index}`,
    source_sha256: `${index}`.repeat(64),
    execution_class: 'saved-page-batch',
  }));
  const pages = buildSavedPagePlans(cases, {
    targetCharacters: 180,
    hardCharacters: 220,
    slugPrefix: 'run-owned:fixture',
  });
  assert.ok(pages.length > 1);
  assert.equal(pages.flatMap((value) => value.cases).length, 3);
  assert.ok(pages.every((value) => value.source_characters <= 220));
  assert.match(pages[0].source, /WJDIFF_BEGIN_[0-9a-f]{32}_000001/u);
  assert.equal(pages[0].slug, 'run-owned:fixture-001');
});

test('page builder defaults to the measured safe Wikidot shard size', () => {
  const cases = Array.from({length: 2}, (_, index) => ({
    case_id: `large-${index}`,
    source: 'x'.repeat(5_000),
    source_sha256: `${index}`.repeat(64),
    execution_class: 'saved-page-batch',
  }));
  const pages = buildSavedPagePlans(cases);
  assert.equal(pages.length, 2);
  assert.ok(pages.every((page) => page.source_characters <= 9_000));
});

test('page builder can select runtime cases for saved-page observation', () => {
  const cases = [
    {
      case_id: 'static',
      source: 'plain',
      source_sha256: '1'.repeat(64),
      execution_class: 'saved-page-batch',
    },
    {
      case_id: 'runtime',
      source: '[[include missing]]',
      source_sha256: '2'.repeat(64),
      execution_class: 'wikijump-runtime',
    },
  ];
  const pages = buildSavedPagePlans(cases, {
    executionClass: 'wikijump-runtime',
    slugPrefix: 'run-owned:ftml-diff-20260726',
    targetCharacters: 8_000,
    hardCharacters: 9_000,
  });
  assert.equal(pages.length, 1);
  assert.deepEqual(pages[0].cases.map((value) => value.case_id), ['runtime']);
});

test('page builder keeps isolated runtime cases on singleton pages', () => {
  const cases = [
    {
      case_id: 'safe-before',
      source: '[[module Rate]]',
      source_sha256: '1'.repeat(64),
      execution_class: 'wikijump-runtime',
      page_scope: 'batch-safe',
    },
    {
      case_id: 'isolated',
      source: '[[toc]]\n+ Heading\n[[[target-page]]]',
      source_sha256: '2'.repeat(64),
      execution_class: 'wikijump-runtime',
      page_scope: 'isolated',
    },
    {
      case_id: 'safe-after-one',
      source: '[[module Rate]]',
      source_sha256: '3'.repeat(64),
      execution_class: 'wikijump-runtime',
      page_scope: 'batch-safe',
    },
    {
      case_id: 'safe-after-two',
      source: '[[module Rate]]',
      source_sha256: '4'.repeat(64),
      execution_class: 'wikijump-runtime',
      page_scope: 'batch-safe',
    },
  ];
  const pages = buildSavedPagePlans(cases, {
    executionClass: 'wikijump-runtime',
    slugPrefix: 'run-owned:ftml-diff-20260726',
    targetCharacters: 8_000,
    hardCharacters: 9_000,
  });
  assert.deepEqual(
    pages.map((page) => page.cases.map((value) => value.case_id)),
    [['safe-before'], ['isolated'], ['safe-after-one', 'safe-after-two']],
  );
  assert.equal(pages[1].source, cases[1].source);
  assert.deepEqual(pages[1].cases, [{
    case_id: 'isolated',
    source_sha256: '2'.repeat(64),
    page_scope: 'isolated',
  }]);
  assert.doesNotMatch(pages[1].source, /WJDIFF_/u);
});

test('failed preview retry builder keeps only pages with missing markers', () => {
  const cases = Array.from({length: 3}, (_, index) => ({
    case_id: `case-${index}`,
    source: `alpha-${index}`,
    source_sha256: `${index}`.repeat(64),
    execution_class: 'saved-page-batch',
  }));
  const pages = buildSavedPagePlans(cases, {
    targetCharacters: 180,
    hardCharacters: 220,
    slugPrefix: 'run-owned:initial',
  });
  const captures = pages.map((page, index) => ({
    page_plan: page,
    page_content_html: index === 0
      ? page.cases.flatMap((value) => [value.marker_begin, value.marker_end]).join('\n')
      : '\n\n',
  }));
  const retries = buildFailedPreviewRetryPlans(cases, captures, {
    targetCharacters: 180,
    hardCharacters: 220,
    slugPrefix: 'run-owned:retry',
  });
  assert.deepEqual(
    retries.flatMap((page) => page.cases.map((value) => value.case_id)),
    pages.slice(1).flatMap((page) => page.cases.map((value) => value.case_id)),
  );
});

test('failed saved-page render retries preserve the runtime execution lane', () => {
  const cases = [{
    case_id: 'runtime',
    source: '[[include missing]]',
    source_sha256: '3'.repeat(64),
    execution_class: 'wikijump-runtime',
  }];
  const [page] = buildSavedPagePlans(cases, {
    executionClass: 'wikijump-runtime',
    slugPrefix: 'run-owned:runtime',
    targetCharacters: 8_000,
    hardCharacters: 9_000,
  });
  const retries = buildFailedPreviewRetryPlans(
    cases,
    [{page_plan: page, capture_status: 'render-failed'}],
    {
      executionClass: 'wikijump-runtime',
      slugPrefix: 'run-owned:runtime-retry',
      targetCharacters: 8_000,
      hardCharacters: 9_000,
    },
  );
  assert.equal(retries.length, 1);
  assert.deepEqual(retries[0].cases.map((value) => value.case_id), ['runtime']);

  const resolved = buildFailedPreviewRetryPlans(
    cases,
    [
      {page_plan: page, capture_status: 'render-failed'},
      {
        page_plan: page,
        page_content_html: `${page.cases[0].marker_begin}\n${page.cases[0].marker_end}`,
      },
    ],
    {
      executionClass: 'wikijump-runtime',
      slugPrefix: 'run-owned:runtime-retry',
      targetCharacters: 8_000,
      hardCharacters: 9_000,
    },
  );
  assert.deepEqual(resolved, []);
});

test('marked fragment extraction requires direct ordered paragraph markers', () => {
  const page = {
    cases: [
      {case_id: 'a', marker_begin: 'BEGIN_A', marker_end: 'END_A'},
      {case_id: 'b', marker_begin: 'BEGIN_B', marker_end: 'END_B'},
    ],
  };
  const fragments = extractMarkedFragments(
    '<p>BEGIN_A</p><p><strong>alpha</strong></p><p>END_A</p><p>BEGIN_B</p><p>beta</p><p>END_B</p>',
    page,
  );
  assert.equal(fragments.get('a'), '<p><strong>alpha</strong></p>');
  assert.equal(fragments.get('b'), '<p>beta</p>');
  assert.equal(
    extractMarkedFragments(
      '<div id="page-content"><p>BEGIN_A</p><p>alpha</p><p>END_A</p></div>',
      {cases: [page.cases[0]]},
    ).get('a'),
    '<p>alpha</p>',
  );
  assert.throws(
    () => extractMarkedFragments('<p>BEGIN_A</p><p>BEGIN_A</p><p>END_A</p>', {cases: [page.cases[0]]}),
    /marker integrity failed/u,
  );
});

test('marked fragment extraction identifies the missing sentinel', () => {
  const page = {cases: [{case_id: 'a', marker_begin: 'BEGIN_A', marker_end: 'END_A'}]};
  assert.throws(
    () => extractMarkedFragments('<div id="page-content"><p>END_A</p></div>', page),
    /marker integrity failed for a: missing BEGIN_A/u,
  );
});

test('marked fragment extraction preserves Wikidot output that ejects an end marker from its paragraph', () => {
  const page = {cases: [{case_id: 'list', marker_begin: 'BEGIN_LIST', marker_end: 'END_LIST'}]};
  const fragments = extractMarkedFragments(
    '<div id="page-content"><p>BEGIN_LIST</p><ul><li>alpha</li></ul>END_LIST</div>',
    page,
  );
  assert.equal(fragments.get('list'), '<ul><li>alpha</li></ul>');
});

test('isolated fragment extraction uses the whole page content without sentinels', () => {
  const page = {
    cases: [{
      case_id: 'isolated',
      source_sha256: '1'.repeat(64),
      page_scope: 'isolated',
    }],
  };
  const fragments = extractMarkedFragments(
    '<div id="page-content"><style>body { color: red; }</style><p>visible</p></div>',
    page,
  );
  assert.equal(fragments.get('isolated'), '<style>body { color: red; }</style><p>visible</p>');
});

test('live fragment comparison requires parsed DOM and visible text parity', () => {
  assert.equal(compareFragment('same', '<p>alpha</p>', '<p>alpha</p>').status, 'match');
  assert.equal(compareFragment('different-tag', '<p><b>alpha</b></p>', '<p><strong>alpha</strong></p>').status, 'mismatch');
  assert.equal(compareFragment('different-text', '<p>alpha</p>', '<p>beta</p>').status, 'mismatch');
});
