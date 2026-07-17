import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import {PLAN_SCHEMA, runCompletionPlan, sha256Value, validatePlan} from '../src/completion-controller.mjs';
import {parseArgs} from '../scripts/run-completion-controller.mjs';

async function temporaryRoot() {
  return fs.mkdtemp(path.join(os.tmpdir(), 'wj-completion-controller-'));
}

function diagnosticPlan(stages) {
  return {schema: PLAN_SCHEMA, run_id: 'test-run', branch: 'en', mode: 'diagnostic', candidate: {wikijump_sha: 'a'.repeat(40), ftml_sha: 'b'.repeat(40)}, stages};
}

function stage(id, kind, extra = {}) {
  return {id, kind, command: process.execPath, args: ['-e', 'process.exit(0)'], outputs: [`./${id}.json`], ...extra};
}

test('CLI requires exact controller paths and supports no-resume', () => {
  assert.deepEqual(parseArgs(['--plan', 'p.json', '--state', 's.json', '--summary', 'v.json', '--no-resume', '--quiet']), {plan: path.resolve('p.json'), state: path.resolve('s.json'), summary: path.resolve('v.json'), resume: false, quiet: true});
  assert.throws(() => parseArgs(['--plan', 'p.json']), /--state is required/);
  assert.throws(() => parseArgs(['--bogus']), /unknown argument/);
});

test('plan requires one manifest stage and earlier dependencies', () => {
  const planPath = '/tmp/controller/plan.json';
  assert.throws(() => validatePlan(diagnosticPlan([stage('import', 'import')]), {planPath}), /exactly one/);
  assert.throws(() => validatePlan(diagnosticPlan([stage('freeze', 'freeze_manifest', {depends_on: ['later']}), stage('later', 'render')]), {planPath}), /earlier stage/);
  const validated = validatePlan(diagnosticPlan([stage('freeze', 'freeze_manifest'), stage('import', 'import')]), {planPath});
  assert.equal(validated.stages[1].depends_on[0], 'freeze');
});

test('complete plans require every terminal gate kind', () => {
  const plan = diagnosticPlan([stage('freeze', 'freeze_manifest')]);
  plan.mode = 'complete';
  assert.throws(() => validatePlan(plan, {planPath: '/tmp/plan.json'}), /missing import stage/);
});

test('plan evidence outputs have one owner and bind verdict and cluster files', () => {
  const planPath = '/tmp/controller/plan.json';
  assert.throws(() => validatePlan(diagnosticPlan([stage('freeze', 'freeze_manifest', {outputs: ['./same.json']}), stage('render', 'render', {outputs: ['./same.json']})]), {planPath}), /owned by both/);
  assert.throws(() => validatePlan(diagnosticPlan([stage('freeze', 'freeze_manifest', {outputs: ['./manifest.json'], verdict: {path: './verdict.json'}})]), {planPath}), /verdict path must be one of its outputs/);
  assert.throws(() => validatePlan(diagnosticPlan([stage('freeze', 'freeze_manifest', {outputs: ['./manifest.json'], cluster_sources: [{path: './clusters.jsonl'}]})]), {planPath}), /cluster source must be one of its outputs/);
});

test('candidate evidence rejects secret-shaped fields', () => {
  const plan = diagnosticPlan([stage('freeze', 'freeze_manifest')]);
  plan.candidate = {runtime: {session_token: 'not-allowed'}};
  assert.throws(() => validatePlan(plan, {planPath: '/tmp/plan.json'}), /looks sensitive/);
});

test('controller runs stages, hashes outputs, checks verdicts, and deduplicates clusters', async () => {
  const root = await temporaryRoot();
  const planPath = path.join(root, 'plan.json');
  const statePath = path.join(root, 'state.json');
  const summaryPath = path.join(root, 'summary.json');
  const inputPath = path.join(root, 'input.txt');
  const firstOutput = path.join(root, 'manifest.json');
  const secondOutput = path.join(root, 'render.json');
  const verdictPath = path.join(root, 'verdict.json');
  const clustersPath = path.join(root, 'clusters.jsonl');
  await fs.writeFile(inputPath, 'source\n');
  const plan = diagnosticPlan([
    stage('freeze', 'freeze_manifest', {inputs: ['./input.txt'], outputs: ['./manifest.json']}),
    stage('render', 'render', {inputs: ['./manifest.json'], outputs: ['./render.json', './verdict.json', './clusters.jsonl'], verdict: {path: './verdict.json', pointer: 'gate.status', pass_values: ['pass']}, cluster_sources: [{path: './clusters.jsonl', key_fields: ['category', 'root_cause']}]}),
  ]);
  await fs.writeFile(planPath, `${JSON.stringify(plan)}\n`);
  const calls = [];
  const runner = async (options) => {
    calls.push(options.label);
    if (options.label === 'freeze') await fs.writeFile(firstOutput, '{"manifest":true}\n');
    else {
      await fs.writeFile(secondOutput, '{"rows":3}\n');
      await fs.writeFile(verdictPath, '{"gate":{"status":"pass"}}\n');
      await fs.writeFile(clustersPath, [JSON.stringify({category: 'syntax', root_cause: 'x', page: 'a'}), JSON.stringify({category: 'syntax', root_cause: 'x', page: 'b'}), JSON.stringify({category: 'network', root_cause: 'y'})].join('\n'));
    }
    return {exitCode: 0, record: {runId: `run-${options.label}`, artifactPaths: {ledger: path.join(root, 'ledger.jsonl')}}};
  };
  const summary = await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  assert.equal(summary.status, 'pass');
  assert.deepEqual(calls, ['freeze', 'render']);
  assert.equal(summary.stages.passed, 2);
  assert.equal(summary.failure_cluster_count, 2);
  assert.equal(summary.failure_clusters[0].occurrences, 2);
  assert.match(summary.stage_receipts[0].outputs[0].sha256, /^[a-f0-9]{64}$/);
  assert.equal(JSON.parse(await fs.readFile(summaryPath, 'utf8')).schema, summary.schema);
});

test('controller resumes only exact fingerprints with unchanged outputs', async () => {
  const root = await temporaryRoot();
  const planPath = path.join(root, 'plan.json');
  const statePath = path.join(root, 'state.json');
  const summaryPath = path.join(root, 'summary.json');
  await fs.writeFile(path.join(root, 'source.txt'), 'a');
  const plan = diagnosticPlan([stage('freeze', 'freeze_manifest', {inputs: ['./source.txt'], outputs: ['./manifest.json']})]);
  await fs.writeFile(planPath, JSON.stringify(plan));
  let calls = 0;
  const runner = async () => {
    calls += 1;
    await fs.writeFile(path.join(root, 'manifest.json'), `call=${calls}\n`);
    return {exitCode: 0, record: {runId: String(calls), artifactPaths: {ledger: path.join(root, 'ledger.jsonl')}}};
  };
  const first = await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  const second = await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  assert.equal(first.status, 'pass');
  assert.equal(second.stages.resumed, 1);
  assert.equal(calls, 1);
  await fs.writeFile(path.join(root, 'manifest.json'), 'mutated\n');
  await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  assert.equal(calls, 2);
});

test('controller reruns when a declared verdict changes after a pass', async () => {
  const root = await temporaryRoot();
  const planPath = path.join(root, 'plan.json');
  const statePath = path.join(root, 'nested', 'state.json');
  const summaryPath = path.join(root, 'nested', 'summary.json');
  const plan = diagnosticPlan([stage('freeze', 'freeze_manifest', {outputs: ['./manifest.json', './verdict.json'], verdict: {path: './verdict.json', pass_values: ['pass']}})]);
  await fs.writeFile(planPath, JSON.stringify(plan));
  let calls = 0;
  const runner = async () => {
    calls += 1;
    await fs.writeFile(path.join(root, 'manifest.json'), '{}');
    await fs.writeFile(path.join(root, 'verdict.json'), '{"status":"pass"}');
    return {exitCode: 0, record: {runId: String(calls), artifactPaths: {ledger: path.join(root, 'ledger.jsonl')}}};
  };
  await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  await fs.writeFile(path.join(root, 'verdict.json'), '{"status":"fail"}');
  const summary = await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  assert.equal(summary.status, 'pass');
  assert.equal(calls, 2);
});

test('controller safely replaces a same-host lock owned by a dead process', async () => {
  const root = await temporaryRoot();
  const planPath = path.join(root, 'plan.json');
  const statePath = path.join(root, 'state.json');
  const summaryPath = path.join(root, 'summary.json');
  await fs.writeFile(planPath, JSON.stringify(diagnosticPlan([stage('freeze', 'freeze_manifest')])));
  await fs.writeFile(`${statePath}.lock`, `${JSON.stringify({pid: 2_000_000_000, hostname: os.hostname()})}\n`);
  const runner = async () => {
    await fs.writeFile(path.join(root, 'freeze.json'), '{}');
    return {exitCode: 0, record: {runId: 'lock-recovery', artifactPaths: {ledger: path.join(root, 'ledger.jsonl')}}};
  };
  const summary = await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  assert.equal(summary.status, 'pass');
  await assert.rejects(fs.stat(`${statePath}.lock`), {code: 'ENOENT'});
});

test('controller stops at first failed verdict and preserves a compact failure receipt', async () => {
  const root = await temporaryRoot();
  const planPath = path.join(root, 'plan.json');
  const statePath = path.join(root, 'state.json');
  const summaryPath = path.join(root, 'summary.json');
  const plan = diagnosticPlan([
    stage('freeze', 'freeze_manifest', {outputs: ['./manifest.json', './verdict.json'], verdict: {path: './verdict.json', pass_values: ['pass']}}),
    stage('later', 'render', {outputs: ['./later.json']}),
  ]);
  await fs.writeFile(planPath, JSON.stringify(plan));
  const calls = [];
  const runner = async (options) => {
    calls.push(options.label);
    await fs.writeFile(path.join(root, 'manifest.json'), '{}');
    await fs.writeFile(path.join(root, 'verdict.json'), '{"status":"fail"}');
    return {exitCode: 0, record: {runId: 'failed-verdict', artifactPaths: {ledger: path.join(root, 'ledger.jsonl')}}};
  };
  const summary = await runCompletionPlan({planPath, statePath, summaryPath, commandRunner: runner});
  assert.equal(summary.status, 'fail');
  assert.deepEqual(calls, ['freeze']);
  assert.equal(summary.stages.failed, 1);
  assert.equal(summary.stages.pending, 1);
  assert.match(summary.stage_receipts[0].reason, /reported "fail"/);
});

test('plan hash binds exact bytes', () => {
  assert.notEqual(sha256Value(Buffer.from('{"a":1}')), sha256Value(Buffer.from('{ "a": 1 }')));
});
