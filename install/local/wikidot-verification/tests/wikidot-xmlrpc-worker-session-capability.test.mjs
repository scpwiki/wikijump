import assert from "node:assert/strict";
import test from "node:test";

import {
  normalizeWikidotXmlrpcWorkerSessionOptions,
  openWikidotXmlrpcWorkerExecutionCapability,
  WIKIDOT_XMLRPC_WORKER_SESSION_DEFAULTS,
} from "../src/wikidot-xmlrpc-worker-session-capability.mjs";

function fakeChild() {
  return {
    once() {},
    stdin: {
      destroy() {},
      end() {},
      on() {},
      write() {},
    },
    stdout: {
      destroy() {},
      on() {},
    },
    unref() {},
  };
}

function execution(overrides = {}) {
  return {
    child: fakeChild(),
    signalProcessGroup() {},
    ...overrides,
  };
}

test("session options accept only positive explicit timeout overrides", () => {
  const defaults = normalizeWikidotXmlrpcWorkerSessionOptions({});
  assert.deepEqual(defaults, WIKIDOT_XMLRPC_WORKER_SESSION_DEFAULTS);
  assert.equal(Object.isFrozen(defaults), true);
  assert.deepEqual(
    normalizeWikidotXmlrpcWorkerSessionOptions({ captureTimeoutMs: 25 }),
    { ...WIKIDOT_XMLRPC_WORKER_SESSION_DEFAULTS, captureTimeoutMs: 25 },
  );

  for (const value of [
    null,
    [],
    { command: "python" },
    { captureTimeoutMs: 0 },
    { captureTimeoutMs: 1.5 },
    { captureTimeoutMs: Number.MAX_SAFE_INTEGER + 1 },
    new Proxy({}, {}),
  ]) {
    assert.throws(
      () => normalizeWikidotXmlrpcWorkerSessionOptions(value),
      /worker session/u,
    );
  }

  const accessor = {};
  Object.defineProperty(accessor, "captureTimeoutMs", {
    enumerable: true,
    get() {
      throw new Error("accessor must not run");
    },
  });
  assert.throws(
    () => normalizeWikidotXmlrpcWorkerSessionOptions(accessor),
    /positive safe integer/u,
  );
});

test("execution capability is exact, frozen, and excludes raw launch configuration", () => {
  const source = execution();
  const opened = openWikidotXmlrpcWorkerExecutionCapability(source);
  assert.equal(opened.child, source.child);
  assert.notEqual(opened.signalProcessGroup, source.signalProcessGroup);
  assert.equal(Object.isFrozen(opened), true);

  for (const value of [
    execution({ command: "python" }),
    { child: fakeChild(), signalProcessGroup: true },
    { child: {}, signalProcessGroup() {} },
    new Proxy(execution(), {}),
  ]) {
    assert.throws(
      () => openWikidotXmlrpcWorkerExecutionCapability(value),
      /worker (execution|session)/u,
    );
  }

  const accessor = execution();
  Object.defineProperty(accessor, "signalProcessGroup", {
    enumerable: true,
    get() {
      throw new Error("accessor must not run");
    },
  });
  assert.throws(
    () => openWikidotXmlrpcWorkerExecutionCapability(accessor),
    /data fields/u,
  );
});

test("execution capability binds group signaling to its immutable child snapshot", () => {
  const child = fakeChild();
  let receiver;
  let receivedSignal;
  const opened = openWikidotXmlrpcWorkerExecutionCapability({
    child,
    signalProcessGroup(signal) {
      receiver = this;
      receivedSignal = signal;
    },
  });

  opened.signalProcessGroup("SIGTERM");
  assert.equal(receiver.child, child);
  assert.equal(Object.isFrozen(receiver), true);
  assert.notEqual(receiver, opened);
  assert.equal(receivedSignal, "SIGTERM");
});
