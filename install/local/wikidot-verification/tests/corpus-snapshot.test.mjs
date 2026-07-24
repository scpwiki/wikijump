import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  CORPUS_SNAPSHOT_HASH_WORKER_URL,
  hashCorpusSnapshotPaths,
} from '../src/corpus-snapshot-hash-worker.mjs';
import { parseArgs as parseFreezeArgs, usage as freezeUsage } from '../scripts/freeze-corpus-snapshot.mjs';
import { buildCorpusSnapshot, discoverCorpusBranches } from '../src/corpus-snapshot.mjs';

const TEST_DIR = path.dirname(fileURLToPath(import.meta.url));
const FREEZE_SCRIPT = path.resolve(TEST_DIR, '../scripts/freeze-corpus-snapshot.mjs');

function write(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, value);
}

function fixtureCorpus(index = {
  schema_version: 1,
  by_site_created_at: { 'scp-wiki|2020-01-01T00:00:00+00:00': ['id'] },
}, fullname = 'alpha') {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-snapshot-'));
  write(path.join(root, 'en', 'index.json'), JSON.stringify(index));
  write(path.join(root, 'en', 'pages', fullname, 'source.wikidot.txt'), 'alpha source\n');
  write(path.join(root, 'en', 'pages', fullname, 'meta.json'), JSON.stringify({ fullname, parent_fullname: null }));
  write(path.join(root, 'en', 'pages', fullname, 'entity_id.txt'), '12345678-1234-1234-1234-123456789abc\n');
  write(path.join(root, 'en', 'pages', fullname, 'files', 'image.bin'), 'attachment');
  write(path.join(root, 'en', '_runs', 'ignored.json'), 'not canonical');
  return root;
}

test('freeze CLI exposes deterministic argument validation', () => {
  assert.deepEqual(parseFreezeArgs([
    '--corpus-root', '/tmp/corpus',
    '--output', '/tmp/snapshot.json',
    '--branch', 'en',
    '--repository', 'wikijump=/tmp/wikijump#develop',
    '--hash-workers', '2',
  ]), {
    corpusRoot: '/tmp/corpus',
    output: '/tmp/snapshot.json',
    branches: ['en'],
    repositories: ['wikijump=/tmp/wikijump#develop'],
    hashWorkers: 2,
  });
  assert.deepEqual(parseFreezeArgs(['--help']), { help: true });
  assert.match(freezeUsage(), /--hash-workers/u);
});

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

test('buildCorpusSnapshot accepts canonical target Wikidot origins', () => {
  for (const targetWiki of [
    'scpaiueouiuiuiui.wikidot.com',
    'http://scpaiueouiuiuiui.wikidot.com',
    'https://scpaiueouiuiuiui.wikidot.com/',
  ]) {
    const root = fixtureCorpus({ target_wiki: targetWiki });
    const snapshot = buildCorpusSnapshot({ corpusRoot: root });

    assert.equal(snapshot.branches[0].site_status, 'resolved');
    assert.equal(snapshot.rows[0].source_site, 'scpaiueouiuiuiui');
    assert.equal(snapshot.rows[0].source_url, 'https://scpaiueouiuiuiui.wikidot.com/alpha');
    assert.equal(snapshot.rows[0].local_https_url, 'https://scpaiueouiuiuiui.wikijump.localhost/alpha');
    assert.equal(snapshot.rows[0].inventory_status, 'ready');
  }
});

test('buildCorpusSnapshot accepts site-slug boundaries from corpus index formats', () => {
  const canonicalSites = ['a', '0', 'scp-wiki', `a${'b'.repeat(61)}z`];
  for (const site of canonicalSites) {
    for (const index of [
      { target_wiki: site },
      { target_wiki: `https://${site}.wikidot.com/` },
      { by_site_created_at: { [`${site}|2020-01-01T00:00:00+00:00`]: ['uuid'] } },
    ]) {
      const snapshot = buildCorpusSnapshot({ corpusRoot: fixtureCorpus(index) });
      assert.equal(snapshot.branches[0].site_status, 'resolved', site);
      assert.equal(snapshot.branches[0].source_site, site);
    }
  }
});

test('buildCorpusSnapshot rejects malformed, Unicode, and noncanonical origins', () => {
  const invalidTargets = [
    '',
    '-scp-wiki',
    'scp-wiki-',
    `a${'b'.repeat(63)}`,
    'SCP-WIKI',
    'scp_wiki',
    'scpé-wiki',
    'scp-wiki\n',
    'scp-wiki\r\n',
    'https://scp-wiki.wikidot.com/\n',
    'https://scp-wiki.wikidot.com/\r\n',
    'https://例え.wikidot.com/',
    'ftp://scp-wiki.wikidot.com/',
    'https://user@example.wikidot.com/',
    'https://scp-wiki.wikidot.com:443/',
    'https://scp-wiki.wikidot.com/path',
    'https://scp-wiki.wikidot.com/?query=1',
    'https://scp-wiki.wikidot.com/#fragment',
    'https://scp-wiki.wikidot.com.attacker.test/',
  ];

  for (const target_wiki of invalidTargets) {
    const snapshot = buildCorpusSnapshot({ corpusRoot: fixtureCorpus({ target_wiki }) });
    assert.equal(snapshot.branches[0].site_status, 'unverified', target_wiki);
    assert.equal(snapshot.rows[0].source_url, null);
  }
});

test('buildCorpusSnapshot URL-encodes Unicode page fullnames under a verified origin', () => {
  const fullname = 'カテゴリ:雪 だるま';
  const snapshot = buildCorpusSnapshot({
    corpusRoot: fixtureCorpus({ target_wiki: 'scp-jp.wikidot.com' }, fullname),
  });

  assert.equal(snapshot.rows[0].source_site, 'scp-jp');
  assert.equal(
    snapshot.rows[0].source_url,
    'https://scp-jp.wikidot.com/%E3%82%AB%E3%83%86%E3%82%B4%E3%83%AA:%E9%9B%AA%20%E3%81%A0%E3%82%8B%E3%81%BE',
  );
});

test('buildCorpusSnapshot fails closed on malformed corpus index shapes', () => {
  for (const index of [
    { by_site_created_at: null },
    { by_site_created_at: [] },
    { by_site_created_at: 'scp-wiki|timestamp' },
    { by_site_created_at: { 'missing-delimiter': ['uuid'] } },
    { by_site_created_at: { '|timestamp': ['uuid'] } },
    { target_wiki: null },
    { target_wiki: { site: 'scp-wiki' } },
  ]) {
    const snapshot = buildCorpusSnapshot({ corpusRoot: fixtureCorpus(index) });
    assert.equal(snapshot.branches[0].site_status, 'unverified');
    assert.deepEqual(snapshot.branches[0].source_sites, []);
  }
});

test('buildCorpusSnapshot fails closed on unverified source origins', () => {
  const unsafeIndexes = [
    [{ target_wiki: 'https://127.0.0.1:1234/private' }, []],
    [{ target_wiki: 'https://scp-wiki.wikidot.com.attacker.example' }, []],
    [{ by_site_created_at: { '127.0.0.1:1234/private|2020-01-01T00:00:00+00:00': ['id'] } }, []],
    [{
      by_site_created_at: { 'scp-wiki|2020-01-01T00:00:00+00:00': ['id'] },
      target_wiki: 'https://127.0.0.1:1234/private',
    }, ['scp-wiki']],
  ];

  for (const [index, verifiedSites] of unsafeIndexes) {
    const root = fixtureCorpus(index);
    const snapshot = buildCorpusSnapshot({ corpusRoot: root });

    assert.equal(snapshot.branches[0].site_status, 'unverified');
    assert.equal(snapshot.branches[0].source_site, null);
    assert.deepEqual(snapshot.branches[0].source_sites, verifiedSites);
    assert.equal(snapshot.rows[0].source_url, null);
    assert.equal(snapshot.rows[0].local_https_url, null);
    assert.equal(snapshot.rows[0].inventory_status, 'invalid');
    assert.deepEqual(snapshot.rows[0].inventory_problems, ['source_site_unverified']);
  }
});

test('buildCorpusSnapshot rejects conflicting canonical source sites as ambiguous', () => {
  const root = fixtureCorpus({
    by_site_created_at: { 'scp-wiki|2020-01-01T00:00:00+00:00': ['id'] },
    target_wiki: 'scp-jp.wikidot.com',
  });
  const snapshot = buildCorpusSnapshot({ corpusRoot: root });

  assert.equal(snapshot.branches[0].site_status, 'ambiguous');
  assert.deepEqual(snapshot.branches[0].source_sites, ['scp-jp', 'scp-wiki']);
  assert.equal(snapshot.rows[0].source_site, null);
  assert.equal(snapshot.rows[0].source_url, null);
  assert.equal(snapshot.rows[0].local_https_url, null);
  assert.equal(snapshot.rows[0].inventory_status, 'invalid');
  assert.deepEqual(snapshot.rows[0].inventory_problems, ['source_site_ambiguous']);
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

test('corpus snapshot hash worker exposes its entrypoint and exact hashes', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'corpus-hash-worker-'));
  const first = path.join(root, 'first.txt');
  const second = path.join(root, 'second.txt');
  write(first, 'alpha');
  write(second, 'beta');

  assert.equal(CORPUS_SNAPSHOT_HASH_WORKER_URL.pathname.endsWith('/src/corpus-snapshot-hash-worker.mjs'), true);
  assert.deepEqual(hashCorpusSnapshotPaths([first, second]), [
    [first, {
      bytes: 5,
      sha256: '8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8',
    }],
    [second, {
      bytes: 4,
      sha256: 'f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753',
    }],
  ]);
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
