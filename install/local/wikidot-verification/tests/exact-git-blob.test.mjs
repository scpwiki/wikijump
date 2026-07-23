import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import zlib from "node:zlib";
import test from "node:test";

import {
  createExactGitBlobReader,
  createExactGitTreeReader,
  ExactGitBlobError,
  readExactGitBlob,
  readExactGitTreeFiles,
} from "../src/exact-git-blob.mjs";

const COMMIT_ENV = Object.freeze({
  GIT_AUTHOR_DATE: "2000-01-01T00:00:00Z",
  GIT_AUTHOR_EMAIL: "oracle@example.invalid",
  GIT_AUTHOR_NAME: "Oracle",
  GIT_COMMITTER_DATE: "2000-01-01T00:00:00Z",
  GIT_COMMITTER_EMAIL: "oracle@example.invalid",
  GIT_COMMITTER_NAME: "Oracle",
});
const GIT_EXECUTABLE = "/usr/bin/git";
const RESOLVER_URL = new URL("../src/exact-git-blob.mjs", import.meta.url).href;
const TEST_GIT_ENVIRONMENT = Object.freeze({
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_NO_REPLACE_OBJECTS: "1",
  GIT_OPTIONAL_LOCKS: "0",
  GIT_PAGER: "cat",
  GIT_TERMINAL_PROMPT: "0",
  LANG: "C",
  LC_ALL: "C",
  PATH: "/usr/bin:/bin",
});

function git(args, { env = {}, input } = {}) {
  return execFileSync(GIT_EXECUTABLE, args, {
    cwd: "/",
    encoding: "buffer",
    env: { ...TEST_GIT_ENVIRONMENT, ...env },
    input,
    stdio: ["pipe", "pipe", "pipe"],
  });
}

function repoGit(gitDirectory, args, options) {
  return git([`--git-dir=${gitDirectory}`, ...args], options);
}

function trimmed(buffer) {
  return buffer.toString("ascii").trim();
}

function gitObjectOid(type, bytes) {
  return crypto
    .createHash("sha1")
    .update(`${type} ${bytes.byteLength}\0`)
    .update(bytes)
    .digest("hex");
}

async function writeLooseObject(gitDirectory, type, bytes) {
  const oid = gitObjectOid(type, bytes);
  const directory = path.join(gitDirectory, "objects", oid.slice(0, 2));
  await fs.mkdir(directory, { recursive: true });
  await fs.writeFile(
    path.join(directory, oid.slice(2)),
    zlib.deflateSync(
      Buffer.concat([Buffer.from(`${type} ${bytes.byteLength}\0`), bytes]),
    ),
  );
  return oid;
}

function writeBlob(gitDirectory, bytes) {
  return trimmed(
    repoGit(gitDirectory, ["hash-object", "-w", "--stdin"], {
      input: bytes,
    }),
  );
}

function writeTree(gitDirectory, entries) {
  const input = Buffer.from(
    entries
      .map(({ mode, oid, type, name }) => `${mode} ${type} ${oid}\t${name}\n`)
      .join(""),
  );
  return trimmed(repoGit(gitDirectory, ["mktree"], { input }));
}

function writeCommit(gitDirectory, tree, message) {
  return trimmed(
    repoGit(gitDirectory, ["commit-tree", tree, "-m", message], {
      env: COMMIT_ENV,
    }),
  );
}

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function expected(state, overrides = {}) {
  return {
    blobOid: state.workerOid,
    blobSha256: sha256(state.worker),
    commitOid: state.commit,
    treeOid: state.tree,
    ...overrides,
  };
}

async function fixture(t) {
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "exact-git-blob-"));
  const gitDirectory = path.join(parent, "repo.git");
  git(["init", "--bare", "--initial-branch=main", gitDirectory]);
  const worker = Buffer.from([0x00, 0xff, 0x0a, 0x61, 0x62, 0x63]);
  const workerOid = writeBlob(gitDirectory, worker);
  const link = Buffer.from("scripts/worker.py", "utf8");
  const linkOid = writeBlob(gitDirectory, link);
  const scripts = writeTree(gitDirectory, [
    { mode: "100755", name: "worker.py", oid: workerOid, type: "blob" },
  ]);
  const replacementBlob = writeBlob(gitDirectory, Buffer.from("replacement"));
  const replacementTree = writeTree(gitDirectory, [
    {
      mode: "100644",
      name: "replacement.py",
      oid: replacementBlob,
      type: "blob",
    },
  ]);
  const replacementCommit = writeCommit(
    gitDirectory,
    replacementTree,
    "second",
  );
  const tree = writeTree(gitDirectory, [
    { mode: "120000", name: "link", oid: linkOid, type: "blob" },
    { mode: "40000", name: "scripts", oid: scripts, type: "tree" },
    {
      mode: "160000",
      name: "submodule",
      oid: replacementCommit,
      type: "commit",
    },
  ]);
  const commit = writeCommit(gitDirectory, tree, "first");
  t.after(() => fs.rm(parent, { force: true, recursive: true }));
  return {
    commit,
    gitDirectory,
    link,
    linkOid,
    replacementCommit,
    tree,
    worker,
    workerOid,
  };
}

function binding(state, filePath = "scripts/worker.py") {
  return {
    gitDirectory: state.gitDirectory,
    path: filePath,
  };
}

function rejectsWith(code) {
  return (error) => error instanceof ExactGitBlobError && error.code === code;
}

function shellQuote(value) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

async function writeExecutable(filePath, source) {
  await fs.writeFile(filePath, source, { mode: 0o755 });
}

function runResolverChild(
  bindingInput,
  expectedInput,
  optionsInput,
  environment,
) {
  const program = [
    `import { readExactGitBlob } from ${JSON.stringify(RESOLVER_URL)};`,
    "const binding = JSON.parse(process.env.EXACT_GIT_BLOB_BINDING);",
    "const expected = JSON.parse(process.env.EXACT_GIT_BLOB_EXPECTED);",
    "const options = JSON.parse(process.env.EXACT_GIT_BLOB_OPTIONS);",
    "const result = await readExactGitBlob(binding, expected, options);",
    "process.stdout.write(JSON.stringify({ blobOid: result.blobOid, bytes: result.readBytes().toString('hex') }));",
  ].join("\n");
  return JSON.parse(
    execFileSync(process.execPath, ["--input-type=module", "--eval", program], {
      cwd: "/",
      encoding: "utf8",
      env: {
        ...environment,
        EXACT_GIT_BLOB_BINDING: JSON.stringify(bindingInput),
        EXACT_GIT_BLOB_EXPECTED: JSON.stringify(expectedInput),
        EXACT_GIT_BLOB_OPTIONS: JSON.stringify(optionsInput),
      },
      stdio: ["ignore", "pipe", "pipe"],
    }),
  );
}

function processExists(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}

async function processIsLive(pid) {
  try {
    const stat = await fs.readFile(`/proc/${pid}/stat`, "utf8");
    return !/\) Z /u.test(stat);
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

async function assertProcessTerminated(pid) {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    if (!(await processIsLive(pid))) return;
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  assert.fail(`process ${pid} remained live after resolver rejection`);
}

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
      } catch {
        try {
          process.kill(pid, "SIGKILL");
        } catch (error) {
          if (error?.code !== "ESRCH") throw error;
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
