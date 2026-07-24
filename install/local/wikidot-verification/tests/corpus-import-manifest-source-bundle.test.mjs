import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {fileURLToPath} from "node:url";
import test from "node:test";

import {
  buildCorpusImportManifest,
  buildManifestSummary,
  formatJsonl,
} from "../src/corpus-import-manifest.mjs";
import {
  cryptoSha256,
  writeSourceBundlePage,
} from "./support/corpus-import-manifest-fixture.mjs";

test('buildCorpusImportManifest accepts source bundles without entity IDs', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', {
    source: 'sandbox start',
    meta: {
      children_count: '2',
      comments_count: 3,
      revisions_count: 4,
      rating: -1,
      tags: ['beta', 'alpha'],
      title: 'Start',
      title_shown: 'Start',
    },
  });

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });
  const rowsAgain = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });

  assert.equal(rows.length, 1);
  assert.equal(rows[0].source_site, 'sandbox-for-codex');
  assert.equal(rows[0].source_branch, 'SANDBOX');
  assert.match(rows[0].source_entity_id, /^[0-9a-f]{8}-[0-9a-f]{4}-5[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  assert.equal(rows[0].source_entity_id, rowsAgain[0].source_entity_id);
  assert.equal(rows[0].children, 2);
  assert.equal(rows[0].comments, 3);
  assert.equal(rows[0].revisions, 4);
  assert.equal(rows[0].rating, -1);
  assert.equal(rows[0].created_at, '1970-01-01T00:00:00+00:00');
  assert.equal(rows[0].updated_at, '1970-01-01T00:00:00+00:00');
  assert.deepEqual(rows[0].tags, ['alpha', 'beta']);
  assert.equal(rows[0].entity_id_path, null);
  assert.equal(rows[0].source_path, path.join(root, 'pages', 'start', 'source.wikidot.txt'));
  assert.equal(rows[0].meta_path, path.join(root, 'pages', 'start', 'meta.json'));
  assert.equal(rows[0].source_capture_method, 'wikidot_xmlrpc_pages.get_one');
  assert.equal(rows[0].source_browser_visibility, 'source_only');
  assert.equal(rows[0].source_visibility_reason, 'missing_browser_visibility_proof');
  assert.equal(rows[0].required_browser, false);
  assert.equal(rows[0].source_required_actor, null);
});

test('buildCorpusImportManifest accepts CRLF source bundle manifests', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  const source = 'sandbox start';
  writeSourceBundlePage(root, 'start', { source });
  fs.writeFileSync(
    path.join(root, 'corpus-manifest.tsv'),
    [
      'site\tfullname\tsource_bytes\tsource_sha256',
      `sandbox-for-codex\tstart\t${Buffer.byteLength(source)}\t${cryptoSha256(source)}`,
      '',
    ].join('\r\n'),
  );

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    sourceSite: '',
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });

  assert.equal(rows.length, 1);
  assert.equal(rows[0].fullname, 'start');
  assert.equal(rows[0].source_site, 'sandbox-for-codex');
});

test('buildCorpusImportManifest falls back when source bundle manifest site is blank', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', { site: '' });

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });

  assert.equal(rows.length, 1);
  assert.equal(rows[0].source_site, 'SANDBOX');
});

test('buildCorpusImportManifest keeps source bundle browser-visible rows browser-required', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', {
    manifest: {
      browser_visibility: 'browser_visible',
      source_browser_status: '200',
    },
  });

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });
  const summary = buildManifestSummary(rows, formatJsonl(rows));

  assert.equal(rows[0].source_browser_visibility, 'browser_visible');
  assert.equal(rows[0].source_browser_status, 200);
  assert.equal(rows[0].source_visibility_reason, 'declared_source_browser_visibility');
  assert.equal(rows[0].required_browser, true);
  assert.equal(summary.required_browser_count, 1);
  assert.deepEqual(summary.source_browser_visibility_counts, { browser_visible: 1 });
});

test('buildCorpusImportManifest requires actor metadata for actor-required source bundle rows', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'member-only', {
    manifest: {
      browser_visibility: 'actor_required',
      required_actor: 'account_a',
    },
  });

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });
  const summary = buildManifestSummary(rows, formatJsonl(rows));

  assert.equal(rows[0].source_browser_visibility, 'actor_required');
  assert.equal(rows[0].source_required_actor, 'account_a');
  assert.equal(rows[0].required_browser, true);
  assert.equal(summary.source_required_actor_count, 1);
  assert.deepEqual(summary.source_browser_visibility_counts, { actor_required: 1 });
});

test('buildCorpusImportManifest uses non-empty source browser aliases', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'member-only', {
    manifest: {
      browser_visibility: '',
      source_browser_visibility: 'actor_required',
      required_actor: '',
      source_required_actor: 'account_b',
      browser_status: '',
      source_browser_status: '200',
      browser_visibility_reason: '',
      source_visibility_reason: 'authenticated source proof',
    },
  });

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });

  assert.equal(rows[0].source_browser_visibility, 'actor_required');
  assert.equal(rows[0].source_required_actor, 'account_b');
  assert.equal(rows[0].source_browser_status, 200);
  assert.equal(rows[0].source_visibility_reason, 'authenticated source proof');
  assert.equal(rows[0].required_browser, true);
});

test('buildCorpusImportManifest fails closed on invalid source bundle browser metadata', () => {
  const missingActorRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(missingActorRoot, 'member-only', {
    manifest: {
      browser_visibility: 'actor_required',
    },
  });
  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: missingActorRoot, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /actor_required source browser visibility requires required_actor/,
  );

  const invalidVisibilityRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(invalidVisibilityRoot, 'start', {
    manifest: {
      browser_visibility: 'maybe',
    },
  });
  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: invalidVisibilityRoot, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /source browser visibility must be one of/,
  );
});

test('buildCorpusImportManifest treats zero-revision source bundle rows without browser proof as source-only', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'xmlrpc-only', {
    meta: {
      revisions_count: 0,
    },
  });

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'JP_TEST',
    sourceBranch: 'JP_TEST',
  });

  assert.equal(rows[0].source_browser_visibility, 'source_only');
  assert.equal(rows[0].source_visibility_reason, 'zero_revisions_without_browser_visibility_proof');
  assert.equal(rows[0].required_browser, false);
});

test('buildCorpusImportManifest does not treat source bundle vote count as rating', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', {
    meta: {
      rating: undefined,
      votes_count: 120,
    },
  });

  const rows = buildCorpusImportManifest({
    sourceBundleRoot: root,
    branch: 'SANDBOX',
    sourceBranch: 'SANDBOX',
  });

  assert.equal(rows[0].rating, 0);
});

test('buildCorpusImportManifest rejects invalid UTF-8 source bundle files', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start');
  fs.writeFileSync(path.join(root, 'pages', 'start', 'source.wikidot.txt'), Buffer.from([0xff]));

  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: root, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /invalid UTF-8/,
  );
});

test('buildCorpusImportManifest rejects negative source bundle counts', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', { meta: { comments_count: -1 } });

  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: root, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /meta\.comments_count must be a non-negative integer/,
  );
});

test('buildCorpusImportManifest validates source bundle string metadata', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', { meta: { created_by: 42 } });

  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: root, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /meta\.created_by must be null or a string/,
  );
});

test('buildCorpusImportManifest rejects mismatched source bundle fullnames', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', { meta: { fullname: 'other' } });

  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: root, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /meta\.fullname other does not match directory name start/,
  );
});

test('buildCorpusImportManifest requires source bundle TSV rows when a TSV exists', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start');
  fs.writeFileSync(
    path.join(root, 'corpus-manifest.tsv'),
    'site\tfullname\ttitle\ttags\tsource_path\tsource_bytes\tsource_sha256\tmeta_path\tcapture_method\n',
  );

  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: root, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /missing row in corpus-manifest\.tsv for start/,
  );
});

test('buildCorpusImportManifest rejects source bundle TSV rows without page directories', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start');
  fs.appendFileSync(
    path.join(root, 'corpus-manifest.tsv'),
    [
      'sandbox-for-codex',
      'missing-page',
      'Missing Page',
      'codex',
      path.join(root, 'pages', 'missing-page', 'source.wikidot.txt'),
      '7',
      cryptoSha256('missing'),
      path.join(root, 'pages', 'missing-page', 'meta.json'),
      'wikidot_xmlrpc_pages.get_one',
    ].join('\t') + '\n',
  );

  assert.throws(
    () => buildCorpusImportManifest({ sourceBundleRoot: root, branch: 'SANDBOX', sourceBranch: 'SANDBOX' }),
    /corpus-manifest\.tsv rows have no matching page directories: missing-page/,
  );
});

test('build-corpus-import-manifest CLI accepts source bundles', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'source-bundle-'));
  writeSourceBundlePage(root, 'start', { source: 'sandbox start' });
  const output = path.join(root, 'manifest.jsonl');
  const summary = path.join(root, 'summary.json');

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/build-corpus-import-manifest.mjs'),
    '--source-bundle',
    root,
    '--output',
    output,
    '--summary',
    summary,
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const rows = fs.readFileSync(output, 'utf8').trim().split('\n').map((line) => JSON.parse(line));
  const summaryJson = JSON.parse(fs.readFileSync(summary, 'utf8'));
  assert.equal(rows.length, 1);
  assert.equal(rows[0].source_site, 'sandbox-for-codex');
  assert.equal(rows[0].source_branch, 'source-bundle');
  assert.equal(summaryJson.row_count, 1);
});
