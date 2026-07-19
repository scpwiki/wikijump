import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  assertWikidotXmlrpcPythonEnvironmentMatchesWorkerAuthority,
  buildWikidotXmlrpcPythonEnvironment,
  openWikidotXmlrpcPythonEnvironment,
  parseWikidotXmlrpcPythonEnvironment,
  putWikidotXmlrpcPythonEnvironment,
  serializeWikidotXmlrpcPythonEnvironment,
} from "../src/wikidot-xmlrpc-python-environment.mjs";
import { buildWikidotXmlrpcWorkerAuthority } from "../src/wikidot-xmlrpc-worker-authority.mjs";
import {
  initializeReferenceObjectStore,
  openReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";

function options(overrides = {}) {
  return {
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
  };
}

function environment(overrides) {
  return buildWikidotXmlrpcPythonEnvironment(options(overrides));
}

function authority(overrides = {}) {
  const input = options(overrides);
  return buildWikidotXmlrpcWorkerAuthority({
    dependencyEnvironmentSha256: input.dependencyEnvironmentSha256,
    dependencyLockFileSha256: input.dependencyLockFileSha256,
    dependencyRecipeSha256: input.dependencyRecipeSha256,
    pythonExecutableSha256: input.pythonExecutableSha256,
    pythonVersion: input.pythonVersion,
    venvConfigSha256: input.venvConfigSha256,
    workerBlobOid: input.workerBlobOid,
    workerFileSha256: input.workerFileSha256,
    workerRepositoryCommit: input.workerRepositoryCommit,
    workerRepositoryTree: input.workerRepositoryTree,
  });
}

async function fixture(t) {
  const parent = await fs.mkdtemp(
    path.join(os.tmpdir(), "xmlrpc-python-environment-"),
  );
  const root = path.join(parent, "store");
  const state = { root, store: await initializeReferenceObjectStore(root) };
  t.after(async () => {
    await state.store.close();
    await fs.rm(parent, { force: true, recursive: true });
  });
  return state;
}

test("schema and canonical bytes bind a path-free Python environment", async () => {
  const schema = JSON.parse(
    await fs.readFile(
      new URL(
        "../schemas/wikidot-xmlrpc-python-environment-v1.schema.json",
        import.meta.url,
      ),
    ),
  );
  const value = environment();
  assert.deepEqual(Object.keys(value).sort(), schema.required);
  assert.deepEqual(
    Object.keys(value).sort(),
    Object.keys(schema.properties).sort(),
  );
  assert.equal(value.protocol_version, 2);
  assert.equal(value.python_implementation, "cpython");
  assert.equal(value.worker, "wikidot_xmlrpc_capture_worker");
  assert.equal(schema.properties.python_version.maxLength, 64);
  assert.equal(
    new RegExp(schema.properties.python_version.pattern).test(
      "9007199254740992.0.0",
    ),
    false,
  );
  assert.deepEqual(
    parseWikidotXmlrpcPythonEnvironment(
      serializeWikidotXmlrpcPythonEnvironment(value),
    ),
    value,
  );
  assert.equal(Object.isFrozen(value), true);
});

test("CAS identity is stable and changes with every environment input", async (t) => {
  const state = await fixture(t);
  const first = await putWikidotXmlrpcPythonEnvironment(
    state.store,
    environment(),
  );
  const repeated = await putWikidotXmlrpcPythonEnvironment(
    state.store,
    environment(),
  );
  assert.equal(first.disposition, "created");
  assert.equal(repeated.disposition, "exists");
  assert.deepEqual(repeated.object, first.object);
  await state.store.close();
  state.store = await openReferenceObjectStore(state.root);
  assert.deepEqual(
    (await openWikidotXmlrpcPythonEnvironment(state.store, first.object))
      .descriptor,
    environment(),
  );
  for (const [field, replacement] of Object.entries({
    dependencyEnvironmentSha256: "0".repeat(64),
    dependencyLockBlobOid: "6".repeat(40),
    dependencyLockFileSha256: "1".repeat(64),
    dependencyRecipeBlobOid: "7".repeat(40),
    dependencyRecipeSha256: "2".repeat(64),
    pythonExecutableSha256: "3".repeat(64),
    pythonVersion: "3.13.14",
    venvConfigSha256: "4".repeat(64),
    workerBlobOid: "8".repeat(40),
    workerFileSha256: "5".repeat(64),
    workerRepositoryCommit: "9".repeat(40),
    workerRepositoryTree: "a".repeat(40),
  })) {
    const changed = await putWikidotXmlrpcPythonEnvironment(
      state.store,
      environment({ [field]: replacement }),
    );
    assert.notEqual(changed.object.sha256, first.object.sha256, field);
  }
});

test("environment preserves every authority-v1 claim and rejects disagreement", () => {
  const value = environment();
  const matched = assertWikidotXmlrpcPythonEnvironmentMatchesWorkerAuthority(
    value,
    authority(),
  );
  assert.deepEqual(matched.descriptor, value);
  assert.equal(Object.isFrozen(matched), true);
  for (const [field, replacement] of Object.entries({
    dependencyEnvironmentSha256: "0".repeat(64),
    dependencyLockFileSha256: "1".repeat(64),
    dependencyRecipeSha256: "2".repeat(64),
    pythonExecutableSha256: "3".repeat(64),
    pythonVersion: "3.13.14",
    venvConfigSha256: "4".repeat(64),
    workerBlobOid: "6".repeat(40),
    workerFileSha256: "5".repeat(64),
    workerRepositoryCommit: "7".repeat(40),
    workerRepositoryTree: "8".repeat(40),
  })) {
    assert.throws(
      () =>
        assertWikidotXmlrpcPythonEnvironmentMatchesWorkerAuthority(
          value,
          authority({ [field]: replacement }),
        ),
      /does not match/u,
      field,
    );
  }
});

test("hostile or noncanonical inputs fail closed without leaking values", () => {
  const value = environment();
  for (const [field, replacement] of [
    ["python_implementation", "pypy"],
    ["python_version", "v3.13.13"],
    ["python_version", `3.${"1".repeat(70)}.0`],
    ["python_version", "9007199254740992.0.0"],
    ["worker_blob_oid", "A".repeat(40)],
    ["worker_file_sha256", "A".repeat(64)],
  ]) {
    assert.throws(() =>
      serializeWikidotXmlrpcPythonEnvironment({
        ...value,
        [field]: replacement,
      }),
    );
  }
  assert.throws(() =>
    buildWikidotXmlrpcPythonEnvironment(
      options({ pythonImplementation: "pypy" }),
    ),
  );
  const secret = "sentinel-environment-secret";
  const accessor = options();
  Object.defineProperty(accessor, "workerBlobOid", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const proxy = new Proxy(options(), {
    ownKeys() {
      throw new Error(secret);
    },
  });
  for (const call of [
    () => buildWikidotXmlrpcPythonEnvironment(accessor),
    () => buildWikidotXmlrpcPythonEnvironment(proxy),
    () => parseWikidotXmlrpcPythonEnvironment({ valueOf: () => secret }),
    () =>
      serializeWikidotXmlrpcPythonEnvironment({
        ...value,
        worker_path: secret,
      }),
  ]) {
    assert.throws(call, (error) => !error.message.includes(secret));
  }
  const reversed = Object.fromEntries(Object.entries(value).reverse());
  for (const bytes of [
    Buffer.from([0xff, 0x0a]),
    Buffer.from("{}\r\n"),
    Buffer.from(`${JSON.stringify(reversed)}\n`),
    Buffer.alloc(16 * 1024 + 1, 0x20),
  ]) {
    assert.throws(() => parseWikidotXmlrpcPythonEnvironment(bytes));
  }
});

test("opening verifies immutable CAS bytes and strict references", async (t) => {
  const state = await fixture(t);
  const stored = await putWikidotXmlrpcPythonEnvironment(
    state.store,
    environment(),
  );
  const objectPath = path.join(
    state.root,
    ...referenceObjectRelativePath(stored.object.sha256).split("/"),
  );
  await assert.rejects(
    openWikidotXmlrpcPythonEnvironment(state.store, {
      algorithm: "sha256",
      bytes: 1,
      sha256: "0".repeat(64),
    }),
    (error) =>
      error.message === "XML-RPC Python environment object cannot be read" &&
      !error.message.includes("/proc/"),
  );
  await fs.chmod(objectPath, 0o600);
  await fs.writeFile(objectPath, Buffer.alloc(stored.object.bytes, 0x20));
  await fs.chmod(objectPath, 0o400);
  await assert.rejects(
    openWikidotXmlrpcPythonEnvironment(state.store, stored.object),
    /object cannot be read/u,
  );
  const secret = "sentinel-reference-secret";
  const proxy = new Proxy(stored.object, {
    ownKeys() {
      throw new Error(secret);
    },
  });
  await assert.rejects(
    openWikidotXmlrpcPythonEnvironment(state.store, proxy),
    (error) => !error.message.includes(secret),
  );
});
