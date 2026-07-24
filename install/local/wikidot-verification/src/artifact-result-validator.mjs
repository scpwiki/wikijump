import {
  addFinding,
  isObject,
  isSafeRelativeArtifactPath,
  validateSchemaVersion,
  validateStatus,
} from "./artifact-validation-common.mjs";

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

export function assertArtifactKind(kind) {
  if (!["auto", "pro", "codex"].includes(kind)) {
    throw new Error(`unsupported artifact kind: ${kind}`);
  }
}

function detectArtifactKind({kind, result}) {
  if (kind !== "auto") return kind;
  return typeof result?.assignment_id === "string" ? "codex" : "pro";
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
    return;
  }

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
    addFinding(
      findings,
      "error",
      "result_assignment_id_mismatch",
      "result.json assignment_id does not match expected assignment ID",
      {
        path: "result.json",
        expected: expectedAssignmentId,
        actual: result.assignment_id,
      },
    );
  }

  if (!Array.isArray(result.validation)) {
    addFinding(
      findings,
      "warning",
      "result_validation_missing",
      "Codex result.json should contain validation command records",
      {path: "result.json"},
    );
  }
}

function validateArtifactReference(reference, findings) {
  const artifactPath = typeof reference === "string" ? reference : reference?.path;
  if (!isSafeRelativeArtifactPath(artifactPath)) {
    addFinding(
      findings,
      "error",
      "artifact_reference_path_invalid",
      "artifact reference must be a safe relative POSIX path",
      {path: "result.json", artifact_path: artifactPath},
    );
    return null;
  }
  return artifactPath;
}

function validateResultArtifactReferences(result, findings) {
  if (!Array.isArray(result?.artifacts)) return [];
  const artifactPaths = [];
  for (const reference of result.artifacts) {
    const artifactPath = validateArtifactReference(reference, findings);
    if (artifactPath !== null) artifactPaths.push(artifactPath);
  }
  return artifactPaths;
}

export function validateArtifactResult({
  kind,
  result,
  findings,
  expectedTaskId,
  expectedAssignmentId,
}) {
  const artifactKind = detectArtifactKind({kind, result});
  if (result !== null) {
    if (artifactKind === "pro") {
      validateProResult(result, findings);
    } else {
      validateCodexResult(result, findings, {expectedTaskId, expectedAssignmentId});
    }
  }
  return {
    artifactKind,
    artifactPaths: validateResultArtifactReferences(result, findings),
  };
}
