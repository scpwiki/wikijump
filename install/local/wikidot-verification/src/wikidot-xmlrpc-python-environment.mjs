import { types as utilTypes } from "node:util";

import { stableStringify } from "./canonical-json.mjs";
import {
  isReferenceObjectStore,
  validateReferenceObject,
} from "./reference-object-store.mjs";
import {
  hashWikidotXmlrpcInstalledEnvironmentManifest,
  parseWikidotXmlrpcInstalledEnvironmentManifest,
  serializeWikidotXmlrpcInstalledEnvironmentManifest,
} from "./wikidot-xmlrpc-installed-environment-manifest.mjs";
import { exactDataRecord as dataObject } from "./wikidot-xmlrpc-exact-data-record.mjs";
import {
  parseWikidotXmlrpcWorkerAuthority,
  serializeWikidotXmlrpcWorkerAuthority,
} from "./wikidot-xmlrpc-worker-authority.mjs";

export const WIKIDOT_XMLRPC_PYTHON_ENVIRONMENT_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_python_environment.v1";
export const WIKIDOT_XMLRPC_WORKER_NAME = "wikidot_xmlrpc_capture_worker";
export const WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION = 2;

const BUILD_KEYS = Object.freeze([
  "dependencyEnvironmentSha256",
  "dependencyLockBlobOid",
  "dependencyLockFileSha256",
  "dependencyRecipeBlobOid",
  "dependencyRecipeSha256",
  "pythonExecutableSha256",
  "pythonImplementation",
  "pythonVersion",
  "venvConfigSha256",
  "workerBlobOid",
  "workerFileSha256",
  "workerRepositoryCommit",
  "workerRepositoryTree",
]);
const ENVIRONMENT_KEYS = Object.freeze([
  "dependency_environment_sha256",
  "dependency_lock_blob_oid",
  "dependency_lock_file_sha256",
  "dependency_recipe_blob_oid",
  "dependency_recipe_sha256",
  "protocol_version",
  "python_executable_sha256",
  "python_implementation",
  "python_version",
  "schema",
  "venv_config_sha256",
  "worker",
  "worker_blob_oid",
  "worker_file_sha256",
  "worker_repository",
  "worker_repository_commit",
  "worker_repository_tree",
]);
const AUTHORITY_FIELDS = Object.freeze([
  "dependency_environment_sha256",
  "dependency_lock_file_sha256",
  "dependency_recipe_sha256",
  "python_executable_sha256",
  "python_version",
  "venv_config_sha256",
  "worker_blob_oid",
  "worker_file_sha256",
  "worker_repository",
  "worker_repository_commit",
  "worker_repository_tree",
]);
const FIXED_ENVIRONMENT = Object.freeze({
  protocol_version: WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION,
  python_implementation: "cpython",
  schema: WIKIDOT_XMLRPC_PYTHON_ENVIRONMENT_SCHEMA,
  worker: WIKIDOT_XMLRPC_WORKER_NAME,
  worker_repository: "Rokurolize/scp-wiki-translation",
});
const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const GIT_SHA1_RE = /^[0-9a-f]{40}$/u;
const MAX_BYTES = 16 * 1024;
const MAX_PYTHON_VERSION_CHARS = 64;
const PYTHON_VERSION_RE =
  /^(?:0|[1-9]\d{0,14})\.(?:0|[1-9]\d{0,14})\.(?:0|[1-9]\d{0,14})$/u;
const SHA256_RE = /^[0-9a-f]{64}$/u;

function assertGitSha1(value, label) {
  if (typeof value !== "string" || !GIT_SHA1_RE.test(value)) {
    throw new Error(`${label} must be a Git SHA-1`);
  }
}

function assertSha256(value, label) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

function assertPythonVersion(value) {
  if (
    typeof value !== "string" ||
    value.length > MAX_PYTHON_VERSION_CHARS ||
    !PYTHON_VERSION_RE.test(value) ||
    value
      .split(".")
      .map((part) => Number(part))
      .some((part) => !Number.isSafeInteger(part))
  ) {
    throw new Error("XML-RPC Python environment fields are invalid");
  }
}

function normalizeEnvironment(value) {
  const input = dataObject(
    value,
    ENVIRONMENT_KEYS,
    "XML-RPC Python environment",
  );
  for (const field of [
    "dependency_environment_sha256",
    "dependency_lock_file_sha256",
    "dependency_recipe_sha256",
    "python_executable_sha256",
    "venv_config_sha256",
    "worker_file_sha256",
  ]) {
    assertSha256(input[field], `Python environment ${field}`);
  }
  for (const field of [
    "dependency_lock_blob_oid",
    "dependency_recipe_blob_oid",
    "worker_blob_oid",
    "worker_repository_commit",
    "worker_repository_tree",
  ]) {
    assertGitSha1(input[field], `Python environment ${field}`);
  }
  if (
    Object.entries(FIXED_ENVIRONMENT).some(
      ([field, expected]) => input[field] !== expected,
    )
  ) {
    throw new Error("XML-RPC Python environment fields are invalid");
  }
  assertPythonVersion(input.python_version);
  return Object.freeze({ ...input });
}

function canonicalBytes(value) {
  const bytes = Buffer.from(
    `${stableStringify(normalizeEnvironment(value))}\n`,
  );
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC Python environment exceeds its byte limit");
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
    throw new Error("XML-RPC Python environment input must be bytes");
  }
  try {
    return Buffer.from(value);
  } catch {
    throw new Error("XML-RPC Python environment input must be bytes");
  }
}

function assertStore(store) {
  if (!isReferenceObjectStore(store)) {
    throw new Error("reference object store is required");
  }
}

function normalizedAuthority(value) {
  try {
    return parseWikidotXmlrpcWorkerAuthority(
      serializeWikidotXmlrpcWorkerAuthority(value),
    );
  } catch {
    throw new Error("XML-RPC worker authority is invalid");
  }
}

function normalizedInstalledEnvironmentManifest(value) {
  try {
    return parseWikidotXmlrpcInstalledEnvironmentManifest(
      serializeWikidotXmlrpcInstalledEnvironmentManifest(value),
    );
  } catch {
    throw new Error("installed environment manifest is invalid");
  }
}

export function buildWikidotXmlrpcPythonEnvironment(options) {
  const input = dataObject(
    options,
    BUILD_KEYS,
    "Python environment build options",
  );
  return normalizeEnvironment({
    ...FIXED_ENVIRONMENT,
    dependency_environment_sha256: input.dependencyEnvironmentSha256,
    dependency_lock_blob_oid: input.dependencyLockBlobOid,
    dependency_lock_file_sha256: input.dependencyLockFileSha256,
    dependency_recipe_blob_oid: input.dependencyRecipeBlobOid,
    dependency_recipe_sha256: input.dependencyRecipeSha256,
    python_executable_sha256: input.pythonExecutableSha256,
    python_implementation: input.pythonImplementation,
    python_version: input.pythonVersion,
    venv_config_sha256: input.venvConfigSha256,
    worker_blob_oid: input.workerBlobOid,
    worker_file_sha256: input.workerFileSha256,
    worker_repository_commit: input.workerRepositoryCommit,
    worker_repository_tree: input.workerRepositoryTree,
  });
}

export function serializeWikidotXmlrpcPythonEnvironment(value) {
  return canonicalBytes(value);
}

export function parseWikidotXmlrpcPythonEnvironment(value) {
  const bytes = inputBytes(value);
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC Python environment exceeds its byte limit");
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
      "XML-RPC Python environment must contain one UTF-8 JSON line",
    );
  }
  const normalized = normalizeEnvironment(parsed);
  if (!canonicalBytes(normalized).equals(bytes)) {
    throw new Error("XML-RPC Python environment bytes are not canonical");
  }
  return normalized;
}

export function assertWikidotXmlrpcPythonEnvironmentMatchesWorkerAuthority(
  environment,
  authority,
) {
  const descriptor = normalizeEnvironment(environment);
  const normalized = normalizedAuthority(authority);
  for (const field of AUTHORITY_FIELDS) {
    if (descriptor[field] !== normalized[field]) {
      throw new Error(
        "XML-RPC Python environment does not match worker authority",
      );
    }
  }
  return Object.freeze({ authority: normalized, descriptor });
}

export function assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
  environment,
  manifest,
) {
  let descriptor;
  try {
    descriptor = normalizeEnvironment(environment);
  } catch {
    throw new Error("XML-RPC Python environment is invalid");
  }
  const normalizedManifest = normalizedInstalledEnvironmentManifest(manifest);
  const files = new Map(
    normalizedManifest.files.map((file) => [file.path, file]),
  );
  const executable = files.get(normalizedManifest.python_executable_path);
  const config = files.get(normalizedManifest.venv_config_path);
  if (
    normalizedManifest.python_implementation !==
      descriptor.python_implementation ||
    normalizedManifest.python_version !== descriptor.python_version ||
    executable.sha256 !== descriptor.python_executable_sha256 ||
    config.sha256 !== descriptor.venv_config_sha256 ||
    hashWikidotXmlrpcInstalledEnvironmentManifest(normalizedManifest) !==
      descriptor.dependency_environment_sha256
  ) {
    throw new Error(
      "XML-RPC Python environment does not match installed environment manifest",
    );
  }
  return Object.freeze({ descriptor, manifest: normalizedManifest });
}

export async function putWikidotXmlrpcPythonEnvironment(store, value) {
  assertStore(store);
  const descriptor = normalizeEnvironment(value);
  const result = await store.putBytes(canonicalBytes(descriptor));
  return Object.freeze({ descriptor, ...result });
}

export async function openWikidotXmlrpcPythonEnvironment(store, reference) {
  assertStore(store);
  const object = validateReferenceObject(
    dataObject(
      reference,
      ["algorithm", "bytes", "sha256"],
      "Python environment reference",
    ),
  );
  let bytes;
  try {
    bytes = await store.readObject(object, { maxBytes: MAX_BYTES });
  } catch {
    throw new Error("XML-RPC Python environment object cannot be read");
  }
  const descriptor = parseWikidotXmlrpcPythonEnvironment(bytes);
  return Object.freeze({ descriptor, object });
}
