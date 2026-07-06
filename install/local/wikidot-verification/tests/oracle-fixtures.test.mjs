import { strict as assert } from 'node:assert';
import test from 'node:test';

import {
  aggregateOracleVerdict,
  compareOracleEntry,
  compareSignatures,
  domSignature,
  ORACLE_FIXTURE_SCHEMA,
} from '../src/oracle-fixtures.mjs';

test('domSignature counts tags, classes, attrs, ids, comments', () => {
  const html =
    '<br/>\n<span class="placard"><span style="font-size:75%;"><strong>X</strong></span></span>' +
    '<!-- note --><div id="a" class="x y"></div>';
  const sig = domSignature(html);
  assert.deepEqual(sig.tags, { br: 1, span: 2, strong: 1, div: 1 });
  assert.deepEqual(sig.classes, { placard: 1, x: 1, y: 1 });
  assert.equal(sig.attrs.class, 2);
  assert.equal(sig.attrs.style, 1);
  assert.equal(sig.id_count, 1);
  assert.equal(sig.comment_count, 1);
});

test('domSignature matches the oracle capture format on a real fragment', () => {
  // Fragment shape from oracle entry bold-simple_link-span-ba9765f9ebf11fc4.
  const html =
    '<br/>\n<span class="placard"><span style="font-size:75%;"><strong>DISCLAIMER:</strong> text</span></span><br/>';
  const sig = domSignature(html);
  assert.deepEqual(sig.tags, { br: 2, span: 2, strong: 1 });
  assert.deepEqual(sig.classes, { placard: 1 });
  assert.deepEqual(sig.attrs, { class: 1, style: 1 });
  assert.equal(sig.id_count, 0);
  assert.equal(sig.comment_count, 0);
});

test('compareSignatures reports count mismatches both directions', () => {
  const diffs = compareSignatures(
    { tags: { p: 2 }, classes: {}, attrs: {}, id_count: 0, comment_count: 0 },
    { tags: { p: 1, div: 1 }, classes: {}, attrs: {}, id_count: 1, comment_count: 0 },
  );
  assert.deepEqual(
    diffs.map((d) => `${d.kind}:${d.key ?? ''}`),
    ['tag:p', 'tag:div', 'id_count:'],
  );
});

test('compareOracleEntry pass and fail', () => {
  const entry = {
    oracle_entry_id: 'x',
    constructs: ['bold'],
    rendered: {
      raw_extracted_html: '<br/>\n<strong>hi</strong><br/>',
      dom_signature: {
        tags: { br: 2, strong: 1 },
        classes: {},
        attrs: {},
        id_count: 0,
        comment_count: 0,
      },
    },
  };
  // Local render wraps the snippet in a paragraph and has no sentinel <br>s;
  // both are declared harness normalizations.
  const pass = compareOracleEntry(entry, '<p><strong>hi</strong></p>');
  assert.equal(pass.status, 'pass');
  assert.deepEqual(pass.normalization, ['boundary_br', 'paragraph_unwrap']);
  const fail = compareOracleEntry(entry, '<b>hi</b>');
  assert.equal(fail.status, 'fail');
  assert.ok(fail.diffs.length > 0);
});

test('integrity check skips entries whose stored signature mismatches raw html', () => {
  const entry = {
    oracle_entry_id: 'z',
    rendered: {
      raw_extracted_html: '<em>x</em>',
      dom_signature: { tags: { strong: 1 }, classes: {}, attrs: {}, id_count: 0, comment_count: 0 },
    },
  };
  const result = compareOracleEntry(entry, '<em>x</em>');
  assert.equal(result.status, 'skipped');
  assert.match(result.reason, /tokenizer disagrees/);
});

test('entry without signature is skipped and forces exit 2', () => {
  const result = compareOracleEntry({ oracle_entry_id: 'y', rendered: {} }, '<p></p>');
  assert.equal(result.status, 'skipped');
  const { verdict, exitCode } = aggregateOracleVerdict({ runId: 'r', results: [result] });
  assert.equal(verdict.schema, ORACLE_FIXTURE_SCHEMA);
  assert.equal(exitCode, 2);
});

test('aggregate exit codes', () => {
  const pass = { oracle_entry_id: 'a', status: 'pass', diffs: [] };
  const fail = { oracle_entry_id: 'b', status: 'fail', diffs: [{}] };
  assert.equal(aggregateOracleVerdict({ runId: 'r', results: [pass] }).exitCode, 0);
  const agg = aggregateOracleVerdict({ runId: 'r', results: [pass, fail] });
  assert.equal(agg.exitCode, 1);
  assert.deepEqual(agg.verdict.aggregate.failing, ['b']);
});
