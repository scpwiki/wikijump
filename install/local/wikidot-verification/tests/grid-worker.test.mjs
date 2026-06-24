import assert from "node:assert/strict";
import {mkdtemp, readFile, readdir, rm, writeFile} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  enqueueGridAssignment,
  initializeGridCampaign,
  readGridEvents,
  reconstructGridState,
  runLaneWorkerOnce,
  writeCodexResultArtifact,
} from "../src/grid-worker.mjs";

async function temporaryDirectory(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "wikijump-grid-worker-"));
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
  await writeFile(
    logPath,
    "worker saw prompt text mentioning active_goal_route_repair_required but no structured stop\n",
  );
  await writeCodexResultArtifact({
    artifactRoot,
    assignment: currentAssignment,
    validation: [
      {
        command_id: "cmd-001",
        exit_code: 0,
        claim: "executed_pass",
      },
    ],
  });
  return {exit_code: 0, stop_code: null};
}

test("reuses one lane for three sequential assignments with separate artifacts", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-next-20260624";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});

  for (const taskId of ["WJ-OPS-002A", "WJ-OPS-002B", "WJ-OPS-002C"]) {
    await enqueueGridAssignment({
      stateRoot,
      campaignId,
      lane: 1,
      assignment: assignment(taskId),
    });
    const status = await runLaneWorkerOnce({
      stateRoot,
      campaignId,
      lane: 1,
      executeAssignment: successfulExecutor,
    });
    assert.equal(status.state, "DONE_REUSABLE");
    assert.equal(status.task_id, taskId);
    assert.match(status.artifact_root, new RegExp(`${taskId}-r1-a1/artifacts$`));
  }

  const reconstructed = await reconstructGridState({stateRoot, campaignId, laneCount: 1});
  assert.equal(reconstructed.lanes[0].state, "DONE_REUSABLE");
  assert.equal(reconstructed.events.filter((event) => event.event === "DONE_REUSABLE").length, 3);
  assert.equal(reconstructed.events.filter((event) => event.event === "ARTIFACT_SCHEMA_PASS").length, 3);
});

test("quarantines invalid artifacts instead of marking the lane reusable", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-schema-invalid";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});
  await enqueueGridAssignment({
    stateRoot,
    campaignId,
    lane: 1,
    assignment: assignment("WJ-OPS-BAD"),
  });

  const status = await runLaneWorkerOnce({
    stateRoot,
    campaignId,
    lane: 1,
    executeAssignment: async ({artifactRoot}) => {
      await writeFile(path.join(artifactRoot, "result.json"), "{not json\n");
      return {exit_code: 0, stop_code: null};
    },
  });

  assert.equal(status.state, "SCHEMA_INVALID");
  const events = await readGridEvents({stateRoot, campaignId});
  assert.ok(events.some((event) => event.event === "ARTIFACT_SCHEMA_FAIL"));
});

test("rejects malformed inbox JSON before it can block a lane", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-reject-json";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});
  const inbox = path.join(stateRoot, "campaigns", campaignId, "lanes", "lane-01", "inbox");
  await writeFile(path.join(inbox, "bad.json"), "{not json\n");

  const status = await runLaneWorkerOnce({
    stateRoot,
    campaignId,
    lane: 1,
    executeAssignment: successfulExecutor,
  });

  assert.equal(status.state, "BLOCKED_INPUT");
  assert.match(status.rejected_path, /rejected/);
  assert.deepEqual(await readdir(inbox), []);
});

test("rejects unsafe assignment records before path construction", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-reject-unsafe";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});
  const inbox = path.join(stateRoot, "campaigns", campaignId, "lanes", "lane-01", "inbox");
  await writeFile(
    path.join(inbox, "bad.json"),
    `${JSON.stringify({...assignment("WJ-OPS-BAD"), campaign_id: campaignId, lane: 1, assignment_id: "../escape"})}\n`,
  );

  const status = await runLaneWorkerOnce({
    stateRoot,
    campaignId,
    lane: 1,
    executeAssignment: successfulExecutor,
  });

  assert.equal(status.state, "BLOCKED_INPUT");
  assert.match(status.rejected_path, /rejected/);
  assert.deepEqual(await readdir(inbox), []);
});

test("rejects lane mismatches once and allows the next assignment to run", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-reject-mismatch";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});
  const inbox = path.join(stateRoot, "campaigns", campaignId, "lanes", "lane-01", "inbox");
  await writeFile(
    path.join(inbox, "000-bad.json"),
    `${JSON.stringify({...assignment("WJ-OPS-WRONG"), campaign_id: campaignId, lane: 2})}\n`,
  );
  await enqueueGridAssignment({
    stateRoot,
    campaignId,
    lane: 1,
    assignment: assignment("WJ-OPS-NEXT"),
  });

  const rejected = await runLaneWorkerOnce({
    stateRoot,
    campaignId,
    lane: 1,
    executeAssignment: successfulExecutor,
  });
  assert.equal(rejected.state, "BLOCKED_INPUT");

  const completed = await runLaneWorkerOnce({
    stateRoot,
    campaignId,
    lane: 1,
    executeAssignment: successfulExecutor,
  });
  assert.equal(completed.state, "DONE_REUSABLE");
  assert.equal(completed.task_id, "WJ-OPS-NEXT");
});

test("uses exact structured stop code instead of scanning prompt/log text", async (t) => {
  const stateRoot = await temporaryDirectory(t);
  const campaignId = "wj-stop-code";
  await initializeGridCampaign({stateRoot, campaignId, laneCount: 1});

  await enqueueGridAssignment({
    stateRoot,
    campaignId,
    lane: 1,
    assignment: assignment("WJ-OPS-GENERIC"),
  });
  const generic = await runLaneWorkerOnce({
    stateRoot,
    campaignId,
    lane: 1,
    executeAssignment: successfulExecutor,
  });
  assert.equal(generic.state, "DONE_REUSABLE");

  await enqueueGridAssignment({
    stateRoot,
    campaignId,
    lane: 1,
    assignment: assignment("WJ-OPS-STOP"),
  });
  const stopped = await runLaneWorkerOnce({
    stateRoot,
    campaignId,
    lane: 1,
    executeAssignment: async ({logPath}) => {
      await writeFile(logPath, "structured route failure\n");
      return {exit_code: 101, stop_code: "active_goal_route_repair_required"};
    },
  });

  assert.equal(stopped.state, "ROUTE_REPAIR_REQUIRED");
  const events = await readGridEvents({stateRoot, campaignId});
  assert.equal(events.filter((event) => event.event === "ROUTE_REPAIR_REQUIRED").length, 1);
});
