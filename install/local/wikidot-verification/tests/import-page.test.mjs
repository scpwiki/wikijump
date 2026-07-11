import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';

import {
  batchSlugs,
  buildApplyInvocation,
  corpusPageStatus,
  mergeApplySummaries,
  parseApplyOutput,
  planImportSet,
  resolveSessionToken,
  slugFromDependencyLabel,
} from '../src/import-page.mjs';

test('corpusPageStatus classifies ok, missing, and incomplete pages', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'import-page-'));
  const pageDir = path.join(root, 'en', 'pages', 'scp-2000');
  fs.mkdirSync(pageDir, { recursive: true });
  fs.writeFileSync(path.join(pageDir, 'source.wikidot.txt'), 'text');
  fs.writeFileSync(path.join(pageDir, 'meta.json'), '{}');
  assert.deepEqual(corpusPageStatus(root, 'en', 'scp-2000'), {
    status: 'incomplete',
    missing_files: ['entity_id.txt'],
  });
  fs.writeFileSync(path.join(pageDir, 'entity_id.txt'), 'id');
  assert.deepEqual(corpusPageStatus(root, 'en', 'scp-2000'), { status: 'ok' });
  assert.deepEqual(corpusPageStatus(root, 'en', 'scp-404'), { status: 'missing' });
});

test('slugFromDependencyLabel strips only the family or site prefix', () => {
  assert.equal(slugFromDependencyLabel('EN:scp-2000'), 'scp-2000');
  assert.equal(slugFromDependencyLabel('EN:component:license-box'), 'component:license-box');
  assert.equal(slugFromDependencyLabel('scp-2000'), 'scp-2000');
});

test('planImportSet orders dependencies before requested pages and dedupes', () => {
  const reports = [
    {
      fixture_id: 'EN:scp-2000',
      status: 'closure_complete',
      import_order: ['EN:component:license-box', 'EN:component:license-box-end', 'EN:scp-2000'],
      dependencies: { out_of_bundle: [] },
    },
    {
      fixture_id: 'EN:scp-9506',
      status: 'out_of_bundle',
      import_order: ['EN:component:license-box', 'EN:theme:basalt', 'EN:scp-9506'],
      dependencies: { out_of_bundle: [{ kind: 'include', label: 'scp-int:component:x' }] },
    },
  ];
  const plan = planImportSet({ requestedSlugs: ['scp-2000', 'scp-9506'], closureReports: reports });
  assert.deepEqual(plan.dependencySlugs, [
    'component:license-box',
    'component:license-box-end',
    'theme:basalt',
  ]);
  assert.deepEqual(plan.importSlugs, [
    'component:license-box',
    'component:license-box-end',
    'theme:basalt',
    'scp-2000',
    'scp-9506',
  ]);
  assert.equal(plan.outOfBundle.length, 1);
  assert.equal(plan.outOfBundle[0].page, 'scp-9506');
  assert.deepEqual(plan.closureStatuses, {
    'scp-2000': 'closure_complete',
    'scp-9506': 'out_of_bundle',
  });
});

test('batchSlugs splits into fixed-size batches and rejects bad sizes', () => {
  assert.deepEqual(batchSlugs(['a', 'b', 'c'], 2), [['a', 'b'], ['c']]);
  assert.deepEqual(batchSlugs([], 40), []);
  assert.throws(() => batchSlugs(['a'], 0), /positive integer/);
});

test('apply invocation pairs a configured RPC URL with an explicit session token', async () => {
  const rpcUrl = 'https://configured.example.test/jsonrpc';
  let loginCalls = 0;
  const sessionToken = await resolveSessionToken({
    sessionToken: 'explicit-token',
    rpcUrl,
    login: async () => {
      loginCalls += 1;
      return 'unexpected-login-token';
    },
  });
  const invocation = buildApplyInvocation({
    batchPath: '/tmp/apply-batch-0.jsonl',
    rpcUrl,
    siteId: 6000005,
    attachmentUserId: '-1',
    sessionToken,
  });

  assert.equal(loginCalls, 0);
  assert.deepEqual(invocation, {
    scriptName: 'apply-corpus-import-manifest.mjs',
    scriptArgs: [
      '--manifest', '/tmp/apply-batch-0.jsonl',
      '--create-mode', 'rpc',
      '--api-url', rpcUrl,
      '--site-id', '6000005',
      '--skip-existing-done',
      '--attachment-user-id', '-1',
      '--presign-host-alias', 'files=127.0.0.1',
    ],
    env: { DEEPWELL_SESSION_TOKEN: 'explicit-token' },
  });
});

test('apply invocation pairs a configured RPC URL with a login-issued token', async () => {
  const rpcUrl = 'https://configured.example.test/jsonrpc';
  const loginUrls = [];
  const sessionToken = await resolveSessionToken({
    sessionToken: null,
    rpcUrl,
    login: async (url) => {
      loginUrls.push(url);
      return 'login-issued-token';
    },
  });
  const invocation = buildApplyInvocation({
    batchPath: '/tmp/apply-batch-1.jsonl',
    rpcUrl,
    siteId: 6000005,
    attachmentUserId: '-1',
    sessionToken,
  });

  assert.deepEqual(loginUrls, [rpcUrl]);
  assert.deepEqual(invocation, {
    scriptName: 'apply-corpus-import-manifest.mjs',
    scriptArgs: [
      '--manifest', '/tmp/apply-batch-1.jsonl',
      '--create-mode', 'rpc',
      '--api-url', rpcUrl,
      '--site-id', '6000005',
      '--skip-existing-done',
      '--attachment-user-id', '-1',
      '--presign-host-alias', 'files=127.0.0.1',
    ],
    env: { DEEPWELL_SESSION_TOKEN: 'login-issued-token' },
  });
});

test('mergeApplySummaries adds numeric fields across batches', () => {
  const merged = mergeApplySummaries([
    { created: 2, attachments_uploaded: 5, import_run_id: 10 },
    { created: 1, skipped_existing_done: 3, import_run_id: 11 },
    null,
  ]);
  assert.equal(merged.created, 3);
  assert.equal(merged.attachments_uploaded, 5);
  assert.equal(merged.skipped_existing_done, 3);
});

test('parseApplyOutput reads row lines and the pretty summary block', () => {
  const stdout = [
    '{"slug":"component:license-box","action":"skipped_existing_done","page_id":1}',
    '{"slug":"scp-2000","action":"created","page_id":2,"attachments_uploaded":1}',
    '{',
    '  "summary": {',
    '    "created": 1,',
    '    "skipped_existing_done": 1',
    '  }',
    '}',
  ].join('\n');
  const parsed = parseApplyOutput(stdout);
  assert.equal(parsed.rows.length, 2);
  assert.equal(parsed.rows[1].slug, 'scp-2000');
  assert.deepEqual(parsed.summary, { created: 1, skipped_existing_done: 1 });
});

test('parseApplyOutput handles pretty dry-run output', () => {
  const stdout = ['{', '  "dry_run": true,', '  "selected_rows": 3', '}'].join('\n');
  const parsed = parseApplyOutput(stdout);
  assert.deepEqual(parsed.rows, []);
  assert.deepEqual(parsed.summary, { dry_run: true, selected_rows: 3 });
});
