import {createHash} from "node:crypto";
import {lstat, readFile} from "node:fs/promises";
import path from "node:path";

const SHA256_RE = /^[0-9a-f]{64}$/;

const PRO_RESULT_STATUSES = new Set([
  "strategy_ready",
  "patch_ready",
  "review_ready",
  "blocked_input",
  "blocked_environment",
  "no_change_required",
]);

const CODEX_RESULT_STATUSES = new Set([
  "patch_ready",
  "proof_collected",
  "analysis_complete",
  "monitor_complete",
  "followup_needed",
  "gap_ledger_only",
  "blocked_input",
  "blocked_environment",
  "stale_base",
  "no_change_required",
]);

function createFinding(severity, code, message, extra = {}) {
  return {severity, code, message, ...extra};
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function addFinding(findings, severity, code, message, extra) {
  findings.push(createFinding(severity, code, message, extra));
}

function detectArtifactKind({kind, result}) {
  if (kind !== "auto") {
    return kind;
  }
  if (typeof result?.assignment_id === "string") {
    return "codex";
  }
  return "pro";
}

function assertArtifactKind(kind) {
  if (!["auto", "pro", "codex"].includes(kind)) {
    throw new Error(`unsupported artifact kind: ${kind}`);
  }
}

function isSafeRelativeArtifactPath(artifactPath) {
  if (typeof artifactPath !== "string" || artifactPath.length === 0) {
    return false;
  }
  if (
    path.posix.isAbsolute(artifactPath) ||
    artifactPath.includes("\\") ||
    artifactPath.includes("\0")
  ) {
    return false;
  }
  const normalized = path.posix.normalize(artifactPath);
  return (
    normalized === artifactPath &&
    normalized !== "." &&
    normalized !== ".." &&
    !normalized.startsWith("../")
  );
}

function resolveContained(rootPath, artifactPath) {
  if (!isSafeRelativeArtifactPath(artifactPath)) {
    throw new Error("path must be a normalized relative POSIX path");
  }

  const root = path.resolve(rootPath);
  const candidate = path.resolve(root, ...artifactPath.split("/"));
  const relative = path.relative(root, candidate);
  if (relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative))) {
    return candidate;
  }

  throw new Error("path escapes artifact root");
}

async function readJsonFile(filePath, findings, codePrefix, artifactPath) {
  let text;
  try {
    text = await readFile(filePath, "utf8");
  } catch (error) {
    addFinding(findings, "error", `${codePrefix}_missing`, `missing ${artifactPath}`, {
      path: artifactPath,
      detail: error.message,
    });
    return null;
  }

  try {
    return JSON.parse(text);
  } catch (error) {
    addFinding(findings, "error", `${codePrefix}_invalid_json`, `${artifactPath} is invalid JSON`, {
      path: artifactPath,
      detail: error.message,
    });
    return null;
  }
}

function validateSchemaVersion(value, findings, artifactPath) {
  if (!Number.isInteger(value) || value < 1) {
    addFinding(findings, "error", "schema_version_invalid", "schema_version must be a positive integer", {
      path: artifactPath,
    });
  }
}

function validateStatus(value, allowedStatuses, findings, artifactPath) {
  if (typeof value !== "string" || value.length === 0) {
    addFinding(findings, "error", "result_status_missing", "result.json must contain a non-empty status", {
      path: artifactPath,
    });
    return;
  }
  if (!allowedStatuses.has(value)) {
    addFinding(findings, "error", "result_status_unknown", `unsupported result status: ${value}`, {
      path: artifactPath,
      status: value,
    });
  }
}

function validateProResult(result, findings) {
  if (!isObject(result)) {
    addFinding(findings, "error", "result_not_object", "result.json must be an object", {
      path: "result.json",
    });
    return;
  }

  validateSchemaVersion(result.schema_version, findings, "result.json");
  validateStatus(result.status, PRO_RESULT_STATUSES, findings, "result.json");

  if (typeof result.repository !== "string" || result.repository.length === 0) {
    addFinding(findings, "error", "result_repository_missing", "Pro result.json must contain repository", {
      path: "result.json",
    });
  }

  if (result.status === "strategy_ready") {
    if (!Array.isArray(result.outputs) || result.outputs.length === 0) {
      addFinding(findings, "error", "pro_outputs_missing", "strategy Pro result must list outputs", {
        path: "result.json",
      });
    }
  } else {
    if (typeof result.task_id !== "string" || result.task_id.length === 0) {
      addFinding(findings, "error", "result_task_id_missing", "patch/review Pro result must contain task_id", {
        path: "result.json",
      });
    }
    if (typeof result.base_commit !== "string" || result.base_commit.length === 0) {
      addFinding(findings, "error", "result_base_commit_missing", "patch/review Pro result must contain base_commit", {
        path: "result.json",
      });
    }
  }
}

function validateCodexResult(result, findings, {expectedTaskId, expectedAssignmentId}) {
  if (!isObject(result)) {
    addFinding(findings, "error", "result_not_object", "result.json must be an object", {
      path: "result.json",
    });
    return;
  }

  validateSchemaVersion(result.schema_version, findings, "result.json");
  validateStatus(result.status, CODEX_RESULT_STATUSES, findings, "result.json");

  for (const field of ["repository", "task_id", "assignment_id"]) {
    if (typeof result[field] !== "string" || result[field].length === 0) {
      addFinding(findings, "error", `result_${field}_missing`, `Codex result.json must contain ${field}`, {
        path: "result.json",
      });
    }
  }

  if (expectedTaskId !== null && result.task_id !== expectedTaskId) {
    addFinding(findings, "error", "result_task_id_mismatch", "result.json task_id does not match expected task ID", {
      path: "result.json",
      expected: expectedTaskId,
      actual: result.task_id,
    });
  }

  if (expectedAssignmentId !== null && result.assignment_id !== expectedAssignmentId) {
    addFinding(findings, "error", "result_assignment_id_mismatch", "result.json assignment_id does not match expected assignment ID", {
      path: "result.json",
      expected: expectedAssignmentId,
      actual: result.assignment_id,
    });
  }

  if (!Array.isArray(result.validation)) {
    addFinding(findings, "warning", "result_validation_missing", "Codex result.json should contain validation command records", {
      path: "result.json",
    });
  }
}

function validateArtifactReference(reference, findings, sourcePath) {
  const artifactPath = typeof reference === "string" ? reference : reference?.path;
  if (!isSafeRelativeArtifactPath(artifactPath)) {
    addFinding(findings, "error", "artifact_reference_path_invalid", "artifact reference must be a safe relative POSIX path", {
      path: sourcePath,
      artifact_path: artifactPath,
    });
    return null;
  }
  return artifactPath;
}

function validateResultArtifactReferences(result, findings) {
  if (!Array.isArray(result?.artifacts)) {
    return [];
  }
  const artifactPaths = [];
  for (const reference of result.artifacts) {
    const artifactPath = validateArtifactReference(reference, findings, "result.json");
    if (artifactPath !== null) {
      artifactPaths.push(artifactPath);
    }
  }
  return artifactPaths;
}

function validateManifestShape(manifest, findings) {
  if (!isObject(manifest)) {
    addFinding(findings, "error", "manifest_not_object", "manifest.json must be an object", {
      path: "manifest.json",
    });
    return [];
  }

  validateSchemaVersion(manifest.schema_version, findings, "manifest.json");

  if (!Array.isArray(manifest.files)) {
    addFinding(findings, "error", "manifest_files_missing", "manifest.json must contain a files array", {
      path: "manifest.json",
    });
    return [];
  }

  return manifest.files;
}

async function validateManifestEntry({artifactRoot, entry, index, seenPaths, findings}) {
  if (!isObject(entry)) {
    addFinding(findings, "error", "manifest_entry_not_object", "manifest file entry must be an object", {
      path: "manifest.json",
      index,
    });
    return;
  }

  if (!isSafeRelativeArtifactPath(entry.path)) {
    addFinding(findings, "error", "manifest_path_invalid", "manifest path must be a safe relative POSIX path", {
      path: "manifest.json",
      index,
      artifact_path: entry.path,
    });
    return;
  }

  if (seenPaths.has(entry.path)) {
    addFinding(findings, "error", "manifest_path_duplicate", "manifest contains a duplicate path", {
      path: "manifest.json",
      index,
      artifact_path: entry.path,
    });
    return;
  }
  seenPaths.add(entry.path);

  if (!Number.isInteger(entry.size) || entry.size < 0) {
    addFinding(findings, "error", "manifest_size_invalid", "manifest size must be a non-negative integer", {
      path: "manifest.json",
      index,
      artifact_path: entry.path,
    });
    return;
  }

  if (typeof entry.sha256 !== "string" || !SHA256_RE.test(entry.sha256)) {
    addFinding(findings, "error", "manifest_sha256_invalid", "manifest sha256 must be lowercase 64-character hex", {
      path: "manifest.json",
      index,
      artifact_path: entry.path,
    });
    return;
  }

  let filePath;
  try {
    filePath = resolveContained(artifactRoot, entry.path);
  } catch (error) {
    addFinding(findings, "error", "manifest_path_escape", "manifest path escapes artifact root", {
      path: "manifest.json",
      index,
      artifact_path: entry.path,
      detail: error.message,
    });
    return;
  }

  let stat;
  try {
    stat = await lstat(filePath);
  } catch (error) {
    addFinding(findings, "error", "manifest_file_missing", "manifest-listed file is missing", {
      path: entry.path,
      detail: error.message,
    });
    return;
  }

  if (!stat.isFile()) {
    addFinding(findings, "error", "manifest_file_not_regular", "manifest-listed path must be a regular file", {
      path: entry.path,
    });
    return;
  }

  const bytes = await readFile(filePath);
  if (bytes.byteLength !== entry.size) {
    addFinding(findings, "error", "manifest_size_mismatch", "manifest size does not match file bytes", {
      path: entry.path,
      expected: entry.size,
      actual: bytes.byteLength,
    });
  }

  const actualSha256 = createHash("sha256").update(bytes).digest("hex");
  if (actualSha256 !== entry.sha256) {
    addFinding(findings, "error", "manifest_sha256_mismatch", "manifest sha256 does not match file bytes", {
      path: entry.path,
      expected: entry.sha256,
      actual: actualSha256,
    });
  }
}

async function validateRequiredArtifactPaths({artifactRoot, requiredFiles, findings}) {
  for (const requiredFile of requiredFiles) {
    if (!isSafeRelativeArtifactPath(requiredFile)) {
      addFinding(findings, "error", "required_path_invalid", "required file must be a safe relative POSIX path", {
        path: requiredFile,
      });
      continue;
    }

    let filePath;
    try {
      filePath = resolveContained(artifactRoot, requiredFile);
    } catch (error) {
      addFinding(findings, "error", "required_path_escape", "required file escapes artifact root", {
        path: requiredFile,
        detail: error.message,
      });
      continue;
    }

    try {
      const stat = await lstat(filePath);
      if (!stat.isFile()) {
        addFinding(findings, "error", "required_file_not_regular", "required file must be a regular file", {
          path: requiredFile,
        });
      }
    } catch (error) {
      addFinding(findings, "error", "required_file_missing", "required file is missing", {
        path: requiredFile,
        detail: error.message,
      });
    }
  }
}

export async function validateArtifactDirectory({
  artifactRoot,
  kind = "auto",
  expectedTaskId = null,
  expectedAssignmentId = null,
  requiredFiles = [],
} = {}) {
  assertArtifactKind(kind);
  if (typeof artifactRoot !== "string" || artifactRoot.length === 0) {
    throw new TypeError("artifactRoot must be a non-empty string");
  }

  const resolvedArtifactRoot = path.resolve(artifactRoot);
  const findings = [];
  const baseRequiredFiles = new Set(["result.json", "manifest.json", ...requiredFiles]);

  const result = await readJsonFile(
    path.join(resolvedArtifactRoot, "result.json"),
    findings,
    "result",
    "result.json",
  );
  const manifest = await readJsonFile(
    path.join(resolvedArtifactRoot, "manifest.json"),
    findings,
    "manifest",
    "manifest.json",
  );

  const artifactKind = detectArtifactKind({kind, result});
  if (result !== null) {
    if (artifactKind === "pro") {
      validateProResult(result, findings);
    } else {
      validateCodexResult(result, findings, {expectedTaskId, expectedAssignmentId});
    }
  }

  const resultArtifactPaths = validateResultArtifactReferences(result, findings);
  for (const artifactPath of resultArtifactPaths) {
    baseRequiredFiles.add(artifactPath);
  }

  await validateRequiredArtifactPaths({
    artifactRoot: resolvedArtifactRoot,
    requiredFiles: [...baseRequiredFiles],
    findings,
  });

  if (manifest !== null) {
    const entries = validateManifestShape(manifest, findings);
    const seenPaths = new Set();
    for (const [index, entry] of entries.entries()) {
      await validateManifestEntry({
        artifactRoot: resolvedArtifactRoot,
        entry,
        index,
        seenPaths,
        findings,
      });
    }
  }

  const errorCount = findings.filter((finding) => finding.severity === "error").length;
  return {
    schema_version: 1,
    validator: "wikijump-artifact-validator-v1",
    artifact_kind: artifactKind,
    artifact_root: resolvedArtifactRoot,
    status: errorCount === 0 ? "pass" : "quarantine",
    summary: {
      errors: errorCount,
      warnings: findings.filter((finding) => finding.severity === "warning").length,
      findings: findings.length,
    },
    findings,
  };
}

export function artifactValidatorExitCode(report) {
  return report.status === "pass" ? 0 : 2;
}
