import { types as utilTypes } from "node:util";

import { stableStringify } from "./canonical-json.mjs";
import {
  isReferenceObjectStore,
  validateReferenceObject,
} from "./reference-object-store.mjs";
import { exactDataRecord as dataObject } from "./wikidot-xmlrpc-exact-data-record.mjs";

export const WIKIDOT_XMLRPC_WORKER_AUTHORITY_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_worker_authority.v1";

// This module only serializes identity claims. A later capsule verifier must
// prove their versioned canonical preimages before it can launch any worker.

const BUILD_KEYS = Object.freeze([
  "dependencyEnvironmentSha256",
  "dependencyLockFileSha256",
  "dependencyRecipeSha256",
  "pythonExecutableSha256",
  "pythonVersion",
  "venvConfigSha256",
  "workerBlobOid",
  "workerFileSha256",
  "workerRepositoryCommit",
  "workerRepositoryTree",
]);
const AUTHORITY_KEYS = Object.freeze([
  "dependency_environment_sha256",
  "dependency_lock_file_sha256",
  "dependency_recipe_sha256",
  "python_executable_sha256",
  "python_version",
  "schema",
  "venv_config_sha256",
  "worker_blob_oid",
  "worker_file_sha256",
  "worker_repository",
  "worker_repository_commit",
  "worker_repository_tree",
]);
const FIXED_AUTHORITY = Object.freeze({
  schema: WIKIDOT_XMLRPC_WORKER_AUTHORITY_SCHEMA,
  worker_repository: "Rokurolize/scp-wiki-translation",
});
const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const GIT_SHA1_RE = /^[0-9a-f]{40}$/u;
const MAX_BYTES = 8 * 1024;
const MAX_PYTHON_VERSION_CHARS = 64;
const PYTHON_VERSION_RE = /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u;
const SHA256_RE = /^[0-9a-f]{64}$/u;

function assertSha256(value, label) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

function normalizeAuthority(value) {
  const input = dataObject(value, AUTHORITY_KEYS, "XML-RPC worker authority");
  for (const field of [
    "dependency_environment_sha256",
    "dependency_lock_file_sha256",
    "dependency_recipe_sha256",
    "python_executable_sha256",
    "venv_config_sha256",
    "worker_file_sha256",
  ]) {
    assertSha256(input[field], `worker authority ${field}`);
  }
  for (const field of [
    "worker_blob_oid",
    "worker_repository_commit",
    "worker_repository_tree",
  ]) {
    if (typeof input[field] !== "string" || !GIT_SHA1_RE.test(input[field])) {
      throw new Error(`worker authority ${field} must be a Git SHA-1`);
    }
  }
  if (
    typeof input.python_version !== "string" ||
    input.python_version.length > MAX_PYTHON_VERSION_CHARS ||
    !PYTHON_VERSION_RE.test(input.python_version) ||
    Object.entries(FIXED_AUTHORITY).some(
      ([field, expected]) => input[field] !== expected,
    )
  ) {
    throw new Error("XML-RPC worker authority fields are invalid");
  }
  return Object.freeze({ ...input });
}

function canonicalBytes(value) {
  const bytes = Buffer.from(`${stableStringify(normalizeAuthority(value))}\n`);
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC worker authority exceeds its byte limit");
  }
  return bytes;
}

function inputBytes(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    utilTypes.isProxy(value) ||
    !(value instanceof Uint8Array)
  ) {
    throw new Error("XML-RPC worker authority input must be bytes");
  }
  try {
    return Buffer.from(value);
  } catch {
    throw new Error("XML-RPC worker authority input must be bytes");
  }
}

function assertStore(store) {
  if (!isReferenceObjectStore(store)) {
    throw new Error("reference object store is required");
  }
}

export function buildWikidotXmlrpcWorkerAuthority(options) {
  const input = dataObject(
    options,
    BUILD_KEYS,
    "worker authority build options",
  );
  return normalizeAuthority({
    dependency_environment_sha256: input.dependencyEnvironmentSha256,
    dependency_lock_file_sha256: input.dependencyLockFileSha256,
    dependency_recipe_sha256: input.dependencyRecipeSha256,
    python_executable_sha256: input.pythonExecutableSha256,
    python_version: input.pythonVersion,
    ...FIXED_AUTHORITY,
    venv_config_sha256: input.venvConfigSha256,
    worker_blob_oid: input.workerBlobOid,
    worker_file_sha256: input.workerFileSha256,
    worker_repository_commit: input.workerRepositoryCommit,
    worker_repository_tree: input.workerRepositoryTree,
  });
}

export function serializeWikidotXmlrpcWorkerAuthority(value) {
  return canonicalBytes(value);
}

export function parseWikidotXmlrpcWorkerAuthority(value) {
  const bytes = inputBytes(value);
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC worker authority exceeds its byte limit");
  }
  let parsed;
  try {
    const text = FATAL_UTF8_DECODER.decode(bytes);
    if (
      !text.endsWith("\n") ||
      text.slice(0, -1).includes("\n") ||
      text.includes("\r")
    ) {
      throw new Error();
    }
    parsed = JSON.parse(text);
  } catch {
    throw new Error(
      "XML-RPC worker authority must contain one UTF-8 JSON line",
    );
  }
  const normalized = normalizeAuthority(parsed);
  if (!canonicalBytes(normalized).equals(bytes)) {
    throw new Error("XML-RPC worker authority bytes are not canonical");
  }
  return normalized;
}

export async function putWikidotXmlrpcWorkerAuthority(store, value) {
  assertStore(store);
  const descriptor = normalizeAuthority(value);
  const result = await store.putBytes(canonicalBytes(descriptor));
  return Object.freeze({ descriptor, ...result });
}

export async function openWikidotXmlrpcWorkerAuthority(store, reference) {
  assertStore(store);
  const object = validateReferenceObject(
    dataObject(
      reference,
      ["algorithm", "bytes", "sha256"],
      "worker authority reference",
    ),
  );
  let bytes;
  try {
    bytes = await store.readObject(object, { maxBytes: MAX_BYTES });
  } catch {
    throw new Error("XML-RPC worker authority object cannot be read");
  }
  const descriptor = parseWikidotXmlrpcWorkerAuthority(bytes);
  return Object.freeze({ descriptor, object });
}
