import {types as utilTypes} from "node:util";

import {stableStringify} from "./canonical-json.mjs";
import {WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES} from "./reference-acquisition-xmlrpc-observation.mjs";
import {validateWikidotXmlrpcWorkerAttestation} from "./wikidot-xmlrpc-worker-attestation.mjs";

export const WIKIDOT_XMLRPC_WORKER_INPUT_MAX_BYTES = 4096;
export const WIKIDOT_XMLRPC_WORKER_INITIALIZE_INPUT_MAX_BYTES = 64 * 1024;
const MAX_CREDENTIAL_BYTES = 4096;
export const WIKIDOT_XMLRPC_WORKER_RESULT_MAX_BYTES = WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES + 4096;
const MAX_JSON_DEPTH = 64;
const MAX_JSON_TOKENS = 1_000_000;
const JSON_NUMBER_RE = /-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/uy;
const FAILURE_MATRIX = new Map([
  ["wikidot_deleted", false],
  ["wikidot_forbidden", false],
  ["wikidot_fault_unclassified", false],
  ["response_rejected", false],
  ["call_deadline_exceeded", true],
  ["transport_exhausted", true],
  ["worker_internal_error", false],
]);
export class WorkerProtocolError extends Error {}
export class WorkerTerminatedError extends Error {}
export class OperatorSignalError extends Error {
  constructor(signal) {
    super(`coordinator interrupted by ${signal}`);
    this.signal = signal;
    this.exitCode = signal === "SIGINT" ? 130 : 143;
  }
}

function exactKeys(value, expected) {
  return (
    value !== null &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    stableStringify(Object.keys(value).sort()) === stableStringify(expected)
  );
}

export function assertPrincipalId(value) {
  if (
    !Number.isSafeInteger(value) ||
    value < 1 ||
    value > Number.MAX_SAFE_INTEGER
  ) {
    throw new WorkerProtocolError("worker principal ID is invalid");
  }
  return value;
}

function assertCredential(value) {
  if (typeof value !== "string" || value.length === 0) {
    throw new WorkerProtocolError("worker credentials are invalid");
  }
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) {
        throw new WorkerProtocolError("worker credentials are invalid");
      }
      index += 1;
    } else if (
      (code >= 0xdc00 && code <= 0xdfff) ||
      code === 0 ||
      code === 10 ||
      code === 13
    ) {
      throw new WorkerProtocolError("worker credentials are invalid");
    }
  }
  if (Buffer.byteLength(value, "utf8") > MAX_CREDENTIAL_BYTES) {
    throw new WorkerProtocolError("worker credentials are invalid");
  }
  return value;
}

export function normalizeCredentials(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new WorkerProtocolError("worker credentials are invalid");
  }
  let keys;
  let prototype;
  try {
    keys = Reflect.ownKeys(value);
    prototype = Reflect.getPrototypeOf(value);
  } catch {
    throw new WorkerProtocolError("worker credentials are invalid");
  }
  if (
    prototype !== Object.prototype ||
    keys.length !== 2 ||
    keys.some((key) => typeof key !== "string") ||
    stableStringify([...keys].sort()) !== stableStringify(["apiKey", "appName"])
  ) {
    throw new WorkerProtocolError("worker credentials are invalid");
  }
  const snapshot = {};
  for (const key of keys) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new WorkerProtocolError("worker credentials are invalid");
    }
    snapshot[key] = assertCredential(descriptor.value);
  }
  return snapshot;
}

function rejectDuplicateKeys(text) {
  const contexts = [];
  let tokens = 0;
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (character === '"') {
      const start = index;
      for (index += 1; index < text.length; index += 1) {
        if (text[index] === "\\") index += 1;
        else if (text[index] === '"') break;
      }
      let cursor = index + 1;
      while (/\s/u.test(text[cursor] ?? "")) cursor += 1;
      if (text[cursor] === ":" && contexts.at(-1) instanceof Set) {
        const key = JSON.parse(text.slice(start, index + 1));
        const keys = contexts.at(-1);
        if (keys.has(key))
          throw new WorkerProtocolError("worker returned duplicate JSON key");
        keys.add(key);
      }
    } else if (character === "-" || /\d/u.test(character)) {
      JSON_NUMBER_RE.lastIndex = index;
      const token = JSON_NUMBER_RE.exec(text)?.[0] ?? "";
      const digits = token.startsWith("-") ? token.slice(1) : token;
      if (
        !/[.eE]/u.test(token) &&
        (digits.length > 16 ||
          (digits.length === 16 && digits > "9007199254740991"))
      )
        throw new WorkerProtocolError("worker JSON contains an unsafe integer");
      index += Math.max(token.length, 1) - 1;
    } else if (character === "{") contexts.push(new Set());
    else if (character === "[") contexts.push(null);
    else if (character === "}" || character === "]") contexts.pop();
    if (
      ["{", "[", ":", ","].includes(character) &&
      (++tokens > MAX_JSON_TOKENS || contexts.length > MAX_JSON_DEPTH)
    )
      throw new WorkerProtocolError("worker JSON exceeds its structural limit");
  }
}

export function parseWikidotXmlrpcWorkerRecord(line) {
  if (
    !(line instanceof Uint8Array) ||
    line.byteLength > WIKIDOT_XMLRPC_WORKER_RESULT_MAX_BYTES ||
    line.at(-1) !== 0x0a ||
    line.includes(0x0d)
  )
    throw new WorkerProtocolError("worker returned invalid JSONL framing");
  try {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(
      line.subarray(0, -1),
    );
    rejectDuplicateKeys(text);
    return JSON.parse(text, (_key, value) => {
      if (typeof value === "number" && !Number.isFinite(value))
        throw new WorkerProtocolError("worker JSON number is non-finite");
      return value;
    });
  } catch (error) {
    if (error instanceof WorkerProtocolError) throw error;
    throw new WorkerProtocolError("worker returned invalid JSON");
  }
}

export function validateReady(record, principalId) {
  if (
    !exactKeys(record, ["ok", "op", "principal_id"]) ||
    record.ok !== true ||
    record.op !== "ready" ||
    record.principal_id !== principalId
  ) {
    throw new WorkerProtocolError("worker ready record is invalid");
  }
}

export function validateAttestation(record, environment) {
  try {
    return validateWikidotXmlrpcWorkerAttestation(environment, record);
  } catch {
    throw new WorkerProtocolError("worker attestation is invalid");
  }
}

export function validateCapture(record, ordinal) {
  if (
    exactKeys(record, ["ok", "op", "ordinal", "response"]) &&
    record.ok === true &&
    record.op === "capture" &&
    record.ordinal === ordinal &&
    record.response !== null &&
    typeof record.response === "object" &&
    !Array.isArray(record.response)
  ) {
    return record;
  }
  if (
    !exactKeys(record, ["code", "ok", "op", "ordinal", "retryable"]) ||
    record.ok !== false ||
    record.op !== "capture" ||
    record.ordinal !== ordinal ||
    FAILURE_MATRIX.get(record.code) !== record.retryable
  ) {
    throw new WorkerProtocolError("worker capture record is invalid");
  }
  return record;
}
