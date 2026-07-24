#!/usr/bin/env node
import {readFile} from "node:fs/promises";
import {runCliIfMain} from "../src/cli-entry.mjs";

import {
  buildTmuxAttachCommand,
  buildTmuxResetCommand,
  dispatchGridAssignmentToTmux,
} from "../src/tmux-dispatcher.mjs";

export function parseArgs(argv) {
  const args = {
    command: null,
    sessionName: "wj-codex-grid",
    window: "0",
    pane: "0",
    dryRun: false,
    confirmReset: false,
  };
  if (argv[0] === "--help" || argv[0] === "-h") {
    args.help = true;
    return args;
  }
  args.command = argv[0] ?? null;

  for (let i = 1; i < argv.length; i += 1) {
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
    } else if (arg === "--assignment") {
      args.assignmentPath = next;
      i += 1;
    } else if (arg === "--session") {
      args.sessionName = next;
      i += 1;
    } else if (arg === "--window") {
      args.window = next;
      i += 1;
    } else if (arg === "--pane") {
      args.pane = next;
      i += 1;
    } else if (arg === "--dry-run") {
      args.dryRun = true;
    } else if (arg === "--confirm-reset") {
      args.confirmReset = true;
    } else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return args;
}

export function usage() {
  return [
    "Usage:",
    "  node scripts/wj-grid-dispatch.mjs enqueue --state-root <dir> --campaign-id <id> --lane <n> --assignment <json> [--session wj-codex-grid] [--window 0] [--pane 0] [--dry-run]",
    "  node scripts/wj-grid-dispatch.mjs attach [--session wj-codex-grid]",
    "  node scripts/wj-grid-dispatch.mjs reset --confirm-reset [--session wj-codex-grid]",
  ].join("\n");
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help || !args.command) {
    console.log(usage());
    return 0;
  }

  if (args.command === "attach") {
    console.log(JSON.stringify({command: buildTmuxAttachCommand({sessionName: args.sessionName})}, null, 2));
    return 0;
  }

  if (args.command === "reset") {
    console.log(
      JSON.stringify({command: buildTmuxResetCommand({sessionName: args.sessionName, confirmReset: args.confirmReset})}, null, 2),
    );
    return 0;
  }

  if (args.command !== "enqueue") {
    throw new Error(usage());
  }
  if (!args.stateRoot || !args.campaignId || !Number.isInteger(args.lane) || !args.assignmentPath) {
    throw new Error(usage());
  }

  const assignment = JSON.parse(await readFile(args.assignmentPath, "utf8"));
  const result = await dispatchGridAssignmentToTmux({
    stateRoot: args.stateRoot,
    campaignId: args.campaignId,
    lane: args.lane,
    assignment,
    sessionName: args.sessionName,
    window: args.window,
    pane: args.pane,
    dryRun: args.dryRun,
  });
  console.log(JSON.stringify(result, null, 2));
  return 0;
}

await runCliIfMain(import.meta.url, main);
