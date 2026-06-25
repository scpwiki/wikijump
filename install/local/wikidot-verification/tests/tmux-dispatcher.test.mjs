import assert from "node:assert/strict";
import {execFile} from "node:child_process";
import {mkdtemp, rm} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";
import {promisify} from "node:util";

import {
  enqueueGridAssignment,
  initializeGridCampaign,
  readGridEvents,
  reconstructGridState,
  runLaneWorkerOnce,
  writeCodexResultArtifact,
} from "../src/grid-worker.mjs";
import {
  DEFAULT_REPO_ROOT,
  buildLaneWorkerShellCommand,
  buildTmuxAttachCommand,
  buildTmuxResetCommand,
  dispatchGridAssignmentToTmux,
} from "../src/tmux-dispatcher.mjs";

const execFileAsync = promisify(execFile);
const DISPATCH_CLI = fileURLToPath(new URL("../scripts/wj-grid-dispatch.mjs", import.meta.url));
const WORKER_CLI = fileURLToPath(new URL("../scripts/wj-grid-worker-once.mjs", import.meta.url));

async function temporaryDirectory(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "wikijump-tmux-dispatcher-"));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return directory;
}

function assignment(id, overrides = {}) {
  return {
    assignment_id: `${id}-r1-a1`,
    task_id: id,
    title: `task ${id}`,
    route: "gpt-5.3-codex-spark",
    repository: "Rokurolize/wikijump",
    base_sha: "1672120d758755382ae3e9c174c49e5ee1cd543b",
    expected_artifacts: ["result.json", "manifest.json", "report.md", "commands.jsonl"],
    ...overrides,
  };
}

async function successfulExecutor({assignment: currentAssignment, artifactRoot, logPath}) {
  await writeCodexResultArtifact({
    artifactRoot,
    assignment: currentAssignment,
    reportText: `# ${currentAssignment.title}\n\nDispatcher test artifact.\n`,
    validation: [
      {
        command_id: "dispatch-test",
        exit_code: 0,
        claim: "dispatcher_test_passed",
      },
    ],
  });
  await import("node:fs/promises").then(({writeFile}) =>
    writeFile(logPath, `completed ${currentAssignment.assignment_id}\n`),
  );
  return {exit_code: 0, stop_code: null};
}

test("builds attach and reset commands without making reset the default path", () => {
  assert.deepEqual(buildTmuxAttachCommand(), ["tmux", "attach", "-t", "wj-codex-grid"]);
  assert.throws(() => buildTmuxResetCommand(), /confirmReset=true/);
  assert.deepEqual(buildTmuxResetCommand({confirmReset: true}), [
    "tmux",
    "kill-session",
    "-t",
    "wj-codex-grid",
  ]);
});

test("dispatches an assignment to a stable tmux pane without killing the session", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-ops-010";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});

  const calls = [];
  const result = await dispatchGridAssignmentToTmux({
    stateRoot,
    campaignId,
    lane: 1,
    assignment: assignment("WJ-OPS-010A"),
    repoRoot: "/tmp/unused-repo-root",
    tmuxRun: async (argv) => {
      calls.push(argv);
      return {stdout: "", stderr: ""};
    },
  });

  assert.equal(result.assignment.assignment_id, "WJ-OPS-010A-r1-a1");
  assert.equal(result.tmux.target, "wj-codex-grid:0.0");
  assert.deepEqual(calls, [result.tmux.command]);
  assert.equal(calls[0][0], "tmux");
  assert.equal(calls[0][1], "send-keys");
  assert.ok(!calls.flat().includes("kill-session"));
  assert.ok(result.tmux.worker_command.includes("wj-grid-worker-once.mjs"));
  assert.ok(
    result.tmux.worker_command.includes(
      path.join("/tmp/unused-repo-root", "install/local/wikidot-verification/scripts/wj-grid-worker-once.mjs"),
    ),
  );

  const events = await readGridEvents({stateRoot, campaignId});
  assert.equal(events.filter((event) => event.event === "ASSIGNED").length, 1);
  assert.equal(events.filter((event) => event.event === "TMUX_DISPATCHED").length, 1);
});

test("dispatch CLI handles top-level help", async () => {
  const {stdout} = await execFileAsync(process.execPath, [DISPATCH_CLI, "--help"]);
  assert.match(stdout, /Usage:/);
  assert.match(stdout, /enqueue/);
});

test("worker CLI accepts dispatcher-emitted executor flag", async () => {
  const {stdout} = await execFileAsync(process.execPath, [WORKER_CLI, "--help"]);
  assert.match(stdout, /--executor/);
  const failure = await execFileAsync(process.execPath, [WORKER_CLI, "--executor", "loopback"]).catch(
    (error) => error,
  );
  assert.equal(failure.code, 1);
  assert.match(failure.stderr, /Usage:/);
});

test("relative state roots are resolved before enqueueing and sending to tmux", async (t) => {
  const repoRoot = await temporaryDirectory(t);
  const campaignId = "wj-ops-010-relative-state";

  const result = await dispatchGridAssignmentToTmux({
    stateRoot: "relative-state",
    campaignId,
    lane: 1,
    assignment: assignment("WJ-OPS-010REL"),
    repoRoot,
    dryRun: true,
  });

  const expectedRoot = path.join(repoRoot, "relative-state");
  assert.equal(result.path, path.join(expectedRoot, "campaigns", campaignId, "lanes", "lane-01", "inbox", "WJ-OPS-010REL-r1-a1.json"));
  assert.ok(result.tmux.worker_command.includes(expectedRoot));
  const events = await readGridEvents({stateRoot: expectedRoot, campaignId});
  assert.equal(events.filter((event) => event.event === "TMUX_DISPATCH_DRY_RUN").length, 1);
});

test("dispatch dry-run records the command but does not call tmux", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-ops-010-dry-run";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});

  let called = false;
  const result = await dispatchGridAssignmentToTmux({
    stateRoot,
    campaignId,
    lane: 1,
    assignment: assignment("WJ-OPS-010DRY"),
    dryRun: true,
    tmuxRun: async () => {
      called = true;
    },
  });

  assert.equal(called, false);
  assert.equal(result.tmux.dry_run, true);
  const events = await readGridEvents({stateRoot, campaignId});
  assert.equal(events.filter((event) => event.event === "TMUX_DISPATCH_DRY_RUN").length, 1);
});

test("records a failed tmux send without claiming dispatch success", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-ops-010-send-fail";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});

  await assert.rejects(
    dispatchGridAssignmentToTmux({
      stateRoot,
      campaignId,
      lane: 1,
      assignment: assignment("WJ-OPS-010FAIL"),
      tmuxRun: async () => {
        throw new Error("missing tmux pane");
      },
    }),
    /missing tmux pane/,
  );

  const events = await readGridEvents({stateRoot, campaignId});
  assert.equal(events.filter((event) => event.event === "TMUX_DISPATCHED").length, 0);
  const failures = events.filter((event) => event.event === "TMUX_DISPATCH_FAILED");
  assert.equal(failures.length, 1);
  assert.equal(failures[0].error, "missing tmux pane");
});

test("uses existing grid helper semantics for sequential pane reuse", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-ops-010-reuse";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});

  const calls = [];
  for (const taskId of ["WJ-OPS-010A", "WJ-OPS-010B"]) {
    await dispatchGridAssignmentToTmux({
      stateRoot,
      campaignId,
      lane: 1,
      assignment: assignment(taskId),
      tmuxRun: async (argv) => {
        calls.push(argv);
        return {stdout: "", stderr: ""};
      },
    });
    const status = await runLaneWorkerOnce({
      stateRoot,
      campaignId,
      lane: 1,
      executeAssignment: successfulExecutor,
    });
    assert.equal(status.state, "DONE_REUSABLE");
    assert.equal(status.task_id, taskId);
  }

  assert.equal(calls.length, 2);
  assert.deepEqual(calls.map((call) => call.slice(0, 3)), [
    ["tmux", "send-keys", "-t"],
    ["tmux", "send-keys", "-t"],
  ]);
  assert.ok(calls.every((call) => call[3] === "wj-codex-grid:0.0"));

  const state = await reconstructGridState({stateRoot, campaignId, laneCount: 1});
  assert.equal(state.lanes[0].state, "DONE_REUSABLE");
  assert.equal(state.events.filter((event) => event.event === "DONE_REUSABLE").length, 2);
  assert.equal(state.events.filter((event) => event.event === "ARTIFACT_SCHEMA_PASS").length, 2);
});

test("quotes lane worker command arguments for tmux send-keys", () => {
  const command = buildLaneWorkerShellCommand({
    stateRoot: "/tmp/state root",
    campaignId: "wj-ops-010",
    lane: 7,
    workerScript: "scripts/worker one.mjs",
    executor: "loopback",
  });
  assert.match(command, /'\/tmp\/state root'/);
  assert.match(command, /'scripts\/worker one\.mjs'/);
  assert.match(command, /'7'/);
});
