import {createHash} from 'node:crypto';
import {createReadStream} from 'node:fs';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

import {runMeasuredCommand} from './command-ledger.mjs';

export const PLAN_SCHEMA = 'wikijump_full_parity.completion_plan.v1';
export const STATE_SCHEMA = 'wikijump_full_parity.completion_state.v1';
export const SUMMARY_SCHEMA = 'wikijump_full_parity.completion_summary.v1';

const STAGE_KINDS = new Set([
  'freeze_manifest',
  'consume_manifest',
  'import',
  'render',
  'browser_capture',
  'browser_replay',
  'compare',
  'workflow',
  'client',
  'certify',
]);
const SAFE_ID = /^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$/;
const SHA256 = /^[a-f0-9]{64}$/;
const GIT_SHA = /^[a-f0-9]{40}$/;
const SENSITIVE_NAME = /(?:password|passwd|pwd|secret|token|credential|access[_-]?key|db[_-]?url)/i;
const KIND_ORDER = new Map([
  ['freeze_manifest', 0],
  ['consume_manifest', 0],
  ['import', 1],
  ['render', 2],
  ['browser_capture', 3],
  ['browser_replay', 4],
  ['compare', 5],
  ['workflow', 6],
  ['client', 6],
  ['certify', 7],
]);

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

export function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  if (isObject(value)) {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(',')}}`;
  }
  return JSON.stringify(value);
}

export function sha256Value(value) {
  return createHash('sha256').update(typeof value === 'string' || Buffer.isBuffer(value) ? value : canonicalJson(value)).digest('hex');
}

export async function sha256File(filePath) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(filePath)) hash.update(chunk);
  return hash.digest('hex');
}

function resolveFrom(baseDirectory, value, label) {
  if (typeof value !== 'string' || value.length === 0 || value.includes('\0')) throw new Error(`${label} must be a non-empty path string`);
  return path.resolve(baseDirectory, value);
}

function stringArray(value, label) {
  if (!Array.isArray(value) || value.some((item) => typeof item !== 'string' || item.includes('\0'))) throw new Error(`${label} must be an array of strings`);
  return value;
}

function validateCandidate(candidate) {
  const serialized = canonicalJson(candidate);
  if (serialized.length > 65_536) throw new Error('plan candidate exceeds 64 KiB');
  const stack = [[candidate, 'candidate']];
  while (stack.length > 0) {
    const [value, location] = stack.pop();
    if (Array.isArray(value)) {
      for (const [index, item] of value.entries()) stack.push([item, `${location}[${index}]`]);
      continue;
    }
    if (!isObject(value)) {
      if (value !== null && !['string', 'number', 'boolean'].includes(typeof value)) throw new Error(`${location} contains an unsupported value`);
      continue;
    }
    for (const [key, item] of Object.entries(value)) {
      if (SENSITIVE_NAME.test(key)) throw new Error(`${location}.${key} looks sensitive and cannot be copied into controller evidence`);
      stack.push([item, `${location}.${key}`]);
    }
  }
  return candidate;
}

function validateVerdict(verdict, stageId, baseDirectory) {
  if (verdict === undefined) return null;
  if (!isObject(verdict)) throw new Error(`stage ${stageId} verdict must be an object`);
  const pointer = verdict.pointer ?? 'status';
  if (typeof pointer !== 'string' || pointer.length === 0 || pointer.split('.').some((part) => part.length === 0)) throw new Error(`stage ${stageId} verdict pointer is invalid`);
  const passValues = verdict.pass_values ?? ['pass'];
  if (!Array.isArray(passValues) || passValues.length === 0 || passValues.some((value) => !['string', 'number', 'boolean'].includes(typeof value))) throw new Error(`stage ${stageId} verdict pass_values is invalid`);
  return {path: resolveFrom(baseDirectory, verdict.path, `stage ${stageId} verdict path`), pointer, pass_values: passValues};
}

function validateClusterSources(sources, stageId, baseDirectory) {
  if (sources === undefined) return [];
  if (!Array.isArray(sources)) throw new Error(`stage ${stageId} cluster_sources must be an array`);
  return sources.map((source, index) => {
    if (!isObject(source)) throw new Error(`stage ${stageId} cluster source ${index} must be an object`);
    const format = source.format ?? 'jsonl';
    if (!['json', 'jsonl'].includes(format)) throw new Error(`stage ${stageId} cluster source ${index} format must be json or jsonl`);
    const keyFields = source.key_fields ?? ['root_cause', 'category', 'signature'];
    if (!Array.isArray(keyFields) || keyFields.length === 0 || keyFields.some((field) => typeof field !== 'string' || field.length === 0)) throw new Error(`stage ${stageId} cluster source ${index} key_fields is invalid`);
    return {path: resolveFrom(baseDirectory, source.path, `stage ${stageId} cluster source path`), format, key_fields: keyFields};
  });
}

export function validatePlan(plan, {planPath}) {
  if (!isObject(plan) || plan.schema !== PLAN_SCHEMA) throw new Error(`plan must use schema ${PLAN_SCHEMA}`);
  if (!SAFE_ID.test(plan.run_id ?? '')) throw new Error('plan run_id is invalid');
  if (!SAFE_ID.test(plan.branch ?? '')) throw new Error('plan branch is invalid');
  if (!Array.isArray(plan.stages) || plan.stages.length === 0) throw new Error('plan stages must be a non-empty array');
  const baseDirectory = path.dirname(path.resolve(planPath));
  const stageIds = new Set();
  const ownedOutputs = new Map();
  const stages = plan.stages.map((stage, index) => {
    if (!isObject(stage) || !SAFE_ID.test(stage.id ?? '')) throw new Error(`stage ${index} id is invalid`);
    if (stageIds.has(stage.id)) throw new Error(`duplicate stage id: ${stage.id}`);
    stageIds.add(stage.id);
    if (!STAGE_KINDS.has(stage.kind)) throw new Error(`stage ${stage.id} kind is invalid`);
    if (typeof stage.command !== 'string' || stage.command.length === 0 || stage.command.includes('\0')) throw new Error(`stage ${stage.id} command is invalid`);
    const dependencies = stage.depends_on ?? (index === 0 ? [] : [plan.stages[index - 1].id]);
    stringArray(dependencies, `stage ${stage.id} depends_on`);
    for (const dependency of dependencies) {
      if (!stageIds.has(dependency) || dependency === stage.id) throw new Error(`stage ${stage.id} dependency must name an earlier stage: ${dependency}`);
    }
    const timeoutMs = stage.timeout_ms ?? null;
    if (timeoutMs !== null && (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0)) throw new Error(`stage ${stage.id} timeout_ms must be a positive integer or null`);
    const inputs = stringArray(stage.inputs ?? [], `stage ${stage.id} inputs`).map((value) => resolveFrom(baseDirectory, value, `stage ${stage.id} input`));
    const outputs = stringArray(stage.outputs ?? [], `stage ${stage.id} outputs`).map((value) => resolveFrom(baseDirectory, value, `stage ${stage.id} output`));
    if (outputs.length === 0) throw new Error(`stage ${stage.id} must declare at least one evidence output`);
    if (new Set(outputs).size !== outputs.length) throw new Error(`stage ${stage.id} outputs must be unique`);
    if (outputs.some((output) => inputs.includes(output))) throw new Error(`stage ${stage.id} inputs and outputs must be disjoint`);
    for (const output of outputs) {
      if (ownedOutputs.has(output)) throw new Error(`output ${output} is owned by both ${ownedOutputs.get(output)} and ${stage.id}`);
      ownedOutputs.set(output, stage.id);
    }
    const verdict = validateVerdict(stage.verdict, stage.id, baseDirectory);
    const clusterSources = validateClusterSources(stage.cluster_sources, stage.id, baseDirectory);
    if (verdict !== null && !outputs.includes(verdict.path)) throw new Error(`stage ${stage.id} verdict path must be one of its outputs`);
    for (const source of clusterSources) if (!outputs.includes(source.path)) throw new Error(`stage ${stage.id} cluster source must be one of its outputs: ${source.path}`);
    return {
      id: stage.id,
      kind: stage.kind,
      command: stage.command,
      args: stringArray(stage.args ?? [], `stage ${stage.id} args`),
      cwd: resolveFrom(baseDirectory, stage.cwd ?? '.', `stage ${stage.id} cwd`),
      timeout_ms: timeoutMs,
      depends_on: dependencies,
      inputs,
      outputs,
      verdict,
      cluster_sources: clusterSources,
    };
  });
  const manifestStages = stages.filter((stage) => stage.kind === 'freeze_manifest' || stage.kind === 'consume_manifest');
  if (manifestStages.length !== 1) throw new Error('plan must contain exactly one freeze_manifest or consume_manifest stage');
  if (stages[0] !== manifestStages[0]) throw new Error('the manifest stage must be first');
  if (plan.mode === 'complete') {
    for (const kind of ['import', 'render', 'browser_capture', 'browser_replay', 'compare', 'workflow', 'client', 'certify']) {
      if (!stages.some((stage) => stage.kind === kind)) throw new Error(`complete plan is missing ${kind} stage`);
    }
    for (let index = 1; index < stages.length; index += 1) {
      if (KIND_ORDER.get(stages[index].kind) < KIND_ORDER.get(stages[index - 1].kind)) throw new Error(`complete plan stage order regresses at ${stages[index].id}`);
    }
  } else if (plan.mode !== 'diagnostic') {
    throw new Error('plan mode must be complete or diagnostic');
  }
  const candidate = validateCandidate(isObject(plan.candidate) ? plan.candidate : {});
  if (plan.mode === 'complete') {
    for (const field of ['wikijump_sha', 'ftml_sha']) if (!GIT_SHA.test(candidate[field] ?? '')) throw new Error(`complete plan candidate ${field} must be a lowercase 40-character Git SHA`);
    for (const field of ['runtime_identity_sha256', 'runtime_config_sha256']) if (!SHA256.test(candidate[field] ?? '')) throw new Error(`complete plan candidate ${field} must be a lowercase SHA-256`);
    if (typeof candidate.artifact_key !== 'string' || candidate.artifact_key.length === 0) throw new Error('complete plan candidate artifact_key must be non-empty');
  }
  return {
    schema: PLAN_SCHEMA,
    run_id: plan.run_id,
    branch: plan.branch,
    mode: plan.mode,
    candidate,
    ledger_path: resolveFrom(baseDirectory, plan.ledger_path ?? './command-ledger.jsonl', 'ledger_path'),
    stages,
  };
}

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, 'utf8'));
}

async function writeJsonAtomic(filePath, value) {
  await fs.mkdir(path.dirname(filePath), {recursive: true, mode: 0o700});
  const temporary = `${filePath}.tmp-${process.pid}`;
  await fs.writeFile(temporary, `${JSON.stringify(value, null, 2)}\n`, {mode: 0o600});
  await fs.rename(temporary, filePath);
}

async function acquireLock(statePath) {
  await fs.mkdir(path.dirname(statePath), {recursive: true, mode: 0o700});
  const lockPath = `${statePath}.lock`;
  let handle = null;
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      handle = await fs.open(lockPath, 'wx', 0o600);
      break;
    } catch (error) {
      if (error.code !== 'EEXIST' || attempt > 0 || !await removeDeadOwnerLock(lockPath)) {
        if (error.code === 'EEXIST') throw new Error(`controller state lock already exists: ${lockPath}`);
        throw error;
      }
    }
  }
  await handle.writeFile(`${JSON.stringify({pid: process.pid, hostname: os.hostname(), acquired_at: new Date().toISOString()})}\n`);
  await handle.sync();
  return async () => {
    await handle.close().catch(() => {});
    await fs.unlink(lockPath).catch(() => {});
  };
}

async function removeDeadOwnerLock(lockPath) {
  let handle;
  try {
    handle = await fs.open(lockPath, 'r');
    const [stat, text] = await Promise.all([handle.stat(), handle.readFile('utf8')]);
    const owner = JSON.parse(text);
    if (owner.hostname !== os.hostname() || !Number.isSafeInteger(owner.pid) || owner.pid <= 0) return false;
    try {
      process.kill(owner.pid, 0);
      return false;
    } catch (error) {
      if (error.code !== 'ESRCH') return false;
    }
    const current = await fs.lstat(lockPath);
    if (current.dev !== stat.dev || current.ino !== stat.ino) return false;
    await fs.unlink(lockPath);
    return true;
  } catch {
    return false;
  } finally {
    await handle?.close().catch(() => {});
  }
}

async function hashPaths(paths) {
  const entries = [];
  for (const filePath of paths) {
    let stat;
    try {
      stat = await fs.stat(filePath);
    } catch (error) {
      if (error.code === 'ENOENT') throw new Error(`required file is missing: ${filePath}`);
      throw error;
    }
    if (!stat.isFile()) throw new Error(`required path is not a regular file: ${filePath}`);
    entries.push({path: filePath, bytes: stat.size, sha256: await sha256File(filePath)});
  }
  return entries;
}

function valueAtPointer(value, pointer) {
  let current = value;
  for (const part of pointer.split('.')) {
    if (!isObject(current) && !Array.isArray(current) || !Object.hasOwn(current, part)) return undefined;
    current = current[part];
  }
  return current;
}

async function checkVerdict(verdict) {
  if (verdict === null) return {passed: true, value: null, path: null, sha256: null};
  const document = await readJson(verdict.path);
  const value = valueAtPointer(document, verdict.pointer);
  return {passed: verdict.pass_values.some((candidate) => Object.is(candidate, value)), value, path: verdict.path, sha256: await sha256File(verdict.path)};
}

function sameOutputReceipt(previous, current) {
  return Array.isArray(previous) && previous.length === current.length && current.every((entry, index) => entry.path === previous[index].path && entry.bytes === previous[index].bytes && entry.sha256 === previous[index].sha256);
}

async function resumableReceipt(previous, fingerprint, stage) {
  if (!isObject(previous) || previous.status !== 'pass' || previous.fingerprint_sha256 !== fingerprint) return null;
  try {
    const outputs = await hashPaths(stage.outputs);
    if (!sameOutputReceipt(previous.outputs, outputs)) return null;
    const verdict = await checkVerdict(stage.verdict);
    if (!verdict.passed || canonicalJson(verdict) !== canonicalJson(previous.verdict)) return null;
    return {outputs, verdict};
  } catch {
    return null;
  }
}

function clusterKey(record, fields) {
  const selected = {};
  for (const field of fields) {
    const value = valueAtPointer(record, field);
    if (value !== undefined && value !== null && value !== '') selected[field] = value;
  }
  return Object.keys(selected).length === 0 ? null : selected;
}

async function readClusterRecords(source) {
  const text = await fs.readFile(source.path, 'utf8');
  if (source.format === 'jsonl') return text.split('\n').filter(Boolean).map((line) => JSON.parse(line));
  const value = JSON.parse(text);
  if (Array.isArray(value)) return value;
  for (const key of ['clusters', 'findings', 'failures', 'rows', 'results']) if (Array.isArray(value?.[key])) return value[key];
  return [value];
}

async function collectClusters(stages) {
  const clusters = new Map();
  for (const stage of stages) {
    for (const source of stage.cluster_sources) {
      let records;
      try {
        records = await readClusterRecords(source);
      } catch (error) {
        if (error.code === 'ENOENT') continue;
        throw error;
      }
      for (const record of records) {
        if (!isObject(record)) continue;
        const key = clusterKey(record, source.key_fields);
        if (key === null) continue;
        const fingerprint = sha256Value(key);
        const existing = clusters.get(fingerprint) ?? {fingerprint_sha256: fingerprint, key, occurrences: 0, stages: new Set(), sources: new Set()};
        existing.occurrences += 1;
        existing.stages.add(stage.id);
        existing.sources.add(source.path);
        clusters.set(fingerprint, existing);
      }
    }
  }
  return [...clusters.values()].map((cluster) => ({...cluster, stages: [...cluster.stages].sort(), sources: [...cluster.sources].sort()})).sort((left, right) => right.occurrences - left.occurrences || left.fingerprint_sha256.localeCompare(right.fingerprint_sha256));
}

function stageFingerprint(stage, inputs, dependencyReceipts) {
  return sha256Value({
    id: stage.id,
    kind: stage.kind,
    command: stage.command,
    args: stage.args,
    cwd: stage.cwd,
    timeout_ms: stage.timeout_ms,
    inputs,
    dependencies: dependencyReceipts,
    outputs: stage.outputs,
    verdict: stage.verdict,
  });
}

function initialState(plan, planSha256) {
  return {schema: STATE_SCHEMA, run_id: plan.run_id, branch: plan.branch, mode: plan.mode, plan_sha256: planSha256, candidate: plan.candidate, status: 'running', started_at: new Date().toISOString(), updated_at: new Date().toISOString(), stages: {}};
}

function buildSummary(plan, state, clusters) {
  const receipts = plan.stages.map((stage) => state.stages[stage.id]).filter(Boolean);
  const count = (status) => receipts.filter((receipt) => receipt.status === status).length;
  return {
    schema: SUMMARY_SCHEMA,
    run_id: plan.run_id,
    branch: plan.branch,
    mode: plan.mode,
    candidate: plan.candidate,
    frozen_manifest: state.stages[plan.stages[0].id]?.outputs?.[0] ?? null,
    plan_sha256: state.plan_sha256,
    status: state.status,
    controller_error: state.controller_error ?? null,
    started_at: state.started_at,
    finished_at: state.finished_at ?? null,
    stages: {total: plan.stages.length, passed: count('pass'), resumed: receipts.filter((receipt) => receipt.resumed === true).length, failed: count('fail'), pending: plan.stages.length - receipts.length},
    failure_cluster_count: clusters.length,
    failure_clusters: clusters,
    stage_receipts: receipts,
  };
}

export async function runCompletionPlan({planPath, statePath, summaryPath, resume = true, quiet = false, commandRunner = runMeasuredCommand}) {
  const absolutePlanPath = path.resolve(planPath);
  const absoluteStatePath = path.resolve(statePath);
  const absoluteSummaryPath = path.resolve(summaryPath);
  const planBytes = await fs.readFile(absolutePlanPath);
  const planSha256 = sha256Value(planBytes);
  const plan = validatePlan(JSON.parse(planBytes.toString('utf8')), {planPath: absolutePlanPath});
  for (const stage of plan.stages) {
    for (const output of stage.outputs) {
      if ([absolutePlanPath, absoluteStatePath, absoluteSummaryPath, plan.ledger_path].includes(output)) throw new Error(`stage ${stage.id} cannot own controller input or bookkeeping path: ${output}`);
    }
  }
  const releaseLock = await acquireLock(absoluteStatePath);
  try {
    let state = initialState(plan, planSha256);
    if (resume) {
      try {
        const previous = await readJson(absoluteStatePath);
        if (previous.schema === STATE_SCHEMA && previous.run_id === plan.run_id && previous.plan_sha256 === planSha256) state = {...previous, status: 'running', updated_at: new Date().toISOString(), finished_at: undefined};
      } catch (error) {
        if (error.code !== 'ENOENT') throw error;
      }
    }
    await writeJsonAtomic(absoluteStatePath, state);

    for (const stage of plan.stages) {
      const dependencies = stage.depends_on.map((id) => state.stages[id]);
      if (dependencies.some((receipt) => receipt?.status !== 'pass')) throw new Error(`stage ${stage.id} has an incomplete dependency`);
      let inputs;
      try {
        inputs = await hashPaths(stage.inputs);
      } catch (error) {
        state.stages[stage.id] = {id: stage.id, kind: stage.kind, status: 'fail', resumed: false, started_at: new Date().toISOString(), finished_at: new Date().toISOString(), reason: error.message};
        state.status = 'fail';
        break;
      }
      const dependencyReceipts = dependencies.map((receipt) => ({id: receipt.id, fingerprint_sha256: receipt.fingerprint_sha256, outputs: receipt.outputs}));
      const fingerprint = stageFingerprint(stage, inputs, dependencyReceipts);
      const previous = state.stages[stage.id];
      const resumed = resume ? await resumableReceipt(previous, fingerprint, stage) : null;
      if (resumed !== null) {
        state.stages[stage.id] = {...previous, resumed: true, resumed_at: new Date().toISOString(), outputs: resumed.outputs, verdict: resumed.verdict};
        state.updated_at = new Date().toISOString();
        await writeJsonAtomic(absoluteStatePath, state);
        continue;
      }

      const startedAt = new Date().toISOString();
      let execution;
      try {
        execution = await commandRunner({family: `completion:${plan.branch}`, label: stage.id, command: stage.command, args: stage.args, cwd: stage.cwd, ledgerPath: plan.ledger_path, timeoutMs: stage.timeout_ms, quiet});
      } catch (error) {
        execution = {exitCode: 1, record: null, error};
      }
      let outputs = [];
      let verdict = null;
      let reason = null;
      if (execution.exitCode !== 0) reason = `command exited ${execution.exitCode}${execution.error ? `: ${execution.error.message}` : ''}`;
      if (reason === null) {
        try {
          outputs = await hashPaths(stage.outputs);
          verdict = await checkVerdict(stage.verdict);
          if (!verdict.passed) reason = `verdict ${verdict.path} reported ${JSON.stringify(verdict.value)}`;
        } catch (error) {
          reason = error.message;
        }
      }
      state.stages[stage.id] = {id: stage.id, kind: stage.kind, status: reason === null ? 'pass' : 'fail', resumed: false, fingerprint_sha256: fingerprint, inputs, outputs, verdict, command_run_id: execution.record?.runId ?? null, command_ledger: execution.record?.artifactPaths?.ledger ?? plan.ledger_path, started_at: startedAt, finished_at: new Date().toISOString(), reason};
      state.updated_at = new Date().toISOString();
      if (reason !== null) {
        state.status = 'fail';
        await writeJsonAtomic(absoluteStatePath, state);
        break;
      }
      await writeJsonAtomic(absoluteStatePath, state);
    }

    if (state.status !== 'fail') state.status = plan.stages.every((stage) => state.stages[stage.id]?.status === 'pass') ? 'pass' : 'fail';
    state.finished_at = new Date().toISOString();
    state.updated_at = state.finished_at;
    await writeJsonAtomic(absoluteStatePath, state);
    let clusters = [];
    try {
      clusters = await collectClusters(plan.stages.filter((stage) => state.stages[stage.id]?.status === 'pass'));
    } catch (error) {
      state.status = 'fail';
      state.controller_error = `failure cluster reduction failed: ${error.message}`;
      state.finished_at = new Date().toISOString();
      state.updated_at = state.finished_at;
      await writeJsonAtomic(absoluteStatePath, state);
    }
    const summary = buildSummary(plan, state, clusters);
    await writeJsonAtomic(absoluteSummaryPath, summary);
    return summary;
  } finally {
    await releaseLock();
  }
}
