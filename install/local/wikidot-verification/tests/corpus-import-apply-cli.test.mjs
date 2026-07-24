import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {fileURLToPath} from "node:url";
import test from "node:test";

import {
  buildCorpusImportManifest,
  formatJsonl,
} from "../src/corpus-import-manifest.mjs";
import {writePage} from "./support/corpus-import-manifest-fixture.mjs";

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
