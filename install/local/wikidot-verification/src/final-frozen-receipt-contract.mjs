// Final-frozen receipt: the machine-checkable claim that one exact source
// identity passed the repository's validation barrier and produced one exact
// set of sealed images. Every field is derived from an executed check or a
// content hash; nothing in it is an attestation.

import path from "node:path";

import {
  isPlainObject,
  readStableRegularFile,
  requireNonEmptyString,
  requirePlainObject,
  requireSha256,
} from "./standing-browser-parity-util.mjs";

export const FINAL_FROZEN_RECEIPT_SCHEMA = "wikijump.final_frozen_receipt.v2";
export const CONSOLIDATED_VALIDATION_SCHEMA =
  "wikijump.consolidated_validation_receipt.v2";

const GIT_OBJECT = /^[0-9a-f]{40}$/u;
const IMAGE_ID = /^sha256:[0-9a-f]{64}$/u;
const REF_KEYS = ["path", "sha256"];
const RECEIPT_KEYS = ["images", "schema", "source", "status", "validation"];
const VALIDATION_KEYS = [
  "base_commit",
  "check_count",
  "groups",
  "path",
  "sha256",
];
const CONSOLIDATED_KEYS = [
  "base_commit",
  "checks",
  "completed_at",
  "groups",
  "mode",
  "preflight",
  "schema",
  "started_at",
  "status",
  "tool_versions",
  "wikijump_commit",
  "wikijump_tree",
];

function exactKeys(value, expected, name) {
  if (
    JSON.stringify(Object.keys(value).sort()) !==
    JSON.stringify([...expected].sort())
  ) {
    throw new Error(`${name} has missing or unknown fields`);
  }
}

function gitObject(value, name) {
  if (!GIT_OBJECT.test(value ?? "")) {
    throw new Error(`${name} must be a full lowercase Git object id`);
  }
  return value;
}

function reference(value, name) {
  const ref = requirePlainObject(value, name);
  exactKeys(ref, REF_KEYS, name);
  const filePath = requireNonEmptyString(ref.path, `${name}.path`);
  if (!path.isAbsolute(filePath)) {
    throw new Error(`${name}.path must be absolute`);
  }
  return Object.freeze({
    path: filePath,
    sha256: requireSha256(ref.sha256, `${name}.sha256`),
  });
}

function references(value, name) {
  if (!Array.isArray(value) || value.length === 0) {
    throw new Error(`${name} must contain one or more file references`);
  }
  const result = value.map((entry, index) =>
    reference(entry, `${name}[${index}]`),
  );
  if (new Set(result.map((entry) => entry.path)).size !== result.length) {
    throw new Error(`${name} contains duplicate file references`);
  }
  return Object.freeze(result);
}

/** A consolidated validation receipt: the final preflight barrier run on one
 * exact clean commit, with every selected check and its result. */
export function validateConsolidatedValidation(value, source) {
  const receipt = requirePlainObject(value, "consolidated validation");
  exactKeys(receipt, CONSOLIDATED_KEYS, "consolidated validation");
  if (receipt.schema !== CONSOLIDATED_VALIDATION_SCHEMA) {
    throw new Error("consolidated validation has an unsupported schema");
  }
  if (receipt.mode !== "final") {
    throw new Error("consolidated validation must run the final barrier");
  }
  gitObject(receipt.base_commit, "consolidated validation base commit");
  if (
    receipt.wikijump_commit !== source.wikijump_commit ||
    receipt.wikijump_tree !== source.wikijump_tree
  ) {
    throw new Error("consolidated validation source identity is stale");
  }
  if (!Array.isArray(receipt.checks) || receipt.checks.length === 0) {
    throw new Error("consolidated validation ran no checks");
  }
  const failed = receipt.checks.filter((check) => check?.status !== "ok");
  for (const check of receipt.checks) {
    requireNonEmptyString(check?.name, "consolidated validation check name");
  }
  if (receipt.status !== "pass" || failed.length > 0) {
    throw new Error(
      `consolidated validation did not pass: ${
        failed.map((check) => check.name).join(", ") || receipt.status
      }`,
    );
  }
  if (!isPlainObject(receipt.groups) || !Object.values(receipt.groups).some(Boolean)) {
    throw new Error("consolidated validation selected no check groups");
  }
  return Object.freeze({
    base_commit: receipt.base_commit,
    check_count: receipt.checks.length,
    groups: Object.freeze({...receipt.groups}),
  });
}

export function validateImageProducer(value, source) {
  const producer = requirePlainObject(value, "image producer output");
  exactKeys(
    producer,
    ["ftml_sha", "images", "status", "wikijump_sha", "wikijump_tree"],
    "image producer output",
  );
  if (producer.status !== "pass" || !isPlainObject(producer.images)) {
    throw new Error(
      "image producer output must be a passing receipt with an images object",
    );
  }
  for (const [producerKey, sourceKey] of [
    ["wikijump_sha", "wikijump_commit"],
    ["wikijump_tree", "wikijump_tree"],
    ["ftml_sha", "ftml_sha"],
  ]) {
    if (producer[producerKey] !== source[sourceKey]) {
      throw new Error(
        `image producer output ${producerKey} does not match the frozen source`,
      );
    }
  }
  const identities = Object.entries(producer.images).map(([role, image]) => {
    const id = typeof image === "string" ? image : image?.id;
    if (!/^[a-z][a-z0-9_-]*$/u.test(role) || !IMAGE_ID.test(id ?? "")) {
      throw new Error(
        `image producer output has no immutable image ID for ${role}`,
      );
    }
    return [role, id];
  });
  if (identities.length === 0) {
    throw new Error("image producer output must contain image identities");
  }
  return Object.freeze(Object.fromEntries(identities.sort()));
}

export function validateFinalFrozenReceipt(value, {source = null} = {}) {
  const receipt = requirePlainObject(value, "final frozen receipt");
  exactKeys(receipt, RECEIPT_KEYS, "final frozen receipt");
  if (
    receipt.schema !== FINAL_FROZEN_RECEIPT_SCHEMA ||
    receipt.status !== "FINAL_FROZEN"
  ) {
    throw new Error("final frozen receipt is not a FINAL_FROZEN v2 receipt");
  }
  const sourceValue = requirePlainObject(receipt.source, "final frozen source");
  exactKeys(
    sourceValue,
    ["ftml_sha", "lockfiles", "wikijump_commit", "wikijump_tree"],
    "final frozen source",
  );
  const normalizedSource = Object.freeze({
    wikijump_commit: gitObject(sourceValue.wikijump_commit, "final frozen Wikijump commit"),
    wikijump_tree: gitObject(sourceValue.wikijump_tree, "final frozen Wikijump tree"),
    ftml_sha: gitObject(sourceValue.ftml_sha, "final frozen FTML commit"),
    lockfiles: references(sourceValue.lockfiles, "final frozen lockfiles"),
  });
  if (source) {
    for (const key of ["wikijump_commit", "wikijump_tree", "ftml_sha"]) {
      if (normalizedSource[key] !== source[key]) {
        throw new Error(`final frozen ${key} does not match the expected source`);
      }
    }
  }
  const validation = requirePlainObject(receipt.validation, "final frozen validation");
  exactKeys(validation, VALIDATION_KEYS, "final frozen validation");
  const images = requirePlainObject(receipt.images, "final frozen images");
  exactKeys(images, ["identities", "producer"], "final frozen images");
  if (!isPlainObject(images.identities) || Object.keys(images.identities).length === 0) {
    throw new Error("final frozen images have no identities");
  }
  return Object.freeze({
    schema: FINAL_FROZEN_RECEIPT_SCHEMA,
    status: "FINAL_FROZEN",
    source: normalizedSource,
    validation: Object.freeze({
      ...reference({path: validation.path, sha256: validation.sha256}, "final frozen validation"),
      base_commit: gitObject(validation.base_commit, "final frozen validation base commit"),
      check_count: validation.check_count,
      groups: validation.groups,
    }),
    images: Object.freeze({
      producer: reference(images.producer, "final frozen image producer"),
      identities: Object.freeze({...images.identities}),
    }),
  });
}

function parseJson(bytes, name) {
  try {
    return JSON.parse(new TextDecoder("utf-8", {fatal: true}).decode(bytes));
  } catch (error) {
    throw new Error(`${name} must contain valid UTF-8 JSON`, {cause: error});
  }
}

async function verifyReference(ref, name) {
  const file = await readStableRegularFile(ref.path, name);
  if (file.sha256 !== ref.sha256) {
    throw new Error(`${name} is stale`);
  }
  return file;
}

export async function verifyFinalFrozenReceipt({receiptPath, source}) {
  const receiptFile = await readStableRegularFile(
    receiptPath,
    "final frozen receipt",
  );
  const receipt = validateFinalFrozenReceipt(
    parseJson(receiptFile.bytes, "final frozen receipt"),
    {source},
  );
  await Promise.all(
    receipt.source.lockfiles.map((entry) =>
      verifyReference(entry, "final frozen lockfile"),
    ),
  );
  const validationFile = await verifyReference(
    receipt.validation,
    "final frozen consolidated validation",
  );
  const validation = validateConsolidatedValidation(
    parseJson(validationFile.bytes, "final frozen consolidated validation"),
    receipt.source,
  );
  if (
    validation.base_commit !== receipt.validation.base_commit ||
    validation.check_count !== receipt.validation.check_count ||
    JSON.stringify(validation.groups) !== JSON.stringify(receipt.validation.groups)
  ) {
    throw new Error("final frozen validation summary disagrees with its receipt");
  }
  const imageProducerFile = await verifyReference(
    receipt.images.producer,
    "final frozen image producer",
  );
  const identities = validateImageProducer(
    parseJson(imageProducerFile.bytes, "final frozen image producer"),
    receipt.source,
  );
  if (JSON.stringify(identities) !== JSON.stringify(receipt.images.identities)) {
    throw new Error("final frozen image identities are stale");
  }
  return Object.freeze({
    path: path.resolve(receiptPath),
    sha256: receiptFile.sha256,
    receipt,
  });
}
