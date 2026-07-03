import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '../../../..');
const scriptPath = path.join(repoRoot, 'install/local/wikidot-verification/scripts/capture-corpus-files.mjs');
const buildManifestScriptPath = path.join(repoRoot, 'install/local/wikidot-verification/scripts/build-corpus-import-manifest.mjs');

test('capture-corpus-files dry-run discovers absolute and page-relative attachments', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'capture-corpus-files-'));
  const pageDir = path.join(root, 'en', 'pages', 'scp-1234');
  fs.mkdirSync(pageDir, { recursive: true });
  fs.writeFileSync(
    path.join(pageDir, 'source.wikidot.txt'),
    [
      '[[image cover.jpg]]',
      '[[image alternate.jpg?rev=1]]',
      '[[include component:image-block',
      'name=detail.png|caption=Detail]]',
      '[[include component:image-block',
      'name=detail-v2.png#caption|caption=Detail]]',
      '[[module CSS]]',
      '.icon { background: url(icon.svg#cache); }',
      '[[/module]]',
      '[[image https://scp-wiki.wdfiles.com/local--files/scp-1234/remote.webp]]',
      '> **Filename:** credits-only.gif, cover.jpg',
    ].join('\n'),
  );

  const result = spawnSync(
    process.execPath,
    [scriptPath, '--corpus-root', root, '--branch', 'en', '--slug', 'scp-1234', '--dry-run'],
    { encoding: 'utf8' },
  );

  assert.equal(result.status, 0, result.stderr);
  const rows = result.stdout
    .split('\n')
    .filter((line) => line.startsWith('{"action"'))
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    rows.map((row) => row.filename).sort(),
    ['alternate.jpg', 'cover.jpg', 'credits-only.gif', 'detail-v2.png', 'detail.png', 'icon.svg', 'remote.webp'],
  );
  assert.equal(rows.find((row) => row.filename === 'remote.webp').original_url, 'https://scp-wiki.wdfiles.com/local--files/scp-1234/remote.webp');
});

test('capture-corpus-files can create missing cross-page attachment owner records without repo assets', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'capture-corpus-files-'));
  const resourceRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'capture-corpus-file-bytes-'));
  const pageDir = path.join(root, 'en', 'pages', 'scp-1234');
  const resourcePath = path.join(
    resourceRoot,
    'resources/scp-1234/scp-sandbox-3_wdfiles_com/local--files/missing-owner/remote.png',
  );
  fs.mkdirSync(pageDir, { recursive: true });
  fs.mkdirSync(path.dirname(resourcePath), { recursive: true });
  fs.writeFileSync(
    path.join(pageDir, 'source.wikidot.txt'),
    '[[image https://scp-sandbox-3.wdfiles.com/local--files/missing-owner/remote.png]]\n',
  );
  fs.writeFileSync(
    path.join(pageDir, 'meta.json'),
    `${JSON.stringify({
      children: 0,
      commented_at: null,
      commented_by: null,
      comments: 0,
      created_at: '2026-01-01T00:00:00+00:00',
      created_by: 'fixture',
      fullname: 'scp-1234',
      parent_fullname: null,
      parent_title: null,
      rating: 0,
      revisions: 1,
      tags: ['scp'],
      title: 'SCP-1234',
      title_shown: 'SCP-1234',
      updated_at: '2026-01-01T00:00:00+00:00',
      updated_by: 'fixture',
    }, null, 2)}\n`,
  );
  fs.writeFileSync(path.join(pageDir, 'entity_id.txt'), '12345678-1234-4234-8234-123456789abc\n');
  fs.writeFileSync(resourcePath, Buffer.from([0x89, 0x50, 0x4e, 0x47]));

  const result = spawnSync(
    process.execPath,
    [
      scriptPath,
      '--corpus-root',
      root,
      '--branch',
      'en',
      '--slug',
      'scp-1234',
      '--resource-source-root',
      resourceRoot,
      '--create-missing-attachment-pages',
    ],
    { encoding: 'utf8' },
  );

  assert.equal(result.status, 0, result.stderr);
  const rows = result.stdout
    .split('\n')
    .filter((line) => line.startsWith('{"source_slug"'))
    .map((line) => JSON.parse(line));
  assert.equal(rows.length, 1);
  assert.equal(rows[0].page, 'missing-owner');
  assert.equal(rows[0].filename, 'remote.png');
  assert.equal(rows[0].created_missing_corpus_page, true);
  assert.equal(rows[0].corpus_path, 'en/pages/missing-owner/files/remote.png');

  const ownerDir = path.join(root, 'en', 'pages', 'missing-owner');
  assert.equal(fs.existsSync(path.join(ownerDir, 'source.wikidot.txt')), true);
  assert.equal(fs.existsSync(path.join(ownerDir, 'entity_id.txt')), true);
  const meta = JSON.parse(fs.readFileSync(path.join(ownerDir, 'meta.json'), 'utf8'));
  assert.equal(meta.fullname, 'missing-owner');
  assert.equal(meta.capture_method, 'wikijump_corpus_attachment_placeholder');
  assert.equal(meta.source_browser_visibility, 'source_only');
  const files = JSON.parse(fs.readFileSync(path.join(ownerDir, 'files.json'), 'utf8'));
  assert.equal(files.length, 1);
  assert.equal(files[0].original_url, 'https://scp-sandbox-3.wdfiles.com/local--files/missing-owner/remote.png');

  const manifestPath = path.join(root, 'manifest.jsonl');
  const summaryPath = path.join(root, 'summary.json');
  const buildResult = spawnSync(
    process.execPath,
    [
      buildManifestScriptPath,
      '--corpus-root',
      root,
      '--branch',
      'en',
      '--source-site',
      'scp-wiki',
      '--source-branch',
      'en',
      '--output',
      manifestPath,
      '--summary',
      summaryPath,
    ],
    { encoding: 'utf8' },
  );

  assert.equal(buildResult.status, 0, buildResult.stderr);
  const manifestRows = fs.readFileSync(manifestPath, 'utf8')
    .trim()
    .split('\n')
    .map((line) => JSON.parse(line));
  const ownerRow = manifestRows.find((row) => row.fullname === 'missing-owner');
  assert.equal(ownerRow.attachments.length, 1);
  assert.equal(ownerRow.revisions, 0);
  assert.deepEqual(ownerRow.tags, ['attachment-placeholder']);
  assert.equal(ownerRow.attachments[0].corpus_path, 'en/pages/missing-owner/files/remote.png');
});
