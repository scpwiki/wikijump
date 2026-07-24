import {performance} from "node:perf_hooks";
import process from "node:process";

import {
  normalizeWikidotXmlrpcWorkerSessionOptions,
  openWikidotXmlrpcWorkerExecutionCapability,
} from "./wikidot-xmlrpc-worker-session-capability.mjs";
import {WIKIDOT_XMLRPC_WORKER_PROTOCOL_VERSION} from "./wikidot-xmlrpc-python-environment.mjs";
import {
  assertPrincipalId,
  normalizeCredentials,
  OperatorSignalError,
  parseWikidotXmlrpcWorkerRecord,
  validateAttestation,
  validateCapture,
  validateReady,
  WIKIDOT_XMLRPC_WORKER_INITIALIZE_INPUT_MAX_BYTES,
  WIKIDOT_XMLRPC_WORKER_INPUT_MAX_BYTES,
  WIKIDOT_XMLRPC_WORKER_RESULT_MAX_BYTES,
  WorkerProtocolError,
  WorkerTerminatedError,
} from "./wikidot-xmlrpc-worker-protocol.mjs";

export {
  OperatorSignalError,
  parseWikidotXmlrpcWorkerRecord,
  WorkerProtocolError,
  WorkerTerminatedError,
} from "./wikidot-xmlrpc-worker-protocol.mjs";

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
        if (this.bytes > WIKIDOT_XMLRPC_WORKER_RESULT_MAX_BYTES)
          this.fail(new WorkerProtocolError("worker result is too large"));
        return;
      }
      const length = this.bytes + newline + 1;
      if (length > WIKIDOT_XMLRPC_WORKER_RESULT_MAX_BYTES) {
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

  async write(record, maxInputBytes = WIKIDOT_XMLRPC_WORKER_INPUT_MAX_BYTES) {
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

  async request(record, timeoutMs, maxInputBytes = WIKIDOT_XMLRPC_WORKER_INPUT_MAX_BYTES) {
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
    const startupRequest = async (request, maxInputBytes = WIKIDOT_XMLRPC_WORKER_INPUT_MAX_BYTES) => {
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
        WIKIDOT_XMLRPC_WORKER_INITIALIZE_INPUT_MAX_BYTES,
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
