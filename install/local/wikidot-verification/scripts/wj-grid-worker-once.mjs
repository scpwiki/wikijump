#!/usr/bin/env node
import {writeFile} from "node:fs/promises";
import {runCliIfMain} from "../src/cli-entry.mjs";

import {runLaneWorkerOnce, writeCodexResultArtifact} from "../src/grid-worker.mjs";

export function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = argv[i + 1];
    if (arg === "--state-root") {
      args.stateRoot = next;
      i += 1;
    } else if (arg === "--campaign-id") {
      args.campaignId = next;
      i += 1;
    } else if (arg === "--lane") {
      args.lane = Number.parseInt(next, 10);
      i += 1;
    } else if (arg === "--executor") {
      args.executor = next;
      i += 1;
    } else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return args;
}

export function usage() {
  return "Usage: node scripts/wj-grid-worker-once.mjs --state-root <dir> --campaign-id <id> --lane <n> [--executor loopback]";
}

async function executeAssignment({assignment, artifactRoot, logPath}) {
  await writeFile(logPath, `dispatcher worker completed ${assignment.assignment_id}\n`);
  await writeCodexResultArtifact({
    artifactRoot,
    assignment,
    reportText: `# ${assignment.title}\n\nDispatcher proof completed.\n`,
    validation: [
      {
        command_id: "dispatcher-worker",
        exit_code: 0,
        claim: "dispatcher_worker_passed",
      },
    ],
  });
  return {exit_code: 0, stop_code: null};
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  if (!args.stateRoot || !args.campaignId || !Number.isInteger(args.lane)) {
    throw new Error(usage());
  }
  const status = await runLaneWorkerOnce({
    stateRoot: args.stateRoot,
    campaignId: args.campaignId,
    lane: args.lane,
    executeAssignment,
  });
  console.log(JSON.stringify(status, null, 2));
  return 0;
}

await runCliIfMain(import.meta.url, main);
