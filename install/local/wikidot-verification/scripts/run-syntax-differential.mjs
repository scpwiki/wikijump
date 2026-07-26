#!/usr/bin/env node

import fs from 'node:fs';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {once} from 'node:events';

import {runCliIfMain} from '../src/cli-entry.mjs';
import {
  aggregateSyntaxComparisons,
  compareSyntaxReference,
  ftmlInputFromReference,
  validateWikidotReference,
} from '../src/syntax-differential.mjs';

export function usage() {
  return 'Usage: run-syntax-differential.mjs --references FILE --renderer COMMAND ' +
    '[--renderer-arg ARG ...] [--timeout-ms N] --output FILE';
}

function optionValue(argv, index, name) {
  const value = argv[index + 1];
  if (value == null || value.startsWith('--')) throw new Error(`${name} requires a value`);
  return value;
}

export function parseArgs(argv) {
  const args = {
    references: null,
    renderer: null,
    rendererArgs: [],
    output: null,
    timeoutMs: 30_000,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--references') args.references = optionValue(argv, index++, arg);
    else if (arg === '--renderer') args.renderer = optionValue(argv, index++, arg);
    else if (arg === '--renderer-arg') args.rendererArgs.push(optionValue(argv, index++, arg));
    else if (arg === '--timeout-ms') args.timeoutMs = Number(optionValue(argv, index++, arg));
    else if (arg === '--output') args.output = optionValue(argv, index++, arg);
    else if (arg === '--help' || arg === '-h') return {help: true};
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!args.references) throw new Error('--references is required');
  if (!args.renderer) throw new Error('--renderer is required');
  if (!args.output) throw new Error('--output is required');
  if (!Number.isSafeInteger(args.timeoutMs) || args.timeoutMs <= 0) {
    throw new Error('--timeout-ms must be a positive integer');
  }
  return args;
}

function readJsonLines(path) {
  return fs
    .readFileSync(path, 'utf8')
    .split('\n')
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line));
}

function rendererIdentity(command, commandArgs) {
  let sha256 = null;
  try {
    sha256 = createHash('sha256').update(fs.readFileSync(command)).digest('hex');
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  return {command, arguments: commandArgs, sha256};
}

export async function renderCases(
  references,
  command,
  commandArgs = [],
  timeoutMs = 30_000,
  inputFromReference = ftmlInputFromReference,
) {
  const child = spawn(command, commandArgs, {stdio: ['pipe', 'pipe', 'pipe']});
  let stdout = '';
  let stderr = '';
  let stdinError = null;
  let timedOut = false;
  let killTimer = null;
  child.stdout.setEncoding('utf8');
  child.stderr.setEncoding('utf8');
  child.stdout.on('data', (chunk) => {
    stdout += chunk;
  });
  child.stderr.on('data', (chunk) => {
    stderr += chunk;
  });
  child.stdin.on('error', (error) => {
    stdinError = error;
  });
  const close = new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('close', resolve);
  });
  const timeout = setTimeout(() => {
    timedOut = true;
    child.kill('SIGTERM');
    killTimer = setTimeout(() => child.kill('SIGKILL'), 1000);
  }, timeoutMs);
  try {
    for (const reference of references) {
      if (!child.stdin.write(`${JSON.stringify(inputFromReference(reference))}\n`)) {
        await Promise.race([once(child.stdin, 'drain'), close]);
      }
      if (child.exitCode !== null) break;
    }
    child.stdin.end();
    const exitCode = await close;
    if (timedOut || exitCode !== 0 || stdinError !== null) {
      let detail = `FTML renderer exited ${exitCode}: ${stderr.trim().slice(0, 500)}`;
      if (stdinError !== null) detail = `FTML renderer input failed: ${stdinError.message}`;
      if (timedOut) detail = `FTML renderer exceeded ${timeoutMs} ms`;
      throw new Error(detail);
    }
  } finally {
    clearTimeout(timeout);
    clearTimeout(killTimer);
  }
  const results = stdout
    .split('\n')
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line));
  if (results.length !== references.length) {
    throw new Error(
      `FTML renderer returned ${results.length} results for ${references.length} cases`,
    );
  }
  return results;
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const references = readJsonLines(args.references).map(validateWikidotReference);
  const caseIds = references.map((reference) => reference.syntax_case.case_id);
  if (caseIds.length === 0 || new Set(caseIds).size !== caseIds.length) {
    throw new Error('Wikidot references must contain unique syntax cases');
  }
  const ftmlReferences = references.filter(
    (reference) => reference.syntax_case.local_execution_tier === 'ftml',
  );
  const renderResults = await renderCases(
    ftmlReferences,
    args.renderer,
    args.rendererArgs,
    args.timeoutMs,
  );
  const byCaseId = new Map(renderResults.map((result) => [result.case_id, result]));
  if (byCaseId.size !== renderResults.length) throw new Error('FTML renderer returned duplicate case IDs');
  const expectedCaseIds = new Set(
    ftmlReferences.map((reference) => reference.syntax_case.case_id),
  );
  if (
    renderResults.length !== expectedCaseIds.size ||
    renderResults.some((result) => !expectedCaseIds.has(result.case_id))
  ) {
    throw new Error('FTML renderer results do not match the requested syntax cases');
  }
  const comparisons = references.map((reference) => {
    if (reference.syntax_case.local_execution_tier !== 'ftml') {
      return {
        schema: 'wikijump_syntax_differential.syntax_comparison.v1',
        case_id: reference.syntax_case.case_id,
        status: 'not-applicable',
        detail: `local execution tier ${reference.syntax_case.local_execution_tier} is outside the FTML runner`,
      };
    }
    return compareSyntaxReference(reference, byCaseId.get(reference.syntax_case.case_id));
  });
  const verdict = aggregateSyntaxComparisons(
    comparisons,
    rendererIdentity(args.renderer, args.rendererArgs),
  );
  fs.writeFileSync(args.output, `${JSON.stringify(verdict, null, 2)}\n`, {flag: 'wx'});
  console.log(JSON.stringify(verdict.summary));
  return verdict.summary['runner-error'] > 0 ? 2 : verdict.summary.mismatch > 0 ? 1 : 0;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
