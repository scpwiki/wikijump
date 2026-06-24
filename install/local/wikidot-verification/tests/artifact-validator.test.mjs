import assert from "node:assert/strict";
import {execFile} from "node:child_process";
import {createHash} from "node:crypto";
import {mkdir, mkdtemp, rm, writeFile} from "node:fs/promises";
import {promisify} from "node:util";
import {fileURLToPath} from "node:url";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  artifactValidatorExitCode,
  validateArtifactDirectory,
} from "../src/artifact-validator.mjs";

const execFileAsync = promisify(execFile);
const cliScriptPath = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../scripts/validate-artifact.mjs",
);

async function temporaryDirectory(t, prefix = "wikijump-artifact-") {
  const directory = await mkdtemp(path.join(os.tmpdir(), prefix));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return directory;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function writeArtifactFile(root, relativePath, contents) {
  const targetPath = path.join(root, ...relativePath.split("/"));
  await mkdir(path.dirname(targetPath), {recursive: true});
  await writeFile(targetPath, contents);
  return {
    path: relativePath,
    size: Buffer.byteLength(contents),
    sha256: sha256(contents),
  };
}

async function writeManifest(root, entries) {
  await writeFile(
    path.join(root, "manifest.json"),
    JSON.stringify(
      {
        schema_version: 1,
        artifact: "test-artifact",
        repository: "Rokurolize/wikijump",
        self_included: false,
        files: entries,
      },
      null,
      2,
    ),
  );
}

async function createValidProArtifact(t) {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    status: "strategy_ready",
    repository: "Rokurolize/wikijump",
    outputs: ["report.md"],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  const reportEntry = await writeArtifactFile(root, "report.md", "strategy report\n");
  await writeManifest(root, [resultEntry, reportEntry]);
  return root;
}

async function createValidProPatchArtifact(t) {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    status: "patch_ready",
    repository: "Rokurolize/wikijump",
    base_commit: "1672120d758755382ae3e9c174c49e5ee1cd543b",
    task_id: "WJ-OPS-001",
    changed_files: [],
    validation: {
      executed: [],
      not_run: [],
    },
    environment: {
      network_used: false,
      browser_used: false,
      database_used: false,
      live_github_checked: false,
    },
    closure_claims: {},
    limitations: [],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  const patchEntry = await writeArtifactFile(root, "patches/task.patch", "diff --git a/x b/x\n");
  await writeManifest(root, [resultEntry, patchEntry]);
  return root;
}

async function createValidCodexArtifact(t) {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    task_id: "WJ-OPS-001",
    assignment_id: "WJ-OPS-001-r1-a1",
    status: "analysis_complete",
    repository: "Rokurolize/wikijump",
    base_sha_expected: "1672120d758755382ae3e9c174c49e5ee1cd543b",
    base_sha_observed: "1672120d758755382ae3e9c174c49e5ee1cd543b",
    worktree: "/tmp/worktree",
    changed_paths: [],
    artifacts: ["report.md", "commands.jsonl"],
    validation: [
      {
        command_id: "cmd-001",
        exit_code: 0,
        claim: "executed_pass",
      },
    ],
    github_mutations: [],
    findings: [],
    blockers: [],
    stop_code: null,
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  const reportEntry = await writeArtifactFile(root, "report.md", "analysis report\n");
  const commandsEntry = await writeArtifactFile(root, "commands.jsonl", "{\"command_id\":\"cmd-001\"}\n");
  await writeManifest(root, [resultEntry, reportEntry, commandsEntry]);
  return root;
}

function findingCodes(report) {
  return report.findings.map((finding) => finding.code);
}

test("validates a Pro strategy artifact and returns a machine-readable pass report", async (t) => {
  const root = await createValidProArtifact(t);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "pro"});

  assert.equal(report.status, "pass");
  assert.equal(report.artifact_kind, "pro");
  assert.deepEqual(report.summary, {errors: 0, warnings: 0, findings: 0});
  assert.equal(artifactValidatorExitCode(report), 0);
});

test("validates a Codex artifact with matching task and assignment IDs", async (t) => {
  const root = await createValidCodexArtifact(t);

  const report = await validateArtifactDirectory({
    artifactRoot: root,
    kind: "codex",
    expectedTaskId: "WJ-OPS-001",
    expectedAssignmentId: "WJ-OPS-001-r1-a1",
  });

  assert.equal(report.status, "pass");
  assert.equal(report.artifact_kind, "codex");
  assert.equal(artifactValidatorExitCode(report), 0);
});

test("auto-detects Pro strategy artifacts", async (t) => {
  const root = await createValidProArtifact(t);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "auto"});

  assert.equal(report.status, "pass");
  assert.equal(report.artifact_kind, "pro");
});

test("auto-detects Pro patch artifacts without treating task_id as Codex-only", async (t) => {
  const root = await createValidProPatchArtifact(t);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "auto"});

  assert.equal(report.status, "pass");
  assert.equal(report.artifact_kind, "pro");
});

test("auto-detects Codex artifacts from assignment_id", async (t) => {
  const root = await createValidCodexArtifact(t);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "auto"});

  assert.equal(report.status, "pass");
  assert.equal(report.artifact_kind, "codex");
});

test("quarantines invalid result JSON without trusting textual completion", async (t) => {
  const root = await temporaryDirectory(t);
  const resultEntry = await writeArtifactFile(root, "result.json", "{not valid json\n");
  await writeManifest(root, [resultEntry]);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "codex"});

  assert.equal(report.status, "quarantine");
  assert.equal(artifactValidatorExitCode(report), 2);
  assert.ok(findingCodes(report).includes("result_invalid_json"));
});

test("quarantines a missing status field", async (t) => {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    repository: "Rokurolize/wikijump",
    outputs: ["report.md"],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  const reportEntry = await writeArtifactFile(root, "report.md", "report\n");
  await writeManifest(root, [resultEntry, reportEntry]);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "pro"});

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("result_status_missing"));
});

test("quarantines manifest path traversal", async (t) => {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    status: "strategy_ready",
    repository: "Rokurolize/wikijump",
    outputs: ["report.md"],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  await writeManifest(root, [resultEntry, {path: "../outside.txt", size: 1, sha256: "0".repeat(64)}]);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "pro"});

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("manifest_path_invalid"));
});

test("quarantines a bare parent-directory manifest path", async (t) => {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    status: "strategy_ready",
    repository: "Rokurolize/wikijump",
    outputs: ["report.md"],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  await writeManifest(root, [resultEntry, {path: "..", size: 1, sha256: "0".repeat(64)}]);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "pro"});

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("manifest_path_invalid"));
});

test("quarantines manifest-listed files that are missing", async (t) => {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    status: "strategy_ready",
    repository: "Rokurolize/wikijump",
    outputs: ["missing.md"],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  await writeManifest(root, [resultEntry, {path: "missing.md", size: 7, sha256: "0".repeat(64)}]);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "pro"});

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("manifest_file_missing"));
});

test("quarantines bad hashes and size mismatches", async (t) => {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    status: "strategy_ready",
    repository: "Rokurolize/wikijump",
    outputs: ["report.md"],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  await writeArtifactFile(root, "report.md", "actual report\n");
  await writeManifest(root, [
    resultEntry,
    {path: "report.md", size: 999, sha256: "0".repeat(64)},
  ]);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "pro"});

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("manifest_size_mismatch"));
  assert.ok(findingCodes(report).includes("manifest_sha256_mismatch"));
});

test("quarantines Codex task and assignment mismatches", async (t) => {
  const root = await createValidCodexArtifact(t);

  const report = await validateArtifactDirectory({
    artifactRoot: root,
    kind: "codex",
    expectedTaskId: "WJ-OPS-002",
    expectedAssignmentId: "WJ-OPS-002-r1-a1",
  });

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("result_task_id_mismatch"));
  assert.ok(findingCodes(report).includes("result_assignment_id_mismatch"));
});

test("quarantines unsafe artifact references from result.json", async (t) => {
  const root = await temporaryDirectory(t);
  const result = {
    schema_version: 1,
    task_id: "WJ-OPS-001",
    assignment_id: "WJ-OPS-001-r1-a1",
    status: "analysis_complete",
    repository: "Rokurolize/wikijump",
    artifacts: ["../outside.md"],
    validation: [],
  };
  const resultEntry = await writeArtifactFile(root, "result.json", `${JSON.stringify(result, null, 2)}\n`);
  await writeManifest(root, [resultEntry]);

  const report = await validateArtifactDirectory({artifactRoot: root, kind: "codex"});

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("artifact_reference_path_invalid"));
});

test("quarantines additional required files that are absent", async (t) => {
  const root = await createValidProArtifact(t);

  const report = await validateArtifactDirectory({
    artifactRoot: root,
    kind: "pro",
    requiredFiles: ["gap-ledger.tsv"],
  });

  assert.equal(report.status, "quarantine");
  assert.ok(findingCodes(report).includes("required_file_missing"));
});

test("CLI rejects a missing expected task ID before validation", async (t) => {
  const root = await createValidCodexArtifact(t);

  await assert.rejects(
    execFileAsync(process.execPath, [
      cliScriptPath,
      root,
      "--kind",
      "codex",
      "--expected-task-id",
    ]),
    {
      code: 1,
    },
  );
});
