#!/usr/bin/env node

// This bootstrap deliberately imports only Node built-ins. Its sole authority is
// to materialize the exact coordinator closure before any coordinator module can
// publish a receipt, observe credentials, start a worker, or make a request.
// Its trust root is the Node and Git executables, the supplied immutable Git
// identity, and same-UID host-process integrity; it is not a privilege boundary
// against a malicious process that already controls that trust root.
import { spawn } from "node:child_process";
import crypto from "node:crypto";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import { constants as osConstants } from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const BOOTSTRAP_PATH =
  "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition.mjs";
const MATERIALIZED_ENTRYPOINT_PATH =
  "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs";
const COORDINATOR_ENTRY_PATH =
  "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs";
const GIT_EXECUTABLE = "/usr/bin/git";
const GIT_SHA1_RE = /^[0-9a-f]{40}$/u;
const MAX_COMMIT_BYTES = 64 * 1024;
const MAX_FILE_BYTES = 2 * 1024 * 1024;
const MAX_METADATA_BYTES = 1024;
const MAX_TOTAL_BYTES = 32 * 1024 * 1024;
const MAX_TREE_BYTES = 1024 * 1024;
const PRIVATE_DIRECTORY_MODE = 0o700;
const PRIVATE_FILE_MODE = 0o400;
const SAFE_EXECUTION_PATH = "/usr/bin:/bin";
const GIT_TIMEOUT_MS = 5_000;
const GIT_KILL_GRACE_MS = 1_000;
const FORBIDDEN_NODE_ENVIRONMENT = Object.freeze([
  "NODE_OPTIONS",
  "NODE_PATH",
  "NODE_REPL_EXTERNAL_MODULE",
]);
const DESCRIPTOR_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_materialized_launch.v1";

export const WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS = Object.freeze(
  [
    BOOTSTRAP_PATH,
    MATERIALIZED_ENTRYPOINT_PATH,
    COORDINATOR_ENTRY_PATH,
    "install/local/wikidot-verification/src/atomic-no-replace.mjs",
    "install/local/wikidot-verification/src/canonical-json.mjs",
    "install/local/wikidot-verification/src/corpus-file-reader.mjs",
    "install/local/wikidot-verification/src/corpus-import-manifest.mjs",
    "install/local/wikidot-verification/src/exact-git-blob.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-attachment.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-attempt.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-completion-index.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-completion.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-inventory-source.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-inventory-validation.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-inventory.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-summary.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-campaign.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-completion.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-implementation.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-observation.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-work-target.mjs",
    "install/local/wikidot-verification/src/reference-object-descriptor.mjs",
    "install/local/wikidot-verification/src/reference-object-store.mjs",
    "install/local/wikidot-verification/src/resource-manifest.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-verdict.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-exact-data-record.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-installed-environment-manifest.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-private-capsule.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-python-environment.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-attestation.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-authority.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-client.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-protocol.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-session-capability.mjs",
  ].sort(),
);

export class WikidotXmlrpcBootstrapError extends Error {
  constructor(code) {
    super(`Wikidot XML-RPC bootstrap ${code}`);
    this.code = code;
    this.name = "WikidotXmlrpcBootstrapError";
    this.stack = `${this.name}: ${this.message}`;
  }
}

function fail(code) {
  throw new WikidotXmlrpcBootstrapError(code);
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

function assertSha1(value, code) {
  if (typeof value !== "string" || !GIT_SHA1_RE.test(value)) fail(code);
  return value;
}

function safeRelativePath(value) {
  if (typeof value !== "string" || value.length === 0 || value.includes("\0")) {
    fail("coordinator_path_invalid");
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
    fail("coordinator_path_invalid");
  }
  return Object.freeze(segments);
}

function parseBootstrapOptions(argv) {
  if (argv.length === 1 && ["--help", "-h"].includes(argv[0])) {
    return Object.freeze({ help: true });
  }
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
  for (const option of [
    "--capsule-parent",
    "--wikijump-commit",
    "--wikijump-git-dir",
    "--wikijump-tree",
  ]) {
    if (!Object.hasOwn(values, option)) fail("arguments_incomplete");
  }
  return Object.freeze({
    arguments: Object.freeze([...argv]),
    capsuleParent: assertAbsolutePath(
      values["--capsule-parent"],
      "capsule_parent_invalid",
    ),
    wikijumpCommit: assertSha1(
      values["--wikijump-commit"],
      "wikijump_commit_invalid",
    ),
    wikijumpGitDirectory: assertAbsolutePath(
      values["--wikijump-git-dir"],
      "wikijump_git_directory_invalid",
    ),
    wikijumpTree: assertSha1(
      values["--wikijump-tree"],
      "wikijump_tree_invalid",
    ),
  });
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
        if (failureCode === undefined)
          resolve(Buffer.concat(chunks, stdoutBytes));
        else reject(new WikidotXmlrpcBootstrapError(failureCode));
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
  if (!Number.isSafeInteger(size) || size > maxBytes)
    fail("git_object_too_large");
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
  return assertSha1(oid, "malformed_commit");
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

async function privateDirectory(value, code) {
  const raw = assertAbsolutePath(value, code);
  let resolved;
  let rawStat;
  let stat;
  try {
    rawStat = await fs.lstat(raw, { bigint: true });
    if (rawStat.isSymbolicLink()) fail(code);
    resolved = await fs.realpath(raw);
    stat = await fs.lstat(resolved, { bigint: true });
  } catch {
    fail(code);
  }
  if (
    !stat.isDirectory() ||
    stat.uid !== BigInt(process.geteuid()) ||
    (stat.mode & 0o777n) !== BigInt(PRIVATE_DIRECTORY_MODE)
  ) {
    fail(code);
  }
  return resolved;
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

function safeDestination(root, sourcePath) {
  const destination = path.resolve(root, ...safeRelativePath(sourcePath));
  if (destination === root || !destination.startsWith(`${root}${path.sep}`)) {
    fail("coordinator_path_invalid");
  }
  return destination;
}

async function makePrivateParents(root, sourcePath) {
  const segments = safeRelativePath(sourcePath);
  let current = root;
  for (const segment of segments.slice(0, -1)) {
    current = path.join(current, segment);
    try {
      await fs.mkdir(current, { mode: PRIVATE_DIRECTORY_MODE });
      await fs.chmod(current, PRIVATE_DIRECTORY_MODE);
    } catch (error) {
      if (error?.code !== "EEXIST") fail("coordinator_directory_create_failed");
    }
    const stat = await fs.lstat(current, { bigint: true }).catch(() => {
      fail("coordinator_directory_create_failed");
    });
    if (
      !stat.isDirectory() ||
      stat.uid !== BigInt(process.geteuid()) ||
      (stat.mode & 0o777n) !== BigInt(PRIVATE_DIRECTORY_MODE)
    ) {
      fail("coordinator_directory_create_failed");
    }
  }
}

async function writePrivateFile(destination, bytes) {
  let handle;
  try {
    handle = await fs.open(
      destination,
      fsConstants.O_WRONLY |
        fsConstants.O_CREAT |
        fsConstants.O_EXCL |
        (fsConstants.O_NOFOLLOW ?? 0),
      PRIVATE_FILE_MODE,
    );
    await handle.writeFile(bytes);
    await handle.chmod(PRIVATE_FILE_MODE);
    await handle.sync();
  } catch {
    fail("coordinator_file_write_failed");
  } finally {
    await handle?.close().catch(() => {});
  }
  let check;
  try {
    check = await fs.open(
      destination,
      fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0),
    );
    const before = await check.stat({ bigint: true });
    const actual = await check.readFile();
    const after = await check.stat({ bigint: true });
    if (
      !before.isFile() ||
      before.uid !== BigInt(process.geteuid()) ||
      (before.mode & 0o777n) !== BigInt(PRIVATE_FILE_MODE) ||
      before.dev !== after.dev ||
      before.ino !== after.ino ||
      before.size !== after.size ||
      !actual.equals(bytes)
    ) {
      fail("coordinator_file_write_failed");
    }
  } catch {
    fail("coordinator_file_write_failed");
  } finally {
    await check?.close().catch(() => {});
  }
}

async function readExactCoordinatorClosure(options) {
  const gitDirectory = await trustedGitDirectory(options.wikijumpGitDirectory);
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
    options.wikijumpCommit,
    "commit",
    MAX_COMMIT_BYTES,
  );
  if (commitTreeOid(commit) !== options.wikijumpTree) {
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
  const files = [];
  let totalBytes = 0;
  for (const sourcePath of WIKIDOT_XMLRPC_COORDINATOR_SOURCE_PATHS) {
    let entries = await readTree(options.wikijumpTree);
    let entry;
    const segments = safeRelativePath(sourcePath);
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
    files.push(
      Object.freeze({
        blobOid: entry.oid,
        byteLength: bytes.byteLength,
        bytes: Buffer.from(bytes),
        path: sourcePath,
        sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
      }),
    );
  }
  return Object.freeze({ files: Object.freeze(files), gitDirectory });
}

export async function materializeExactCoordinator(options) {
  const parent = await privateDirectory(
    options.capsuleParent,
    "capsule_parent_invalid",
  );
  const closure = await readExactCoordinatorClosure(options);
  let root;
  try {
    root = await fs.mkdtemp(path.join(parent, "wikijump-xmlrpc-coordinator-"));
    await fs.chmod(root, PRIVATE_DIRECTORY_MODE);
    root = await privateDirectory(root, "coordinator_directory_create_failed");
    for (const file of closure.files) {
      await makePrivateParents(root, file.path);
      await writePrivateFile(safeDestination(root, file.path), file.bytes);
    }
    return Object.freeze({
      entrypoint: safeDestination(root, MATERIALIZED_ENTRYPOINT_PATH),
      files: Object.freeze(
        closure.files.map((file) =>
          Object.freeze({
            blob_oid: file.blobOid,
            bytes: file.byteLength,
            path: file.path,
            sha256: file.sha256,
          }),
        ),
      ),
      root,
    });
  } catch (error) {
    if (root !== undefined) {
      await fs
        .rm(root, { force: true, maxRetries: 2, recursive: true })
        .catch(() => {});
    }
    if (error instanceof WikidotXmlrpcBootstrapError) throw error;
    fail("coordinator_materialization_failed");
  }
}

function scrubCredentials(environment = process.env) {
  delete environment.WIKIDOT_APP_NAME;
  delete environment.WIKIDOT_API_KEY;
}

function childEnvironment() {
  const environment = Object.create(null);
  for (const name of ["WIKIDOT_APP_NAME", "WIKIDOT_API_KEY"]) {
    const value = process.env[name];
    if (typeof value === "string") environment[name] = value;
  }
  environment.LANG = process.env.LANG ?? "C";
  environment.LC_ALL = process.env.LC_ALL ?? "C";
  environment.PATH = SAFE_EXECUTION_PATH;
  return environment;
}

function assertSafeNodeStartup() {
  if (
    FORBIDDEN_NODE_ENVIRONMENT.some(
      (name) =>
        typeof process.env[name] === "string" && process.env[name].length !== 0,
    ) ||
    process.execArgv.some((argument) =>
      [
        "--eval",
        "--experimental-loader",
        "--import",
        "--loader",
        "--require",
        "-r",
      ].some(
        (option) => argument === option || argument.startsWith(`${option}=`),
      ),
    )
  ) {
    fail("node_startup_invalid");
  }
}

function launchDescriptor(coordinator, options) {
  return Object.freeze({
    coordinator_path: COORDINATOR_ENTRY_PATH,
    entrypoint_path: MATERIALIZED_ENTRYPOINT_PATH,
    files: coordinator.files,
    materialization_root: coordinator.root,
    schema: DESCRIPTOR_SCHEMA,
    wikijump_commit: options.wikijumpCommit,
    wikijump_tree: options.wikijumpTree,
  });
}

function writeLaunchDescriptor(stream, descriptor) {
  return new Promise((resolve, reject) => {
    const bytes = Buffer.from(JSON.stringify(descriptor), "utf8");
    const failWrite = (error) => reject(error);
    stream.once("error", failWrite);
    stream.end(bytes, () => {
      stream.off("error", failWrite);
      resolve();
    });
  });
}

function signalExitCode(signal) {
  const number = osConstants.signals[signal];
  return Number.isSafeInteger(number) ? 128 + number : 1;
}

function signalCoordinator(child, signal) {
  if (child.pid === undefined) return;
  try {
    process.kill(-child.pid, signal);
    return;
  } catch {
    child.kill(signal);
  }
}

function waitForCoordinator(child) {
  return new Promise((resolve, reject) => {
    const handlers = new Map();
    const removeHandlers = () => {
      for (const [signal, handler] of handlers) process.off(signal, handler);
    };
    for (const signal of ["SIGHUP", "SIGINT", "SIGTERM"]) {
      const handler = () => {
        if (child.exitCode !== null || child.signalCode !== null) return;
        signalCoordinator(child, signal);
      };
      handlers.set(signal, handler);
      process.on(signal, handler);
    }
    child.once("error", (error) => {
      removeHandlers();
      reject(error);
    });
    child.once("close", (code, signal) => {
      removeHandlers();
      resolve(signal === null ? (code ?? 1) : signalExitCode(signal));
    });
  });
}

export async function runBootstrap(argv) {
  let coordinator = null;
  try {
    assertSafeNodeStartup();
    const options = parseBootstrapOptions(argv);
    if (options.help === true) {
      process.stdout.write(
        "Usage: run-wikidot-xmlrpc-acquisition.mjs requires the coordinator identity and acquisition options.\n",
      );
      return 0;
    }
    coordinator = await materializeExactCoordinator(options);
    const environment = childEnvironment();
    scrubCredentials();
    const child = spawn(
      process.execPath,
      [coordinator.entrypoint, ...options.arguments],
      {
        cwd: "/",
        detached: true,
        env: environment,
        shell: false,
        stdio: ["inherit", "inherit", "inherit", "pipe", "pipe"],
      },
    );
    const completion = waitForCoordinator(child);
    try {
      const descriptor = launchDescriptor(coordinator, options);
      await Promise.all([
        writeLaunchDescriptor(child.stdio[3], descriptor),
        writeLaunchDescriptor(child.stdio[4], descriptor),
      ]);
    } catch (error) {
      signalCoordinator(child, "SIGTERM");
      await completion.catch(() => {});
      throw error;
    }
    return await completion;
  } finally {
    scrubCredentials();
    if (coordinator !== null) {
      await fs
        .rm(coordinator.root, { force: true, maxRetries: 2, recursive: true })
        .catch(() => {});
    }
  }
}

export async function main(argv) {
  return runBootstrap(argv);
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
        "XML-RPC acquisition bootstrap failed before launch\n",
      );
      process.exitCode = 1;
    });
}
