import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  addFinding,
  isSafeRelativeArtifactPath,
  resolveContained,
} from "../src/artifact-validation-common.mjs";
import {validateArtifactManifest} from "../src/artifact-manifest-validator.mjs";
import {
  assertArtifactKind,
  validateArtifactResult,
} from "../src/artifact-result-validator.mjs";

test("artifact validation common helpers reject non-canonical relative paths", () => {
  assert.equal(isSafeRelativeArtifactPath("reports/result.json"), true);
  for (const invalid of ["", ".", "..", "../result.json", "/tmp/result.json", "reports\\result.json"]) {
    assert.equal(isSafeRelativeArtifactPath(invalid), false, invalid);
  }
  assert.equal(
    resolveContained("/tmp/artifact", "reports/result.json"),
    path.resolve("/tmp/artifact/reports/result.json"),
  );
  assert.throws(() => resolveContained("/tmp/artifact", "../result.json"), /normalized relative/u);

  const findings = [];
  addFinding(findings, "warning", "example", "example finding", {path: "result.json"});
  assert.deepEqual(findings, [{
    severity: "warning",
    code: "example",
    message: "example finding",
    path: "result.json",
  }]);
});

test("artifact result validator dispatches schema families and preserves artifact paths", () => {
  assert.doesNotThrow(() => assertArtifactKind("auto"));
  assert.throws(() => assertArtifactKind("unknown"), /unsupported artifact kind/u);

  const findings = [];
  const validated = validateArtifactResult({
    kind: "auto",
    result: {
      schema_version: 1,
      status: "analysis_complete",
      repository: "Rokurolize/wikijump",
      task_id: "WJ-OPS-001",
      assignment_id: "WJ-OPS-001-r1-a1",
      artifacts: ["report.md", {path: "commands.jsonl"}],
      validation: [],
    },
    findings,
    expectedTaskId: "WJ-OPS-001",
    expectedAssignmentId: "WJ-OPS-001-r1-a1",
  });
  assert.deepEqual(validated, {
    artifactKind: "codex",
    artifactPaths: ["report.md", "commands.jsonl"],
  });
  assert.deepEqual(findings, []);
});

test("artifact manifest validator binds required paths, size, and digest", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "artifact-manifest-validator-"));
  t.after(() => fs.rm(root, {recursive: true, force: true}));
  const contents = Buffer.from("report\n");
  await fs.writeFile(path.join(root, "report.md"), contents);
  const findings = [];
  await validateArtifactManifest({
    artifactRoot: root,
    manifest: {
      schema_version: 1,
      files: [{
        path: "report.md",
        size: contents.length,
        sha256: createHash("sha256").update(contents).digest("hex"),
      }],
    },
    requiredFiles: ["report.md"],
    findings,
  });
  assert.deepEqual(findings, []);

  await validateArtifactManifest({
    artifactRoot: root,
    manifest: {
      schema_version: 1,
      files: [{path: "report.md", size: 999, sha256: "0".repeat(64)}],
    },
    requiredFiles: ["missing.md"],
    findings,
  });
  assert.deepEqual(
    findings.map(({code}) => code).sort(),
    ["manifest_sha256_mismatch", "manifest_size_mismatch", "required_file_missing"],
  );
});
