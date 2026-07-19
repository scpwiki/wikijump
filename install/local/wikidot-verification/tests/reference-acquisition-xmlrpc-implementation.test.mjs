import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  buildWikidotXmlrpcImplementation,
  openWikidotXmlrpcImplementation,
  parseWikidotXmlrpcImplementation,
  putWikidotXmlrpcImplementation,
  serializeWikidotXmlrpcImplementation,
} from "../src/reference-acquisition-xmlrpc-implementation.mjs";
import {
  initializeReferenceObjectStore,
  openReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";

function options(overrides = {}) {
  return {
    coordinatorFileSha256: "a".repeat(64),
    dependencyLockFileSha256: "b".repeat(64),
    nodeVersion: "v26.4.0",
    pythonVersion: "3.14.0",
    wikijumpCommit: "1".repeat(40),
    wikijumpTree: "2".repeat(40),
    workerFileSha256: "c".repeat(64),
    workerRepositoryCommit: "3".repeat(40),
    workerRepositoryTree: "4".repeat(40),
    ...overrides,
  };
}

function implementation(overrides) {
  return buildWikidotXmlrpcImplementation(options(overrides));
}

async function fixture(t) {
  const parent = await fs.mkdtemp(
    path.join(os.tmpdir(), "xmlrpc-implementation-"),
  );
  const state = {
    root: path.join(parent, "store"),
    store: undefined,
  };
  state.store = await initializeReferenceObjectStore(state.root);
  t.after(async () => {
    await state.store.close();
    await fs.rm(parent, { force: true, recursive: true });
  });
  return state;
}

test("schema and canonical bytes bind exact implementation authority", async () => {
  const schema = JSON.parse(
    await fs.readFile(
      new URL(
        "../schemas/wikidot-xmlrpc-implementation-v1.schema.json",
        import.meta.url,
      ),
    ),
  );
  const value = implementation();
  assert.deepEqual(Object.keys(value).sort(), schema.required);
  assert.deepEqual(
    Object.keys(value).sort(),
    Object.keys(schema.properties).sort(),
  );
  assert.deepEqual(
    parseWikidotXmlrpcImplementation(
      serializeWikidotXmlrpcImplementation(value),
    ),
    value,
  );
  assert.equal(value.coordinator_repository, "Rokurolize/wikijump");
  assert.equal(value.dependency_lock_path, "uv.lock");
  assert.equal(value.rate_refill_per_second, 0.25);
  assert.equal(value.worker_repository, "Rokurolize/scp-wiki-translation");
  assert(Object.isFrozen(value));
});

test("content identity is stable across restart and changes with source bytes", async (t) => {
  const state = await fixture(t);
  let { store } = state;
  const first = await putWikidotXmlrpcImplementation(store, implementation());
  const repeated = await putWikidotXmlrpcImplementation(
    store,
    implementation(),
  );
  assert.equal(first.disposition, "created");
  assert.equal(repeated.disposition, "exists");
  assert.deepEqual(repeated.object, first.object);
  await store.close();
  store = await openReferenceObjectStore(state.root);
  state.store = store;
  const reopened = await openWikidotXmlrpcImplementation(store, first.object);
  assert.deepEqual(reopened.descriptor, implementation());
  const changed = await putWikidotXmlrpcImplementation(
    store,
    implementation({ workerFileSha256: "d".repeat(64) }),
  );
  assert.notEqual(changed.object.sha256, first.object.sha256);
});

test("authority and hostile inputs fail closed without leaking values", () => {
  const value = implementation();
  for (const [field, changed] of [
    ["endpoint", "https://example.invalid"],
    ["node_version", "26.4.0"],
    ["python_version", "v3.14.0"],
    ["rate_capacity", 2],
    ["worker_path", "scripts/other.py"],
    ["wikijump_commit", "A".repeat(40)],
  ]) {
    assert.throws(() =>
      serializeWikidotXmlrpcImplementation({ ...value, [field]: changed }),
    );
  }
  const secret = "sentinel-secret-must-not-survive";
  const accessor = options();
  Object.defineProperty(accessor, "nodeVersion", {
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
  const coercible = {
    valueOf() {
      throw new Error(secret);
    },
  };
  for (const call of [
    () => buildWikidotXmlrpcImplementation(accessor),
    () => buildWikidotXmlrpcImplementation(proxy),
    () => parseWikidotXmlrpcImplementation(coercible),
    () => serializeWikidotXmlrpcImplementation({ ...value, api_key: secret }),
  ]) {
    assert.throws(call, (error) => !error.message.includes(secret));
  }
});

test("parser rejects malformed and noncanonical byte streams", () => {
  const value = implementation();
  const reversed = Object.fromEntries(Object.entries(value).reverse());
  for (const bytes of [
    Buffer.from([0xff, 0x0a]),
    Buffer.from("{}\r\n"),
    Buffer.from(`${JSON.stringify(reversed)}\n`),
    Buffer.alloc(16 * 1024 + 1, 0x20),
  ]) {
    assert.throws(() => parseWikidotXmlrpcImplementation(bytes));
  }
});

test("opening verifies immutable CAS bytes and strict references", async (t) => {
  const state = await fixture(t);
  const stored = await putWikidotXmlrpcImplementation(
    state.store,
    implementation(),
  );
  const objectPath = path.join(
    state.root,
    ...referenceObjectRelativePath(stored.object.sha256).split("/"),
  );
  await fs.chmod(objectPath, 0o600);
  await fs.writeFile(objectPath, Buffer.alloc(stored.object.bytes, 0x20));
  await fs.chmod(objectPath, 0o400);
  await assert.rejects(
    openWikidotXmlrpcImplementation(state.store, stored.object),
    /corrupt/u,
  );
  const secret = "sentinel-reference-secret";
  const proxy = new Proxy(stored.object, {
    ownKeys() {
      throw new Error(secret);
    },
  });
  await assert.rejects(
    openWikidotXmlrpcImplementation(state.store, proxy),
    (error) => !error.message.includes(secret),
  );
});
