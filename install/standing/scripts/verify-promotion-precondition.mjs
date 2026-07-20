#!/usr/bin/env node

import { createHash } from "node:crypto";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import { verifyStandingCandidateParityAdmission } from "../../local/wikidot-verification/src/standing-browser-promotion-admission.mjs";
import { validateCandidateParityIdentity } from "../../local/wikidot-verification/src/standing-browser-parity-receipt.mjs";
import {
  requirePlainObject,
  requireSha256,
  sealJsonNoReplace,
} from "../../local/wikidot-verification/src/standing-browser-parity-util.mjs";

export const STANDING_PROMOTION_PRECONDITION_SCHEMA =
  "wikijump.standing_promotion_precondition.v1";

const REQUIRED_ARGUMENTS = Object.freeze([
  "receipt",
  "candidate-identity",
  "live-reference",
  "live-completion-policy",
  "build-evidence",
  "staging-home",
  "output",
]);

const REQUIRED_IMAGE_ROLES = Object.freeze([
  "cache",
  "caddy",
  "database",
  "deepwell",
  "files",
  "framerail",
  "wws",
]);

const BUILD_MANIFEST_EXCLUSIONS = Object.freeze([
  "evidence-manifest.sha256",
  "seal.json",
]);

function requireEqual(actual, expected, name) {
  if (actual !== expected) throw new Error(`${name} does not match`);
  return actual;
}

function requireRegularFile(stat, name) {
  if (
    !stat?.isFile() ||
    stat.isSymbolicLink() ||
    (stat.nlink !== 1 && stat.nlink !== 1n)
  ) {
    throw new Error(`${name} must be a regular file`);
  }
}

function statFingerprint(stat) {
  return Object.freeze({
    dev: String(stat.dev),
    ino: String(stat.ino),
    nlink: String(stat.nlink),
    mode: String(stat.mode),
    size: String(stat.size),
    mtimeNs: String(stat.mtimeNs),
    ctimeNs: String(stat.ctimeNs),
  });
}

function requireSameFingerprint(before, after, name) {
  if (JSON.stringify(before) !== JSON.stringify(after)) {
    throw new Error(`${name} changed while it was being read`);
  }
}

async function readStableRegularFile(filePath, name) {
  const beforeStat = await fs
    .lstat(filePath, { bigint: true })
    .catch(() => null);
  requireRegularFile(beforeStat, name);
  const before = statFingerprint(beforeStat);
  if (!fsConstants.O_NOFOLLOW) {
    throw new Error("stable file verification requires O_NOFOLLOW support");
  }
  const handle = await fs.open(
    filePath,
    fsConstants.O_RDONLY | fsConstants.O_NOFOLLOW,
  );
  let bytes;
  try {
    const openedStat = await handle.stat({ bigint: true });
    requireRegularFile(openedStat, name);
    requireSameFingerprint(before, statFingerprint(openedStat), name);
    bytes = await handle.readFile();
  } finally {
    await handle.close();
  }
  const afterStat = await fs
    .lstat(filePath, { bigint: true })
    .catch(() => null);
  requireRegularFile(afterStat, name);
  requireSameFingerprint(before, statFingerprint(afterStat), name);
  return Object.freeze({
    bytes,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  });
}

function jsonValueFromStableFile(file, name) {
  try {
    return JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(file.bytes),
    );
  } catch (error) {
    throw new Error(`${name} must contain valid UTF-8 JSON`, { cause: error });
  }
}

function jsonObjectFromStableFile(file, name) {
  return requirePlainObject(jsonValueFromStableFile(file, name), name);
}

async function resolveNonSymbolicDirectory(directoryPath, name) {
  const requested = path.resolve(directoryPath);
  const stat = await fs.lstat(requested).catch(() => null);
  if (!stat?.isDirectory() || stat.isSymbolicLink()) {
    throw new Error(`${name} must be a non-symbolic directory`);
  }
  const resolved = await fs.realpath(requested);
  const resolvedStat = await fs.lstat(resolved).catch(() => null);
  if (!resolvedStat?.isDirectory() || resolvedStat.isSymbolicLink()) {
    throw new Error(`${name} changed while it was being resolved`);
  }
  return resolved;
}

function isWithinOrEqual(root, candidate) {
  const relative = path.relative(root, candidate);
  return (
    relative === "" ||
    (!relative.startsWith(`..${path.sep}`) &&
      relative !== ".." &&
      !path.isAbsolute(relative))
  );
}

async function resolveOutputPath({
  outputPath,
  stagingHomeRoot,
  buildEvidenceRoot,
  inputPaths,
}) {
  const requested = path.resolve(outputPath);
  const parent = await fs.realpath(path.dirname(requested)).catch(() => null);
  if (!parent) {
    throw new Error("output parent directory must already exist");
  }
  const parentStat = await fs.lstat(parent).catch(() => null);
  if (!parentStat?.isDirectory() || parentStat.isSymbolicLink()) {
    throw new Error("output parent directory must be a non-symbolic directory");
  }
  const resolved = path.join(parent, path.basename(requested));
  if (isWithinOrEqual(stagingHomeRoot, resolved)) {
    throw new Error("output must not be within the rendered staging home");
  }
  if (isWithinOrEqual(buildEvidenceRoot, resolved)) {
    throw new Error("output must not be within sealed build evidence");
  }
  const resolvedInputs = await Promise.all(
    inputPaths.map(async ({ name, value }) => ({
      name,
      value: await fs.realpath(value).catch(() => path.resolve(value)),
    })),
  );
  const collision = resolvedInputs.find(({ value }) => value === resolved);
  if (collision) {
    throw new Error(`output must not equal ${collision.name}`);
  }
  return resolved;
}

function exactImageMap(value) {
  if (!Array.isArray(value) || value.length !== REQUIRED_IMAGE_ROLES.length) {
    throw new Error(
      "sealed final image inventory must contain every runtime role",
    );
  }
  const pairs = value.map((entry) => {
    const object = requirePlainObject(
      entry,
      "sealed final image inventory entry",
    );
    if (!/^[a-z][a-z0-9_-]*$/u.test(object.role ?? "")) {
      throw new Error("sealed final image inventory has an invalid role");
    }
    if (!/^sha256:[0-9a-f]{64}$/u.test(object.image_id ?? "")) {
      throw new Error(
        "sealed final image inventory has a mutable or invalid image id",
      );
    }
    if (object.os !== "linux" || object.architecture !== "amd64") {
      throw new Error(
        "sealed final image inventory must contain linux/amd64 images",
      );
    }
    return [object.role, object.image_id];
  });
  pairs.sort(([left], [right]) => left.localeCompare(right));
  if (
    JSON.stringify(pairs.map(([role]) => role)) !==
    JSON.stringify(REQUIRED_IMAGE_ROLES)
  ) {
    throw new Error("sealed final image inventory has an incomplete role set");
  }
  if (new Set(pairs.map(([, imageId]) => imageId)).size !== pairs.length) {
    throw new Error("sealed final image inventory reuses an image id");
  }
  return Object.freeze(Object.fromEntries(pairs));
}

function sameImageMap(left, right) {
  return (
    JSON.stringify(Object.entries(left).sort()) ===
    JSON.stringify(Object.entries(right).sort())
  );
}

function assertSafeRelativePath(relative, name) {
  if (
    relative === "" ||
    relative.startsWith("/") ||
    relative.includes("\\") ||
    relative.includes("\n") ||
    relative.includes("\r") ||
    relative.includes("\0") ||
    relative
      .split("/")
      .some((part) => part === "" || part === "." || part === "..")
  ) {
    throw new Error(`${name} contains an unsafe path: ${relative}`);
  }
  return relative;
}

function sortPaths(paths) {
  return [...paths].sort((left, right) =>
    Buffer.compare(Buffer.from(left), Buffer.from(right)),
  );
}

function requireSamePathSet(expected, actual, name) {
  if (
    JSON.stringify(sortPaths(expected)) !== JSON.stringify(sortPaths(actual))
  ) {
    throw new Error(`${name} does not match the sealed manifest`);
  }
}

async function collectRegularFiles(root, label, relative = "") {
  const entries = await fs.readdir(path.join(root, relative), {
    withFileTypes: true,
  });
  const files = [];
  for (const entry of entries) {
    const next = path.join(relative, entry.name);
    const absolute = path.join(root, next);
    const stat = await fs.lstat(absolute);
    if (stat.isSymbolicLink()) {
      throw new Error(`${label} contains a symbolic link: ${next}`);
    }
    if (stat.isDirectory()) {
      files.push(...(await collectRegularFiles(root, label, next)));
    } else if (stat.isFile()) {
      files.push(assertSafeRelativePath(next, label));
    } else {
      throw new Error(`${label} contains a non-regular entry: ${next}`);
    }
  }
  return files;
}

async function renderedHomeBinding(stagingHomePath) {
  const root = await resolveNonSymbolicDirectory(
    stagingHomePath,
    "staging home",
  );
  const files = sortPaths(await collectRegularFiles(root, "rendered home"));
  const contents = await Promise.all(
    files.map(async (relative) => [
      relative,
      await readStableRegularFile(
        path.join(root, relative),
        `rendered home file ${relative}`,
      ),
    ]),
  );
  requireSamePathSet(
    files,
    await collectRegularFiles(root, "rendered home"),
    "rendered home",
  );
  const manifest = createHash("sha256");
  for (const [relative, file] of contents) {
    manifest.update(`${file.sha256}  ./${relative}\n`);
  }
  return Object.freeze({ root, manifest_sha256: manifest.digest("hex") });
}

// This exactly matches the host controller's sorted `sha256sum` manifest
// algorithm so the sealed candidate identity can bind the rendered topology.
export async function renderedHomeManifestSha256(stagingHomePath) {
  return (await renderedHomeBinding(stagingHomePath)).manifest_sha256;
}

function parseEvidenceManifest(file) {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(file.bytes);
  if (!text.endsWith("\n") || text === "\n") {
    throw new Error(
      "build evidence manifest must contain newline-terminated entries",
    );
  }
  const entries = new Map();
  for (const line of text.slice(0, -1).split("\n")) {
    if (line.startsWith("\\")) {
      throw new Error("build evidence manifest uses unsupported escaped paths");
    }
    const match = /^([0-9a-f]{64})  \.\/(.+)$/u.exec(line);
    if (!match) throw new Error("build evidence manifest has an invalid entry");
    const [, sha256, relative] = match;
    assertSafeRelativePath(relative, "build evidence manifest");
    if (BUILD_MANIFEST_EXCLUSIONS.includes(relative) || entries.has(relative)) {
      throw new Error(
        "build evidence manifest has an excluded or duplicate path",
      );
    }
    entries.set(relative, sha256);
  }
  return entries;
}

function requireNonEmptyString(value, name) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${name} must be a non-empty string`);
  }
  return value;
}

async function readBuildBinding(buildEvidenceRoot) {
  // The host controller runs its broader sealed-build validator before this
  // adapter. This binding layer verifies the manifest and the exact values
  // that must agree with the independently measured candidate.
  const root = buildEvidenceRoot;
  const [sealFile, manifestFile] = await Promise.all([
    readStableRegularFile(path.join(root, "seal.json"), "build seal"),
    readStableRegularFile(
      path.join(root, "evidence-manifest.sha256"),
      "build evidence manifest",
    ),
  ]);
  const seal = jsonObjectFromStableFile(sealFile, "build seal");
  const manifestEntries = parseEvidenceManifest(manifestFile);
  const expectedFiles = [...manifestEntries.keys()];
  const actualFiles = (
    await collectRegularFiles(root, "build evidence")
  ).filter((relative) => !BUILD_MANIFEST_EXCLUSIONS.includes(relative));
  requireSamePathSet(expectedFiles, actualFiles, "build evidence file set");
  const files = new Map(
    await Promise.all(
      sortPaths(expectedFiles).map(async (relative) => [
        relative,
        await readStableRegularFile(
          path.join(root, relative),
          `build evidence file ${relative}`,
        ),
      ]),
    ),
  );
  requireSamePathSet(
    expectedFiles,
    (await collectRegularFiles(root, "build evidence")).filter(
      (relative) => !BUILD_MANIFEST_EXCLUSIONS.includes(relative),
    ),
    "build evidence file set",
  );
  for (const [relative, expectedSha256] of manifestEntries) {
    requireEqual(
      files.get(relative).sha256,
      expectedSha256,
      `build evidence manifest digest for ${relative}`,
    );
  }
  const verdictFile = files.get("verdict.json");
  const imagesFile = files.get("images/final-images.json");
  if (!verdictFile || !imagesFile) {
    throw new Error(
      "build evidence manifest is missing required promotion inputs",
    );
  }
  const verdict = jsonObjectFromStableFile(verdictFile, "build verdict");
  const images = jsonValueFromStableFile(imagesFile, "final image inventory");
  if (
    seal.schema !== "wikijump.standing_provenance_build_seal.v1" ||
    seal.status !== "sealed"
  ) {
    throw new Error("build seal has an unsupported status or schema");
  }
  requireNonEmptyString(seal.run_id, "build seal run_id");
  if (
    seal.evidence_manifest_verified !== true ||
    JSON.stringify(seal.evidence_manifest_exclusions) !==
      JSON.stringify(BUILD_MANIFEST_EXCLUSIONS)
  ) {
    throw new Error("build seal has an invalid evidence manifest contract");
  }
  requireEqual(
    requireSha256(
      seal.evidence_manifest_sha256,
      "build seal evidence manifest SHA-256",
    ),
    manifestFile.sha256,
    "build seal evidence manifest SHA-256",
  );
  requireEqual(
    requireSha256(seal.verdict_sha256, "build seal verdict SHA-256"),
    verdictFile.sha256,
    "build seal verdict SHA-256",
  );
  if (
    verdict.schema !== "wikijump.standing_provenance_build.v1" ||
    verdict.status !== "pass" ||
    verdict.promotion_eligible !== true
  ) {
    throw new Error("build verdict is not a passing promotion-eligible build");
  }
  requireEqual(
    requireNonEmptyString(verdict.run_id, "build verdict run_id"),
    seal.run_id,
    "build seal and verdict run id",
  );
  requireEqual(
    verdict.final_images,
    "images/final-images.json",
    "build verdict final image path",
  );
  for (const [key, name] of [
    ["wikijump_commit", "build verdict Wikijump commit"],
    ["wikijump_tree", "build verdict Wikijump tree"],
    ["ftml_sha", "build verdict FTML SHA"],
  ]) {
    if (!/^[0-9a-f]{40}$/u.test(verdict[key] ?? "")) {
      throw new Error(`${name} must be a Git object id`);
    }
  }
  return Object.freeze({
    seal_sha256: sealFile.sha256,
    evidence_manifest_sha256: manifestFile.sha256,
    verdict_sha256: verdictFile.sha256,
    final_images_sha256: imagesFile.sha256,
    wikijump_commit: verdict.wikijump_commit,
    wikijump_tree: verdict.wikijump_tree,
    ftml_sha: verdict.ftml_sha,
    images: exactImageMap(images),
  });
}

function bindAdmissionToPromotion({
  admission,
  identity,
  identitySha256,
  build,
  stagingManifestSha256,
}) {
  if (admission?.status !== "pass") {
    throw new Error("candidate parity admission did not pass");
  }
  requireEqual(
    admission.candidate_identity_sha256,
    identitySha256,
    "admission candidate identity SHA-256",
  );
  for (const key of [
    "wikijump_commit",
    "wikijump_tree",
    "ftml_sha",
    "artifact_key",
  ]) {
    requireEqual(
      admission.candidate?.[key],
      key === "artifact_key" ? identity.artifact_key : identity.candidate[key],
      `admission candidate ${key}`,
    );
  }
  for (const key of ["wikijump_commit", "wikijump_tree", "ftml_sha"]) {
    requireEqual(
      identity.candidate[key],
      build[key],
      `candidate ${key} versus sealed build`,
    );
  }
  requireEqual(
    identity.build.seal_sha256,
    build.seal_sha256,
    "candidate build seal SHA-256",
  );
  requireEqual(
    identity.build.verdict_sha256,
    build.verdict_sha256,
    "candidate build verdict SHA-256",
  );
  requireEqual(
    identity.build.final_images_sha256,
    build.final_images_sha256,
    "candidate final image inventory SHA-256",
  );
  if (!sameImageMap(identity.candidate.images, build.images)) {
    throw new Error("candidate image map does not match the sealed build");
  }
  requireEqual(
    identity.candidate.config.promotion_base_manifest_sha256,
    stagingManifestSha256,
    "candidate promotion-base manifest SHA-256",
  );
}

async function readPromotionBinding({
  candidateIdentityPath,
  buildEvidenceRoot,
  stagingHomeRoot,
}) {
  const [identityFile, build, stagingHome] = await Promise.all([
    readStableRegularFile(candidateIdentityPath, "candidate parity identity"),
    readBuildBinding(buildEvidenceRoot),
    renderedHomeBinding(stagingHomeRoot),
  ]);
  return Object.freeze({
    identity_sha256: identityFile.sha256,
    identity: validateCandidateParityIdentity(
      jsonObjectFromStableFile(identityFile, "candidate parity identity"),
    ),
    build,
    staging_home: stagingHome,
  });
}

function requireStablePromotionBinding(before, after) {
  requireEqual(
    after.identity_sha256,
    before.identity_sha256,
    "candidate parity identity changed during promotion binding",
  );
  for (const key of [
    "seal_sha256",
    "evidence_manifest_sha256",
    "verdict_sha256",
    "final_images_sha256",
    "wikijump_commit",
    "wikijump_tree",
    "ftml_sha",
  ]) {
    requireEqual(
      after.build[key],
      before.build[key],
      `sealed build ${key} changed during promotion binding`,
    );
  }
  if (!sameImageMap(after.build.images, before.build.images)) {
    throw new Error(
      "sealed build image inventory changed during promotion binding",
    );
  }
  requireEqual(
    after.staging_home.manifest_sha256,
    before.staging_home.manifest_sha256,
    "rendered staging home changed during promotion binding",
  );
}

export async function verifyStandingPromotionPrecondition({
  receiptPath,
  candidateIdentityPath,
  liveReferencePath,
  liveCompletionPolicyPath,
  buildEvidencePath,
  stagingHomePath,
  outputPath,
  now,
  verifyAdmission = verifyStandingCandidateParityAdmission,
}) {
  const [buildEvidenceRoot, stagingHomeRoot] = await Promise.all([
    resolveNonSymbolicDirectory(buildEvidencePath, "build evidence"),
    resolveNonSymbolicDirectory(stagingHomePath, "staging home"),
  ]);
  const sealedOutputPath = await resolveOutputPath({
    outputPath,
    buildEvidenceRoot,
    stagingHomeRoot,
    inputPaths: [
      { name: "candidate parity receipt", value: receiptPath },
      { name: "candidate parity identity", value: candidateIdentityPath },
      { name: "live reference", value: liveReferencePath },
      { name: "live completion policy", value: liveCompletionPolicyPath },
    ],
  });
  const admission = await verifyAdmission({
    receiptPath,
    candidateIdentityPath,
    liveReferencePath,
    liveCompletionPolicyPath,
    now,
  });
  const binding = await readPromotionBinding({
    candidateIdentityPath,
    buildEvidenceRoot,
    stagingHomeRoot,
  });
  bindAdmissionToPromotion({
    admission,
    identity: binding.identity,
    identitySha256: binding.identity_sha256,
    build: binding.build,
    stagingManifestSha256: binding.staging_home.manifest_sha256,
  });
  requireStablePromotionBinding(
    binding,
    await readPromotionBinding({
      candidateIdentityPath,
      buildEvidenceRoot,
      stagingHomeRoot,
    }),
  );
  const result = Object.freeze({
    schema: STANDING_PROMOTION_PRECONDITION_SCHEMA,
    status: "pass",
    verified_at: new Date().toISOString(),
    admission: {
      candidate_parity_receipt_sha256:
        admission.candidate_parity_receipt_sha256,
      candidate_identity_sha256: admission.candidate_identity_sha256,
      live_reference_sha256: admission.live_reference_sha256,
      live_completion_policy_sha256: admission.live_completion_policy_sha256,
      source_runner_sha256: admission.source_runner_sha256,
      source_observation_sha256: admission.source_observation_sha256,
      source_execution_identity_sha256:
        admission.source_execution_identity_sha256,
    },
    candidate: {
      artifact_key: binding.identity.artifact_key,
      wikijump_commit: binding.identity.candidate.wikijump_commit,
      wikijump_tree: binding.identity.candidate.wikijump_tree,
      ftml_sha: binding.identity.candidate.ftml_sha,
      compose_project: binding.identity.candidate.compose_project,
      expires_at: binding.identity.candidate.expires_at,
    },
    build: binding.build,
    staging_home: { manifest_sha256: binding.staging_home.manifest_sha256 },
  });
  const sealed = await sealJsonNoReplace(sealedOutputPath, result);
  return Object.freeze({ ...result, output: sealed });
}

function parseArgs(argv) {
  const values = {};
  for (let index = 2; index < argv.length; index += 1) {
    const flag = argv[index];
    if (!flag.startsWith("--")) throw new Error(`unknown argument: ${flag}`);
    const key = flag.slice(2);
    if (!REQUIRED_ARGUMENTS.includes(key))
      throw new Error(`unknown argument: ${flag}`);
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`${flag} requires a value`);
    }
    if (values[key]) throw new Error(`${flag} may be supplied only once`);
    values[key] = path.resolve(value);
    index += 1;
  }
  for (const key of REQUIRED_ARGUMENTS) {
    if (!values[key]) throw new Error(`--${key} is required`);
  }
  return values;
}

function printHelp() {
  console.log(`Usage: verify-promotion-precondition.mjs --receipt FILE --candidate-identity FILE --live-reference FILE --live-completion-policy FILE --build-evidence DIRECTORY --staging-home DIRECTORY --output FILE

Verifies the reviewed source browser-parity admission and binds it to the exact sealed build and rendered standing topology. It has no Docker, maintenance, canonical-home, or network side effects.`);
}

async function main() {
  if (process.argv.includes("--help") || process.argv.includes("-h")) {
    printHelp();
    return;
  }
  const args = parseArgs(process.argv);
  const result = await verifyStandingPromotionPrecondition({
    receiptPath: args.receipt,
    candidateIdentityPath: args["candidate-identity"],
    liveReferencePath: args["live-reference"],
    liveCompletionPolicyPath: args["live-completion-policy"],
    buildEvidencePath: args["build-evidence"],
    stagingHomePath: args["staging-home"],
    outputPath: args.output,
  });
  console.log(
    JSON.stringify({
      schema: "wikijump.standing_promotion_precondition_cli_result.v1",
      status: "pass",
      output: result.output.path,
      sha256: result.output.sha256,
    }),
  );
}

if (import.meta.url === new URL(process.argv[1], "file:").href) {
  main().catch((error) => {
    console.error(
      JSON.stringify({
        schema: "wikijump.standing_promotion_precondition_cli_result.v1",
        status: "error",
        error: error?.message ?? String(error),
      }),
    );
    process.exitCode = 1;
  });
}
