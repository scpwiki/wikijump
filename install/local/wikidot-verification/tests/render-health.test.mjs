import { strict as assert } from 'node:assert';
import test from 'node:test';

import {
  aggregateVerdict,
  classifyRenderedPage,
  countEscapedOccurrences,
  renderDashboardHtml,
  stripNonContent,
  RENDER_HEALTH_SCHEMA,
} from '../src/render-health.mjs';

test('clean page classifies S0 pass', () => {
  const page = classifyRenderedPage({
    fixtureId: 'EN:clean',
    httpStatus: 200,
    html: '<html><body><p>Hello world</p></body></html>',
    source: 'Hello world',
  });
  assert.equal(page.status, 'pass');
  assert.equal(page.severity, 'S0');
  assert.deepEqual(page.categories, []);
});

test('leaked collapsible marker is S3 failed', () => {
  const page = classifyRenderedPage({
    fixtureId: 'EN:leak',
    httpStatus: 200,
    html: '<body><p>[[collapsible show="+"]]</p></body>',
    source: '[[collapsible show="+"]]\ncontent\n[[/collapsible]]',
  });
  assert.equal(page.status, 'failed');
  assert.equal(page.severity, 'S3');
  assert.deepEqual(page.categories, ['leaked-marker']);
});

test('unresolved include is its own category', () => {
  const page = classifyRenderedPage({
    fixtureId: 'EN:include',
    httpStatus: 200,
    html: '<body>[[include :scp-wiki:component:foo]]</body>',
    source: '[[include :scp-wiki:component:foo]]',
  });
  assert.deepEqual(page.categories, ['unresolved-include']);
});

test('escaped markers in @@...@@ do not count as leaks', () => {
  const source = 'Use this:\n@@[[include :scp-wiki:theme:x]]@@\n';
  const page = classifyRenderedPage({
    fixtureId: 'JP:escaped',
    httpStatus: 200,
    html: '<body><p>Use this: [[include :scp-wiki:theme:x]]</p></body>',
    source,
  });
  assert.equal(page.status, 'pass');
  assert.deepEqual(page.categories, []);
});

test('escaped count only discounts matching occurrences', () => {
  const source = '@@[[image foo.png]]@@';
  assert.equal(countEscapedOccurrences(source, /\[\[image\b/g), 1);
  assert.equal(countEscapedOccurrences(source, /\[\[include\b/g), 0);
  const page = classifyRenderedPage({
    fixtureId: 'EN:mixed',
    httpStatus: 200,
    html: '<body>[[image foo.png]] [[image bar.png]]</body>',
    source,
  });
  // one escaped + one genuinely leaked
  assert.deepEqual(page.categories, ['leaked-marker']);
  assert.equal(page.findings[0].count, 1);
});

test('code blocks are stripped from content scanning', () => {
  const html = '<body><pre><code>[[module ListPages]]</code></pre><p>fine</p></body>';
  assert.ok(!stripNonContent(html).includes('[[module'));
  const page = classifyRenderedPage({ fixtureId: 'EN:code', httpStatus: 200, html, source: 'x' });
  assert.equal(page.status, 'pass');
});

test('404 is route-missing S4', () => {
  const page = classifyRenderedPage({ fixtureId: 'EN:missing', httpStatus: 404 });
  assert.deepEqual(page.categories, ['route-missing']);
  assert.equal(page.severity, 'S4');
});

test('500 and network errors are local-runtime-error', () => {
  assert.deepEqual(classifyRenderedPage({ fixtureId: 'a', httpStatus: 500 }).categories, [
    'local-runtime-error',
  ]);
  assert.deepEqual(classifyRenderedPage({ fixtureId: 'b', httpStatus: 0 }).categories, [
    'local-runtime-error',
  ]);
});

test('unhandled status becomes taxonomy-unknown and needs review', () => {
  const page = classifyRenderedPage({ fixtureId: 'EN:redirect', httpStatus: 302 });
  assert.deepEqual(page.categories, ['taxonomy-unknown']);
  assert.equal(page.needs_review, true);
});

test('empty render for non-empty source is S4', () => {
  const page = classifyRenderedPage({
    fixtureId: 'EN:empty',
    httpStatus: 200,
    html: '<body>   </body>',
    source: 'lots of source text',
  });
  assert.ok(page.categories.includes('empty-render'));
  assert.equal(page.severity, 'S4');
});

test('failed local--files request maps to missing-local-asset', () => {
  const page = classifyRenderedPage({
    fixtureId: 'EN:asset',
    httpStatus: 200,
    html: '<body>ok</body>',
    source: 'x',
    failedRequests: ['https://x/local--files/page/img.png', 'https://cdn/app.css'],
  });
  assert.deepEqual(page.categories.sort(), ['failed-request', 'missing-local-asset']);
});

test('known-unsupported disposition downgrades to S2 and does not fail', () => {
  const page = classifyRenderedPage({
    fixtureId: 'EN:module',
    httpStatus: 200,
    html: '<body>[[module Rate]]</body>',
    source: '[[module Rate]]',
    dispositions: { 'leaked-marker': 'known-unsupported' },
  });
  assert.equal(page.status, 'unsupported-known');
  assert.equal(page.severity, 'S2');
});

test('aggregate verdict computes health rate and exit codes', () => {
  const pages = [
    classifyRenderedPage({ fixtureId: 'a', httpStatus: 200, html: '<p>ok</p>', source: 'ok' }),
    classifyRenderedPage({ fixtureId: 'b', httpStatus: 404 }),
  ];
  const { verdict, exitCode } = aggregateVerdict({
    runId: 'r1',
    family: 'EN',
    pages,
    threshold: 0.9,
  });
  assert.equal(verdict.schema, RENDER_HEALTH_SCHEMA);
  assert.equal(verdict.aggregate.pages_total, 2);
  assert.equal(verdict.aggregate.pages_healthy, 1);
  assert.equal(verdict.aggregate.health_rate, 0.5);
  assert.equal(verdict.aggregate.category_counts['route-missing'], 1);
  assert.equal(exitCode, 1);
});

test('taxonomy-unknown forces exit 2 even above threshold', () => {
  const pages = [classifyRenderedPage({ fixtureId: 'a', httpStatus: 302 })];
  const { exitCode } = aggregateVerdict({ runId: 'r', family: 'EN', pages, threshold: 0 });
  assert.equal(exitCode, 2);
});

test('exit 0 when healthy above threshold', () => {
  const pages = [
    classifyRenderedPage({ fixtureId: 'a', httpStatus: 200, html: '<p>ok</p>', source: 'ok' }),
  ];
  const { exitCode } = aggregateVerdict({ runId: 'r', family: 'EN', pages, threshold: 0.9 });
  assert.equal(exitCode, 0);
});

test('dashboard html embeds aggregate numbers and escapes content', () => {
  const pages = [
    classifyRenderedPage({
      fixtureId: 'EN:<bad>',
      httpStatus: 200,
      html: '<body>[[div class="x"]]</body>',
      source: 'x',
    }),
  ];
  const { verdict } = aggregateVerdict({ runId: 'r1', family: 'EN', pages });
  const html = renderDashboardHtml({
    verdict,
    importSummary: { imported: 495, total: 500 },
    previous: { aggregate: { health_rate: 0.8 } },
  });
  assert.ok(html.includes('EN:&lt;bad&gt;'));
  assert.ok(html.includes('495/500'));
  assert.ok(html.includes('trend vs previous'));
  assert.ok(html.includes('leaked-marker'));
});
