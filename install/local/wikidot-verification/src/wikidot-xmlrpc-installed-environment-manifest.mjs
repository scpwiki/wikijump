import { types as utilTypes } from "node:util";

import { sha256Hex, stableStringify } from "./canonical-json.mjs";
import {
  isReferenceObjectStore,
  validateReferenceObject,
} from "./reference-object-store.mjs";

export const WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_installed_environment_manifest.v1";
export const WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_MAX_BYTES =
  16 * 1024 * 1024;

const BUILD_KEYS = Object.freeze([
  "files",
  "pythonExecutablePath",
  "pythonImplementation",
  "pythonVersion",
  "venvConfigPath",
]);
const FILE_KEYS = Object.freeze(["bytes", "executable", "path", "sha256"]);
const MANIFEST_KEYS = Object.freeze([
  "files",
  "python_executable_path",
  "python_implementation",
  "python_version",
  "schema",
  "scope",
  "venv_config_path",
]);
const FIXED_FIELDS = Object.freeze({
  schema: WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_SCHEMA,
  scope: "declared_application_dependencies",
});
const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const MAX_DEPTH = 32;
const MAX_ENTRIES = 200_000;
const MAX_FILE_BYTES = 1024 * 1024 * 1024;
const MAX_PATH_BYTES = 4096;
const MAX_TOTAL_FILE_BYTES = 4 * 1024 * 1024 * 1024;
const MAX_PYTHON_VERSION_CHARS = 64;
const PYTHON_VERSION_RE =
  /^(?:0|[1-9]\d{0,14})\.(?:0|[1-9]\d{0,14})\.(?:0|[1-9]\d{0,14})$/u;
const SHA256_RE = /^[0-9a-f]{64}$/u;

function dataArray(value, label) {
  if (!Array.isArray(value) || utilTypes.isProxy(value)) {
    throw new Error(`${label} must be a data array`);
  }
  let keys;
  let lengthDescriptor;
  try {
    keys = Reflect.ownKeys(value);
    lengthDescriptor = Reflect.getOwnPropertyDescriptor(value, "length");
  } catch {
    throw new Error(`${label} must be a data array`);
  }
  if (
    lengthDescriptor === undefined ||
    !("value" in lengthDescriptor) ||
    !Number.isSafeInteger(lengthDescriptor.value) ||
    lengthDescriptor.value < 0 ||
    keys.length !== lengthDescriptor.value + 1
  ) {
    throw new Error(`${label} must be a dense data array`);
  }
  const snapshot = [];
  for (let index = 0; index < lengthDescriptor.value; index += 1) {
    let descriptor;
    try {
      descriptor = Reflect.getOwnPropertyDescriptor(value, String(index));
    } catch {
      throw new Error(`${label} must be a data array`);
    }
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error(`${label} must be a dense data array`);
    }
    snapshot.push(descriptor.value);
  }
  if (
    keys.some(
      (key) =>
        key !== "length" &&
        (typeof key !== "string" ||
          !Number.isSafeInteger(Number(key)) ||
          String(Number(key)) !== key ||
          Number(key) < 0 ||
          Number(key) >= lengthDescriptor.value),
    )
  ) {
    throw new Error(`${label} has unexpected fields`);
  }
  return Object.freeze(snapshot);
}

function dataObject(value, expectedKeys, label) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error(`${label} must be a data object`);
  }
  let prototype;
  let keys;
  let descriptors;
  try {
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
    descriptors = keys.map((key) =>
      Reflect.getOwnPropertyDescriptor(value, key),
    );
  } catch {
    throw new Error(`${label} must be a data object`);
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some((key) => typeof key !== "string") ||
    stableStringify([...keys].sort()) !== stableStringify(expectedKeys)
  ) {
    throw new Error(`${label} has unexpected fields or prototype`);
  }
  const snapshot = {};
  for (const [index, key] of keys.entries()) {
    const descriptor = descriptors[index];
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error(`${label} must contain only enumerable data fields`);
    }
    Object.defineProperty(snapshot, key, {
      enumerable: true,
      value: descriptor.value,
    });
  }
  return Object.freeze(snapshot);
}

function assertWellFormedString(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} is invalid`);
  }
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) {
        throw new Error(`${label} is invalid`);
      }
      index += 1;
    } else if (code >= 0xdc00 && code <= 0xdfff) {
      throw new Error(`${label} is invalid`);
    }
  }
}

function normalizeRelativePath(value, label) {
  assertWellFormedString(value, label);
  if (
    value.startsWith("/") ||
    value.includes("\\") ||
    value.includes("\0") ||
    Buffer.byteLength(value, "utf8") > MAX_PATH_BYTES
  ) {
    throw new Error(`${label} is invalid`);
  }
  const segments = value.split("/");
  if (
    segments.length > MAX_DEPTH ||
    segments.some(
      (segment) =>
        segment.length === 0 ||
        segment === "." ||
        segment === ".." ||
        /[\u0000-\u001f\u007f]/u.test(segment),
    )
  ) {
    throw new Error(`${label} is invalid`);
  }
  return value;
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
    throw new Error("installed environment manifest Python fields are invalid");
  }
}

function compareUtf8(left, right) {
  return Buffer.compare(Buffer.from(left, "utf8"), Buffer.from(right, "utf8"));
}

function normalizeFile(value) {
  const input = dataObject(value, FILE_KEYS, "installed environment file");
  if (
    !Number.isSafeInteger(input.bytes) ||
    input.bytes < 0 ||
    Object.is(input.bytes, -0) ||
    input.bytes > MAX_FILE_BYTES ||
    typeof input.executable !== "boolean" ||
    typeof input.sha256 !== "string" ||
    !SHA256_RE.test(input.sha256)
  ) {
    throw new Error("installed environment file is invalid");
  }
  return Object.freeze({
    bytes: input.bytes,
    executable: input.executable,
    path: normalizeRelativePath(input.path, "installed environment file path"),
    sha256: input.sha256,
  });
}

function normalizeFiles(value) {
  const input = dataArray(value, "installed environment files");
  if (input.length === 0 || input.length > MAX_ENTRIES) {
    throw new Error("installed environment manifest file count is invalid");
  }
  const files = input.map(normalizeFile);
  files.sort((left, right) => compareUtf8(left.path, right.path));
  if (
    files.some(
      (file, index) => index > 0 && file.path === files[index - 1].path,
    )
  ) {
    throw new Error("installed environment manifest has duplicate file paths");
  }
  const totalFileBytes = files.reduce((total, file) => total + file.bytes, 0);
  if (totalFileBytes > MAX_TOTAL_FILE_BYTES) {
    throw new Error(
      "installed environment manifest exceeds its file byte limit",
    );
  }
  const paths = new Set(files.map((file) => file.path));
  for (const file of files) {
    let ancestor = file.path;
    while (ancestor.includes("/")) {
      ancestor = ancestor.slice(0, ancestor.lastIndexOf("/"));
      if (paths.has(ancestor)) {
        throw new Error(
          "installed environment manifest has an invalid file tree",
        );
      }
    }
  }
  return Object.freeze(files);
}

function manifestByteBudget(value) {
  const prefix = Buffer.byteLength('{"files":[');
  const suffix = Buffer.byteLength(
    `],"python_executable_path":${JSON.stringify(value.python_executable_path)},"python_implementation":${JSON.stringify(value.python_implementation)},"python_version":${JSON.stringify(value.python_version)},"schema":${JSON.stringify(value.schema)},"scope":${JSON.stringify(value.scope)},"venv_config_path":${JSON.stringify(value.venv_config_path)}}\n`,
    "utf8",
  );
  let bytes = prefix + suffix + Math.max(0, value.files.length - 1);
  for (const file of value.files) {
    bytes += Buffer.byteLength(
      `{"bytes":${file.bytes},"executable":${file.executable},"path":${JSON.stringify(file.path)},"sha256":"${file.sha256}"}`,
      "utf8",
    );
    if (bytes > WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_MAX_BYTES) {
      throw new Error("installed environment manifest exceeds its byte limit");
    }
  }
}

function manifestFile(files, path, label) {
  const file = files.find((candidate) => candidate.path === path);
  if (file === undefined) {
    throw new Error(`${label} is not in the installed environment manifest`);
  }
  return file;
}

function normalizeManifest(value) {
  const input = dataObject(
    value,
    MANIFEST_KEYS,
    "installed environment manifest",
  );
  if (
    input.schema !== FIXED_FIELDS.schema ||
    input.scope !== FIXED_FIELDS.scope ||
    input.python_implementation !== "cpython"
  ) {
    throw new Error(
      "installed environment manifest authority fields are invalid",
    );
  }
  assertPythonVersion(input.python_version);
  const files = normalizeFiles(input.files);
  const pythonExecutablePath = normalizeRelativePath(
    input.python_executable_path,
    "installed environment Python executable path",
  );
  const venvConfigPath = normalizeRelativePath(
    input.venv_config_path,
    "installed environment venv config path",
  );
  if (pythonExecutablePath === venvConfigPath) {
    throw new Error("installed environment manifest role paths are invalid");
  }
  const manifest = Object.freeze({
    files,
    python_executable_path: pythonExecutablePath,
    python_implementation: "cpython",
    python_version: input.python_version,
    ...FIXED_FIELDS,
    venv_config_path: venvConfigPath,
  });
  const executable = manifestFile(
    files,
    pythonExecutablePath,
    "installed environment Python executable",
  );
  const config = manifestFile(
    files,
    venvConfigPath,
    "installed environment venv config",
  );
  if (!executable.executable || config.executable) {
    throw new Error("installed environment manifest role files are invalid");
  }
  manifestByteBudget(manifest);
  return manifest;
}

function canonicalBytes(value) {
  const bytes = Buffer.from(`${stableStringify(normalizeManifest(value))}\n`);
  if (
    bytes.byteLength > WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_MAX_BYTES
  ) {
    throw new Error("installed environment manifest exceeds its byte limit");
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
    throw new Error("installed environment manifest input must be bytes");
  }
  try {
    return Buffer.from(value);
  } catch {
    throw new Error("installed environment manifest input must be bytes");
  }
}

function assertStore(store) {
  if (!isReferenceObjectStore(store)) {
    throw new Error("reference object store is required");
  }
}

// Canonicalizes a declared regular-file application-dependency capsule image. Every entry is an ordinary file; a future collector/materializer must reject source symlinks or copy a verified referent as a regular file before it produces this manifest. The declaration is not proof of a complete system runtime; the private capsule must enforce the wider execution boundary before spawning.
export function buildWikidotXmlrpcInstalledEnvironmentManifest(options) {
  const input = dataObject(
    options,
    BUILD_KEYS,
    "installed environment manifest build options",
  );
  const manifest = normalizeManifest({
    files: input.files,
    python_executable_path: input.pythonExecutablePath,
    python_implementation: input.pythonImplementation,
    python_version: input.pythonVersion,
    ...FIXED_FIELDS,
    venv_config_path: input.venvConfigPath,
  });
  canonicalBytes(manifest);
  return manifest;
}

export function serializeWikidotXmlrpcInstalledEnvironmentManifest(value) {
  return canonicalBytes(value);
}

export function parseWikidotXmlrpcInstalledEnvironmentManifest(value) {
  const bytes = inputBytes(value);
  if (
    bytes.byteLength > WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_MAX_BYTES
  ) {
    throw new Error("installed environment manifest exceeds its byte limit");
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
      "installed environment manifest must contain one UTF-8 JSON line",
    );
  }
  const normalized = normalizeManifest(parsed);
  if (!canonicalBytes(normalized).equals(bytes)) {
    throw new Error("installed environment manifest bytes are not canonical");
  }
  return normalized;
}

export function hashWikidotXmlrpcInstalledEnvironmentManifest(value) {
  return sha256Hex(serializeWikidotXmlrpcInstalledEnvironmentManifest(value));
}

export async function putWikidotXmlrpcInstalledEnvironmentManifest(
  store,
  value,
) {
  assertStore(store);
  const descriptor = normalizeManifest(value);
  const result = await store.putBytes(canonicalBytes(descriptor));
  return Object.freeze({ descriptor, ...result });
}

export async function openWikidotXmlrpcInstalledEnvironmentManifest(
  store,
  reference,
) {
  assertStore(store);
  const object = validateReferenceObject(
    dataObject(
      reference,
      ["algorithm", "bytes", "sha256"],
      "installed environment manifest reference",
    ),
  );
  let bytes;
  try {
    bytes = await store.readObject(object, {
      maxBytes: WIKIDOT_XMLRPC_INSTALLED_ENVIRONMENT_MANIFEST_MAX_BYTES,
    });
  } catch {
    throw new Error("installed environment manifest object cannot be read");
  }
  let descriptor;
  try {
    descriptor = parseWikidotXmlrpcInstalledEnvironmentManifest(bytes);
  } catch {
    throw new Error("installed environment manifest object is not canonical");
  }
  return Object.freeze({
    descriptor,
    object,
  });
}
