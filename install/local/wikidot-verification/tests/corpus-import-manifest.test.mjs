import assert from 'node:assert/strict';
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
