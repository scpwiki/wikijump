import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

import {
  buildCorpusImportManifest,
  buildManifestSummary,
  formatJsonl,
} from '../src/corpus-import-manifest.mjs';
import { assertEmptyDbImportTarget } from '../src/corpus-import-empty-target.mjs';

function writePage(root, branch, fullname, { entityId, meta = {}, source = 'content' } = {}) {
  const pageDir = path.join(root, branch, 'pages', fullname);
  fs.mkdirSync(pageDir, { recursive: true });
  const completeMeta = {
    children: 0,
    commented_at: null,
    commented_by: null,
    comments: 0,
    created_at: '2008-07-25T20:49:21+00:00',
    created_by: 'Lt Masipag',
    fullname,
    parent_fullname: null,
    parent_title: null,
    rating: 10634,
    revisions: 57,
    tags: ['scp', 'euclid'],
    title: 'SCP-173',
    title_shown: 'SCP-173',
    updated_at: '2025-04-02T12:17:27+00:00',
    updated_by: 'ParallelPotatoes',
    ...meta,
  };
  fs.writeFileSync(path.join(pageDir, 'source.wikidot.txt'), source);
  fs.writeFileSync(path.join(pageDir, 'meta.json'), `${JSON.stringify(completeMeta, null, 2)}\n`);
  fs.writeFileSync(path.join(pageDir, 'entity_id.txt'), `${entityId}\n`);
}

function writePageAttachment(root, branch, fullname, { filename, bytes, originalUrl } = {}) {
  const pageDir = path.join(root, branch, 'pages', fullname);
  const filesDir = path.join(pageDir, 'files');
  const filePath = path.join(filesDir, filename);
  fs.mkdirSync(filesDir, { recursive: true });
  fs.writeFileSync(filePath, bytes);
  fs.writeFileSync(
    path.join(pageDir, 'files.json'),
    `${JSON.stringify([
      {
        filename,
        original_url: originalUrl,
        wikidot_path: new URL(originalUrl).pathname,
        path: `files/${filename}`,
        sha256: cryptoSha256(bytes),
        mime: 'image/png',
        size: bytes.length,
      },
    ], null, 2)}\n`,
  );
}

function writeSourceBundlePage(root, fullname, { entityId = null, site = 'sandbox-for-codex', meta = {}, manifest = {}, source = 'content' } = {}) {
  const pageDir = path.join(root, 'pages', fullname);
  fs.mkdirSync(pageDir, { recursive: true });
  const sourceBytes = Buffer.byteLength(source);
  const sourceSha256 = cryptoSha256(source);
  const completeMeta = {
    capture_method: 'wikidot_xmlrpc_pages.get_one',
    category: '_default',
    children_count: '0',
    comments_count: 0,
    fullname,
    name: fullname,
    parent_fullname: null,
    rating: 0,
    revisions_count: 1,
    size: sourceBytes,
    source_bytes: sourceBytes,
    source_sha256: sourceSha256,
    tags: ['codex'],
    title: fullname,
    title_shown: fullname,
    votes_count: 0,
    xmlrpc_fullname: fullname,
    ...meta,
  };
  fs.writeFileSync(path.join(pageDir, 'source.wikidot.txt'), source);
  fs.writeFileSync(path.join(pageDir, 'meta.json'), `${JSON.stringify(completeMeta, null, 2)}\n`);
  if (entityId !== null) fs.writeFileSync(path.join(pageDir, 'entity_id.txt'), `${entityId}\n`);
  const manifestPath = path.join(root, 'corpus-manifest.tsv');
  const baseHeaders = ['site', 'fullname', 'title', 'tags', 'source_path', 'source_bytes', 'source_sha256', 'meta_path', 'capture_method'];
  const manifestHeaders = [...baseHeaders, ...Object.keys(manifest)];
  if (!fs.existsSync(manifestPath)) {
    fs.writeFileSync(manifestPath, `${manifestHeaders.join('\t')}\n`);
  }
  const baseColumns = {
    site,
    fullname,
    title: completeMeta.title,
    tags: completeMeta.tags.join('|'),
    source_path: path.join(pageDir, 'source.wikidot.txt'),
    source_bytes: String(sourceBytes),
    source_sha256: sourceSha256,
    meta_path: path.join(pageDir, 'meta.json'),
    capture_method: completeMeta.capture_method,
  };
  fs.appendFileSync(manifestPath, `${manifestHeaders.map((header) => baseColumns[header] ?? manifest[header] ?? '').join('\t')}\n`);
}

function cryptoSha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

test('buildCorpusImportManifest emits deterministic rows and summary', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '11111111-1111-4111-8111-111111111111',
    source: '[[module Rate]]\nSCP-173',
  });
  writePage(root, 'en', 'component:license-box', {
    entityId: '22222222-2222-4222-8222-222222222222',
    meta: {
      fullname: 'component:license-box',
      title: 'License Box',
      title_shown: 'License Box',
      parent_fullname: 'scp-173',
      rating: 0,
      revisions: 3,
      tags: ['component'],
    },
    source: 'license component',
  });

  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const jsonl = formatJsonl(rows);
  const summary = buildManifestSummary(rows, jsonl);

  assert.equal(rows.length, 2);
  assert.deepEqual(rows.map((row) => row.fullname), ['component:license-box', 'scp-173']);
  assert.equal(rows[1].source_site, 'scp-wiki');
  assert.equal(rows[1].rating, 10634);
  assert.deepEqual(rows[1].tags, ['euclid', 'scp']);
  assert.match(rows[1].source_sha256, /^[0-9a-f]{64}$/);
  assert.equal(summary.row_count, 2);
  assert.equal(summary.parent_count, 1);
  assert.match(summary.manifest_sha256, /^[0-9a-f]{64}$/);
  assert.equal(formatJsonl(rows), jsonl, 'formatting should be deterministic');
});

test('buildCorpusImportManifest includes validated per-page corpus attachments', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  const bytes = Buffer.from([1, 2, 3, 4]);
  writePage(root, 'en', 'scp-173', {
    entityId: '12121212-1212-4121-8121-121212121212',
    source: '[[image https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png]]',
  });
  writePageAttachment(root, 'en', 'scp-173', {
    filename: 'pixel.png',
    bytes,
    originalUrl: 'https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png',
  });

  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const jsonl = formatJsonl(rows);
  const summary = buildManifestSummary(rows, jsonl);

  assert.equal(rows.length, 1);
  assert.equal(rows[0].attachments.length, 1);
  assert.deepEqual(rows[0].attachments[0], {
    filename: 'pixel.png',
    original_url: 'https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png',
    wikidot_path: '/local--files/scp-173/pixel.png',
    sha256: cryptoSha256(bytes),
    size: 4,
    mime: 'image/png',
    file_path: path.join(root, 'en', 'pages', 'scp-173', 'files', 'pixel.png'),
    corpus_path: 'en/pages/scp-173/files/pixel.png',
    metadata_path: path.join(root, 'en', 'pages', 'scp-173', 'files.json'),
  });
  assert.equal(summary.attachment_count, 1);
  assert.equal(summary.attachment_page_count, 1);
});

test('buildCorpusImportManifest reads files/_state.json capture-state attachment manifests', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  const bytes = Buffer.from([5, 6, 7, 8, 9]);
  writePage(root, 'en', 'theme:pataphysics', {
    entityId: '14141414-1414-4141-8141-141414141414',
  });
  const pageDir = path.join(root, 'en', 'pages', 'theme:pataphysics');
  const filesDir = path.join(pageDir, 'files');
  fs.mkdirSync(filesDir, { recursive: true });
  fs.writeFileSync(path.join(filesDir, 'pata-logo.png'), bytes);
  fs.writeFileSync(
    path.join(filesDir, '_state.json'),
    `${JSON.stringify({
      files: {
        'pata-logo.png': {
          download_url: 'http://scp-wiki.wdfiles.com/local--files/theme%3Apataphysics/pata-logo.png',
          mime_type: 'image/png',
          sha256: `sha256:${cryptoSha256(bytes)}`,
          size: bytes.length,
          uploaded_at: '2020-01-01T00:00:00+00:00',
        },
      },
    }, null, 2)}\n`,
  );

  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });

  assert.equal(rows.length, 1);
  assert.equal(rows[0].attachments.length, 1);
  assert.deepEqual(rows[0].attachments[0], {
    filename: 'pata-logo.png',
    original_url: 'http://scp-wiki.wdfiles.com/local--files/theme%3Apataphysics/pata-logo.png',
    wikidot_path: '/local--files/theme%3Apataphysics/pata-logo.png',
    sha256: cryptoSha256(bytes),
    size: bytes.length,
    mime: 'image/png',
    file_path: path.join(filesDir, 'pata-logo.png'),
    corpus_path: 'en/pages/theme:pataphysics/files/pata-logo.png',
    metadata_path: path.join(filesDir, '_state.json'),
  });
});

test('files.json takes precedence over files/_state.json when both exist', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  const bytes = Buffer.from([1, 2, 3, 4]);
  writePage(root, 'en', 'scp-173', {
    entityId: '15151515-1515-4151-8151-151515151515',
  });
  writePageAttachment(root, 'en', 'scp-173', {
    filename: 'pixel.png',
    bytes,
    originalUrl: 'https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png',
  });
  fs.writeFileSync(
    path.join(root, 'en', 'pages', 'scp-173', 'files', '_state.json'),
    JSON.stringify({ files: { 'other.png': { download_url: 'not-a-url' } } }),
  );

  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  assert.equal(rows[0].attachments.length, 1);
  assert.equal(rows[0].attachments[0].filename, 'pixel.png');
});

test('buildCorpusImportManifest rejects attachment byte hash mismatches', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '13131313-1313-4131-8131-131313131313',
  });
  writePageAttachment(root, 'en', 'scp-173', {
    filename: 'pixel.png',
    bytes: Buffer.from([1]),
    originalUrl: 'https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png',
  });
  const manifestPath = path.join(root, 'en', 'pages', 'scp-173', 'files.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  manifest[0].sha256 = '0'.repeat(64);
  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

  assert.throws(
    () => buildCorpusImportManifest({ corpusRoot: root, branch: 'en' }),
    /sha256 mismatch/,
  );
});

test('buildCorpusImportManifest rejects duplicate entity IDs before import', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  const entityId = '33333333-3333-4333-8333-333333333333';
  writePage(root, 'en', 'scp-173', { entityId });
  writePage(root, 'en', 'scp-174', {
    entityId,
    meta: {
      fullname: 'scp-174',
      title: 'SCP-174',
      title_shown: 'SCP-174',
    },
  });

  assert.throws(
    () => buildCorpusImportManifest({ corpusRoot: root, branch: 'en' }),
    /duplicate source_entity_id/,
  );
});

test('buildCorpusImportManifest rejects incomplete page records', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  const pageDir = path.join(root, 'en', 'pages', 'scp-173');
  fs.mkdirSync(pageDir, { recursive: true });
  fs.writeFileSync(path.join(pageDir, 'source.wikidot.txt'), 'content');
  fs.writeFileSync(path.join(pageDir, 'meta.json'), '{}\n');

  assert.throws(
    () => buildCorpusImportManifest({ corpusRoot: root, branch: 'en' }),
    /missing entity_id.txt/,
  );
});

test('buildCorpusImportManifest uses locale-independent code point ordering', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  writePage(root, 'en', '_404', {
    entityId: '77777777-7777-4777-8777-777777777777',
    meta: { fullname: '_404', title: '404', title_shown: '404' },
  });
  writePage(root, 'en', '0-texts-found', {
    entityId: '88888888-8888-4888-8888-888888888888',
    meta: { fullname: '0-texts-found', title: '0 Texts Found', title_shown: '0 Texts Found' },
  });

  const rows = buildCorpusImportManifest({ corpusRoot: root, branch: 'en' });

  assert.deepEqual(rows.map((row) => row.fullname), ['0-texts-found', '_404']);
});

test('buildCorpusImportManifest rejects negative counts before apply', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '99999999-9999-4999-8999-999999999999',
    meta: { comments: -1 },
  });

  assert.throws(
    () => buildCorpusImportManifest({ corpusRoot: root, branch: 'en' }),
    /meta\.comments must be a non-negative integer/,
  );
});

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

test('apply-corpus-import-manifest rejects DB create mode without a text hash command', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '66666666-6666-4666-8666-666666666666',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--slug',
    'scp-173',
    '--create-mode',
    'db',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    env: { ...process.env, DEEPWELL_TEXT_HASH_COMMAND: '' },
    maxBuffer: 1024 * 1024,
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /text-hash-command|DEEPWELL_TEXT_HASH_COMMAND/);
});

test('apply-corpus-import-manifest accepts secrets only through environment variables', async () => {
  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const script = path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs');
  const secret = 'desloppify-secret-must-not-be-echoed';

  for (const option of ['--session-token', '--attachment-s3-secret-access-key']) {
    const result = spawnSync(process.execPath, [script, option, secret], {
      cwd: packageRoot,
      encoding: 'utf8',
      maxBuffer: 1024 * 1024,
    });

    assert.notEqual(result.status, 0);
    assert.match(result.stderr, new RegExp(`unknown argument: ${option}`));
    assert.doesNotMatch(result.stderr, new RegExp(secret));
  }
});

test('apply-corpus-import-manifest help contains no embedded credential or secret argument', async () => {
  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--help',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /--db-url <postgres-url>/);
  assert.doesNotMatch(result.stdout, /postgres:\/\/wikijump:wikijump@/);
  assert.doesNotMatch(result.stdout, /--session-token|--attachment-s3-secret-access-key/);
});

test('apply-corpus-import-manifest dry-run filters by slug without touching services', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '44444444-4444-4444-8444-444444444444',
    source: 'SCP-173',
  });
  writePage(root, 'en', 'scp-174', {
    entityId: '55555555-5555-4555-8555-555555555555',
    meta: {
      fullname: 'scp-174',
      title: 'SCP-174',
      title_shown: 'SCP-174',
    },
    source: 'SCP-174',
  });

  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--slug',
    'scp-173',
    '--dry-run',
    '--create-mode',
    'db',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const output = JSON.parse(result.stdout);
  assert.deepEqual(output, {
    dry_run: true,
    selected_rows: 1,
    complete_inventory: false,
  });
});

test('apply-corpus-import-manifest accepts opt-in DB rerender dry-run', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '77777777-7777-4777-8777-777777777777',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--slug',
    'scp-173',
    '--dry-run',
    '--create-mode',
    'db',
    '--rerender-after-db-create',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const output = JSON.parse(result.stdout);
  assert.deepEqual(output, {
    dry_run: true,
    selected_rows: 1,
    complete_inventory: false,
  });
});

test('apply-corpus-import-manifest accepts empty-DB assumption for DB dry-runs', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '99999999-9999-4999-8999-999999999999',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--slug',
    'scp-173',
    '--dry-run',
    '--create-mode',
    'db',
    '--assume-empty-db-import',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const output = JSON.parse(result.stdout);
  assert.deepEqual(output, {
    dry_run: true,
    selected_rows: 1,
    complete_inventory: false,
  });
});

test('apply-corpus-import-manifest rejects unsafe empty-DB assumption combinations', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '99999999-9999-4999-8999-999999999998',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const rpcMode = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--dry-run',
    '--assume-empty-db-import',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });
  const adoptMode = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--dry-run',
    '--create-mode',
    'db',
    '--assume-empty-db-import',
    '--adopt-existing',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.notEqual(rpcMode.status, 0);
  assert.match(rpcMode.stderr, /assume-empty-db-import requires --create-mode db/);
  assert.notEqual(adoptMode.status, 0);
  assert.match(adoptMode.stderr, /assume-empty-db-import cannot be combined with --adopt-existing or --replace-existing/);
});

test('apply-corpus-import-manifest documents disabled empty-DB writes and dry-run planning', async () => {
  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--help',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Non-dry-run --assume-empty-db-import is disabled/);
  assert.match(result.stdout, /dry-run accepts the flag for planning without probing or changing the target/);
  assert.doesNotMatch(result.stdout, /database uniqueness fail closed/);
});

test('apply-corpus-import-manifest rejects the racy empty-DB fast path before side effects', async () => {
  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest', path.join(packageRoot, 'package.json'),
    '--create-mode', 'db',
    '--assume-empty-db-import',
    '--text-hash-command', 'unused',
  ], { cwd: packageRoot, encoding: 'utf8' });

  assert.equal(result.status, 1);
  assert.match(result.stderr, /disabled until its empty-target guard and DB shell writes are atomic/);
  assert.doesNotMatch(result.stderr, /ENOENT|postgres|docker|S3/u);
});

test('apply-corpus-import-manifest empty-DB preflight fails closed on active pages', async () => {
  const calls = [];
  const sqlExecutor = {
    async runSql(sql, options) {
      calls.push({ sql, options });
      return '42|existing-page';
    },
  };

  await assertEmptyDbImportTarget({ assumeEmptyDbImport: false, siteId: 6000005 }, sqlExecutor);
  assert.equal(calls.length, 0);

  await assert.rejects(
    assertEmptyDbImportTarget({ assumeEmptyDbImport: true, siteId: 6000005 }, sqlExecutor),
    /requires an empty active page set for site 6000005; found page 42 \(existing-page\)/,
  );
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0].options, { capture: true });
  assert.match(calls[0].sql, /WHERE site_id = 6000005/);
  assert.match(calls[0].sql, /AND deleted_at IS NULL/);

  const emptyExecutor = { runSql: async () => '' };
  await assert.doesNotReject(
    assertEmptyDbImportTarget({ assumeEmptyDbImport: true, siteId: 6000005 }, emptyExecutor),
  );

  for (const siteId of [NaN, 6000005.5, '6000005', null]) {
    await assert.rejects(
      assertEmptyDbImportTarget({ assumeEmptyDbImport: true, siteId }, sqlExecutor),
      /expected integer site ID/,
    );
  }
  assert.equal(calls.length, 1, 'malformed site IDs must fail before executing SQL');

  const scriptPath = path.join(
    path.dirname(path.dirname(fileURLToPath(import.meta.url))),
    'scripts/apply-corpus-import-manifest.mjs',
  );
  const script = fs.readFileSync(scriptPath, 'utf8');
  assert.match(script, /verify_empty_db_import_target/);
});

test('apply-corpus-import-manifest rejects conflicting DB rerender flags', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--dry-run',
    '--create-mode',
    'db',
    '--rerender-after-db-create',
    '--skip-rerender',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /rerender-after-db-create cannot be combined with --skip-rerender/);
});

test('apply-corpus-import-manifest rejects conflicting attachments-only replacement flags', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--attachments-only-existing',
    '--replace-existing',
    '--create-mode',
    'db',
    '--dry-run',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /attachments-only-existing cannot be combined with --replace-existing/);
});

test('apply-corpus-import-manifest rejects conflicting skip attachment flags', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: 'cccccccc-cccc-4ccc-8ccc-cccccccccccc',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--attachments-only-existing',
    '--skip-attachments',
    '--dry-run',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /skip-attachments cannot be combined with --attachments-only-existing/);
});

test('apply-corpus-import-manifest dry-run accepts skipped attachments without a session token', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: 'dddddddd-dddd-4ddd-8ddd-dddddddddddd',
    source: '[[image https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png]]',
  });
  writePageAttachment(root, 'en', 'scp-173', {
    filename: 'pixel.png',
    bytes: Buffer.from([9, 8, 7]),
    originalUrl: 'https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--dry-run',
    '--skip-attachments',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    env: { ...process.env, DEEPWELL_SESSION_TOKEN: '' },
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const output = JSON.parse(result.stdout);
  assert.deepEqual(output, {
    dry_run: true,
    selected_rows: 1,
    complete_inventory: true,
  });
});

test('apply-corpus-import-manifest dry-run direct attachment mode plans selected rows', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: 'eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee',
    source: '[[image https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png]]',
  });
  writePage(root, 'en', 'scp-174', {
    entityId: 'ffffffff-ffff-4fff-8fff-ffffffffffff',
    meta: {
      fullname: 'scp-174',
      title: 'SCP-174',
      title_shown: 'SCP-174',
    },
    source: '[[image https://scp-wiki.wikidot.com/local--files/scp-174/pixel.png]]',
  });
  writePageAttachment(root, 'en', 'scp-173', {
    filename: 'pixel.png',
    bytes: Buffer.from([1, 2, 3]),
    originalUrl: 'https://scp-wiki.wikidot.com/local--files/scp-173/pixel.png',
  });
  writePageAttachment(root, 'en', 'scp-174', {
    filename: 'pixel.png',
    bytes: Buffer.from([1, 2, 3]),
    originalUrl: 'https://scp-wiki.wikidot.com/local--files/scp-174/pixel.png',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const result = spawnSync(process.execPath, [
    path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs'),
    '--manifest',
    manifestPath,
    '--dry-run',
    '--attachment-create-mode',
    'direct',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    env: { ...process.env, DEEPWELL_SESSION_TOKEN: '' },
    maxBuffer: 1024 * 1024,
  });

  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  const output = JSON.parse(result.stdout);
  assert.deepEqual(output, {
    dry_run: true,
    selected_rows: 2,
    complete_inventory: true,
    attachment_direct_plan: {
      attachments_requested: 2,
      unique_blobs: 1,
      duplicate_blobs: 1,
      total_bytes: 6,
      unique_bytes: 3,
    },
  });
});

test('apply-corpus-import-manifest rejects unsafe direct attachment mode combinations', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-apply-'));
  writePage(root, 'en', 'scp-173', {
    entityId: '12345678-1234-4234-8234-123456789abc',
    source: 'SCP-173',
  });
  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
  });
  const manifestPath = path.join(root, 'manifest.jsonl');
  fs.writeFileSync(manifestPath, formatJsonl(rows));

  const { spawnSync } = await import('node:child_process');
  const packageRoot = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
  const scriptPath = path.join(packageRoot, 'scripts/apply-corpus-import-manifest.mjs');
  const directWriteWithoutDb = spawnSync(process.execPath, [
    scriptPath,
    '--manifest',
    manifestPath,
    '--attachment-create-mode',
    'direct',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });
  const directWriteWithoutActor = spawnSync(process.execPath, [
    scriptPath,
    '--manifest',
    manifestPath,
    '--attachment-create-mode',
    'direct',
    '--db-url',
    'postgres://wikijump:wikijump@127.0.0.1:1/wikijump',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });
  const skippedDirect = spawnSync(process.execPath, [
    scriptPath,
    '--manifest',
    manifestPath,
    '--dry-run',
    '--skip-attachments',
    '--attachment-create-mode',
    'direct',
  ], {
    cwd: packageRoot,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });

  assert.notEqual(directWriteWithoutDb.status, 0);
  assert.match(directWriteWithoutDb.stderr, /direct requires --db-url|direct requires --db-url or DEEPWELL_VERIFY_DB_URL/);
  assert.notEqual(directWriteWithoutActor.status, 0);
  assert.match(directWriteWithoutActor.stderr, /direct requires --attachment-user-id or non-default --user-id/);
  assert.notEqual(skippedDirect.status, 0);
  assert.match(skippedDirect.stderr, /skip-attachments cannot be combined with --attachment-create-mode direct/);
});

test('buildCorpusImportManifest fullnames filter selects only the named pages', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  writePage(root, 'en', 'scp-2000', { entityId: '16161616-1616-4161-8161-161616161616' });
  writePage(root, 'en', 'scp-173', { entityId: '17171717-1717-4171-8171-171717171717' });
  writePage(root, 'en', 'component:license-box', { entityId: '18181818-1818-4181-8181-181818181818' });

  const rows = buildCorpusImportManifest({
    corpusRoot: root,
    branch: 'en',
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
    fullnames: ['scp-2000', 'component:license-box'],
  });
  assert.deepEqual(rows.map((row) => row.fullname), ['component:license-box', 'scp-2000']);
});

test('buildCorpusImportManifest fullnames filter fails closed on unknown pages', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-manifest-'));
  writePage(root, 'en', 'scp-2000', { entityId: '19191919-1919-4191-8191-191919191919' });

  assert.throws(
    () => buildCorpusImportManifest({
      corpusRoot: root,
      branch: 'en',
      sourceSite: 'scp-wiki',
      sourceBranch: 'en',
      fullnames: ['scp-2000', 'scp-404-not-there'],
    }),
    /fullnames not found in corpus .*scp-404-not-there/,
  );
  assert.throws(
    () => buildCorpusImportManifest({
      corpusRoot: root,
      branch: 'en',
      sourceSite: 'scp-wiki',
      sourceBranch: 'en',
      fullnames: [],
    }),
    /at least one page/,
  );
});
