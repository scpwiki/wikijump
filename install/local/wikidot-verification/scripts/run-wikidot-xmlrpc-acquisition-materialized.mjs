#!/usr/bin/env node

// This entrypoint imports only Node built-ins until it has validated the
// private descriptor received from the bootstrap over file descriptor 3.
// It independently resolves that descriptor from the claimed exact Git tree;
// the same-UID bootstrap and Git identity remain the explicit trust root.
import crypto from "node:crypto";
import { spawn } from "node:child_process";
import { constants as fsConstants, createReadStream } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";

const BOOTSTRAP_PATH =
  "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition.mjs";
const ENTRYPOINT_PATH =
  "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs";
const COORDINATOR_PATH =
  "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs";
const DESCRIPTOR_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_materialized_launch.v1";
const GIT_EXECUTABLE = "/usr/bin/git";
const GIT_KILL_GRACE_MS = 1_000;
const GIT_TIMEOUT_MS = 5_000;
const MAX_DESCRIPTOR_BYTES = 64 * 1024;
const MAX_COMMIT_BYTES = 64 * 1024;
const MAX_FILE_BYTES = 2 * 1024 * 1024;
const MAX_METADATA_BYTES = 1024;
const MAX_TOTAL_BYTES = 32 * 1024 * 1024;
const MAX_TREE_BYTES = 1024 * 1024;
const PRIVATE_DIRECTORY_MODE = 0o700;
const PRIVATE_FILE_MODE = 0o400;
const SAFE_EXECUTION_PATH = "/usr/bin:/bin";
const SHA1_RE = /^[0-9a-f]{40}$/u;
const SHA256_RE = /^[0-9a-f]{64}$/u;

function fail(code) {
  throw new Error(`Wikidot XML-RPC materialized entrypoint ${code}`);
}

function assertAbsolutePath(value, code) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > 4096 ||
    value.includes("\0") ||
    !path.isAbsolute(value)
  ) {
    fail(code);
  }
  return value;
}

function gitEnvironment() {
  return Object.freeze({
    GIT_CONFIG_GLOBAL: "/dev/null",
    GIT_CONFIG_NOSYSTEM: "1",
    GIT_NO_LAZY_FETCH: "1",
    GIT_NO_REPLACE_OBJECTS: "1",
    GIT_OPTIONAL_LOCKS: "0",
    GIT_PAGER: "cat",
    GIT_TERMINAL_PROMPT: "0",
    LANG: "C",
    LC_ALL: "C",
    PATH: SAFE_EXECUTION_PATH,
  });
}

function killProcessGroup(child) {
  if (child.pid === undefined) return;
  try {
    process.kill(-child.pid, "SIGKILL");
    return;
  } catch {
    child.kill("SIGKILL");
  }
}

function runGit(gitDirectory, args, maxBytes) {
  return new Promise((resolve, reject) => {
    let child;
    let closeGraceTimer;
    let failureCode;
    let settled = false;
    let stderrBytes = 0;
    let stdoutBytes = 0;
    const chunks = [];
    const settle = (callback) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      clearTimeout(closeGraceTimer);
      callback();
    };
    const finish = () => {
      settle(() => {
        if (failureCode === undefined) {
          resolve(Buffer.concat(chunks, stdoutBytes));
        } else {
          reject(
            new Error(`Wikidot XML-RPC materialized entrypoint ${failureCode}`),
          );
        }
      });
    };
    const terminate = (code) => {
      if (failureCode !== undefined) return;
      failureCode = code;
      if (child === undefined) {
        finish();
        return;
      }
      killProcessGroup(child);
      closeGraceTimer = setTimeout(finish, GIT_KILL_GRACE_MS);
    };
    const timer = setTimeout(() => terminate("git_timeout"), GIT_TIMEOUT_MS);
    try {
      child = spawn(
        GIT_EXECUTABLE,
        ["--no-replace-objects", `--git-dir=${gitDirectory}`, ...args],
        {
          cwd: "/",
          detached: true,
          env: gitEnvironment(),
          shell: false,
          stdio: ["ignore", "pipe", "pipe"],
        },
      );
    } catch {
      terminate("git_failed");
      return;
    }
    child.once("error", () => terminate("git_failed"));
    if (child.stdout === null || child.stderr === null) {
      terminate("git_failed");
      return;
    }
    child.stdout.on("data", (chunk) => {
      if (settled || failureCode !== undefined) return;
      stdoutBytes += chunk.byteLength;
      if (stdoutBytes > maxBytes) terminate("git_output_too_large");
      else chunks.push(chunk);
    });
    child.stderr.on("data", (chunk) => {
      if (settled || failureCode !== undefined) return;
      stderrBytes += chunk.byteLength;
      if (stderrBytes > MAX_METADATA_BYTES) terminate("git_failed");
    });
    child.once("close", (code, signal) => {
      if (settled) return;
      if (failureCode === undefined && (code !== 0 || signal !== null)) {
        failureCode = "git_failed";
      }
      finish();
    });
  });
}

function exactLine(bytes, code) {
  if (bytes.byteLength < 2 || bytes.at(-1) !== 0x0a || bytes.includes(0x0d)) {
    fail(code);
  }
  const line = bytes.subarray(0, -1).toString("ascii");
  if (!/^[\x20-\x7e]+$/u.test(line)) fail(code);
  return line;
}

function gitObjectHash(type, bytes) {
  return crypto
    .createHash("sha1")
    .update(`${type} ${bytes.byteLength}\0`)
    .update(bytes)
    .digest("hex");
}

async function readGitObject(gitDirectory, oid, type, maxBytes) {
  const actualType = exactLine(
    await runGit(gitDirectory, ["cat-file", "-t", oid], MAX_METADATA_BYTES),
    "malformed_git_output",
  );
  if (actualType !== type) fail("git_object_type_mismatch");
  const sizeText = exactLine(
    await runGit(gitDirectory, ["cat-file", "-s", oid], MAX_METADATA_BYTES),
    "malformed_git_output",
  );
  if (!/^(?:0|[1-9]\d*)$/u.test(sizeText)) fail("malformed_git_output");
  const size = Number(sizeText);
  if (!Number.isSafeInteger(size) || size > maxBytes) {
    fail("git_object_too_large");
  }
  const bytes = await runGit(gitDirectory, ["cat-file", type, oid], size);
  if (bytes.byteLength !== size || gitObjectHash(type, bytes) !== oid) {
    fail("git_object_identity_mismatch");
  }
  return bytes;
}

function commitTreeOid(bytes) {
  const newline = bytes.indexOf(0x0a);
  if (newline !== 45 || bytes.subarray(0, 5).toString("ascii") !== "tree ") {
    fail("malformed_commit");
  }
  const oid = bytes.subarray(5, newline).toString("ascii");
  if (!SHA1_RE.test(oid)) fail("malformed_commit");
  return oid;
}

function parseTree(bytes) {
  const entries = [];
  const names = new Set();
  for (let offset = 0; offset < bytes.byteLength;) {
    const space = bytes.indexOf(0x20, offset);
    const nul = bytes.indexOf(0x00, space + 1);
    if (space <= offset || nul <= space + 1 || nul + 21 > bytes.byteLength) {
      fail("malformed_tree");
    }
    const mode = bytes.subarray(offset, space).toString("ascii");
    const name = bytes.subarray(space + 1, nul);
    const nameKey = name.toString("hex");
    if (
      !/^(?:40000|100644|100755|120000|160000)$/u.test(mode) ||
      names.has(nameKey)
    ) {
      fail("malformed_tree");
    }
    names.add(nameKey);
    entries.push(
      Object.freeze({
        mode,
        name,
        oid: bytes.subarray(nul + 1, nul + 21).toString("hex"),
      }),
    );
    offset = nul + 21;
  }
  return Object.freeze(entries);
}

function treeEntry(entries, segment) {
  const expected = Buffer.from(segment, "ascii");
  const matches = entries.filter((entry) => entry.name.equals(expected));
  if (matches.length !== 1) fail("git_path_not_found");
  return matches[0];
}

async function trustedGitDirectory(value) {
  const raw = assertAbsolutePath(value, "wikijump_git_directory_invalid");
  let resolved;
  let stat;
  try {
    resolved = await fs.realpath(raw);
    stat = await fs.stat(resolved, { bigint: true });
  } catch {
    fail("wikijump_git_directory_invalid");
  }
  if (!stat.isDirectory()) fail("wikijump_git_directory_invalid");
  return resolved;
}

function validateExactDataRecord(value, keys, code) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    fail(code);
  }
  const actual = Object.keys(value).sort();
  if (
    actual.length !== keys.length ||
    actual.some((key, index) => key !== keys[index])
  ) {
    fail(code);
  }
  return value;
}

function safeRelativePath(value, code) {
  if (typeof value !== "string" || value.length === 0 || value.includes("\0")) {
    fail(code);
  }
  const segments = value.split("/");
  if (
    segments.some(
      (segment) =>
        !/^[A-Za-z0-9_][A-Za-z0-9._-]*$/u.test(segment) ||
        segment === "." ||
        segment === "..",
    )
  ) {
    fail(code);
  }
  return segments;
}

function safeDestination(root, relative, code) {
  const destination = path.resolve(root, ...safeRelativePath(relative, code));
  if (destination === root || !destination.startsWith(`${root}${path.sep}`)) {
    fail(code);
  }
  return destination;
}

function normalizeDescriptor(value) {
  const input = validateExactDataRecord(
    value,
    [
      "coordinator_path",
      "entrypoint_path",
      "files",
      "materialization_root",
      "schema",
      "wikijump_commit",
      "wikijump_tree",
    ],
    "descriptor_invalid",
  );
  if (
    input.schema !== DESCRIPTOR_SCHEMA ||
    input.entrypoint_path !== ENTRYPOINT_PATH ||
    input.coordinator_path !== COORDINATOR_PATH ||
    typeof input.materialization_root !== "string" ||
    !path.isAbsolute(input.materialization_root) ||
    input.materialization_root.includes("\0") ||
    !SHA1_RE.test(input.wikijump_commit) ||
    !SHA1_RE.test(input.wikijump_tree) ||
    !Array.isArray(input.files) ||
    input.files.length < 3 ||
    input.files.length > 128
  ) {
    fail("descriptor_invalid");
  }
  const files = [];
  let previous = null;
  for (const value of input.files) {
    const file = validateExactDataRecord(
      value,
      ["blob_oid", "bytes", "path", "sha256"],
      "descriptor_invalid",
    );
    const normalized = safeRelativePath(file.path, "descriptor_invalid").join(
      "/",
    );
    if (
      normalized !== file.path ||
      (previous !== null && previous >= normalized) ||
      !SHA1_RE.test(file.blob_oid) ||
      !SHA256_RE.test(file.sha256) ||
      !Number.isSafeInteger(file.bytes) ||
      file.bytes < 0 ||
      file.bytes > 2 * 1024 * 1024
    ) {
      fail("descriptor_invalid");
    }
    previous = normalized;
    files.push(Object.freeze({ ...file }));
  }
  const paths = new Set(files.map((file) => file.path));
  for (const required of [BOOTSTRAP_PATH, ENTRYPOINT_PATH, COORDINATOR_PATH]) {
    if (!paths.has(required)) fail("descriptor_incomplete");
  }
  return Object.freeze({ ...input, files: Object.freeze(files) });
}

async function readDescriptor() {
  let bytes;
  try {
    bytes = await readBoundedDescriptor(3);
  } catch (error) {
    if (error?.message === "descriptor_too_large") {
      fail("descriptor_invalid");
    }
    fail("descriptor_unavailable");
  }
  if (bytes.byteLength === 0 || bytes.byteLength > MAX_DESCRIPTOR_BYTES) {
    fail("descriptor_invalid");
  }
  let value;
  try {
    value = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
  } catch {
    fail("descriptor_invalid");
  }
  return normalizeDescriptor(value);
}

function readBoundedDescriptor(fileDescriptor) {
  return new Promise((resolve, reject) => {
    let stream;
    let settled = false;
    let totalBytes = 0;
    const chunks = [];
    const settle = (callback) => {
      if (settled) return;
      settled = true;
      callback();
    };
    try {
      stream = createReadStream(null, {
        autoClose: false,
        emitClose: false,
        fd: fileDescriptor,
        highWaterMark: 4096,
      });
    } catch (error) {
      reject(error);
      return;
    }
    stream.on("data", (chunk) => {
      if (settled) return;
      totalBytes += chunk.byteLength;
      if (totalBytes > MAX_DESCRIPTOR_BYTES) {
        settle(() => {
          stream.destroy();
          reject(new Error("descriptor_too_large"));
        });
        return;
      }
      chunks.push(chunk);
    });
    stream.once("error", (error) => settle(() => reject(error)));
    stream.once("end", () =>
      settle(() => resolve(Buffer.concat(chunks, totalBytes))),
    );
  });
}

async function verifyPrivateRoot(root) {
  let raw;
  let resolved;
  let stat;
  try {
    raw = await fs.lstat(root, { bigint: true });
    if (raw.isSymbolicLink()) fail("root_invalid");
    resolved = await fs.realpath(root);
    stat = await fs.lstat(resolved, { bigint: true });
  } catch {
    fail("root_invalid");
  }
  if (
    resolved !== root ||
    !stat.isDirectory() ||
    stat.uid !== BigInt(process.geteuid()) ||
    (stat.mode & 0o777n) !== BigInt(PRIVATE_DIRECTORY_MODE)
  ) {
    fail("root_invalid");
  }
}

async function verifyPrivateParents(root, relative) {
  let current = root;
  for (const segment of safeRelativePath(relative, "file_invalid").slice(
    0,
    -1,
  )) {
    current = path.join(current, segment);
    let stat;
    try {
      stat = await fs.lstat(current, { bigint: true });
    } catch {
      fail("file_invalid");
    }
    if (
      !stat.isDirectory() ||
      stat.uid !== BigInt(process.geteuid()) ||
      (stat.mode & 0o777n) !== BigInt(PRIVATE_DIRECTORY_MODE)
    ) {
      fail("file_invalid");
    }
  }
}

async function verifyFile(root, file) {
  await verifyPrivateParents(root, file.path);
  const destination = safeDestination(root, file.path, "file_invalid");
  let handle;
  try {
    handle = await fs.open(
      destination,
      fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0),
    );
    const before = await handle.stat({ bigint: true });
    const bytes = await handle.readFile();
    const after = await handle.stat({ bigint: true });
    if (
      !before.isFile() ||
      before.uid !== BigInt(process.geteuid()) ||
      (before.mode & 0o777n) !== BigInt(PRIVATE_FILE_MODE) ||
      before.dev !== after.dev ||
      before.ino !== after.ino ||
      before.size !== after.size ||
      bytes.byteLength !== file.bytes ||
      crypto.createHash("sha256").update(bytes).digest("hex") !== file.sha256
    ) {
      fail("file_invalid");
    }
  } catch {
    fail("file_invalid");
  } finally {
    await handle?.close().catch(() => {});
  }
}

function inputIdentity(argv) {
  const values = Object.create(null);
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (
      typeof option !== "string" ||
      !option.startsWith("--") ||
      typeof value !== "string" ||
      value.startsWith("--") ||
      Object.hasOwn(values, option)
    ) {
      fail("arguments_invalid");
    }
    values[option] = value;
  }
  return Object.freeze({
    commit: values["--wikijump-commit"],
    gitDirectory: values["--wikijump-git-dir"],
    tree: values["--wikijump-tree"],
  });
}

async function verifyDescriptorAgainstGit(descriptor, identity) {
  const gitDirectory = await trustedGitDirectory(identity.gitDirectory);
  const objectFormat = exactLine(
    await runGit(
      gitDirectory,
      ["rev-parse", "--show-object-format"],
      MAX_METADATA_BYTES,
    ),
    "malformed_git_output",
  );
  if (objectFormat !== "sha1") fail("unsupported_git_object_format");
  const commit = await readGitObject(
    gitDirectory,
    descriptor.wikijump_commit,
    "commit",
    MAX_COMMIT_BYTES,
  );
  if (commitTreeOid(commit) !== descriptor.wikijump_tree) {
    fail("commit_tree_mismatch");
  }
  const trees = new Map();
  const readTree = (oid) => {
    let pending = trees.get(oid);
    if (pending === undefined) {
      pending = readGitObject(gitDirectory, oid, "tree", MAX_TREE_BYTES).then(
        parseTree,
      );
      trees.set(oid, pending);
    }
    return pending;
  };
  let totalBytes = 0;
  for (const file of descriptor.files) {
    let entries = await readTree(descriptor.wikijump_tree);
    let entry;
    const segments = safeRelativePath(file.path, "git_path_invalid");
    for (const [index, segment] of segments.entries()) {
      entry = treeEntry(entries, segment);
      if (index + 1 < segments.length) {
        if (entry.mode !== "40000") fail("git_path_not_tree");
        entries = await readTree(entry.oid);
      }
    }
    if (entry === undefined || !new Set(["100644", "100755"]).has(entry.mode)) {
      fail("git_path_not_regular_blob");
    }
    const bytes = await readGitObject(
      gitDirectory,
      entry.oid,
      "blob",
      MAX_FILE_BYTES,
    );
    totalBytes += bytes.byteLength;
    if (totalBytes > MAX_TOTAL_BYTES) fail("coordinator_too_large");
    if (
      entry.oid !== file.blob_oid ||
      bytes.byteLength !== file.bytes ||
      crypto.createHash("sha256").update(bytes).digest("hex") !== file.sha256
    ) {
      fail("descriptor_git_mismatch");
    }
  }
}

export async function verifyMaterializedDescriptor(
  value,
  argv,
  executablePath = process.argv[1],
) {
  const descriptor = normalizeDescriptor(value);
  await verifyPrivateRoot(descriptor.materialization_root);
  if (
    path.resolve(executablePath ?? "") !==
    safeDestination(
      descriptor.materialization_root,
      ENTRYPOINT_PATH,
      "entrypoint_invalid",
    )
  ) {
    fail("entrypoint_invalid");
  }
  const identity = inputIdentity(argv);
  if (
    identity.commit !== descriptor.wikijump_commit ||
    identity.tree !== descriptor.wikijump_tree
  ) {
    fail("identity_invalid");
  }
  await verifyDescriptorAgainstGit(descriptor, identity);
  for (const file of descriptor.files) {
    await verifyFile(descriptor.materialization_root, file);
  }
  const bootstrap = await import(
    pathToFileURL(
      safeDestination(
        descriptor.materialization_root,
        BOOTSTRAP_PATH,
        "bootstrap_invalid",
      ),
    ).href
  );
  const expected = bootstrap.WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS;
  if (
    !Array.isArray(expected) ||
    expected.length !== descriptor.files.length ||
    expected.some((value, index) => value !== descriptor.files[index].path)
  ) {
    fail("closure_invalid");
  }
  return descriptor;
}

export async function main(argv) {
  const descriptor = await readDescriptor();
  await verifyMaterializedDescriptor(descriptor, argv);
  const coordinator = await import(
    pathToFileURL(
      safeDestination(
        descriptor.materialization_root,
        COORDINATOR_PATH,
        "coordinator_invalid",
      ),
    ).href
  );
  return coordinator.main(argv);
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  main(process.argv.slice(2))
    .then((code) => {
      process.exitCode = code;
    })
    .catch(() => {
      process.stderr.write(
        "XML-RPC materialized entrypoint failed before launch\n",
      );
      process.exitCode = 1;
    });
}
