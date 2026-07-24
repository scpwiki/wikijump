import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import { test } from 'node:test';
import {fileURLToPath} from 'node:url';

import {parseArgs as parseImportPageArgs, usage as importPageUsage} from '../scripts/import-page.mjs';
import {
  batchSlugs,
  buildApplyInvocation,
  corpusPageStatus,
  mergeApplySummaries,
  parseApplyOutput,
  planImportSet,
  resolveSessionToken,
  rpcCall,
  slugFromDependencyLabel,
  validateRpcUrl,
} from '../src/import-page.mjs';

test('import-page CLI derives fail-closed defaults without running imports', () => {
  assert.deepEqual(parseImportPageArgs([
    '--slug', 'scp-2000',
    '--corpus-root', '/tmp/corpus',
    '--output-dir', '/tmp/output',
    '--branch', 'en',
    '--site', 'scp-wiki',
    '--batch-size', '2',
    '--max-depth', '3',
    '--adopt-existing',
    '--skip-health',
    '--dry-run',
  ], {}), {
    slugs: ['scp-2000'],
    corpusRoot: '/tmp/corpus',
    inventory: null,
    outputDir: '/tmp/output',
    branch: 'en',
    family: 'EN',
    site: 'scp-wiki',
    siteId: null,
    sourceSite: 'scp-wiki',
    sourceBranch: 'en',
    host: 'scp-wiki.wikijump.localhost',
    rpcUrl: 'http://127.0.0.1:2747/jsonrpc',
    sessionToken: null,
    dbContainer: null,
    attachmentUserId: '-1',
    batchSize: 2,
    maxDepth: 3,
    adoptExisting: true,
    skipHealth: true,
    dryRun: true,
  });
  assert.deepEqual(parseImportPageArgs(['--help'], {}), {help: true});
  assert.match(importPageUsage(), /--skip-health/u);
  assert.throws(
    () => parseImportPageArgs(['--slug', 'scp-2000', '--output-dir', '/tmp/output'], {}),
    /--corpus-root is required/u,
  );
});

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

test('RPC URL validation rejects invalid schemes and credentials', () => {
  assert.throws(() => validateRpcUrl('not a url'), /valid absolute URL/);
  assert.throws(() => validateRpcUrl('file:///tmp/jsonrpc'), /scheme must be http or https/);
  assert.throws(
    () => validateRpcUrl('https://user:password@example.test/jsonrpc'),
    /must not contain credentials/,
  );
  assert.throws(
    () => validateRpcUrl('http://rpc.example.test/jsonrpc'),
    /must use HTTPS for non-loopback hosts/,
  );
});

test('RPC URL warnings identify remote HTTPS origins without secrets', () => {
  assert.deepEqual(validateRpcUrl('http://127.0.0.1:2747/jsonrpc').warnings, []);
  assert.deepEqual(validateRpcUrl('http://[::1]:2747/jsonrpc').warnings, []);
  const warnings = validateRpcUrl('https://rpc.example.test/jsonrpc').warnings;
  assert.equal(warnings.length, 1);
  assert.match(warnings.join('\n'), /RPC origin https:\/\/rpc\.example\.test/);
  assert.doesNotMatch(warnings.join('\n'), /token|password|secret-value/u);
});

test('login RPC uses the configured mock origin and returns its session token', async (t) => {
  const requests = [];
  const server = http.createServer(async (request, response) => {
    let body = '';
    for await (const chunk of request) body += chunk;
    requests.push({url: request.url, body: JSON.parse(body)});
    response.writeHead(200, {'content-type': 'application/json'});
    response.end(JSON.stringify({jsonrpc: '2.0', id: 1, result: {session_token: 'mock-session'}}));
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise((resolve, reject) => {
    server.close((error) => error ? reject(error) : resolve());
  }));
  const {port} = server.address();
  const rpcUrl = `http://127.0.0.1:${port}/custom-jsonrpc`;

  const result = await rpcCall(rpcUrl, 'login', {name_or_email: 'admin@example.test'});
  assert.equal(result.session_token, 'mock-session');
  assert.equal(requests.length, 1);
  assert.equal(requests[0].url, '/custom-jsonrpc');
  assert.equal(requests[0].body.method, 'login');
});

test('RPC calls reject cross-origin redirects without forwarding request bodies', async (t) => {
  let targetRequests = 0;
  const target = http.createServer((_request, response) => {
    targetRequests += 1;
    response.writeHead(200, {'content-type': 'application/json'});
    response.end(JSON.stringify({jsonrpc: '2.0', id: 1, result: {}}));
  });
  await new Promise((resolve) => target.listen(0, '127.0.0.1', resolve));
  const targetPort = target.address().port;

  const redirect = http.createServer((_request, response) => {
    response.writeHead(307, {location: `http://127.0.0.1:${targetPort}/capture`});
    response.end();
  });
  await new Promise((resolve) => redirect.listen(0, '127.0.0.1', resolve));
  const redirectPort = redirect.address().port;

  t.after(() => Promise.all([
    new Promise((resolve, reject) => target.close((error) => error ? reject(error) : resolve())),
    new Promise((resolve, reject) => redirect.close((error) => error ? reject(error) : resolve())),
  ]));

  await assert.rejects(
    rpcCall(`http://127.0.0.1:${redirectPort}/jsonrpc`, 'login', {
      password: 'redirect-secret',
    }),
  );
  assert.equal(targetRequests, 0);
});

test('apply RPC transport also rejects redirects before sending session headers onward', () => {
  const source = fs.readFileSync(
    fileURLToPath(new URL('../scripts/apply-corpus-import-manifest.mjs', import.meta.url)),
    'utf8',
  );
  const rpcStart = source.indexOf('async function rpc(');
  const rpcEnd = source.indexOf('\nfunction parseRows(', rpcStart);
  assert.notEqual(rpcStart, -1);
  assert.notEqual(rpcEnd, -1);
  assert.match(source.slice(rpcStart, rpcEnd), /redirect:\s*['"]error['"]/u);
});

test('session tokens are environment-only and never included in argv', () => {
  const secret = 'token-value-that-must-not-be-logged';
  const invocation = buildApplyInvocation({
    batchPath: '/tmp/batch.jsonl',
    rpcUrl: 'https://rpc.example.test/jsonrpc',
    siteId: 1,
    attachmentUserId: '-1',
    sessionToken: secret,
  });

  assert.equal(invocation.env.DEEPWELL_SESSION_TOKEN, secret);
  assert.doesNotMatch(invocation.scriptArgs.join(' '), new RegExp(secret));
  assert.doesNotMatch(JSON.stringify({
    scriptName: invocation.scriptName,
    scriptArgs: invocation.scriptArgs,
  }), new RegExp(secret));

  const script = fileURLToPath(new URL('../scripts/import-page.mjs', import.meta.url));
  const rejected = spawnSync(process.execPath, [script, '--session-token', secret], {
    encoding: 'utf8',
  });
  assert.equal(rejected.status, 2);
  assert.match(rejected.stderr, /Unknown argument: --session-token/);
  assert.doesNotMatch(`${rejected.stdout}\n${rejected.stderr}`, new RegExp(secret));
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

test('parseApplyOutput rejects truncated, missing, and duplicate terminal summaries', () => {
  assert.throws(
    () => parseApplyOutput('{\n  "summary": {\n    "created": 1'),
    /incomplete JSON object/,
  );
  assert.throws(
    () => parseApplyOutput('{"slug":"scp-2000","action":"created"}'),
    /missing its terminal summary/,
  );
  assert.throws(
    () => parseApplyOutput('{"summary":{"created":1}}\n{"dry_run":true,"selected_rows":1}'),
    /multiple terminal summaries/,
  );
  assert.deepEqual(
    parseApplyOutput('{"slug":"scp-2000","action":"failed"}', {requireTerminal: false}),
    {rows: [{slug: 'scp-2000', action: 'failed'}], summary: null},
  );
});
