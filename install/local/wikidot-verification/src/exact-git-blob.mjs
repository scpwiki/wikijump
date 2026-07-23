import { spawn } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { types as utilTypes } from "node:util";

const BINDING_KEYS = Object.freeze(["gitDirectory", "path"]);
const TREE_BINDING_KEYS = Object.freeze(["gitDirectory"]);
const EXPECTED_KEYS = Object.freeze([
  "blobOid",
  "blobSha256",
  "commitOid",
  "treeOid",
]);
const OPTION_KEYS = Object.freeze(["maxBytes"]);
const TREE_EXPECTED_KEYS = Object.freeze(["commitOid", "treeOid"]);
const TREE_OPTION_KEYS = Object.freeze([
  "maxBytesPerFile",
  "maxFiles",
  "maxTotalBytes",
]);
const READER_CONFIGURATION_KEYS = Object.freeze(["gitExecutable"]);
const GIT_SHA1_RE = /^[0-9a-f]{40}$/u;
const MAX_BLOB_BYTES = 8 * 1024 * 1024;
const MAX_COMMIT_BYTES = 64 * 1024;
const MAX_METADATA_BYTES = 1024;
const MAX_PATH_DEPTH = 16;
const MAX_HOST_PATH_LENGTH = 4096;
const MAX_TREE_BYTES = 1024 * 1024;
const MAX_TREE_FILES = 256;
const MAX_TREE_TOTAL_BYTES = 64 * 1024 * 1024;
const GIT_TIMEOUT_MS = 5_000;
const GIT_KILL_GRACE_MS = 1_000;
const SAFE_EXECUTION_PATH = "/usr/bin:/bin";
const GIT_TREE_MODES = Object.freeze([
  Buffer.from("40000", "ascii"),
  Buffer.from("100644", "ascii"),
  Buffer.from("100755", "ascii"),
  Buffer.from("120000", "ascii"),
  Buffer.from("160000", "ascii"),
]);
const SHA256_RE = /^[0-9a-f]{64}$/u;
const TRUSTED_GIT_EXECUTABLE = "/usr/bin/git";

export class ExactGitBlobError extends Error {
  constructor(code) {
    super(`exact Git blob ${code}`);
    this.name = "ExactGitBlobError";
    this.code = code;
    // Error stacks would disclose the verifier's local source path across the trust boundary.
    this.stack = `${this.name}: ${this.message}`;
  }
}

function fail(code) {
  throw new ExactGitBlobError(code);
}

function dataObject(value, expectedKeys, code) {
  if (value === null || typeof value !== "object") fail(code);
  let prototype;
  let keys;
  let descriptors;
  try {
    if (Array.isArray(value) || utilTypes.isProxy(value)) fail(code);
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
    descriptors = keys.map((key) =>
      Reflect.getOwnPropertyDescriptor(value, key),
    );
  } catch {
    fail(code);
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some((key) => typeof key !== "string") ||
    keys.length !== expectedKeys.length ||
    expectedKeys.some((key) => !keys.includes(key))
  ) {
    fail(code);
  }
  const snapshot = {};
  for (const [index, key] of keys.entries()) {
    const descriptor = descriptors[index];
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      fail(code);
    }
    Object.defineProperty(snapshot, key, {
      enumerable: true,
      value: descriptor.value,
    });
  }
  return Object.freeze(snapshot);
}

function assertSha1(value, code = "invalid_expected_identity") {
  if (typeof value !== "string" || !GIT_SHA1_RE.test(value)) {
    fail(code);
  }
}

function normalizePath(value) {
  if (typeof value !== "string" || value.length === 0 || value.includes("\0")) {
    fail("invalid_git_path");
  }
  const segments = value.split("/");
  if (
    segments.length > MAX_PATH_DEPTH ||
    segments.some(
      (segment) =>
        !/^[A-Za-z0-9_][A-Za-z0-9._-]*$/u.test(segment) ||
        segment === "." ||
        segment === "..",
    )
  ) {
    fail("invalid_git_path");
  }
  return Object.freeze(segments);
}

function normalizeReaderConfiguration(value) {
  const input = dataObject(
    value,
    READER_CONFIGURATION_KEYS,
    "invalid_reader_configuration",
  );
  if (
    typeof input.gitExecutable !== "string" ||
    input.gitExecutable.length === 0 ||
    input.gitExecutable.length > MAX_HOST_PATH_LENGTH ||
    input.gitExecutable.includes("\0") ||
    !path.isAbsolute(input.gitExecutable)
  ) {
    fail("invalid_reader_configuration");
  }
  return input;
}

async function normalizeBinding(value, configuredGitExecutable) {
  const input = dataObject(value, BINDING_KEYS, "invalid_git_binding");
  const segments = normalizePath(input.path);
  if (
    typeof input.gitDirectory !== "string" ||
    input.gitDirectory.length === 0 ||
    input.gitDirectory.length > MAX_HOST_PATH_LENGTH ||
    input.gitDirectory.includes("\0") ||
    !path.isAbsolute(input.gitDirectory)
  ) {
    fail("invalid_git_binding");
  }
  let gitDirectory;
  let gitExecutable;
  try {
    [gitDirectory, gitExecutable] = await Promise.all([
      fs.realpath(input.gitDirectory),
      fs.realpath(configuredGitExecutable),
    ]);
    const [gitDirectoryStat, gitExecutableStat] = await Promise.all([
      fs.stat(gitDirectory),
      fs.stat(gitExecutable),
    ]);
    if (
      !gitDirectoryStat.isDirectory() ||
      !gitExecutableStat.isFile() ||
      (gitExecutableStat.mode & 0o111) === 0
    ) {
      fail("invalid_git_binding");
    }
  } catch (error) {
    if (error instanceof ExactGitBlobError) throw error;
    fail("invalid_git_binding");
  }
  return Object.freeze({
    gitDirectory,
    gitExecutable,
    segments,
  });
}

async function normalizeTreeBinding(value, configuredGitExecutable) {
  const input = dataObject(value, TREE_BINDING_KEYS, "invalid_tree_binding");
  if (
    typeof input.gitDirectory !== "string" ||
    input.gitDirectory.length === 0 ||
    input.gitDirectory.length > MAX_HOST_PATH_LENGTH ||
    input.gitDirectory.includes("\0") ||
    !path.isAbsolute(input.gitDirectory)
  ) {
    fail("invalid_tree_binding");
  }
  let gitDirectory;
  let gitExecutable;
  try {
    [gitDirectory, gitExecutable] = await Promise.all([
      fs.realpath(input.gitDirectory),
      fs.realpath(configuredGitExecutable),
    ]);
    const [gitDirectoryStat, gitExecutableStat] = await Promise.all([
      fs.stat(gitDirectory),
      fs.stat(gitExecutable),
    ]);
    if (
      !gitDirectoryStat.isDirectory() ||
      !gitExecutableStat.isFile() ||
      (gitExecutableStat.mode & 0o111) === 0
    ) {
      fail("invalid_tree_binding");
    }
  } catch (error) {
    if (error instanceof ExactGitBlobError) throw error;
    fail("invalid_tree_binding");
  }
  return Object.freeze({ gitDirectory, gitExecutable });
}

function normalizeExpected(value) {
  const input = dataObject(value, EXPECTED_KEYS, "invalid_expected_identity");
  for (const field of ["blobOid", "commitOid", "treeOid"]) {
    assertSha1(input[field]);
  }
  if (
    typeof input.blobSha256 !== "string" ||
    !SHA256_RE.test(input.blobSha256)
  ) {
    fail("invalid_expected_identity");
  }
  return input;
}

function normalizeOptions(value) {
  const input = dataObject(value, OPTION_KEYS, "invalid_read_options");
  if (
    !Number.isSafeInteger(input.maxBytes) ||
    input.maxBytes <= 0 ||
    input.maxBytes > MAX_BLOB_BYTES
  ) {
    fail("invalid_read_options");
  }
  return input;
}

function normalizeTreeExpected(value) {
  const input = dataObject(value, TREE_EXPECTED_KEYS, "invalid_tree_identity");
  for (const field of TREE_EXPECTED_KEYS) {
    assertSha1(input[field], "invalid_tree_identity");
  }
  return input;
}

function normalizeTreeOptions(value) {
  const input = dataObject(
    value,
    TREE_OPTION_KEYS,
    "invalid_tree_read_options",
  );
  for (const [field, maximum] of [
    ["maxBytesPerFile", MAX_BLOB_BYTES],
    ["maxFiles", MAX_TREE_FILES],
    ["maxTotalBytes", MAX_TREE_TOTAL_BYTES],
  ]) {
    if (
      !Number.isSafeInteger(input[field]) ||
      input[field] <= 0 ||
      input[field] > maximum
    ) {
      fail("invalid_tree_read_options");
    }
  }
  return input;
}

function normalizeTreePaths(value, maximum) {
  if (!Array.isArray(value) || utilTypes.isProxy(value)) {
    fail("invalid_tree_paths");
  }
  let keys;
  let length;
  try {
    keys = Reflect.ownKeys(value);
    length = Reflect.getOwnPropertyDescriptor(value, "length")?.value;
  } catch {
    fail("invalid_tree_paths");
  }
  if (
    !Number.isSafeInteger(length) ||
    length < 1 ||
    length > maximum ||
    keys.length !== length + 1
  ) {
    fail("invalid_tree_paths");
  }
  const seen = new Set();
  const paths = [];
  for (let index = 0; index < length; index += 1) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, String(index));
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      fail("invalid_tree_paths");
    }
    const segments = normalizePath(descriptor.value);
    const normalized = segments.join("/");
    if (seen.has(normalized)) fail("invalid_tree_paths");
    seen.add(normalized);
    paths.push(Object.freeze({ normalized, segments }));
  }
  if (
    keys.some(
      (key) =>
        key !== "length" &&
        (typeof key !== "string" ||
          !Number.isSafeInteger(Number(key)) ||
          String(Number(key)) !== key ||
          Number(key) < 0 ||
          Number(key) >= length),
    )
  ) {
    fail("invalid_tree_paths");
  }
  return Object.freeze(paths);
}

function gitEnvironment() {
  return {
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
  };
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

function runGit(gitExecutable, gitDirectory, args, maxBytes) {
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
        if (failureCode !== undefined) {
          reject(new ExactGitBlobError(failureCode));
        } else {
          resolve(Buffer.concat(chunks, stdoutBytes));
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
        gitExecutable,
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
  if (bytes.length < 2 || bytes.at(-1) !== 0x0a || bytes.includes(0x0d)) {
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

async function readGitObject(gitExecutable, gitDirectory, oid, type, maxBytes) {
  const actualType = exactLine(
    await runGit(
      gitExecutable,
      gitDirectory,
      ["cat-file", "-t", oid],
      MAX_METADATA_BYTES,
    ),
    "malformed_git_output",
  );
  if (actualType !== type) fail("git_object_type_mismatch");
  const sizeText = exactLine(
    await runGit(
      gitExecutable,
      gitDirectory,
      ["cat-file", "-s", oid],
      MAX_METADATA_BYTES,
    ),
    "malformed_git_output",
  );
  if (!/^(?:0|[1-9]\d*)$/u.test(sizeText)) fail("malformed_git_output");
  const size = Number(sizeText);
  if (!Number.isSafeInteger(size) || size > maxBytes) {
    fail(type === "blob" ? "blob_too_large" : "git_object_too_large");
  }
  const bytes = await runGit(
    gitExecutable,
    gitDirectory,
    ["cat-file", type, oid],
    size,
  );
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
  if (!GIT_SHA1_RE.test(oid)) fail("malformed_commit");
  return oid;
}

function gitTreeMode(bytes) {
  const mode = GIT_TREE_MODES.find((candidate) => candidate.equals(bytes));
  if (mode === undefined) fail("malformed_tree");
  return mode.toString("ascii");
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
    const mode = gitTreeMode(bytes.subarray(offset, space));
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
  return entries;
}

function treeEntry(entries, segment) {
  const expected = Buffer.from(segment, "ascii");
  const matches = entries.filter((entry) => entry.name.equals(expected));
  if (matches.length !== 1) fail("git_path_not_found");
  return matches[0];
}

async function assertSha1Repository(gitExecutable, gitDirectory) {
  const format = exactLine(
    await runGit(
      gitExecutable,
      gitDirectory,
      ["rev-parse", "--show-object-format"],
      MAX_METADATA_BYTES,
    ),
    "malformed_git_output",
  );
  if (format !== "sha1") fail("unsupported_git_object_format");
}

/**
 * Construct a reader from operator-owned host configuration. Do not derive this from serialized worker authority or a request.
 */
export function createExactGitBlobReader(configuration) {
  const trusted = normalizeReaderConfiguration(configuration);
  return async function readExactGitBlob(
    binding,
    expected,
    options = { maxBytes: MAX_BLOB_BYTES },
  ) {
    const claimed = normalizeExpected(expected);
    const limits = normalizeOptions(options);
    const local = await normalizeBinding(binding, trusted.gitExecutable);
    await assertSha1Repository(local.gitExecutable, local.gitDirectory);
    const commit = await readGitObject(
      local.gitExecutable,
      local.gitDirectory,
      claimed.commitOid,
      "commit",
      MAX_COMMIT_BYTES,
    );
    if (commitTreeOid(commit) !== claimed.treeOid) fail("commit_tree_mismatch");
    let tree = await readGitObject(
      local.gitExecutable,
      local.gitDirectory,
      claimed.treeOid,
      "tree",
      MAX_TREE_BYTES,
    );
    for (const [index, segment] of local.segments.entries()) {
      const entry = treeEntry(parseTree(tree), segment);
      if (index + 1 < local.segments.length) {
        if (entry.mode !== "40000") fail("git_path_not_tree");
        tree = await readGitObject(
          local.gitExecutable,
          local.gitDirectory,
          entry.oid,
          "tree",
          MAX_TREE_BYTES,
        );
      } else {
        if (!new Set(["100644", "100755"]).has(entry.mode)) {
          fail("git_path_not_regular_blob");
        }
        if (entry.oid !== claimed.blobOid) fail("tree_blob_mismatch");
      }
    }
    const blob = await readGitObject(
      local.gitExecutable,
      local.gitDirectory,
      claimed.blobOid,
      "blob",
      limits.maxBytes,
    );
    const sha256 = crypto.createHash("sha256").update(blob).digest("hex");
    if (sha256 !== claimed.blobSha256) fail("blob_sha256_mismatch");
    const immutableBytes = Buffer.from(blob);
    return Object.freeze({
      blobOid: claimed.blobOid,
      byteLength: immutableBytes.byteLength,
      readBytes() {
        // Buffers are mutable, so each capability use receives a detached copy.
        return Buffer.from(immutableBytes);
      },
      sha256,
    });
  };
}

/**
 * Resolve a fixed, caller-owned set of regular source files from one exact
 * commit/tree pair. The returned bytes are detached from both the Git object
 * database and caller-owned mutable input records.
 */
export function createExactGitTreeReader(configuration) {
  const trusted = normalizeReaderConfiguration(configuration);
  return async function readExactGitTreeFiles(
    binding,
    expected,
    paths,
    options = {
      maxBytesPerFile: MAX_BLOB_BYTES,
      maxFiles: MAX_TREE_FILES,
      maxTotalBytes: MAX_TREE_TOTAL_BYTES,
    },
  ) {
    const claimed = normalizeTreeExpected(expected);
    const limits = normalizeTreeOptions(options);
    const selectedPaths = normalizeTreePaths(paths, limits.maxFiles);
    const local = await normalizeTreeBinding(binding, trusted.gitExecutable);
    await assertSha1Repository(local.gitExecutable, local.gitDirectory);
    const commit = await readGitObject(
      local.gitExecutable,
      local.gitDirectory,
      claimed.commitOid,
      "commit",
      MAX_COMMIT_BYTES,
    );
    if (commitTreeOid(commit) !== claimed.treeOid) fail("commit_tree_mismatch");

    const trees = new Map();
    const readTree = (oid) => {
      let pending = trees.get(oid);
      if (pending === undefined) {
        pending = readGitObject(
          local.gitExecutable,
          local.gitDirectory,
          oid,
          "tree",
          MAX_TREE_BYTES,
        ).then(parseTree);
        trees.set(oid, pending);
      }
      return pending;
    };

    let totalBytes = 0;
    const files = [];
    for (const selected of selectedPaths) {
      let entries = await readTree(claimed.treeOid);
      let entry;
      for (const [index, segment] of selected.segments.entries()) {
        entry = treeEntry(entries, segment);
        if (index + 1 < selected.segments.length) {
          if (entry.mode !== "40000") fail("git_path_not_tree");
          entries = await readTree(entry.oid);
        }
      }
      if (
        entry === undefined ||
        !new Set(["100644", "100755"]).has(entry.mode)
      ) {
        fail("git_path_not_regular_blob");
      }
      const blob = await readGitObject(
        local.gitExecutable,
        local.gitDirectory,
        entry.oid,
        "blob",
        limits.maxBytesPerFile,
      );
      totalBytes += blob.byteLength;
      if (totalBytes > limits.maxTotalBytes) fail("git_tree_output_too_large");
      const immutableBytes = Buffer.from(blob);
      const sha256 = crypto
        .createHash("sha256")
        .update(immutableBytes)
        .digest("hex");
      files.push(
        Object.freeze({
          blobOid: entry.oid,
          byteLength: immutableBytes.byteLength,
          path: selected.normalized,
          sha256,
          readBytes() {
            return Buffer.from(immutableBytes);
          },
        }),
      );
    }
    return Object.freeze({
      commitOid: claimed.commitOid,
      files: Object.freeze(files),
      treeOid: claimed.treeOid,
    });
  };
}

export const readExactGitBlob = createExactGitBlobReader({
  gitExecutable: TRUSTED_GIT_EXECUTABLE,
});
export const readExactGitTreeFiles = createExactGitTreeReader({
  gitExecutable: TRUSTED_GIT_EXECUTABLE,
});
