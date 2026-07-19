import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  buildWikidotXmlrpcWorkerAuthority,
  openWikidotXmlrpcWorkerAuthority,
  parseWikidotXmlrpcWorkerAuthority,
  putWikidotXmlrpcWorkerAuthority,
  serializeWikidotXmlrpcWorkerAuthority,
} from "../src/wikidot-xmlrpc-worker-authority.mjs";
import {
  initializeReferenceObjectStore,
  openReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";

function options(overrides = {}) {
  return {
    dependencyEnvironmentSha256: "a".repeat(64),
    dependencyLockFileSha256: "b".repeat(64),
    dependencyRecipeSha256: "c".repeat(64),
    pythonExecutableSha256: "d".repeat(64),
    pythonVersion: "3.13.13",
    venvConfigSha256: "e".repeat(64),
    workerBlobOid: "1".repeat(40),
    workerFileSha256: "f".repeat(64),
    workerRepositoryCommit: "2".repeat(40),
    workerRepositoryTree: "3".repeat(40),
    ...overrides,
  };
}

function authority(overrides) {
  return buildWikidotXmlrpcWorkerAuthority(options(overrides));
}

async function fixture(t) {
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-worker-"));
  const root = path.join(parent, "store");
  const state = { root, store: await initializeReferenceObjectStore(root) };
  t.after(async () => {
    await state.store.close();
    await fs.rm(parent, { force: true, recursive: true });
  });
  return state;
}

test("schema and canonical bytes bind a path-free worker authority", async () => {
  const schema = JSON.parse(
    await fs.readFile(
      new URL(
        "../schemas/wikidot-xmlrpc-worker-authority-v1.schema.json",
        import.meta.url,
      ),
    ),
  );
  const value = authority();
  assert.deepEqual(Object.keys(value).sort(), schema.required);
  assert.deepEqual(
    Object.keys(value).sort(),
    Object.keys(schema.properties).sort(),
  );
  assert.equal(schema.properties.python_version.maxLength, 64);
  assert.deepEqual(
    parseWikidotXmlrpcWorkerAuthority(
      serializeWikidotXmlrpcWorkerAuthority(value),
    ),
    value,
  );
  assert.equal(value.worker_repository, "Rokurolize/scp-wiki-translation");
  assert.equal(Object.isFrozen(value), true);
});

test("CAS identity is stable across restart and changes with every authority input", async (t) => {
  const state = await fixture(t);
  const first = await putWikidotXmlrpcWorkerAuthority(state.store, authority());
  const repeated = await putWikidotXmlrpcWorkerAuthority(
    state.store,
    authority(),
  );
  assert.equal(first.disposition, "created");
  assert.equal(repeated.disposition, "exists");
  assert.deepEqual(repeated.object, first.object);
  await state.store.close();
  state.store = await openReferenceObjectStore(state.root);
  assert.deepEqual(
    (await openWikidotXmlrpcWorkerAuthority(state.store, first.object))
      .descriptor,
    authority(),
  );
  for (const [field, replacement] of Object.entries({
    dependencyEnvironmentSha256: "0".repeat(64),
    dependencyLockFileSha256: "1".repeat(64),
    dependencyRecipeSha256: "2".repeat(64),
    pythonExecutableSha256: "3".repeat(64),
    pythonVersion: "3.13.14",
    venvConfigSha256: "4".repeat(64),
    workerBlobOid: "4".repeat(40),
    workerFileSha256: "5".repeat(64),
    workerRepositoryCommit: "5".repeat(40),
    workerRepositoryTree: "6".repeat(40),
  })) {
    const changed = await putWikidotXmlrpcWorkerAuthority(
      state.store,
      authority({ [field]: replacement }),
    );
    assert.notEqual(changed.object.sha256, first.object.sha256, field);
  }
});

test("authority and hostile inputs fail closed without leaking paths or secrets", () => {
  const value = authority();
  for (const [field, changed] of [
    ["python_version", "v3.13.13"],
    ["python_version", `3.${"1".repeat(70)}.0`],
    ["worker_blob_oid", "A".repeat(40)],
    ["worker_file_sha256", "A".repeat(64)],
  ]) {
    assert.throws(() =>
      serializeWikidotXmlrpcWorkerAuthority({ ...value, [field]: changed }),
    );
  }
  assert.throws(() =>
    buildWikidotXmlrpcWorkerAuthority(
      options({ pythonVersion: `3.${"1".repeat(70)}.0` }),
    ),
  );
  const secret = "sentinel-authority-secret";
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
    () => buildWikidotXmlrpcWorkerAuthority(accessor),
    () => buildWikidotXmlrpcWorkerAuthority(proxy),
    () => parseWikidotXmlrpcWorkerAuthority({ valueOf: () => secret }),
    () =>
      serializeWikidotXmlrpcWorkerAuthority({
        ...value,
        worker_path: secret,
      }),
  ]) {
    assert.throws(call, (error) => !error.message.includes(secret));
  }
});

test("parser rejects malformed and noncanonical byte streams", () => {
  const value = authority();
  const reversed = Object.fromEntries(Object.entries(value).reverse());
  for (const bytes of [
    Buffer.from([0xff, 0x0a]),
    Buffer.from("{}\r\n"),
    Buffer.from(`${JSON.stringify(reversed)}\n`),
    Buffer.alloc(8 * 1024 + 1, 0x20),
  ]) {
    assert.throws(() => parseWikidotXmlrpcWorkerAuthority(bytes));
  }
});

test("opening verifies immutable CAS bytes and strict references", async (t) => {
  const state = await fixture(t);
  const stored = await putWikidotXmlrpcWorkerAuthority(
    state.store,
    authority(),
  );
  const objectPath = path.join(
    state.root,
    ...referenceObjectRelativePath(stored.object.sha256).split("/"),
  );
  await assert.rejects(
    openWikidotXmlrpcWorkerAuthority(state.store, {
      algorithm: "sha256",
      bytes: 1,
      sha256: "0".repeat(64),
    }),
    (error) =>
      error.message === "XML-RPC worker authority object cannot be read" &&
      !error.message.includes("/proc/"),
  );
  await fs.chmod(objectPath, 0o600);
  await fs.writeFile(objectPath, Buffer.alloc(stored.object.bytes, 0x20));
  await fs.chmod(objectPath, 0o400);
  await assert.rejects(
    openWikidotXmlrpcWorkerAuthority(state.store, stored.object),
    /object cannot be read/u,
  );
  const secret = "sentinel-reference-secret";
  const proxy = new Proxy(stored.object, {
    ownKeys() {
      throw new Error(secret);
    },
  });
  await assert.rejects(
    openWikidotXmlrpcWorkerAuthority(state.store, proxy),
    (error) => !error.message.includes(secret),
  );
});
