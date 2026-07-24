import { types as utilTypes } from "node:util";

import { stableStringify } from "./canonical-json.mjs";
import {
  parseWikidotXmlrpcPythonEnvironment,
  serializeWikidotXmlrpcPythonEnvironment,
  WIKIDOT_XMLRPC_WORKER_NAME,
  WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION,
} from "./wikidot-xmlrpc-python-environment.mjs";

const ATTESTATION_KEYS = Object.freeze([
  "ok",
  "op",
  "protocol_version",
  "runtime",
  "worker",
]);
const RUNTIME_KEYS = Object.freeze(["implementation", "version"]);

function validateExactDataRecord(value, expectedKeys, label) {
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

function runtimeVersion(value) {
  if (!Array.isArray(value) || utilTypes.isProxy(value)) {
    throw new Error("worker attestation runtime version is invalid");
  }
  let prototype;
  let keys;
  let descriptors;
  try {
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
    descriptors = [0, 1, 2].map((index) =>
      Reflect.getOwnPropertyDescriptor(value, String(index)),
    );
  } catch {
    throw new Error("worker attestation runtime version is invalid");
  }
  if (
    prototype !== Array.prototype ||
    stableStringify([...keys].sort()) !==
      stableStringify(["0", "1", "2", "length"])
  ) {
    throw new Error("worker attestation runtime version is invalid");
  }
  const parts = descriptors.map((descriptor) => descriptor?.value);
  if (
    descriptors.some(
      (descriptor) =>
        descriptor === undefined ||
        !descriptor.enumerable ||
        !("value" in descriptor),
    ) ||
    parts.some(
      (part) => !Number.isSafeInteger(part) || part < 0 || Object.is(part, -0),
    )
  ) {
    throw new Error("worker attestation runtime version is invalid");
  }
  return Object.freeze(parts);
}

function normalizedEnvironment(value) {
  try {
    return parseWikidotXmlrpcPythonEnvironment(
      serializeWikidotXmlrpcPythonEnvironment(value),
    );
  } catch {
    throw new Error("worker attestation environment is invalid");
  }
}

function expectedVersion(environment) {
  return Object.freeze(
    environment.python_version.split(".").map((part) => Number(part)),
  );
}

/**
 * Validates the credential-free v2 response before a caller may send any
 * initialize record. This self-report does not prove executable bytes; that
 * remains the responsibility of the later private capsule verifier.
 */
export function validateWikidotXmlrpcWorkerAttestation(environment, value) {
  const expected = normalizedEnvironment(environment);
  const input = validateExactDataRecord(value, ATTESTATION_KEYS, "worker attestation");
  const runtime = validateExactDataRecord(
    input.runtime,
    RUNTIME_KEYS,
    "worker attestation runtime",
  );
  const version = runtimeVersion(runtime.version);
  if (
    input.ok !== true ||
    input.op !== "attestation" ||
    !Number.isSafeInteger(input.protocol_version) ||
    input.protocol_version !== WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION ||
    input.protocol_version !== expected.protocol_version ||
    input.worker !== WIKIDOT_XMLRPC_WORKER_NAME ||
    input.worker !== expected.worker ||
    runtime.implementation !== expected.python_implementation ||
    stableStringify(version) !== stableStringify(expectedVersion(expected))
  ) {
    throw new Error("worker attestation does not match Python environment");
  }
  return Object.freeze({
    environment: expected,
    protocolVersion: input.protocol_version,
    runtime: Object.freeze({
      implementation: runtime.implementation,
      version,
    }),
    worker: input.worker,
  });
}
