#!/usr/bin/env node
// V1 import-health report (agent-runnable): parse an
// apply-corpus-import-manifest.mjs log and emit a verdict JSON. Exit codes:
// 0 all rows done-or-classified (and >= --threshold if given),
// 1 import rate below threshold, 2 unclassified failure states.
//
// Usage:
//   import-health-report.mjs --log <import.log> --output <verdict.json> \
//     [--run-id <id>] [--family EN] [--threshold 0.95]

import fs from 'node:fs';

import {runCliIfMain} from '../src/cli-entry.mjs';

import { applyThreshold, buildImportHealthVerdict, parseImportLog } from '../src/import-health.mjs';

export function parseArgs(argv) {
  const args = { log: null, output: null, runId: null, family: 'EN', threshold: null };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => argv[++i];
    if (arg === '--log') args.log = next();
    else if (arg === '--output') args.output = next();
    else if (arg === '--run-id') args.runId = next();
    else if (arg === '--family') args.family = next();
    else if (arg === '--threshold') args.threshold = Number(next());
    else if (arg === '--help' || arg === '-h') return {help: true};
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!args.log) throw new Error('--log is required');
  if (!args.output) throw new Error('--output is required');
  return args;
}

export function usage() {
  return 'Usage: import-health-report.mjs --log <import.log> --output <verdict.json> [--run-id id] [--family EN] [--threshold 0.95]';
}

export function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const { rows, summary } = parseImportLog(fs.readFileSync(args.log, 'utf8'));
  const runId = args.runId ?? `v1-import-run-${summary?.import_run_id ?? 'unknown'}`;
  const { verdict, exitCode } = buildImportHealthVerdict({
    runId,
    family: args.family,
    rows,
    summary,
  });
  fs.writeFileSync(args.output, JSON.stringify(verdict, null, 1));
  console.log(JSON.stringify({ ...verdict.aggregate, run_id: runId }, null, 2));
  return exitCode !== 0 ? exitCode : applyThreshold(verdict, args.threshold);
}

await runCliIfMain(import.meta.url, main);
