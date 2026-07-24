import {createHash} from "node:crypto";
import {lstat, readFile} from "node:fs/promises";

import {
  addFinding,
  isObject,
  isSafeRelativeArtifactPath,
  resolveContained,
  SHA256_RE,
  validateSchemaVersion,
} from "./artifact-validation-common.mjs";

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

export async function validateArtifactManifest({
  artifactRoot,
  manifest,
  requiredFiles,
  findings,
}) {
  await validateRequiredArtifactPaths({artifactRoot, requiredFiles, findings});
  if (manifest === null) return;

  const entries = validateManifestShape(manifest, findings);
  const seenPaths = new Set();
  for (const [index, entry] of entries.entries()) {
    await validateManifestEntry({artifactRoot, entry, index, seenPaths, findings});
  }
}
