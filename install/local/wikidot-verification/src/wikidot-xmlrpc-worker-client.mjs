import { performance } from "node:perf_hooks";
import process from "node:process";
import { types as utilTypes } from "node:util";

import { stableStringify } from "./canonical-json.mjs";
import { WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES } from "./reference-acquisition-xmlrpc-observation.mjs";
import {
  normalizeWikidotXmlrpcWorkerSessionOptions,
  openWikidotXmlrpcWorkerExecutionCapability,
} from "./wikidot-xmlrpc-worker-session-capability.mjs";
import { validateWikidotXmlrpcWorkerAttestation } from "./wikidot-xmlrpc-worker-attestation.mjs";
import { WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION } from "./wikidot-xmlrpc-python-environment.mjs";

const MAX_INPUT_BYTES = 4096;
const MAX_INITIALIZE_INPUT_BYTES = 64 * 1024;
const MAX_CREDENTIAL_BYTES = 4096;
const MAX_RESULT_BYTES = WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES + 4096;
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

function assertPrincipalId(value) {
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

function normalizeCredentials(value) {
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
    line.byteLength > MAX_RESULT_BYTES ||
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

class BoundedLineReader {
  constructor(stream, onFailure) {
    this.fragments = [];
    this.bytes = 0;
    this.waiter = null;
    this.failure = null;
    this.closed = false;
    this.trailingBytes = 0;
    this.onFailure = onFailure;
    stream.on("data", (chunk) => this.consume(chunk));
    stream.on("error", () =>
      this.fail(new WorkerTerminatedError("worker output failed")),
    );
  }

  consume(input) {
    if (this.failure !== null || this.closed) return;
    let chunk = input;
    while (chunk.byteLength > 0) {
      if (this.waiter === null) {
        this.fail(new WorkerProtocolError("worker returned unsolicited bytes"));
        return;
      }
      const newline = chunk.indexOf(0x0a);
      if (newline === -1) {
        this.fragments.push(chunk);
        this.bytes += chunk.byteLength;
        if (this.bytes > MAX_RESULT_BYTES)
          this.fail(new WorkerProtocolError("worker result is too large"));
        return;
      }
      const length = this.bytes + newline + 1;
      if (length > MAX_RESULT_BYTES) {
        this.fail(new WorkerProtocolError("worker result is too large"));
        return;
      }
      const line = Buffer.concat(
        [...this.fragments, chunk.subarray(0, newline + 1)],
        length,
      );
      this.fragments = [];
      this.bytes = 0;
      chunk = chunk.subarray(newline + 1);
      if (chunk.byteLength > 0) {
        const error = new WorkerProtocolError(
          "worker returned unsolicited bytes",
        );
        const waiter = this.waiter;
        this.waiter = null;
        clearTimeout(waiter.timer);
        waiter.reject(error);
        this.fail(error);
        return;
      }
      const waiter = this.waiter;
      this.waiter = null;
      clearTimeout(waiter.timer);
      try {
        waiter.resolve(parseWikidotXmlrpcWorkerRecord(line));
      } catch (error) {
        waiter.reject(error);
        this.fail(error);
        return;
      }
    }
  }

  next(timeoutMs) {
    if (this.waiter !== null) {
      throw new Error("only one worker response may be in flight");
    }
    if (this.failure !== null) return Promise.reject(this.failure);
    if (this.closed) {
      return Promise.reject(
        new WorkerTerminatedError("worker closed before responding"),
      );
    }
    return new Promise((resolve, reject) => {
      const timer = setTimeout(
        () =>
          this.fail(
            new WorkerTerminatedError("worker response deadline exceeded"),
          ),
        timeoutMs,
      );
      this.waiter = { reject, resolve, timer };
    });
  }

  finish() {
    if (this.closed) return;
    this.closed = true;
    this.trailingBytes = this.bytes;
    this.fragments = [];
    this.bytes = 0;
    if (this.waiter !== null) {
      const waiter = this.waiter;
      this.waiter = null;
      clearTimeout(waiter.timer);
      waiter.reject(
        new WorkerTerminatedError("worker closed before a complete record"),
      );
    }
  }

  fail(error) {
    if (this.failure !== null) return;
    this.failure = error;
    if (this.waiter !== null) {
      const waiter = this.waiter;
      this.waiter = null;
      clearTimeout(waiter.timer);
      waiter.reject(error);
    }
    this.onFailure(error);
  }
}

function validateReady(record, principalId) {
  if (
    !exactKeys(record, ["ok", "op", "principal_id"]) ||
    record.ok !== true ||
    record.op !== "ready" ||
    record.principal_id !== principalId
  ) {
    throw new WorkerProtocolError("worker ready record is invalid");
  }
}

function validateAttestation(record, environment) {
  try {
    return validateWikidotXmlrpcWorkerAttestation(environment, record);
  } catch {
    throw new WorkerProtocolError("worker attestation is invalid");
  }
}

function validateCapture(record, ordinal) {
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

export class WikidotXmlrpcWorkerClient {
  constructor(execution, options = {}) {
    const timeouts = normalizeWikidotXmlrpcWorkerSessionOptions(options);
    const capability = openWikidotXmlrpcWorkerExecutionCapability(execution);
    this.startupTimeoutMs = timeouts.startupTimeoutMs;
    this.captureTimeoutMs = timeouts.captureTimeoutMs;
    this.exitGraceMs = timeouts.exitGraceMs;
    this.signal = null;
    this.lastGroupSignal = null;
    this.terminationPromise = null;
    this.child = capability.child;
    this.signalProcessGroup = capability.signalProcessGroup;
    this.closePromise = new Promise((resolve) =>
      this.child.once("close", (code, signal) => {
        this.closed = { code, signal };
        this.reader.finish();
        this.removeSignalHandlers();
        resolve(this.closed);
      }),
    );
    this.reader = new BoundedLineReader(
      this.child.stdout,
      () => void this.terminate("SIGTERM").catch(() => {}),
    );
    this.child.stdin.on("error", () =>
      this.reader.fail(new WorkerTerminatedError("worker input failed")),
    );
    this.child.once("error", () =>
      this.reader.fail(new WorkerTerminatedError("worker could not start")),
    );
    this.signalHandlers = new Map(
      ["SIGINT", "SIGTERM"].map((name) => [
        name,
        () => this.handleSignal(name),
      ]),
    );
    for (const [name, handler] of this.signalHandlers) {
      process.on(name, handler);
    }
  }

  handleSignal(signal) {
    if (this.signal === null) {
      this.signal = signal;
      void this.terminate(signal).catch(() => {});
      this.reader.fail(new OperatorSignalError(signal));
    } else {
      this.killGroup("SIGKILL");
    }
  }

  removeSignalHandlers() {
    for (const [name, handler] of this.signalHandlers) {
      process.off(name, handler);
    }
    this.signalHandlers.clear();
  }

  killGroup(signal) {
    if (this.lastGroupSignal === "SIGKILL" || this.lastGroupSignal === signal) {
      return;
    }
    try {
      this.signalProcessGroup(signal);
      this.lastGroupSignal = signal;
    } catch (error) {
      if (error.code !== "ESRCH") throw error;
      this.lastGroupSignal = signal;
    }
  }

  assertNotSignaled() {
    if (this.signal !== null) throw new OperatorSignalError(this.signal);
  }

  assertHealthy() {
    this.assertNotSignaled();
    if (this.reader.failure !== null) throw this.reader.failure;
  }

  async write(record, maxInputBytes = MAX_INPUT_BYTES) {
    this.assertHealthy();
    const bytes = Buffer.from(`${JSON.stringify(record)}\n`);
    if (bytes.byteLength > maxInputBytes) {
      throw new WorkerProtocolError("worker request exceeds its byte limit");
    }
    await new Promise((resolve, reject) =>
      this.child.stdin.write(bytes, (error) =>
        error
          ? reject(new WorkerTerminatedError("worker input failed"))
          : resolve(),
      ),
    );
  }

  async request(record, timeoutMs, maxInputBytes = MAX_INPUT_BYTES) {
    const response = this.reader.next(timeoutMs);
    response.catch(() => {});
    try {
      await this.write(record, maxInputBytes);
      return await response;
    } catch (error) {
      await this.terminate("SIGTERM");
      this.assertNotSignaled();
      if (!(error instanceof WorkerProtocolError)) {
        await response;
      }
      throw error;
    }
  }

  async start(principalId, environment, credentials) {
    let normalizedPrincipalId;
    try {
      normalizedPrincipalId = assertPrincipalId(principalId);
    } catch (error) {
      await this.terminate("SIGTERM");
      throw error;
    }
    const deadline = performance.now() + this.startupTimeoutMs;
    const startupRequest = async (request, maxInputBytes = MAX_INPUT_BYTES) => {
      const remainingMs = deadline - performance.now();
      if (remainingMs <= 0) {
        await this.terminate("SIGTERM");
        throw new WorkerTerminatedError("worker startup deadline exceeded");
      }
      return this.request(request, remainingMs, maxInputBytes);
    };
    const attestation = await startupRequest({
      op: "attest",
      protocol_version: WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION,
    });
    try {
      validateAttestation(attestation, environment);
    } catch (error) {
      await this.terminate("SIGTERM");
      throw error;
    }
    try {
      const normalizedCredentials = normalizeCredentials(credentials);
      const record = await startupRequest(
        {
          api_key: normalizedCredentials.apiKey,
          app_name: normalizedCredentials.appName,
          op: "initialize",
          principal_id: normalizedPrincipalId,
          protocol_version: WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION,
        },
        MAX_INITIALIZE_INPUT_BYTES,
      );
      validateReady(record, normalizedPrincipalId);
      this.assertHealthy();
    } catch (error) {
      await this.terminate("SIGTERM");
      throw error;
    }
  }

  async capture(ordinal, fullname) {
    const record = await this.request(
      { fullname, op: "capture", ordinal },
      this.captureTimeoutMs,
    );
    try {
      const validated = validateCapture(record, ordinal);
      this.assertHealthy();
      return validated;
    } catch (error) {
      await this.terminate("SIGTERM");
      throw error;
    }
  }

  async waitForClose() {
    return new Promise((resolve) => {
      const timer = setTimeout(() => resolve(null), this.exitGraceMs);
      this.closePromise.then((closed) => {
        clearTimeout(timer);
        resolve(closed);
      });
    });
  }

  async terminate(signal = "SIGTERM") {
    if (this.terminationPromise !== null) {
      if (signal === "SIGKILL") this.killGroup("SIGKILL");
      return this.terminationPromise;
    }
    this.terminationPromise = (async () => {
      this.child.stdin.destroy();
      this.killGroup(signal);
      if (this.closed !== undefined) return this.closed;
      let closed = await this.waitForClose();
      if (closed === null) {
        this.killGroup("SIGKILL");
        closed = await this.waitForClose();
      }
      if (closed === null) {
        this.child.stdin.destroy();
        this.child.stdout.destroy();
        this.child.unref();
        this.reader.finish();
        this.removeSignalHandlers();
        throw new WorkerTerminatedError(
          "worker process group did not terminate",
        );
      }
      return closed;
    })();
    return this.terminationPromise;
  }

  async expectExit(code) {
    this.assertNotSignaled();
    let closed = await this.waitForClose();
    if (closed === null) {
      try {
        await this.terminate("SIGTERM");
      } catch (error) {
        this.assertNotSignaled();
        throw error;
      }
      closed = this.closed;
    }
    this.killGroup("SIGTERM");
    this.assertHealthy();
    if (
      closed.code !== code ||
      closed.signal !== null ||
      this.reader.trailingBytes !== 0
    ) {
      throw new WorkerTerminatedError("worker exit did not match its result");
    }
  }

  async closeClean() {
    this.assertNotSignaled();
    if (this.closed !== undefined) {
      this.killGroup("SIGTERM");
      throw new WorkerTerminatedError(
        "worker exited before coordinator shutdown",
      );
    }
    this.child.stdin.end();
    let closed = await this.waitForClose();
    if (closed === null) {
      try {
        await this.terminate("SIGTERM");
      } catch (error) {
        this.assertNotSignaled();
        throw error;
      }
      closed = this.closed;
    }
    this.killGroup("SIGTERM");
    this.assertHealthy();
    if (
      closed.code !== 0 ||
      closed.signal !== null ||
      this.reader.trailingBytes !== 0
    ) {
      throw new WorkerTerminatedError("worker did not shut down cleanly");
    }
  }
}
