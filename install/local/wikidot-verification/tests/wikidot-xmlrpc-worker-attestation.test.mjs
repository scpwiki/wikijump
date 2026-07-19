import assert from "node:assert/strict";
import test from "node:test";

import { buildWikidotXmlrpcPythonEnvironment } from "../src/wikidot-xmlrpc-python-environment.mjs";
import { validateWikidotXmlrpcWorkerAttestation } from "../src/wikidot-xmlrpc-worker-attestation.mjs";

function environment(overrides = {}) {
  return buildWikidotXmlrpcPythonEnvironment({
    dependencyEnvironmentSha256: "a".repeat(64),
    dependencyLockBlobOid: "1".repeat(40),
    dependencyLockFileSha256: "b".repeat(64),
    dependencyRecipeBlobOid: "2".repeat(40),
    dependencyRecipeSha256: "c".repeat(64),
    pythonExecutableSha256: "d".repeat(64),
    pythonImplementation: "cpython",
    pythonVersion: "3.13.13",
    venvConfigSha256: "e".repeat(64),
    workerBlobOid: "3".repeat(40),
    workerFileSha256: "f".repeat(64),
    workerRepositoryCommit: "4".repeat(40),
    workerRepositoryTree: "5".repeat(40),
    ...overrides,
  });
}

function attestation(overrides = {}) {
  return {
    ok: true,
    op: "attestation",
    protocol_version: 2,
    runtime: { implementation: "cpython", version: [3, 13, 13] },
    worker: "wikidot_xmlrpc_capture_worker",
    ...overrides,
  };
}

test("a precise v2 worker attestation yields a frozen credential-free capability", () => {
  const expected = environment();
  const result = validateWikidotXmlrpcWorkerAttestation(
    expected,
    attestation(),
  );
  assert.deepEqual(result, {
    environment: expected,
    protocolVersion: 2,
    runtime: { implementation: "cpython", version: [3, 13, 13] },
    worker: "wikidot_xmlrpc_capture_worker",
  });
  assert.equal(Object.isFrozen(result), true);
  assert.equal(Object.isFrozen(result.environment), true);
  assert.equal(Object.isFrozen(result.runtime), true);
  assert.equal(Object.isFrozen(result.runtime.version), true);
});

test("attestation mismatch, wrong shape, and runtime divergence fail closed", () => {
  for (const value of [
    attestation({ ok: false }),
    attestation({ op: "ready" }),
    attestation({ protocol_version: 1 }),
    attestation({ worker: "other_worker" }),
    attestation({ runtime: { implementation: "pypy", version: [3, 13, 13] } }),
    attestation({
      runtime: { implementation: "cpython", version: [3, 13, 14] },
    }),
    { ...attestation(), extra: true },
    { ...attestation(), runtime: { implementation: "cpython" } },
    attestation({ runtime: { implementation: "cpython", version: [3, 13] } }),
    attestation({
      runtime: { implementation: "cpython", version: [3, 13, -1] },
    }),
    attestation({
      runtime: { implementation: "cpython", version: [-0, 13, 13] },
    }),
    attestation({
      runtime: { implementation: "cpython", version: [3, 13, 2 ** 53] },
    }),
  ]) {
    assert.throws(
      () => validateWikidotXmlrpcWorkerAttestation(environment(), value),
      /attestation|runtime/u,
    );
  }
  assert.throws(
    () =>
      validateWikidotXmlrpcWorkerAttestation(
        environment({ pythonVersion: "3.13.14" }),
        attestation(),
      ),
    /does not match/u,
  );
});

test("attestation rejects proxy, accessor, and mutated runtime data without leaking values", () => {
  const secret = "sentinel-attestation-secret";
  const accessor = attestation();
  Object.defineProperty(accessor, "worker", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const proxy = new Proxy(attestation(), {
    ownKeys() {
      throw new Error(secret);
    },
  });
  const version = [3, 13, 13];
  version.extra = secret;
  for (const value of [
    accessor,
    proxy,
    attestation({ runtime: { implementation: "cpython", version } }),
    attestation({ runtime: new Proxy({}, {}) }),
  ]) {
    assert.throws(
      () => validateWikidotXmlrpcWorkerAttestation(environment(), value),
      (error) => !error.message.includes(secret),
    );
  }
});

test("an invalid expected environment prevents any attestation from becoming capable", () => {
  const secret = "sentinel-environment-secret";
  const poisoned = { ...environment() };
  Object.defineProperty(poisoned, "worker", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  assert.throws(
    () => validateWikidotXmlrpcWorkerAttestation(poisoned, attestation()),
    (error) =>
      error.message === "worker attestation environment is invalid" &&
      !error.message.includes(secret),
  );
});
