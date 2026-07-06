import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  buildBundleRegistry,
  isThemeOrComponentSlug,
  parseIncludeTarget,
  resolveDependencyClosure,
  scanWikitextDependencies,
  stripEscapedRegions,
  summarizeClosureReports,
} from '../src/dependency-closure.mjs';

function row(site, slug, extra = {}) {
  return { local_site: site, slug, fixture_id: `${site.toUpperCase()}:${slug}`, ...extra };
}

function makeReadSource(sources) {
  return (pageRow) => sources[`${pageRow.local_site}:${pageRow.slug}`] ?? null;
}

test('parseIncludeTarget splits cross-site targets on the leading colon only', () => {
  assert.deepEqual(parseIncludeTarget('scp-wiki:component:license-box', true), {
    site: 'scp-wiki',
    page: 'component:license-box',
  });
  assert.deepEqual(parseIncludeTarget('component:license-box', false), {
    site: null,
    page: 'component:license-box',
  });
});

test('scanWikitextDependencies finds includes, css modules, data modules and local files', () => {
  const scan = scanWikitextDependencies(
    '[[include :scp-wiki:theme:blankstyle]]\n' +
      '[[include component:info-start |lang=en]]\n' +
      '[[module CSS]]\nbody {}\n[[/module]]\n' +
      '[[module ListPages category="*"]]\n' +
      '<img src="/local--files/scp-002/photo.jpg">',
  );
  assert.equal(scan.includes.length, 2);
  assert.deepEqual(scan.includes[0].site, 'scp-wiki');
  assert.equal(scan.includes[1].page, 'component:info-start');
  assert.equal(scan.cssModuleCount, 1);
  assert.deepEqual(scan.dataModules, ['listpages']);
  assert.deepEqual(scan.localFiles, [{ page: 'scp-002', file: 'photo.jpg' }]);
});

test('escaped regions do not contribute dependencies', () => {
  const scan = scanWikitextDependencies(
    '@@[[include :scp-wiki:component:escaped]]@@\n' +
      '[[code]]\n[[include :scp-wiki:component:in-code]]\n[[/code]]\n' +
      '[!--\nusage:\n[[include :scp-wiki:component:in-comment]]\n--]\n' +
      '[[include component:real]]',
  );
  assert.equal(scan.includes.length, 1);
  assert.equal(scan.includes[0].page, 'component:real');
  assert.equal(stripEscapedRegions('a@@b@@c'), 'ac');
  // @@ spans are inline-only: a stray @@ must not swallow following lines.
  assert.ok(stripEscapedRegions('x@@y\n[[include a]]\nz').includes('[[include a]]'));
});

test('isThemeOrComponentSlug', () => {
  assert.ok(isThemeOrComponentSlug('theme:blankstyle'));
  assert.ok(isThemeOrComponentSlug('component:license-box'));
  assert.ok(!isThemeOrComponentSlug('scp-173'));
});

test('closure_complete with transitive include and import order', () => {
  const rows = [
    row('scp-wiki', 'scp-002', { parent_fullname: 'hub' }),
    row('scp-wiki', 'hub'),
    row('scp-wiki', 'component:license-box'),
    row('scp-wiki', 'theme:blankstyle'),
  ];
  const registry = buildBundleRegistry(rows);
  const readSource = makeReadSource({
    'scp-wiki:scp-002':
      '[[include component:license-box]]\n[[include :scp-wiki:theme:blankstyle]]',
    'scp-wiki:hub': 'plain',
    'scp-wiki:component:license-box': 'plain',
    'scp-wiki:theme:blankstyle': 'plain',
  });
  const report = resolveDependencyClosure({ row: rows[0], registry, readSource });
  assert.equal(report.status, 'closure_complete');
  assert.deepEqual(report.dependencies.out_of_bundle, []);
  assert.equal(report.import_order.at(-1), 'SCP-WIKI:scp-002');
  assert.ok(
    report.import_order.indexOf('SCP-WIKI:hub') <
      report.import_order.indexOf('SCP-WIKI:component:license-box'),
  );
});

test('out-of-bundle include fails closed and is recorded', () => {
  const rows = [row('scp-jp', 'page-a')];
  const registry = buildBundleRegistry(rows);
  const readSource = makeReadSource({
    'scp-jp:page-a': '[[include :scp-wiki:component:not-bundled]]',
  });
  const report = resolveDependencyClosure({ row: rows[0], registry, readSource });
  assert.equal(report.status, 'out_of_bundle');
  assert.deepEqual(report.dependencies.out_of_bundle, [
    { dependency: 'scp-wiki:component:not-bundled', kind: 'theme-component' },
  ]);
});

test('include cycles are detected and reported, not skipped silently', () => {
  const rows = [row('scp-wiki', 'a'), row('scp-wiki', 'b')];
  const registry = buildBundleRegistry(rows);
  const readSource = makeReadSource({
    'scp-wiki:a': '[[include b]]',
    'scp-wiki:b': '[[include a]]',
  });
  const report = resolveDependencyClosure({ row: rows[0], registry, readSource });
  assert.equal(report.status, 'cycle');
  assert.equal(report.dependencies.cycles.length, 1);
});

test('pages that only include themselves classify as self_include_cycle', () => {
  const rows = [row('scp-wiki', 'component:theme-squares')];
  const registry = buildBundleRegistry(rows);
  const readSource = makeReadSource({
    'scp-wiki:component:theme-squares': '[[include :scp-wiki:component:theme-squares]]',
  });
  const report = resolveDependencyClosure({ row: rows[0], registry, readSource });
  assert.equal(report.status, 'self_include_cycle');
});

test('registry keeps the site-native row on key collisions and records them', () => {
  const enRow = row('scp-wiki', 'component:license-box', { family: 'EN' });
  const jpRow = { ...row('scp-wiki', 'component:license-box', { family: 'JP' }), fixture_id: 'JP:component:license-box' };
  for (const order of [[enRow, jpRow], [jpRow, enRow]]) {
    const registry = buildBundleRegistry(order);
    assert.equal(registry.get('scp-wiki:component:license-box').family, 'EN');
    assert.equal(registry.collisions.length, 1);
    assert.equal(registry.collisions[0].kept, 'SCP-WIKI:component:license-box');
    assert.equal(registry.collisions[0].dropped, 'JP:component:license-box');
  }
});

test('summarizeClosureReports exit codes', () => {
  const classified = {
    status: 'out_of_bundle',
    dependencies: { out_of_bundle: [{ dependency: 'x', kind: 'include' }] },
  };
  const unclassified = {
    status: 'out_of_bundle',
    dependencies: { out_of_bundle: [{ dependency: 'y', kind: 'mystery' }] },
  };
  assert.equal(summarizeClosureReports([classified]).exit_code, 0);
  const summary = summarizeClosureReports([classified, unclassified]);
  assert.equal(summary.exit_code, 1);
  assert.equal(summary.unclassified_out_of_bundle, 1);
});
