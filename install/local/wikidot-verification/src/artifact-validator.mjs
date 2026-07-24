import {constants as fsConstants} from "node:fs";
import {lstat, open} from "node:fs/promises";
import path from "node:path";

import {addFinding} from "./artifact-validation-common.mjs";
import {validateArtifactManifest} from "./artifact-manifest-validator.mjs";
import {
  assertArtifactKind,
  validateArtifactResult,
} from "./artifact-result-validator.mjs";

export const MAX_METADATA_JSON_BYTES = 1024 * 1024;
const FATAL_UTF8_DECODER = new TextDecoder("utf-8", {fatal: true});

export async function readBoundedFileHandle(fileHandle, maxBytes) {
  const bytes = Buffer.allocUnsafe(maxBytes + 1);
  let bytesRead = 0;

  while (bytesRead < bytes.length) {
    const result = await fileHandle.read(
      bytes,
      bytesRead,
      bytes.length - bytesRead,
      null,
    );
    if (result.bytesRead === 0) break;
    bytesRead += result.bytesRead;
  }

  return {
    bytes: bytes.subarray(0, bytesRead),
    bytesRead,
    tooLarge: bytesRead > maxBytes,
  };
}

async function readJsonFile(filePath, findings, codePrefix, artifactPath) {
  let stat;
  try {
    stat = await lstat(filePath);
  } catch (error) {
    addFinding(findings, "error", `${codePrefix}_missing`, `missing ${artifactPath}`, {
      path: artifactPath,
      detail: error.message,
    });
    return null;
  }

  if (!stat.isFile()) {
    addFinding(findings, "error", `${codePrefix}_not_regular`, `${artifactPath} must be a regular file`, {
      path: artifactPath,
    });
    return null;
  }

  if (stat.size > MAX_METADATA_JSON_BYTES) {
    addFinding(findings, "error", `${codePrefix}_too_large`, `${artifactPath} exceeds metadata size limit`, {
      path: artifactPath,
      max_bytes: MAX_METADATA_JSON_BYTES,
      actual_bytes: stat.size,
    });
    return null;
  }

  let text;
  let fileHandle;
  let closeFailed = false;
  try {
    fileHandle = await open(filePath, fsConstants.O_RDONLY | fsConstants.O_NOFOLLOW);
    stat = await fileHandle.stat();
    if (!stat.isFile()) {
      addFinding(findings, "error", `${codePrefix}_not_regular`, `${artifactPath} must be a regular file`, {
        path: artifactPath,
      });
      return null;
    }
    if (stat.size > MAX_METADATA_JSON_BYTES) {
      addFinding(findings, "error", `${codePrefix}_too_large`, `${artifactPath} exceeds metadata size limit`, {
        path: artifactPath,
        max_bytes: MAX_METADATA_JSON_BYTES,
        actual_bytes: stat.size,
      });
      return null;
    }
    const boundedRead = await readBoundedFileHandle(fileHandle, MAX_METADATA_JSON_BYTES);
    if (boundedRead.tooLarge) {
      addFinding(findings, "error", `${codePrefix}_too_large`, `${artifactPath} exceeds metadata size limit`, {
        path: artifactPath,
        max_bytes: MAX_METADATA_JSON_BYTES,
        actual_bytes: Math.max(stat.size, boundedRead.bytesRead),
      });
      return null;
    }
    try {
      text = FATAL_UTF8_DECODER.decode(boundedRead.bytes);
    } catch {
      addFinding(findings, "error", `${codePrefix}_invalid_encoding`, `${artifactPath} is not valid UTF-8`, {
        path: artifactPath,
      });
      return null;
    }
  } catch (error) {
    addFinding(findings, "error", `${codePrefix}_unreadable`, `could not read ${artifactPath}`, {
      path: artifactPath,
      detail: error.message,
    });
    return null;
  } finally {
    try {
      await fileHandle?.close();
    } catch (error) {
      closeFailed = true;
      addFinding(findings, "error", `${codePrefix}_unreadable`, `could not close ${artifactPath} after reading`, {
        path: artifactPath,
        detail: error.message,
      });
    }
  }

  if (closeFailed) return null;

  try {
    return JSON.parse(text);
  } catch {
    addFinding(findings, "error", `${codePrefix}_invalid_json`, `${artifactPath} is invalid JSON`, {
      path: artifactPath,
    });
    return null;
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

  const {artifactKind, artifactPaths: resultArtifactPaths} = validateArtifactResult({
    kind,
    result,
    findings,
    expectedTaskId,
    expectedAssignmentId,
  });
  for (const artifactPath of resultArtifactPaths) {
    baseRequiredFiles.add(artifactPath);
  }

  await validateArtifactManifest({
    artifactRoot: resolvedArtifactRoot,
    manifest,
    requiredFiles: [...baseRequiredFiles],
    findings,
  });

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
