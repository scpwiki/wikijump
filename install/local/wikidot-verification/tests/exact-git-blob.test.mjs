import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  createExactGitBlobReader,
  createExactGitTreeReader,
  readExactGitBlob,
  readExactGitTreeFiles,
} from "../src/exact-git-blob.mjs";
import {
  binding,
  expected,
  fixture,
  rejectsWith,
  sha256,
} from "./support/exact-git-blob-fixture.mjs";

test("resolver returns exact binary committed bytes from a bare SHA-1 repository", async (t) => {
  const state = await fixture(t);
  await assert.rejects(
    fs.access(path.join(state.gitDirectory, "scripts", "worker.py")),
  );
  const result = await readExactGitBlob(binding(state), expected(state), {
    maxBytes: state.worker.byteLength,
  });
  assert.deepEqual(Object.keys(result).sort(), [
    "blobOid",
    "byteLength",
    "readBytes",
    "sha256",
  ]);
  assert.equal(Object.isFrozen(result), true);
  assert.equal(result.blobOid, state.workerOid);
  assert.equal(result.byteLength, state.worker.byteLength);
  assert.equal(result.sha256, sha256(state.worker));
  assert.deepEqual(result.readBytes(), state.worker);
  const mutableCopy = result.readBytes();
  mutableCopy.fill(0);
  assert.deepEqual(result.readBytes(), state.worker);
});

test("tree resolver returns a fixed set of exact regular source blobs", async (t) => {
  const state = await fixture(t);
  const result = await readExactGitTreeFiles(
    { gitDirectory: state.gitDirectory },
    { commitOid: state.commit, treeOid: state.tree },
    ["scripts/worker.py"],
    {
      maxBytesPerFile: state.worker.byteLength,
      maxFiles: 1,
      maxTotalBytes: state.worker.byteLength,
    },
  );
  assert.deepEqual(Object.keys(result).sort(), [
    "commitOid",
    "files",
    "treeOid",
  ]);
  assert.equal(Object.isFrozen(result), true);
  assert.equal(result.commitOid, state.commit);
  assert.equal(result.treeOid, state.tree);
  assert.equal(result.files.length, 1);
  assert.equal(result.files[0].path, "scripts/worker.py");
  assert.equal(result.files[0].blobOid, state.workerOid);
  assert.equal(result.files[0].sha256, sha256(state.worker));
  assert.deepEqual(result.files[0].readBytes(), state.worker);
  const mutableCopy = result.files[0].readBytes();
  mutableCopy.fill(0);
  assert.deepEqual(result.files[0].readBytes(), state.worker);

  await assert.rejects(
    readExactGitTreeFiles(
      { gitDirectory: state.gitDirectory },
      { commitOid: state.commit, treeOid: state.tree },
      ["link"],
      { maxBytesPerFile: 1024, maxFiles: 1, maxTotalBytes: 1024 },
    ),
    rejectsWith("git_path_not_regular_blob"),
  );
  await assert.rejects(
    readExactGitTreeFiles(
      { gitDirectory: state.gitDirectory },
      { commitOid: state.commit, treeOid: state.tree },
      ["scripts/worker.py", "scripts/worker.py"],
      { maxBytesPerFile: 1024, maxFiles: 2, maxTotalBytes: 1024 },
    ),
    rejectsWith("invalid_tree_paths"),
  );
  assert.throws(
    () => createExactGitTreeReader({ gitExecutable: "git" }),
    rejectsWith("invalid_reader_configuration"),
  );
});

test("resolver accepts default limits and snapshots exact inputs before I/O", async (t) => {
  const state = await fixture(t);
  const defaultResult = await readExactGitBlob(binding(state), expected(state));
  assert.deepEqual(defaultResult.readBytes(), state.worker);

  const mutableBinding = binding(state);
  const mutableExpected = expected(state);
  const mutableOptions = { maxBytes: state.worker.byteLength };
  const pending = readExactGitBlob(
    mutableBinding,
    mutableExpected,
    mutableOptions,
  );
  mutableBinding.path = "link";
  mutableExpected.blobOid = state.linkOid;
  mutableExpected.blobSha256 = sha256(state.link);
  mutableOptions.maxBytes = 1;
  const result = await pending;
  assert.equal(result.blobOid, state.workerOid);
  assert.deepEqual(result.readBytes(), state.worker);
});

test("resolver accepts null-prototype records and rejects non-data exact shapes", async (t) => {
  const state = await fixture(t);
  const nullPrototypeBinding = Object.assign(
    Object.create(null),
    binding(state),
  );
  const nullPrototypeExpected = Object.assign(
    Object.create(null),
    expected(state),
  );
  const nullPrototypeOptions = Object.assign(Object.create(null), {
    maxBytes: state.worker.byteLength,
  });
  const result = await readExactGitBlob(
    nullPrototypeBinding,
    nullPrototypeExpected,
    nullPrototypeOptions,
  );
  assert.deepEqual(result.readBytes(), state.worker);

  const accessorBinding = binding(state);
  Object.defineProperty(accessorBinding, "path", {
    enumerable: true,
    get() {
      return "scripts/worker.py";
    },
  });
  const symbolBinding = binding(state);
  symbolBinding[Symbol("extra")] = true;
  const revokedBinding = Proxy.revocable(binding(state), {});
  revokedBinding.revoke();
  for (const [readBinding, claim, options, code] of [
    [
      { ...binding(state), extra: true },
      expected(state),
      { maxBytes: state.worker.byteLength },
      "invalid_git_binding",
    ],
    [
      accessorBinding,
      expected(state),
      { maxBytes: state.worker.byteLength },
      "invalid_git_binding",
    ],
    [
      symbolBinding,
      expected(state),
      { maxBytes: state.worker.byteLength },
      "invalid_git_binding",
    ],
    [
      new Proxy(binding(state), {}),
      expected(state),
      { maxBytes: state.worker.byteLength },
      "invalid_git_binding",
    ],
    [
      revokedBinding.proxy,
      expected(state),
      { maxBytes: state.worker.byteLength },
      "invalid_git_binding",
    ],
    [
      binding(state),
      { ...expected(state), extra: true },
      { maxBytes: state.worker.byteLength },
      "invalid_expected_identity",
    ],
    [binding(state), expected(state), {}, "invalid_read_options"],
    [binding(state), expected(state), null, "invalid_read_options"],
    [binding(state), expected(state), { maxBytes: 0 }, "invalid_read_options"],
    [
      binding(state),
      expected(state),
      { maxBytes: 8 * 1024 * 1024 + 1 },
      "invalid_read_options",
    ],
  ]) {
    await assert.rejects(
      readExactGitBlob(readBinding, claim, options),
      rejectsWith(code),
    );
  }
  assert.throws(
    () => createExactGitBlobReader({ gitExecutable: "git" }),
    rejectsWith("invalid_reader_configuration"),
  );
});
