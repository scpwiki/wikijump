import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import fsp from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fork } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

import {
  initializeReferenceObjectStore,
  openReferenceObjectStore,
  referenceObjectRelativePath,
  referenceObjectStoreDescriptorBytes,
  validateReferenceObject,
} from "../src/reference-object-store.mjs";

const CHILD_PATH = fileURLToPath(
  new URL("./reference-object-store-child.mjs", import.meta.url),
);
const DESCRIPTOR_FIXTURE_PATH = fileURLToPath(
  new URL("../fixtures/reference-object-store-v1/store.json", import.meta.url),
);
const VECTOR_FIXTURE_PATH = fileURLToPath(
  new URL(
    "../fixtures/reference-object-store-v1/vectors.json",
    import.meta.url,
  ),
);
const VECTORS = JSON.parse(
  fs.readFileSync(VECTOR_FIXTURE_PATH, "utf8"),
).vectors;

async function temporaryStore(t) {
  const temporaryRoot = await fsp.mkdtemp(
    path.join(os.tmpdir(), "reference-object-store-"),
  );
  const root = path.join(temporaryRoot, "store");
  const store = await initializeReferenceObjectStore(root);
  t.after(async () => {
    await store.close();
    await fsp.rm(temporaryRoot, { force: true, recursive: true });
  });
  return { root, store, temporaryRoot };
}

function nextMessage(child) {
  return new Promise((resolve, reject) => {
    const onError = (error) => {
      child.off("message", onMessage);
      reject(error);
    };
    const onExit = (code) => {
      child.off("message", onMessage);
      reject(new Error(`child exited before replying with status ${code}`));
    };
    const onMessage = (message) => {
      child.off("error", onError);
      child.off("exit", onExit);
      resolve(message);
    };
    child.once("error", onError);
    child.once("exit", onExit);
    child.once("message", onMessage);
  });
}

function objectPath(root, digest) {
  return path.join(root, ...referenceObjectRelativePath(digest).split("/"));
}

test("initializes the normative descriptor and stores every golden byte vector", async (t) => {
  const fixture = await temporaryStore(t);
  const descriptorFixture = await fsp.readFile(DESCRIPTOR_FIXTURE_PATH);
  assert.deepEqual(
    await fsp.readFile(path.join(fixture.root, "store.json")),
    descriptorFixture,
  );
  assert.deepEqual(referenceObjectStoreDescriptorBytes(), descriptorFixture);
  const reopened = await openReferenceObjectStore(fixture.root);
  t.after(() => reopened.close());
  for (const vector of VECTORS) {
    const bytes = Buffer.from(vector.input_hex, "hex");
    const result = await reopened.putBytes(bytes);
    assert.equal(result.disposition, "created");
    assert.deepEqual(result.object, {
      algorithm: "sha256",
      bytes: vector.bytes,
      sha256: vector.sha256,
    });
    assert.equal(
      referenceObjectRelativePath(vector.sha256),
      vector.relative_path,
    );
    assert.deepEqual(
      await fsp.readFile(objectPath(fixture.root, vector.sha256)),
      bytes,
    );
    await reopened.verifyObject(result.object);
  }
  assert.equal(
    crypto.createHash("sha256").update(descriptorFixture).digest("hex"),
    "dfc3db9423713751f1f8bda474b934632fa969232f6a44dabb28e765a6288f79",
  );
  if (process.platform !== "win32") {
    assert.equal((await fsp.stat(fixture.root)).mode & 0o777, 0o700);
    assert.equal(
      (await fsp.stat(path.join(fixture.root, "store.json"))).mode & 0o777,
      0o400,
    );
  }
});

test("deduplicates exact bytes and rejects invalid expectations before publication", async (t) => {
  const fixture = await temporaryStore(t);
  const bytes = Buffer.from("immutable\0bytes");
  const first = await fixture.store.putBytes(bytes);
  const second = await fixture.store.putBytes(bytes);
  assert.equal(first.disposition, "created");
  assert.equal(second.disposition, "exists");
  assert.deepEqual(second.object, first.object);
  assert.deepEqual(
    await fsp.readdir(
      path.dirname(objectPath(fixture.root, first.object.sha256)),
    ),
    [first.object.sha256],
  );

  const unpublished = Buffer.from("never published");
  const unpublishedSha256 = crypto
    .createHash("sha256")
    .update(unpublished)
    .digest("hex");
  await assert.rejects(
    fixture.store.putBytes(unpublished, {
      expectedBytes: unpublished.length + 1,
    }),
    /byte length mismatch/u,
  );
  await assert.rejects(
    fixture.store.putBytes(unpublished, { expectedSha256: "0".repeat(64) }),
    /SHA-256 mismatch/u,
  );
  await assert.rejects(fsp.access(objectPath(fixture.root, unpublishedSha256)));

  for (const invalid of [
    { algorithm: "sha256", bytes: 1, sha256: "A".repeat(64) },
    { algorithm: "md5", bytes: 1, sha256: "a".repeat(64) },
    { algorithm: "sha256", bytes: -1, sha256: "a".repeat(64) },
    { algorithm: "sha256", bytes: 1, sha256: "a".repeat(64), extra: true },
  ]) {
    assert.throws(() => validateReferenceObject(invalid));
  }
  assert.throws(
    () => referenceObjectRelativePath("../escape"),
    /lowercase SHA-256/u,
  );
  await fixture.store.close();
  await fixture.store.close();
  await assert.rejects(fixture.store.putBytes(bytes), /store is closed/u);
});

test("fails closed on corrupt, symlinked, non-regular, and swapped state", async (t) => {
  const bytes = Buffer.from("expected bytes");
  const digest = crypto.createHash("sha256").update(bytes).digest("hex");

  const corrupt = await temporaryStore(t);
  const corruptPath = objectPath(corrupt.root, digest);
  await fsp.mkdir(path.dirname(corruptPath), { mode: 0o700 });
  await fsp.writeFile(corruptPath, "wrong", { mode: 0o400 });
  await assert.rejects(corrupt.store.putBytes(bytes), /corrupt/u);
  assert.equal(await fsp.readFile(corruptPath, "utf8"), "wrong");

  const symlinked = await temporaryStore(t);
  const symlinkPath = objectPath(symlinked.root, digest);
  const target = path.join(symlinked.temporaryRoot, "target");
  await fsp.mkdir(path.dirname(symlinkPath), { mode: 0o700 });
  await fsp.writeFile(target, "outside");
  await fsp.symlink(target, symlinkPath);
  await assert.rejects(symlinked.store.putBytes(bytes));
  assert.equal(await fsp.readFile(target, "utf8"), "outside");

  const nonRegular = await temporaryStore(t);
  const directoryPath = objectPath(nonRegular.root, digest);
  await fsp.mkdir(directoryPath, { mode: 0o700, recursive: true });
  await assert.rejects(nonRegular.store.putBytes(bytes), /regular file/u);
  assert.equal((await fsp.lstat(directoryPath)).isDirectory(), true);

  const descriptor = await temporaryStore(t);
  const descriptorPath = path.join(descriptor.root, "store.json");
  await fsp.chmod(descriptorPath, 0o600);
  await fsp.appendFile(descriptorPath, " ");
  await fsp.chmod(descriptorPath, 0o400);
  await assert.rejects(
    openReferenceObjectStore(descriptor.root),
    /canonical reference store descriptor/u,
  );
  await fsp.chmod(descriptorPath, 0o600);
  await fsp.truncate(descriptorPath, 4 * 1024 * 1024 * 1024);
  await fsp.chmod(descriptorPath, 0o400);
  await assert.rejects(
    openReferenceObjectStore(descriptor.root),
    /canonical reference store descriptor/u,
  );

  const swapped = await temporaryStore(t);
  const sha256Path = path.join(swapped.root, "objects", "sha256");
  const outsidePath = path.join(swapped.temporaryRoot, "outside");
  await fsp.mkdir(outsidePath, { mode: 0o700 });
  await fsp.rename(sha256Path, `${sha256Path}-original`);
  await fsp.symlink(outsidePath, sha256Path);
  await assert.rejects(swapped.store.putBytes(Buffer.from("must stay inside")));
  assert.deepEqual(await fsp.readdir(outsidePath), []);
});

test(
  "same-object multi-process races publish once and verify every loser",
  { timeout: 20_000 },
  async (t) => {
    const fixture = await temporaryStore(t);
    const children = [];
    t.after(() => {
      for (const child of children) child.kill("SIGKILL");
    });
    const bytes = crypto.randomBytes(256 * 1024);
    const inputPath = path.join(fixture.temporaryRoot, "race-input.bin");
    await fsp.writeFile(inputPath, bytes);
    for (let index = 0; index < 8; index += 1) {
      children.push(
        fork(CHILD_PATH, [fixture.root, inputPath], { silent: true }),
      );
    }
    const ready = await Promise.all(
      children.map((child) => nextMessage(child)),
    );
    assert.deepEqual(
      new Set(ready.map((message) => message.type)),
      new Set(["ready"]),
    );
    const results = children.map((child) => nextMessage(child));
    for (const child of children) child.send({ type: "go" });
    const replies = await Promise.all(results);
    const errors = replies.filter((reply) => reply.type === "error");
    assert.deepEqual(errors, [], JSON.stringify(errors, null, 2));
    assert.equal(
      replies.filter((reply) => reply.disposition === "created").length,
      1,
    );
    assert.equal(
      replies.filter((reply) => reply.disposition === "exists").length,
      7,
    );
    assert.equal(new Set(replies.map((reply) => reply.object.sha256)).size, 1);
    const digest = replies[0].object.sha256;
    assert.deepEqual(
      await fsp.readFile(objectPath(fixture.root, digest)),
      bytes,
    );
    assert.deepEqual(
      await fsp.readdir(path.dirname(objectPath(fixture.root, digest))),
      [digest],
    );
  },
);
