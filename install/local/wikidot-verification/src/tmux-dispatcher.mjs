import {execFile} from "node:child_process";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {promisify} from "node:util";

import {appendGridEvent, enqueueGridAssignment} from "./grid-worker.mjs";

const execFileAsync = promisify(execFile);
const SAFE_TMUX_TARGET_RE = /^[A-Za-z0-9_.:-]+$/;

export const DEFAULT_TMUX_SESSION = "wj-codex-grid";
export const DEFAULT_WORKER_SCRIPT = "install/local/wikidot-verification/scripts/wj-grid-worker-once.mjs";
export const DEFAULT_REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");

function assertTmuxTarget(value, name) {
  if (typeof value !== "string" || !SAFE_TMUX_TARGET_RE.test(value)) {
    throw new Error(`${name} must be a safe tmux target`);
  }
}

export function shellQuote(value) {
  const text = String(value);
  if (text.length === 0) {
    return "''";
  }
  return `'${text.replaceAll("'", "'\\''")}'`;
}

export function buildTmuxTarget({sessionName = DEFAULT_TMUX_SESSION, window = "0", pane = "0"} = {}) {
  assertTmuxTarget(sessionName, "sessionName");
  assertTmuxTarget(window, "window");
  assertTmuxTarget(pane, "pane");
  return `${sessionName}:${window}.${pane}`;
}

export function buildTmuxAttachCommand({sessionName = DEFAULT_TMUX_SESSION} = {}) {
  assertTmuxTarget(sessionName, "sessionName");
  return ["tmux", "attach", "-t", sessionName];
}

export function buildTmuxResetCommand({sessionName = DEFAULT_TMUX_SESSION, confirmReset = false} = {}) {
  assertTmuxTarget(sessionName, "sessionName");
  if (confirmReset !== true) {
    throw new Error("Refusing destructive reset without confirmReset=true");
  }
  return ["tmux", "kill-session", "-t", sessionName];
}

export function buildLaneWorkerShellCommand({
  stateRoot,
  campaignId,
  lane,
  workerScript = DEFAULT_WORKER_SCRIPT,
  executor = "codex",
  nodeBin = "node",
} = {}) {
  if (typeof stateRoot !== "string" || stateRoot.length === 0) {
    throw new TypeError("stateRoot must be a non-empty string");
  }
  if (typeof campaignId !== "string" || campaignId.length === 0) {
    throw new TypeError("campaignId must be a non-empty string");
  }
  if (!Number.isInteger(lane) || lane < 1) {
    throw new Error("lane must be a positive integer");
  }
  return [
    nodeBin,
    workerScript,
    "--state-root",
    stateRoot,
    "--campaign-id",
    campaignId,
    "--lane",
    String(lane),
    "--executor",
    executor,
  ]
    .map(shellQuote)
    .join(" ");
}

export function buildTmuxSendKeysCommand({target, command}) {
  if (typeof command !== "string" || command.length === 0) {
    throw new TypeError("command must be a non-empty string");
  }
  assertTmuxTarget(target, "target");
  return ["tmux", "send-keys", "-t", target, command, "C-m"];
}

async function defaultTmuxRun(argv) {
  const [command, ...args] = argv;
  return execFileAsync(command, args, {timeout: 30_000});
}

export async function dispatchGridAssignmentToTmux({
  stateRoot,
  campaignId,
  lane,
  assignment,
  sessionName = DEFAULT_TMUX_SESSION,
  window = "0",
  pane = "0",
  workerScript = DEFAULT_WORKER_SCRIPT,
  executor = "codex",
  nodeBin = "node",
  dryRun = false,
  tmuxRun = defaultTmuxRun,
  repoRoot = DEFAULT_REPO_ROOT,
} = {}) {
  const resolvedStateRoot = resolvePathFromRepoRoot(repoRoot, stateRoot);
  const queued = await enqueueGridAssignment({stateRoot: resolvedStateRoot, campaignId, lane, assignment});
  const target = buildTmuxTarget({sessionName, window, pane});
  const workerCommand = buildLaneWorkerShellCommand({
    stateRoot: resolvedStateRoot,
    campaignId,
    lane,
    workerScript: resolveWorkerScriptFromRepoRoot(repoRoot, workerScript),
    executor,
    nodeBin,
  });
  const tmuxCommand = buildTmuxSendKeysCommand({target, command: workerCommand});

  if (dryRun) {
    await appendGridEvent({
      stateRoot: resolvedStateRoot,
      campaignId,
      event: "TMUX_DISPATCH_DRY_RUN",
      lane,
      task_id: queued.assignment.task_id,
      assignment_id: queued.assignment.assignment_id,
      tmux_target: target,
      worker_command: workerCommand,
    });
  } else {
    try {
      await tmuxRun(tmuxCommand);
    } catch (error) {
      await appendGridEvent({
        stateRoot: resolvedStateRoot,
        campaignId,
        event: "TMUX_DISPATCH_FAILED",
        lane,
        task_id: queued.assignment.task_id,
        assignment_id: queued.assignment.assignment_id,
        tmux_target: target,
        worker_command: workerCommand,
        error: error.message,
      });
      throw error;
    }
    await appendGridEvent({
      stateRoot: resolvedStateRoot,
      campaignId,
      event: "TMUX_DISPATCHED",
      lane,
      task_id: queued.assignment.task_id,
      assignment_id: queued.assignment.assignment_id,
      tmux_target: target,
      worker_command: workerCommand,
    });
  }

  return {
    ...queued,
    tmux: {
      target,
      command: tmuxCommand,
      worker_command: workerCommand,
      dry_run: dryRun,
      attach_command: buildTmuxAttachCommand({sessionName}),
    },
  };
}

export function resolvePathFromRepoRoot(repoRoot, inputPath) {
  if (path.isAbsolute(inputPath)) {
    return inputPath;
  }
  return path.resolve(repoRoot, inputPath);
}

export function resolveWorkerScriptFromRepoRoot(repoRoot, workerScript = DEFAULT_WORKER_SCRIPT) {
  return resolvePathFromRepoRoot(repoRoot, workerScript);
}
