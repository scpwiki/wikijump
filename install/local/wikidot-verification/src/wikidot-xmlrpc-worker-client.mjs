import { spawn } from "node:child_process";
import process from "node:process";

import { stableStringify } from "./corpus-import-manifest.mjs";
import { WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES } from "./reference-acquisition-xmlrpc-observation.mjs";

const MAX_INPUT_BYTES = 4096;
const MAX_RESULT_BYTES = WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES + 4096;
const MAX_JSON_DEPTH = 64;
const MAX_JSON_TOKENS = 1_000_000;
const JSON_NUMBER_RE = /-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/uy;
const STARTUP_TIMEOUT_MS = 180_000;
const CAPTURE_TIMEOUT_MS = 180_000;
const EXIT_GRACE_MS = 5_000;
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
  constructor({
    command,
    args,
    env,
    spawnImpl = spawn,
    startupTimeoutMs = STARTUP_TIMEOUT_MS,
    captureTimeoutMs = CAPTURE_TIMEOUT_MS,
    exitGraceMs = EXIT_GRACE_MS,
  }) {
    this.startupTimeoutMs = startupTimeoutMs;
    this.captureTimeoutMs = captureTimeoutMs;
    this.exitGraceMs = exitGraceMs;
    this.signal = null;
    this.terminationPromise = null;
    this.child = spawnImpl(command, args, {
      detached: true,
      env,
      stdio: ["pipe", "pipe", "ignore"],
    });
    this.closePromise = new Promise((resolve) =>
      this.child.once("close", (code, signal) => {
        this.closed = { code, signal };
        this.reader.finish();
        this.removeSignalHandlers();
        resolve(this.closed);
      }),
    );
    this.leaderExited = false;
    this.child.once("exit", () => {
      this.leaderExited = true;
    });
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
    if (!this.child.pid || this.leaderExited) return;
    try {
      process.kill(-this.child.pid, signal);
    } catch (error) {
      if (error.code !== "ESRCH") throw error;
    }
  }

  assertNotSignaled() {
    if (this.signal !== null) throw new OperatorSignalError(this.signal);
  }

  assertHealthy() {
    this.assertNotSignaled();
    if (this.reader.failure !== null) throw this.reader.failure;
  }

  async write(record) {
    this.assertHealthy();
    const bytes = Buffer.from(`${JSON.stringify(record)}\n`);
    if (bytes.byteLength > MAX_INPUT_BYTES) {
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

  async request(record, timeoutMs) {
    const response = this.reader.next(timeoutMs);
    response.catch(() => {});
    try {
      await this.write(record);
      return await response;
    } catch (error) {
      await this.terminate("SIGTERM");
      this.assertNotSignaled();
      if (!(error instanceof WorkerProtocolError)) {
        try {
          await response;
        } catch (responseError) {
          throw responseError;
        }
      }
      throw error;
    }
  }

  async start(principalId) {
    const record = await this.request(
      { op: "initialize", principal_id: principalId },
      this.startupTimeoutMs,
    );
    try {
      validateReady(record, principalId);
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
    if (this.closed !== undefined) return this.closed;
    if (this.terminationPromise !== null) return this.terminationPromise;
    this.terminationPromise = (async () => {
      this.child.stdin.destroy();
      this.killGroup(signal);
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
