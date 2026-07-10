import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

import { buildCorpusSnapshot, discoverCorpusBranches } from '../src/corpus-snapshot.mjs';

const TEST_DIR = path.dirname(fileURLToPath(import.meta.url));
const FREEZE_SCRIPT = path.resolve(TEST_DIR, '../scripts/freeze-corpus-snapshot.mjs');

function write(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, value);
}

function fixtureCorpus() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-snapshot-'));
  write(path.join(root, 'en', 'index.json'), JSON.stringify({
    schema_version: 1,
    by_site_created_at: { 'scp-wiki|2020-01-01T00:00:00+00:00': ['id'] },
  }));
  write(path.join(root, 'en', 'pages', 'alpha', 'source.wikidot.txt'), 'alpha source\n');
  write(path.join(root, 'en', 'pages', 'alpha', 'meta.json'), JSON.stringify({ fullname: 'alpha', parent_fullname: null }));
  write(path.join(root, 'en', 'pages', 'alpha', 'entity_id.txt'), '12345678-1234-1234-1234-123456789abc\n');
  write(path.join(root, 'en', 'pages', 'alpha', 'files', 'image.bin'), 'attachment');
  write(path.join(root, 'en', '_runs', 'ignored.json'), 'not canonical');
  return root;
}

test('discoverCorpusBranches ignores operational directories', () => {
  const root = fixtureCorpus();
  fs.mkdirSync(path.join(root, '_dryrun', 'pages'), { recursive: true });
  assert.deepEqual(discoverCorpusBranches(root), ['en']);
});

test('buildCorpusSnapshot freezes canonical files and emits browser inventory rows', () => {
  const root = fixtureCorpus();
  const snapshot = buildCorpusSnapshot({ corpusRoot: root, repositories: [{ name: 'wikijump', commit: 'abc' }] });

  assert.equal(snapshot.schema, 'wikijump_full_parity.corpus_inventory_lock.v1');
  assert.equal(snapshot.totals.branch_count, 1);
  assert.equal(snapshot.totals.page_count, 1);
  assert.equal(snapshot.totals.invalid_page_count, 0);
  assert.match(snapshot.manifest_sha256, /^[0-9a-f]{64}$/u);
  assert.equal(snapshot.rows[0].fixture_id, 'EN:alpha');
  assert.equal(snapshot.rows[0].source_site, 'scp-wiki');
  assert.equal(snapshot.rows[0].source_url, 'https://scp-wiki.wikidot.com/alpha');
  assert.equal(snapshot.rows[0].local_https_url, 'https://scp-wiki.wikijump.localhost/alpha');
  assert.equal(snapshot.rows[0].file_count, 4);
  assert.equal(snapshot.branches[0].files.some((file) => file.path.includes('_runs')), false);
});

test('buildCorpusSnapshot records incomplete pages instead of silently dropping them', () => {
  const root = fixtureCorpus();
  fs.unlinkSync(path.join(root, 'en', 'pages', 'alpha', 'entity_id.txt'));
  const snapshot = buildCorpusSnapshot({ corpusRoot: root });

  assert.equal(snapshot.totals.invalid_page_count, 1);
  assert.equal(snapshot.rows[0].inventory_status, 'invalid');
  assert.deepEqual(snapshot.rows[0].inventory_problems, ['missing:entity_id.txt']);
});

test('manifest identity is stable for identical corpus content', () => {
  const root = fixtureCorpus();
  const first = buildCorpusSnapshot({ corpusRoot: root });
  const second = buildCorpusSnapshot({ corpusRoot: root });
  assert.equal(first.manifest_sha256, second.manifest_sha256);
});

test('freeze CLI hashes canonical files with a worker pool', () => {
  const root = fixtureCorpus();
  const output = path.join(root, 'snapshot.json');
  execFileSync(process.execPath, [
    FREEZE_SCRIPT,
    '--corpus-root', root,
    '--output', output,
    '--hash-workers', '2',
  ]);

  const snapshot = JSON.parse(fs.readFileSync(output, 'utf8'));
  assert.equal(snapshot.totals.page_count, 1);
  assert.equal(snapshot.branches[0].files.length, 5);
  assert.match(snapshot.branches[0].tree_sha256, /^[0-9a-f]{64}$/u);
});
