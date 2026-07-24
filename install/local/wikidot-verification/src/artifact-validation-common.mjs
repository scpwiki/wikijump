import path from "node:path";

export const SHA256_RE = /^[0-9a-f]{64}$/;

export function createFinding(severity, code, message, extra = {}) {
  return {severity, code, message, ...extra};
}

export function addFinding(findings, severity, code, message, extra) {
  findings.push(createFinding(severity, code, message, extra));
}

export function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function isSafeRelativeArtifactPath(artifactPath) {
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

export function resolveContained(rootPath, artifactPath) {
  if (!isSafeRelativeArtifactPath(artifactPath)) {
    throw new Error("path must be a normalized relative POSIX path");
  }

  const root = path.resolve(rootPath);
  const candidate = path.resolve(root, ...artifactPath.split("/"));
  const relative = path.relative(root, candidate);
  if (
    relative === "" ||
    (!relative.startsWith(`..${path.sep}`) &&
      relative !== ".." &&
      !path.isAbsolute(relative))
  ) {
    return candidate;
  }

  throw new Error("path escapes artifact root");
}

export function validateSchemaVersion(value, findings, artifactPath) {
  if (!Number.isInteger(value) || value < 1) {
    addFinding(
      findings,
      "error",
      "schema_version_invalid",
      "schema_version must be a positive integer",
      {path: artifactPath},
    );
  }
}

export function validateStatus(value, allowedStatuses, findings, artifactPath) {
  if (typeof value !== "string" || value.length === 0) {
    addFinding(
      findings,
      "error",
      "result_status_missing",
      "result.json must contain a non-empty status",
      {path: artifactPath},
    );
    return;
  }
  if (!allowedStatuses.has(value)) {
    addFinding(
      findings,
      "error",
      "result_status_unknown",
      `unsupported result status: ${value}`,
      {path: artifactPath, status: value},
    );
  }
}
