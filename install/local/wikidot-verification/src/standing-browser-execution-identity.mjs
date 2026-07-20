import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import {
  requireNonEmptyString,
  requirePlainObject,
  requireSha256,
  sha256File,
  sha256Value,
} from "./standing-browser-parity-util.mjs";

const execFileAsync = promisify(execFile);
const SOURCE_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = path.resolve(SOURCE_DIR, "../../../..");

export const STANDING_BROWSER_EXECUTION_IDENTITY_SCHEMA =
  "wikijump.standing_browser_execution_identity.v1";

export const STANDING_BROWSER_EXECUTION_MODULES = Object.freeze([
  "install/local/wikidot-verification/scripts/run-standing-browser-parity.mjs",
  "install/local/wikidot-verification/src/atomic-no-replace.mjs",
  "install/local/wikidot-verification/src/browser-request-gate.mjs",
  "install/local/wikidot-verification/src/capture-egress-proxy.mjs",
  "install/local/wikidot-verification/src/standing-browser-canaries.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-browser-session.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-contract.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-observation.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-receipt.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-reference.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-runner.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-util.mjs",
  "install/local/wikidot-verification/src/standing-browser-pseudo-layout.mjs",
  "install/local/wikidot-verification/src/standing-browser-runtime-identity.mjs",
  "install/local/wikidot-verification/src/standing-browser-screenshot.mjs",
]);

function requireGitObject(value, name) {
  if (typeof value !== "string" || !/^[0-9a-f]{40}$/u.test(value)) {
    throw new Error(`${name} must be a full lowercase Git object id`);
  }
  return value;
}

function validateModuleManifest(value) {
  if (
    !Array.isArray(value) ||
    value.length !== STANDING_BROWSER_EXECUTION_MODULES.length
  ) {
    throw new Error(
      "standing browser execution identity has an incomplete module manifest",
    );
  }
  const expected = [...STANDING_BROWSER_EXECUTION_MODULES].sort();
  const actual = value.map((entry) =>
    requirePlainObject(entry, "execution module manifest entry"),
  );
  const paths = actual.map((entry) =>
    requireNonEmptyString(entry.path, "execution module path"),
  );
  if (JSON.stringify(paths) !== JSON.stringify(expected)) {
    throw new Error(
      "standing browser execution identity module manifest does not name exactly the loaded modules",
    );
  }
  return Object.freeze(
    actual.map((entry) => ({
      path: entry.path,
      sha256: requireSha256(
        entry.sha256,
        `execution module ${entry.path} SHA-256`,
      ),
    })),
  );
}

function ftmlPinFromLock(lockContents) {
  const section = lockContents.match(
    /\[\[package\]\]\nname = "ftml"\n[\s\S]*?(?=\n\[\[package\]\]|$)/u,
  )?.[0];
  const source = section?.match(/^source = "([^"]+)"$/mu)?.[1];
  const revision = source?.match(/#([0-9a-f]{40})$/u)?.[1];
  if (!revision)
    throw new Error(
      "deepwell/Cargo.lock does not contain an exact FTML revision",
    );
  return revision;
}

export function validateCandidateExecutionIdentity(value, candidateIdentity) {
  const execution = requirePlainObject(
    value,
    "candidate browser execution identity",
  );
  if (execution.schema !== STANDING_BROWSER_EXECUTION_IDENTITY_SCHEMA) {
    throw new Error(
      `candidate browser execution identity must use ${STANDING_BROWSER_EXECUTION_IDENTITY_SCHEMA}`,
    );
  }
  if (execution.source_clean !== true) {
    throw new Error(
      "candidate browser execution identity source checkout is not clean",
    );
  }
  if (
    requireGitObject(
      execution.wikijump_commit,
      "candidate browser execution Wikijump commit",
    ) !== candidateIdentity.candidate.wikijump_commit ||
    requireGitObject(
      execution.wikijump_tree,
      "candidate browser execution Wikijump tree",
    ) !== candidateIdentity.candidate.wikijump_tree ||
    requireGitObject(
      execution.ftml_sha,
      "candidate browser execution FTML SHA",
    ) !== candidateIdentity.candidate.ftml_sha
  ) {
    throw new Error(
      "candidate browser execution identity does not bind the sealed candidate source identity",
    );
  }
  const modules = validateModuleManifest(execution.modules);
  const moduleManifestSha256 = requireSha256(
    execution.module_manifest_sha256,
    "candidate browser execution module manifest SHA-256",
  );
  if (sha256Value(modules) !== moduleManifestSha256) {
    throw new Error(
      "candidate browser execution module manifest hash is invalid",
    );
  }
  return Object.freeze({
    schema: STANDING_BROWSER_EXECUTION_IDENTITY_SCHEMA,
    source_clean: true,
    wikijump_commit: candidateIdentity.candidate.wikijump_commit,
    wikijump_tree: candidateIdentity.candidate.wikijump_tree,
    ftml_sha: candidateIdentity.candidate.ftml_sha,
    modules,
    module_manifest_sha256: moduleManifestSha256,
  });
}

async function command(args) {
  const { stdout } = await execFileAsync(
    "git",
    ["-C", REPOSITORY_ROOT, ...args],
    {
      encoding: "utf8",
      timeout: 10_000,
      maxBuffer: 16 * 1024 * 1024,
    },
  );
  return stdout.trim();
}

export async function collectCandidateExecutionIdentity(candidateIdentity) {
  const [status, head, tree, lockContents] = await Promise.all([
    command(["status", "--porcelain=v1", "--untracked-files=all"]),
    command(["rev-parse", "HEAD"]),
    command(["rev-parse", "HEAD^{tree}"]),
    fs.readFile(path.join(REPOSITORY_ROOT, "deepwell", "Cargo.lock"), "utf8"),
  ]);
  if (status !== "") {
    throw new Error("candidate browser runner source checkout must be clean");
  }
  const modules = [];
  for (const relativePath of [...STANDING_BROWSER_EXECUTION_MODULES].sort()) {
    const filePath = path.join(REPOSITORY_ROOT, relativePath);
    const stat = await fs.lstat(filePath);
    if (!stat.isFile() || stat.isSymbolicLink()) {
      throw new Error(
        `candidate browser execution module is not a regular source file: ${relativePath}`,
      );
    }
    modules.push({ path: relativePath, sha256: await sha256File(filePath) });
  }
  return validateCandidateExecutionIdentity(
    {
      schema: STANDING_BROWSER_EXECUTION_IDENTITY_SCHEMA,
      source_clean: true,
      wikijump_commit: head,
      wikijump_tree: tree,
      ftml_sha: ftmlPinFromLock(lockContents),
      modules,
      module_manifest_sha256: sha256Value(modules),
    },
    candidateIdentity,
  );
}
