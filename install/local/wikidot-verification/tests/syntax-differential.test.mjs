import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';

import {
  main as runSyntaxDifferential,
  parseArgs,
  renderCases,
} from '../scripts/run-syntax-differential.mjs';
import {
  canonicalDom,
  compareSyntaxReference,
  ftmlInputFromReference,
  sha256,
  validateWikidotReference,
  visibleText,
} from '../src/syntax-differential.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const fixtureRoot = path.join(here, '..', 'fixtures', 'syntax-differential');

function reference(caseId, source, rawHtml) {
  return {
    schema: 'wikijump_syntax_differential.wikidot_reference.v1',
    syntax_case: {
      schema: 'wikijump_syntax_differential.syntax_case.v1',
      case_id: caseId,
      source,
      wikidot_observation_tier: 'page-preview',
      local_execution_tier: 'ftml',
      title: caseId,
    },
    source_sha256: sha256(source),
    captured_at: '2026-07-26T00:00:00+00:00',
    provenance: {
      site: 'sandbox-for-codex',
      site_domain: 'sandbox-for-codex.wikidot.com',
      module: 'edit/PagePreviewModule',
      wikidot_py_version: '4.4.1',
      wikidot_py_commit: '4af7c8eaec00a3e7a29fe502234e0aeeef968233',
      requirements_sha256: 'ccaa7c0b982723942857abdce48212dcacfd293491d390271e4d7cb5417867fb',
      authenticated: false,
      mutated: false,
    },
    raw_html: rawHtml,
    raw_html_sha256: sha256(rawHtml),
  };
}

function runtimeReference(caseId, source, rawHtml) {
  const value = reference(caseId, source, rawHtml);
  value.syntax_case.local_execution_tier = 'wikijump-runtime';
  return value;
}

function renderResult(caseId, html) {
  return {
    schema: 'wikijump_syntax_differential.ftml_render_result.v1',
    case_id: caseId,
    status: 'rendered',
    html,
    parse_errors: [],
    engine: {layout: 'wikidot'},
  };
}

test('visibleText follows parsed browser text and preserves preformatted whitespace', () => {
  assert.equal(
    visibleText('<p>A  B<style>.x{}</style><script>ignored()</script>&amp; C</p>'),
    'A B& C',
  );
  assert.equal(visibleText('<ul>\n<li>one</li>\n<li>two</li>\n</ul>'), 'one\ntwo');
  assert.equal(visibleText('<dl>\n<dt>term</dt>\n<dd>definition</dd>\n</dl>'), 'term\ndefinition');
  assert.equal(visibleText('<pre>  alpha\n beta  </pre>'), '  alpha\n beta  ');
});

test('syntax comparison requires both DOM signature and visible text parity', () => {
  const live = reference('bold', '**hello**', '<p><strong>hello</strong></p>');
  assert.equal(compareSyntaxReference(live, renderResult('bold', '<p><strong>hello</strong></p>')).status, 'match');

  const textMismatch = compareSyntaxReference(
    live,
    renderResult('bold', '<p><strong>goodbye</strong></p>'),
  );
  assert.equal(textMismatch.status, 'mismatch');
  assert.equal(textMismatch.checks.dom_signature.status, 'match');
  assert.equal(textMismatch.checks.visible_text.status, 'mismatch');

  const structureMismatch = compareSyntaxReference(live, renderResult('bold', '<p><b>hello</b></p>'));
  assert.equal(structureMismatch.status, 'mismatch');
  assert.equal(structureMismatch.checks.dom_signature.status, 'mismatch');
});

test('canonical DOM requires hierarchy, attribute values, and preformatted whitespace', () => {
  assert.notDeepEqual(
    canonicalDom('<strong><em>x</em></strong>'),
    canonicalDom('<em><strong>x</strong></em>'),
  );
  assert.notDeepEqual(
    canonicalDom('<a href="/right">x</a>'),
    canonicalDom('<a href="/wrong">x</a>'),
  );
  assert.notDeepEqual(canonicalDom('<pre>a  b</pre>'), canonicalDom('<pre>a b</pre>'));
  assert.notDeepEqual(canonicalDom('<pre>   </pre>'), canonicalDom('<pre></pre>'));
  assert.notDeepEqual(canonicalDom('<code>\n</code>'), canonicalDom('<code></code>'));
  assert.deepEqual(canonicalDom('<p>a</p>\n<p>b</p>'), canonicalDom('<p>a</p><p>b</p>'));
  assert.deepEqual(
    canonicalDom('<table><tr><td>x</td></tr></table>'),
    canonicalDom('<table><tbody><tr><td>x</td></tr></tbody></table>'),
  );
  assert.deepEqual(
    canonicalDom('<img src="http://sandbox-for-codex.wdfiles.com/local--files//local.png">'),
    canonicalDom('<img src="https://sandbox-for-codex.wjfiles.com/local--files//local.png">'),
  );
  assert.notDeepEqual(
    canonicalDom('<img src="http://sandbox-for-codex.wdfiles.com/local--files/page/one.png">'),
    canonicalDom('<img src="https://sandbox-for-codex.wjfiles.com/local--files/page/two.png">'),
  );
  assert.deepEqual(
    canonicalDom('<a class="bibcite" id="bibcite-2-7926a">2</a>'),
    canonicalDom('<a class="bibcite" id="bibcite-2-12345a">2</a>'),
  );
  assert.notDeepEqual(
    canonicalDom('<a id="bibcite-2-7926a">2</a>'),
    canonicalDom('<a id="bibcite-2-12345a">2</a>'),
  );
});

test('FTML input carries the immutable preview page context', () => {
  assert.deepEqual(ftmlInputFromReference(reference('context', 'alpha', '<p>alpha</p>')).page_context, {
    site: 'sandbox-for-codex',
    page: '',
  });
});

test('checked-in syntax cases exactly match their frozen Wikidot references', async () => {
  const cases = (await fs.readFile(path.join(fixtureRoot, 'preview-cases.jsonl'), 'utf8'))
    .trim()
    .split('\n')
    .map(JSON.parse);
  const references = (await fs.readFile(path.join(fixtureRoot, 'preview-references.jsonl'), 'utf8'))
    .trim()
    .split('\n')
    .map(JSON.parse)
    .map(validateWikidotReference);
  assert.equal(references.length, cases.length);
  assert.deepEqual(
    references.map((value) => value.syntax_case),
    cases,
  );
});

test('syntax references require anonymous immutable acquisition provenance', () => {
  const value = reference('mutated', 'alpha', '<p>alpha</p>');
  value.provenance.mutated = true;
  assert.throws(() => validateWikidotReference(value), /provenance is invalid/u);
  const missingTime = reference('missing-time', 'alpha', '<p>alpha</p>');
  delete missingTime.captured_at;
  assert.throws(() => validateWikidotReference(missingTime), /capture time is invalid/u);
  const impossibleTime = reference('impossible-time', 'alpha', '<p>alpha</p>');
  impossibleTime.captured_at = '2026-02-30T00:00:00Z';
  assert.throws(() => validateWikidotReference(impossibleTime), /capture time is invalid/u);
  const emptySite = reference('empty-site', 'alpha', '<p>alpha</p>');
  emptySite.provenance.site = '';
  assert.throws(() => validateWikidotReference(emptySite), /provenance is invalid/u);
});

test('syntax differential runner streams multiple frozen cases through one renderer process', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'syntax-differential-'));
  const referencesPath = path.join(root, 'references.jsonl');
  const rendererPath = path.join(root, 'renderer.mjs');
  const outputPath = path.join(root, 'verdict.json');
  const references = [
    reference('a', '**A**', '<p><strong>A</strong></p>'),
    reference('b', '**B**', '<p><strong>B</strong></p>'),
    runtimeReference('runtime', '[[[target]]]', '<a class="newpage">target</a>'),
  ];
  await fs.writeFile(referencesPath, `${references.map(JSON.stringify).join('\n')}\n`);
  await fs.writeFile(
    rendererPath,
    [
      "import readline from 'node:readline';",
      "const input = readline.createInterface({input: process.stdin});",
      "for await (const line of input) {",
      "  const value = JSON.parse(line);",
      "  const text = value.source.slice(2, -2);",
      "  console.log(JSON.stringify({schema: 'wikijump_syntax_differential.ftml_render_result.v1', case_id: value.case_id, status: 'rendered', html: `<p><strong>${text}</strong></p>`, parse_errors: [], engine: {layout: 'wikidot'}}));",
      "}",
    ].join('\n'),
  );

  const exitCode = await runSyntaxDifferential([
    '--references', referencesPath,
    '--renderer', process.execPath,
    '--renderer-arg', rendererPath,
    '--output', outputPath,
  ]);
  assert.equal(exitCode, 0);
  const verdict = JSON.parse(await fs.readFile(outputPath, 'utf8'));
  assert.deepEqual(verdict.summary, {
    total: 3,
    match: 2,
    mismatch: 0,
    'runner-error': 0,
    'not-applicable': 1,
  });
});

test('syntax differential CLI requires all evidence paths', () => {
  assert.throws(() => parseArgs([]), /--references is required/u);
  assert.throws(() => parseArgs(['--references', 'r']), /--renderer is required/u);
  assert.throws(
    () => parseArgs(['--references', 'r', '--renderer', 'f']),
    /--output is required/u,
  );
  assert.throws(
    () => parseArgs(['--references', 'r', '--renderer', 'f', '--output', 'o', '--timeout-ms', '0']),
    /--timeout-ms must be a positive integer/u,
  );
});

test('syntax differential runner terminates a renderer that stops responding', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'syntax-differential-timeout-'));
  const rendererPath = path.join(root, 'renderer.mjs');
  await fs.writeFile(rendererPath, 'process.stdin.resume(); setInterval(() => {}, 1000);\n');
  await assert.rejects(
    renderCases(
      [reference('slow', '**slow**', '<p><strong>slow</strong></p>')],
      process.execPath,
      [rendererPath],
      50,
    ),
    /exceeded 50 ms/u,
  );
});

test('syntax differential runner rejects a renderer that exits before returning results', async () => {
  await assert.rejects(
    renderCases(
      [reference('missing', '**missing**', '<p><strong>missing</strong></p>')],
      process.execPath,
      ['-e', 'process.stdin.resume()'],
    ),
    /returned 0 results for 1 cases/u,
  );
});
