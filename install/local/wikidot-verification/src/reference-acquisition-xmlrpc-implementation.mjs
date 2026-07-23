import { types as utilTypes } from "node:util";

import { stableStringify } from "./corpus-import-manifest.mjs";
import {
  isReferenceObjectStore,
  validateReferenceObject,
} from "./reference-object-store.mjs";
import { exactDataRecord as dataObject } from "./wikidot-xmlrpc-exact-data-record.mjs";

export const WIKIDOT_XMLRPC_IMPLEMENTATION_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_implementation.v1";

const BUILD_KEYS = Object.freeze([
  "coordinatorFileSha256",
  "dependencyLockFileSha256",
  "nodeVersion",
  "pythonVersion",
  "wikijumpCommit",
  "wikijumpTree",
  "workerFileSha256",
  "workerRepositoryCommit",
  "workerRepositoryTree",
]);
const IMPLEMENTATION_KEYS = Object.freeze([
  "coordinator_file_sha256",
  "coordinator_path",
  "coordinator_repository",
  "dependency_lock_file_sha256",
  "dependency_lock_path",
  "endpoint",
  "fallback_used",
  "method",
  "node_version",
  "python_version",
  "rate_capacity",
  "rate_refill_per_second",
  "read_only",
  "retry_max_attempts",
  "retry_max_delay_seconds",
  "retry_wait_max_seconds",
  "schema",
  "site",
  "wikijump_commit",
  "wikijump_tree",
  "worker_file_sha256",
  "worker_path",
  "worker_repository",
  "worker_repository_commit",
  "worker_repository_tree",
]);
const FIXED_AUTHORITY = Object.freeze({
  coordinator_path:
    "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
  coordinator_repository: "Rokurolize/wikijump",
  dependency_lock_path: "uv.lock",
  endpoint: "https://www.wikidot.com/xml-rpc-api.php",
  fallback_used: false,
  method: "pages.get_one",
  rate_capacity: 1,
  rate_refill_per_second: 0.25,
  read_only: true,
  retry_max_attempts: 5,
  retry_max_delay_seconds: 120,
  retry_wait_max_seconds: 60,
  schema: WIKIDOT_XMLRPC_IMPLEMENTATION_SCHEMA,
  site: "scp-wiki",
  worker_path: "scripts/wikidot_xmlrpc_capture_worker.py",
  worker_repository: "Rokurolize/scp-wiki-translation",
});
const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const GIT_SHA1_RE = /^[0-9a-f]{40}$/u;
const MAX_BYTES = 16 * 1024;
const NODE_VERSION_RE = /^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u;
const PYTHON_VERSION_RE = /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u;
const SHA256_RE = /^[0-9a-f]{64}$/u;

function assertSha256(value, label) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

function normalizeImplementation(value) {
  const input = dataObject(
    value,
    IMPLEMENTATION_KEYS,
    "XML-RPC implementation",
  );
  for (const field of [
    "coordinator_file_sha256",
    "dependency_lock_file_sha256",
    "worker_file_sha256",
  ]) {
    assertSha256(input[field], `implementation ${field}`);
  }
  for (const field of [
    "wikijump_commit",
    "wikijump_tree",
    "worker_repository_commit",
    "worker_repository_tree",
  ]) {
    if (!GIT_SHA1_RE.test(input[field])) {
      throw new Error(`implementation ${field} must be a Git SHA-1`);
    }
  }
  if (
    !NODE_VERSION_RE.test(input.node_version) ||
    !PYTHON_VERSION_RE.test(input.python_version) ||
    Object.entries(FIXED_AUTHORITY).some(
      ([field, expected]) => input[field] !== expected,
    )
  ) {
    throw new Error("XML-RPC implementation authority fields are invalid");
  }
  return Object.freeze({ ...input });
}

function canonicalBytes(value) {
  const bytes = Buffer.from(
    `${stableStringify(normalizeImplementation(value))}\n`,
  );
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC implementation exceeds its byte limit");
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
    throw new Error("XML-RPC implementation input must be bytes");
  }
  try {
    return Buffer.from(value);
  } catch {
    throw new Error("XML-RPC implementation input must be bytes");
  }
}

function assertStore(store) {
  if (!isReferenceObjectStore(store)) {
    throw new Error("reference object store is required");
  }
}

export function buildWikidotXmlrpcImplementation(options) {
  const input = dataObject(options, BUILD_KEYS, "implementation build options");
  return normalizeImplementation({
    coordinator_file_sha256: input.coordinatorFileSha256,
    dependency_lock_file_sha256: input.dependencyLockFileSha256,
    ...FIXED_AUTHORITY,
    node_version: input.nodeVersion,
    python_version: input.pythonVersion,
    wikijump_commit: input.wikijumpCommit,
    wikijump_tree: input.wikijumpTree,
    worker_file_sha256: input.workerFileSha256,
    worker_repository_commit: input.workerRepositoryCommit,
    worker_repository_tree: input.workerRepositoryTree,
  });
}

export function serializeWikidotXmlrpcImplementation(value) {
  return canonicalBytes(value);
}

export function parseWikidotXmlrpcImplementation(value) {
  const bytes = inputBytes(value);
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC implementation exceeds its byte limit");
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
    throw new Error("XML-RPC implementation must contain one UTF-8 JSON line");
  }
  const normalized = normalizeImplementation(parsed);
  if (!canonicalBytes(normalized).equals(bytes)) {
    throw new Error("XML-RPC implementation bytes are not canonical");
  }
  return normalized;
}

export async function putWikidotXmlrpcImplementation(store, value) {
  assertStore(store);
  const descriptor = normalizeImplementation(value);
  const result = await store.putBytes(canonicalBytes(descriptor));
  return Object.freeze({ descriptor, ...result });
}

export async function openWikidotXmlrpcImplementation(store, reference) {
  assertStore(store);
  const object = validateReferenceObject(
    dataObject(
      reference,
      ["algorithm", "bytes", "sha256"],
      "implementation reference",
    ),
  );
  const descriptor = parseWikidotXmlrpcImplementation(
    await store.readObject(object, { maxBytes: MAX_BYTES }),
  );
  return Object.freeze({ descriptor, object });
}
