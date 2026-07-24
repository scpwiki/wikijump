import assert from "node:assert/strict";
import {execFileSync} from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import zlib from "node:zlib";

import {ExactGitBlobError} from "../../src/exact-git-blob.mjs";

export const COMMIT_ENV = Object.freeze({
  GIT_AUTHOR_DATE: "2000-01-01T00:00:00Z",
  GIT_AUTHOR_EMAIL: "oracle@example.invalid",
  GIT_AUTHOR_NAME: "Oracle",
  GIT_COMMITTER_DATE: "2000-01-01T00:00:00Z",
  GIT_COMMITTER_EMAIL: "oracle@example.invalid",
  GIT_COMMITTER_NAME: "Oracle",
});
export const GIT_EXECUTABLE = "/usr/bin/git";
export const RESOLVER_URL = new URL("../../src/exact-git-blob.mjs", import.meta.url).href;
export const TEST_GIT_ENVIRONMENT = Object.freeze({
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

export function git(args, { env = {}, input } = {}) {
  return execFileSync(GIT_EXECUTABLE, args, {
    cwd: "/",
    encoding: "buffer",
    env: { ...TEST_GIT_ENVIRONMENT, ...env },
    input,
    stdio: ["pipe", "pipe", "pipe"],
  });
}

export function repoGit(gitDirectory, args, options) {
  return git([`--git-dir=${gitDirectory}`, ...args], options);
}

export function trimmed(buffer) {
  return buffer.toString("ascii").trim();
}

export function gitObjectOid(type, bytes) {
  return crypto
    .createHash("sha1")
    .update(`${type} ${bytes.byteLength}\0`)
    .update(bytes)
    .digest("hex");
}

export async function writeLooseObject(gitDirectory, type, bytes) {
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

export function writeBlob(gitDirectory, bytes) {
  return trimmed(
    repoGit(gitDirectory, ["hash-object", "-w", "--stdin"], {
      input: bytes,
    }),
  );
}

export function writeTree(gitDirectory, entries) {
  const input = Buffer.from(
    entries
      .map(({ mode, oid, type, name }) => `${mode} ${type} ${oid}\t${name}\n`)
      .join(""),
  );
  return trimmed(repoGit(gitDirectory, ["mktree"], { input }));
}

export function writeCommit(gitDirectory, tree, message) {
  return trimmed(
    repoGit(gitDirectory, ["commit-tree", tree, "-m", message], {
      env: COMMIT_ENV,
    }),
  );
}

export function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

export function expected(state, overrides = {}) {
  return {
    blobOid: state.workerOid,
    blobSha256: sha256(state.worker),
    commitOid: state.commit,
    treeOid: state.tree,
    ...overrides,
  };
}

export async function fixture(t) {
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

export function binding(state, filePath = "scripts/worker.py") {
  return {
    gitDirectory: state.gitDirectory,
    path: filePath,
  };
}

export function rejectsWith(code) {
  return (error) => error instanceof ExactGitBlobError && error.code === code;
}

export function shellQuote(value) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

export async function writeExecutable(filePath, source) {
  await fs.writeFile(filePath, source, { mode: 0o755 });
}

export function runResolverChild(
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

export function processExists(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}

export async function processIsLive(pid) {
  try {
    const stat = await fs.readFile(`/proc/${pid}/stat`, "utf8");
    return !/\) Z /u.test(stat);
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}

export async function assertProcessTerminated(pid) {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    if (!(await processIsLive(pid))) return;
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  assert.fail(`process ${pid} remained live after resolver rejection`);
}
