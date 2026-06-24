import {createHash} from "node:crypto";
import {
  mkdir,
  open,
  readFile,
  readdir,
  rename,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import {validateArtifactDirectory} from "./artifact-validator.mjs";

const SAFE_ID_RE = /^[A-Za-z0-9][A-Za-z0-9._:-]*$/;
const ROUTE_REPAIR_STOP_CODE = "active_goal_route_repair_required";

function assertSafeId(value, name) {
  if (typeof value !== "string" || !SAFE_ID_RE.test(value)) {
    throw new Error(`${name} must be a safe identifier`);
  }
}

function laneName(lane) {
  if (!Number.isInteger(lane) || lane < 1 || lane > 99) {
    throw new Error("lane must be an integer from 1 through 99");
  }
  return `lane-${String(lane).padStart(2, "0")}`;
}

function nowIso() {
  return new Date().toISOString();
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function campaignDirectory(stateRoot, campaignId) {
  assertSafeId(campaignId, "campaignId");
  return path.resolve(stateRoot, "campaigns", campaignId);
}

function laneDirectory(stateRoot, campaignId, lane) {
  return path.join(campaignDirectory(stateRoot, campaignId), "lanes", laneName(lane));
}

async function appendJsonLine(filePath, record) {
  await mkdir(path.dirname(filePath), {recursive: true});
  const handle = await open(filePath, "a");
  try {
    await handle.write(`${JSON.stringify(record)}\n`);
  } finally {
    await handle.close();
  }
}

async function writeJsonFile(filePath, value) {
  await mkdir(path.dirname(filePath), {recursive: true});
  await writeFile(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

async function readJsonFile(filePath) {
  return JSON.parse(await readFile(filePath, "utf8"));
}

async function regularFileExists(filePath) {
  try {
    return (await stat(filePath)).isFile();
  } catch (error) {
    if (error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

export async function initializeGridCampaign({
  stateRoot,
  campaignId,
  laneCount,
  originDevelop = null,
} = {}) {
  if (typeof stateRoot !== "string" || stateRoot.length === 0) {
    throw new TypeError("stateRoot must be a non-empty string");
  }
  assertSafeId(campaignId, "campaignId");
  if (!Number.isInteger(laneCount) || laneCount < 1) {
    throw new Error("laneCount must be a positive integer");
  }

  const campaignRoot = campaignDirectory(stateRoot, campaignId);
  await mkdir(campaignRoot, {recursive: true});
  for (let lane = 1; lane <= laneCount; lane += 1) {
    const laneRoot = laneDirectory(stateRoot, campaignId, lane);
    await mkdir(path.join(laneRoot, "inbox"), {recursive: true});
    await mkdir(path.join(laneRoot, "assignments"), {recursive: true});
    await mkdir(path.join(laneRoot, "rejected"), {recursive: true});
    const statusPath = path.join(laneRoot, "status.json");
    if (!(await regularFileExists(statusPath))) {
      await writeJsonFile(statusPath, {
        schema_version: 1,
        campaign_id: campaignId,
        lane,
        state: "IDLE",
        assignment_id: null,
        updated_at: nowIso(),
      });
    }
  }

  const manifest = {
    schema_version: 1,
    campaign_id: campaignId,
    lane_count: laneCount,
    origin_develop: originDevelop,
    created_or_observed_at: nowIso(),
  };
  await writeJsonFile(path.join(campaignRoot, "campaign.json"), manifest);
  await appendGridEvent({
    stateRoot,
    campaignId,
    event: "CAMPAIGN_INITIALIZED",
    lane_count: laneCount,
    origin_develop: originDevelop,
  });
  return {campaignRoot, manifest};
}

export async function appendGridEvent({stateRoot, campaignId, ...event}) {
  const record = {
    campaign_id: campaignId,
    ...event,
    time: nowIso(),
  };
  await appendJsonLine(path.join(campaignDirectory(stateRoot, campaignId), "events.jsonl"), record);
  return record;
}

function normalizeAssignment(assignment) {
  if (assignment === null || typeof assignment !== "object" || Array.isArray(assignment)) {
    throw new TypeError("assignment must be an object");
  }
  for (const field of ["assignment_id", "task_id", "campaign_id", "route"]) {
    assertSafeId(assignment[field], field);
  }
  if (!Number.isInteger(assignment.lane) || assignment.lane < 1) {
    throw new Error("assignment.lane must be a positive integer");
  }
  if (typeof assignment.title !== "string" || assignment.title.length === 0) {
    throw new Error("assignment.title must be a non-empty string");
  }
  if (!Array.isArray(assignment.expected_artifacts) || assignment.expected_artifacts.length === 0) {
    throw new Error("assignment.expected_artifacts must be a non-empty array");
  }
  return {
    schema_version: 1,
    assigned_at: nowIso(),
    result_schema: "codex-result-v1",
    ...assignment,
  };
}

export async function enqueueGridAssignment({stateRoot, campaignId, lane, assignment}) {
  assertSafeId(campaignId, "campaignId");
  const normalized = normalizeAssignment({...assignment, campaign_id: campaignId, lane});
  const inbox = path.join(laneDirectory(stateRoot, campaignId, lane), "inbox");
  await mkdir(inbox, {recursive: true});
  const finalPath = path.join(inbox, `${normalized.assignment_id}.json`);
  const tempPath = `${finalPath}.tmp-${process.pid}-${Date.now()}`;
  await writeJsonFile(tempPath, normalized);
  await rename(tempPath, finalPath);
  await appendGridEvent({
    stateRoot,
    campaignId,
    event: "ASSIGNED",
    lane,
    task_id: normalized.task_id,
    assignment_id: normalized.assignment_id,
  });
  return {assignment: normalized, path: finalPath};
}

async function nextAssignmentPath(laneRoot) {
  const inbox = path.join(laneRoot, "inbox");
  const entries = (await readdir(inbox)).filter((entry) => entry.endsWith(".json")).sort();
  if (entries.length === 0) {
    return null;
  }
  return path.join(inbox, entries[0]);
}

async function rejectInboxAssignment({laneRoot, inboxPath, reason, rawAssignment}) {
  const rejectedRoot = path.join(laneRoot, "rejected");
  await mkdir(rejectedRoot, {recursive: true});
  const rejectedPath = path.join(rejectedRoot, `${Date.now()}-${path.basename(inboxPath)}`);
  await rename(inboxPath, rejectedPath);
  await writeJsonFile(`${rejectedPath}.reason.json`, {
    schema_version: 1,
    reason,
    raw_assignment_id: rawAssignment?.assignment_id ?? null,
    rejected_at: nowIso(),
  });
  return rejectedPath;
}

async function updateLaneStatus({stateRoot, campaignId, lane, status}) {
  const statusRecord = {
    schema_version: 1,
    campaign_id: campaignId,
    lane,
    updated_at: nowIso(),
    ...status,
  };
  await writeJsonFile(path.join(laneDirectory(stateRoot, campaignId, lane), "status.json"), statusRecord);
  return statusRecord;
}

export async function writeCodexResultArtifact({
  artifactRoot,
  assignment,
  status = "analysis_complete",
  reportText = "analysis report\n",
  validation = [],
} = {}) {
  const result = {
    schema_version: 1,
    task_id: assignment.task_id,
    assignment_id: assignment.assignment_id,
    status,
    repository: assignment.repository ?? "Rokurolize/wikijump",
    base_sha_expected: assignment.base_sha ?? null,
    base_sha_observed: assignment.base_sha ?? null,
    worktree: assignment.worktree ?? null,
    changed_paths: [],
    artifacts: ["report.md", "commands.jsonl"],
    validation,
    github_mutations: [],
    findings: [],
    blockers: [],
    stop_code: null,
  };
  await mkdir(artifactRoot, {recursive: true});
  const resultBytes = `${JSON.stringify(result, null, 2)}\n`;
  const reportBytes = reportText;
  const commandBytes = validation.map((item) => JSON.stringify(item)).join("\n") + (validation.length === 0 ? "" : "\n");
  await writeFile(path.join(artifactRoot, "result.json"), resultBytes);
  await writeFile(path.join(artifactRoot, "report.md"), reportBytes);
  await writeFile(path.join(artifactRoot, "commands.jsonl"), commandBytes);
  await writeJsonFile(path.join(artifactRoot, "manifest.json"), {
    schema_version: 1,
    artifact: assignment.assignment_id,
    repository: result.repository,
    self_included: false,
    files: [
      {
        path: "result.json",
        size: Buffer.byteLength(resultBytes),
        sha256: sha256(resultBytes),
      },
      {
        path: "report.md",
        size: Buffer.byteLength(reportBytes),
        sha256: sha256(reportBytes),
      },
      {
        path: "commands.jsonl",
        size: Buffer.byteLength(commandBytes),
        sha256: sha256(commandBytes),
      },
    ],
  });
}

export async function runLaneWorkerOnce({
  stateRoot,
  campaignId,
  lane,
  executeAssignment,
} = {}) {
  if (typeof executeAssignment !== "function") {
    throw new TypeError("executeAssignment must be a function");
  }
  const laneRoot = laneDirectory(stateRoot, campaignId, lane);
  const inboxPath = await nextAssignmentPath(laneRoot);
  if (inboxPath === null) {
    return updateLaneStatus({
      stateRoot,
      campaignId,
      lane,
      status: {state: "IDLE", assignment_id: null},
    });
  }

  let rawAssignment;
  let assignment;
  try {
    rawAssignment = await readJsonFile(inboxPath);
    assignment = normalizeAssignment(rawAssignment);
  } catch (error) {
    const rejectedPath = await rejectInboxAssignment({
      laneRoot,
      inboxPath,
      reason: error.message,
      rawAssignment,
    });
    await appendGridEvent({
      stateRoot,
      campaignId,
      event: "ASSIGNMENT_REJECTED",
      lane,
      reason: error.message,
      rejected_path: rejectedPath,
    });
    return updateLaneStatus({
      stateRoot,
      campaignId,
      lane,
      status: {
        state: "BLOCKED_INPUT",
        assignment_id: null,
        reason: error.message,
        rejected_path: rejectedPath,
      },
    });
  }

  if (assignment.campaign_id !== campaignId || assignment.lane !== lane) {
    const rejectedPath = await rejectInboxAssignment({
      laneRoot,
      inboxPath,
      reason: "assignment campaign or lane mismatch",
      rawAssignment,
    });
    await appendGridEvent({
      stateRoot,
      campaignId,
      event: "ASSIGNMENT_REJECTED",
      lane,
      task_id: assignment.task_id,
      assignment_id: assignment.assignment_id,
      reason: "assignment campaign or lane mismatch",
      rejected_path: rejectedPath,
    });
    return updateLaneStatus({
      stateRoot,
      campaignId,
      lane,
      status: {
        state: "BLOCKED_INPUT",
        assignment_id: assignment.assignment_id,
        reason: "assignment campaign or lane mismatch",
        rejected_path: rejectedPath,
      },
    });
  }

  const assignmentRoot = path.join(laneRoot, "assignments", assignment.assignment_id);
  await rm(assignmentRoot, {recursive: true, force: true});
  await mkdir(assignmentRoot, {recursive: true});
  await rename(inboxPath, path.join(assignmentRoot, "assignment.json"));
  const artifactRoot = path.join(assignmentRoot, "artifacts");
  const logPath = path.join(assignmentRoot, "worker.log");

  await updateLaneStatus({
    stateRoot,
    campaignId,
    lane,
    status: {state: "RUNNING", assignment_id: assignment.assignment_id, task_id: assignment.task_id},
  });
  await appendGridEvent({
    stateRoot,
    campaignId,
    event: "ASSIGNMENT_STARTED",
    lane,
    task_id: assignment.task_id,
    assignment_id: assignment.assignment_id,
  });

  let execution;
  try {
    execution = await executeAssignment({assignment, artifactRoot, logPath});
  } catch (error) {
    execution = {exit_code: 1, error: error.message, stop_code: null};
    await writeFile(logPath, `${error.stack ?? error.message}\n`);
  }
  const exitCode = Number.isInteger(execution?.exit_code) ? execution.exit_code : 1;
  const stopCode = execution?.stop_code ?? null;
  await writeJsonFile(path.join(assignmentRoot, "process.json"), {
    schema_version: 1,
    assignment_id: assignment.assignment_id,
    exit_code: exitCode,
    stop_code: stopCode,
    completed_at: nowIso(),
  });
  await appendGridEvent({
    stateRoot,
    campaignId,
    event: "PROCESS_EXIT",
    lane,
    task_id: assignment.task_id,
    assignment_id: assignment.assignment_id,
    exit_code: exitCode,
    stop_code: stopCode,
  });

  if (stopCode === ROUTE_REPAIR_STOP_CODE) {
    await appendGridEvent({
      stateRoot,
      campaignId,
      event: "ROUTE_REPAIR_REQUIRED",
      lane,
      task_id: assignment.task_id,
      assignment_id: assignment.assignment_id,
    });
    return updateLaneStatus({
      stateRoot,
      campaignId,
      lane,
      status: {
        state: "ROUTE_REPAIR_REQUIRED",
        assignment_id: assignment.assignment_id,
        task_id: assignment.task_id,
        stop_code: stopCode,
      },
    });
  }

  const validationReport = await validateArtifactDirectory({
    artifactRoot,
    kind: "codex",
    expectedTaskId: assignment.task_id,
    expectedAssignmentId: assignment.assignment_id,
    requiredFiles: assignment.expected_artifacts,
  });
  await writeJsonFile(path.join(assignmentRoot, "artifact-validation.json"), validationReport);

  if (validationReport.status !== "pass") {
    await appendGridEvent({
      stateRoot,
      campaignId,
      event: "ARTIFACT_SCHEMA_FAIL",
      lane,
      task_id: assignment.task_id,
      assignment_id: assignment.assignment_id,
      findings: validationReport.summary.findings,
    });
    return updateLaneStatus({
      stateRoot,
      campaignId,
      lane,
      status: {
        state: "SCHEMA_INVALID",
        assignment_id: assignment.assignment_id,
        task_id: assignment.task_id,
        artifact_root: artifactRoot,
      },
    });
  }

  await appendGridEvent({
    stateRoot,
    campaignId,
    event: "ARTIFACT_SCHEMA_PASS",
    lane,
    task_id: assignment.task_id,
    assignment_id: assignment.assignment_id,
  });

  const finalState = exitCode === 0 ? "DONE_REUSABLE" : "FAILED_FINAL";
  await appendGridEvent({
    stateRoot,
    campaignId,
    event: finalState,
    lane,
    task_id: assignment.task_id,
    assignment_id: assignment.assignment_id,
  });
  return updateLaneStatus({
    stateRoot,
    campaignId,
    lane,
    status: {
      state: finalState,
      assignment_id: assignment.assignment_id,
      task_id: assignment.task_id,
      artifact_root: artifactRoot,
    },
  });
}

export async function readGridEvents({stateRoot, campaignId}) {
  const eventPath = path.join(campaignDirectory(stateRoot, campaignId), "events.jsonl");
  if (!(await regularFileExists(eventPath))) {
    return [];
  }
  const text = await readFile(eventPath, "utf8");
  return text
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line));
}

export async function reconstructGridState({stateRoot, campaignId, laneCount}) {
  const lanes = [];
  for (let lane = 1; lane <= laneCount; lane += 1) {
    const statusPath = path.join(laneDirectory(stateRoot, campaignId, lane), "status.json");
    lanes.push(await readJsonFile(statusPath));
  }
  return {
    schema_version: 1,
    campaign_id: campaignId,
    events: await readGridEvents({stateRoot, campaignId}),
    lanes,
  };
}
