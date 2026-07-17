#!/usr/bin/env node
import path from 'node:path';
import process from 'node:process';
import {fileURLToPath} from 'node:url';

import {runCompletionPlan} from '../src/completion-controller.mjs';

const SCRIPT_PATH = fileURLToPath(import.meta.url);

export function parseArgs(argv) {
  const args = {resume: true, quiet: false};
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (['--plan', '--state', '--summary'].includes(flag)) {
      const value = argv[index + 1];
      if (!value || value.startsWith('--')) throw new Error(`${flag} requires a value`);
      args[flag.slice(2)] = path.resolve(value);
      index += 1;
    } else if (flag === '--no-resume') args.resume = false;
    else if (flag === '--quiet') args.quiet = true;
    else if (flag === '--help' || flag === '-h') args.help = true;
    else throw new Error(`unknown argument: ${flag}`);
  }
  if (!args.help) for (const required of ['plan', 'state', 'summary']) if (!args[required]) throw new Error(`--${required} is required`);
  return args;
}

function usage() {
  return 'Usage: run-completion-controller.mjs --plan PLAN.json --state STATE.json --summary SUMMARY.json [--no-resume] [--quiet]';
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const summary = await runCompletionPlan({planPath: args.plan, statePath: args.state, summaryPath: args.summary, resume: args.resume, quiet: args.quiet});
  console.log(JSON.stringify({status: summary.status, run_id: summary.run_id, branch: summary.branch, stages: summary.stages, failure_cluster_count: summary.failure_cluster_count, summary: args.summary}));
  return summary.status === 'pass' ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  main(process.argv.slice(2)).then((code) => { process.exitCode = code; }).catch((error) => { console.error(error.message); process.exitCode = 1; });
}
