import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {fileURLToPath} from "node:url";
import test from "node:test";

import {assertEmptyDbImportTarget} from "../src/corpus-import-empty-target.mjs";
import {
  buildCorpusImportManifest,
  formatJsonl,
} from "../src/corpus-import-manifest.mjs";
import {
  writePage,
  writePageAttachment,
} from "./support/corpus-import-manifest-fixture.mjs";

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
