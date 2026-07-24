import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import test from "node:test";

import {
  createExactGitBlobReader,
  ExactGitBlobError,
  readExactGitBlob,
} from "../src/exact-git-blob.mjs";
import {
  assertProcessTerminated,
  binding,
  expected,
  fixture,
  git,
  processExists,
  rejectsWith,
  repoGit,
  runResolverChild,
  sha256,
  shellQuote,
  writeBlob,
  writeCommit,
  writeExecutable,
  writeLooseObject,
} from "./support/exact-git-blob-fixture.mjs";

test("resolver ignores a poisoned caller PATH when opening an exact blob", async (t) => {
  const state = await fixture(t);
  const fakeDirectory = path.join(path.dirname(state.gitDirectory), "fake-bin");
  const marker = path.join(fakeDirectory, "fake-git-ran");
  await fs.mkdir(fakeDirectory);
  await writeExecutable(
    path.join(fakeDirectory, "git"),
    `#!/bin/sh\n: > ${shellQuote(marker)}\nexit 91\n`,
  );
  const result = runResolverChild(
    binding(state),
    expected(state),
    { maxBytes: state.worker.byteLength },
    { PATH: `${fakeDirectory}:/usr/bin:/bin` },
  );
  assert.equal(result.blobOid, state.workerOid);
  assert.equal(result.bytes, state.worker.toString("hex"));
  await assert.rejects(fs.access(marker));
});

test("resolver terminates a failing Git process group before it rejects", async (t) => {
  const state = await fixture(t);
  const fakeExecutable = path.join(
    path.dirname(state.gitDirectory),
    "hanging-git",
  );
  const parentPidPath = path.join(
    path.dirname(state.gitDirectory),
    "parent.pid",
  );
  const descendantPidPath = path.join(
    path.dirname(state.gitDirectory),
    "descendant.pid",
  );
  const trackedPids = [];
  t.after(() => {
    for (const pid of trackedPids) {
      if (!processExists(pid)) continue;
      try {
        process.kill(-pid, "SIGKILL");
      } catch (error) {
        if (error?.code !== "ESRCH") throw error;
        try {
          process.kill(pid, "SIGKILL");
        } catch (fallbackError) {
          if (fallbackError?.code !== "ESRCH") throw fallbackError;
        }
      }
    }
  });
  await writeExecutable(
    fakeExecutable,
    [
      "#!/bin/sh",
      "sleep 60 &",
      "child=$!",
      `printf '%s\\n' "$$" > ${shellQuote(parentPidPath)}`,
      `printf '%s\\n' "$child" > ${shellQuote(descendantPidPath)}`,
      "count=0",
      'while [ "$count" -lt 1200 ]; do',
      "  printf x",
      "  count=$((count + 1))",
      "done",
      "while :; do sleep 60; done",
      "",
    ].join("\n"),
  );
  const hangingReader = createExactGitBlobReader({
    gitExecutable: fakeExecutable,
  });
  await assert.rejects(
    hangingReader(binding(state), expected(state), {
      maxBytes: state.worker.byteLength,
    }),
    rejectsWith("git_output_too_large"),
  );
  const parentPid = Number.parseInt(
    await fs.readFile(parentPidPath, "utf8"),
    10,
  );
  const descendantPid = Number.parseInt(
    await fs.readFile(descendantPidPath, "utf8"),
    10,
  );
  trackedPids.push(parentPid, descendantPid);
  assert.equal(Number.isSafeInteger(parentPid), true);
  assert.equal(Number.isSafeInteger(descendantPid), true);
  await assertProcessTerminated(parentPid);
  await assertProcessTerminated(descendantPid);
});

test("resolver proves the claimed commit, tree, blob, mode, and content hash", async (t) => {
  const state = await fixture(t);
  for (const [readBinding, claim, options, code] of [
    [
      binding(state),
      expected(state, { treeOid: "0".repeat(40) }),
      {},
      "commit_tree_mismatch",
    ],
    [
      binding(state),
      expected(state, { blobOid: state.linkOid }),
      {},
      "tree_blob_mismatch",
    ],
    [
      binding(state),
      expected(state, { blobSha256: "0".repeat(64) }),
      {},
      "blob_sha256_mismatch",
    ],
    [
      binding(state, "link"),
      expected(state, {
        blobOid: state.linkOid,
        blobSha256: sha256(state.link),
      }),
      {},
      "git_path_not_regular_blob",
    ],
    [
      binding(state, "scripts"),
      expected(state),
      {},
      "git_path_not_regular_blob",
    ],
    [
      binding(state, "submodule"),
      expected(state, {
        blobOid: state.replacementCommit,
        blobSha256: "0".repeat(64),
      }),
      {},
      "git_path_not_regular_blob",
    ],
    [
      binding(state),
      expected(state),
      { maxBytes: state.worker.byteLength - 1 },
      "blob_too_large",
    ],
  ]) {
    await assert.rejects(
      readExactGitBlob(readBinding, claim, {
        maxBytes: options.maxBytes ?? state.worker.byteLength,
      }),
      rejectsWith(code),
    );
  }
});

test("resolver rejects refs, unsafe paths, unsupported object formats, and host paths", async (t) => {
  const state = await fixture(t);
  for (const [readBinding, claim, code] of [
    [binding(state, "../worker.py"), expected(state), "invalid_git_path"],
    [binding(state, "scripts//worker.py"), expected(state), "invalid_git_path"],
    [binding(state, "scripts/:(glob)*"), expected(state), "invalid_git_path"],
    [
      binding(state),
      expected(state, { commitOid: state.commit.toUpperCase() }),
      "invalid_expected_identity",
    ],
    [
      binding(state),
      expected(state, { commitOid: "HEAD" }),
      "invalid_expected_identity",
    ],
    [
      binding(state),
      expected(state, { commitOid: state.workerOid }),
      "git_object_type_mismatch",
    ],
  ]) {
    await assert.rejects(
      readExactGitBlob(readBinding, claim, {
        maxBytes: state.worker.byteLength,
      }),
      rejectsWith(code),
    );
  }
  const pathSentinel = "git-directory-sentinel";
  await assert.rejects(
    readExactGitBlob(
      {
        gitDirectory: path.join(os.tmpdir(), pathSentinel),
        path: "scripts/worker.py",
      },
      expected(state),
      { maxBytes: state.worker.byteLength },
    ),
    (error) => {
      assert.equal(error instanceof ExactGitBlobError, true);
      assert.equal(error.message.includes(pathSentinel), false);
      assert.equal(
        error.stack,
        "ExactGitBlobError: exact Git blob invalid_git_binding",
      );
      return true;
    },
  );
  const sha256Directory = path.join(
    path.dirname(state.gitDirectory),
    "sha256.git",
  );
  git(["init", "--bare", "--object-format=sha256", sha256Directory]);
  await assert.rejects(
    readExactGitBlob(
      {
        gitDirectory: sha256Directory,
        path: "scripts/worker.py",
      },
      expected(state),
      { maxBytes: state.worker.byteLength },
    ),
    rejectsWith("unsupported_git_object_format"),
  );
});

test("resolver rejects malformed tree mode bytes that decode as regular blobs", async (t) => {
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "exact-git-blob-"));
  const gitDirectory = path.join(parent, "repo.git");
  git(["init", "--bare", "--initial-branch=main", gitDirectory]);
  t.after(() => fs.rm(parent, { force: true, recursive: true }));

  const bytes = Buffer.from("malformed tree mode should not prove path\n");
  const blobOid = writeBlob(gitDirectory, bytes);
  const malformedTree = Buffer.concat([
    Buffer.from([0xb1, 0xb0, 0xb0, 0xb6, 0xb4, 0xb4, 0x20]),
    Buffer.from("victim.txt\0", "ascii"),
    Buffer.from(blobOid, "hex"),
  ]);
  const treeOid = await writeLooseObject(gitDirectory, "tree", malformedTree);
  const commitOid = writeCommit(gitDirectory, treeOid, "malformed tree");

  assert.throws(
    () => repoGit(gitDirectory, ["ls-tree", treeOid, "victim.txt"]),
    /malformed mode in tree entry/u,
  );
  await assert.rejects(
    readExactGitBlob(
      { gitDirectory, path: "victim.txt" },
      {
        blobOid,
        blobSha256: sha256(bytes),
        commitOid,
        treeOid,
      },
      { maxBytes: bytes.byteLength },
    ),
    rejectsWith("malformed_tree"),
  );
});

test("resolver ignores replace refs when opening a pinned commit", async (t) => {
  const state = await fixture(t);
  repoGit(state.gitDirectory, [
    "replace",
    state.commit,
    state.replacementCommit,
  ]);
  const result = await readExactGitBlob(binding(state), expected(state), {
    maxBytes: state.worker.byteLength,
  });
  assert.deepEqual(result.readBytes(), state.worker);
});
