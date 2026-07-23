import { spawn } from "node:child_process";
import crypto from "node:crypto";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { types as utilTypes } from "node:util";

import { readExactGitTreeFiles } from "./exact-git-blob.mjs";
import { stableStringify } from "./canonical-json.mjs";
import { buildWikidotXmlrpcInstalledEnvironmentManifest } from "./wikidot-xmlrpc-installed-environment-manifest.mjs";

const CAPSULE_OPTIONS = Object.freeze(["capsuleParent", "runtime", "source"]);
const PRIVATE_DIRECTORY_MODE = 0o700;
const PRIVATE_EXECUTABLE_MODE = 0o500;
const PRIVATE_FILE_MODE = 0o400;
const PROBE_MAX_BYTES = 64 * 1024;
const PROBE_TIMEOUT_MS = 30_000;
const RUNTIME_OPTIONS = Object.freeze([
  "pythonExecutablePath",
  "pythonVersion",
  "root",
  "venvConfigPath",
]);
const SOURCE_OPTIONS = Object.freeze(["commitOid", "gitDirectory", "treeOid"]);
const SOURCE_PROBE_MODULES = Object.freeze([
  "httpx",
  "scp_wiki_wikidot.bounded_xmlrpc",
  "scp_wiki_wikidot.client",
  "scp_wiki_wikidot.config",
  "scp_wiki_wikidot.transport",
  "tenacity",
  "wikidot",
  "wikidot_paths",
]);
const WORKER_PATH = "scripts/wikidot_xmlrpc_capture_worker.py";

// This is an explicit application import closure. The worker itself inserts
// its materialized application root before importing this package; no mutable
// source checkout is visible to the spawned interpreter.
export const WIKIDOT_XMLRPC_WORKER_SOURCE_PATHS = Object.freeze([
  "pyproject.toml",
  "scripts/wikidot_paths.py",
  WORKER_PATH,
  "scp_wiki_wikidot/__init__.py",
  "scp_wiki_wikidot/bounded_xmlrpc.py",
  "scp_wiki_wikidot/client.py",
  "scp_wiki_wikidot/config.py",
  "scp_wiki_wikidot/corpus_synchronization_engine.py",
  "scp_wiki_wikidot/errors.py",
  "scp_wiki_wikidot/identity.py",
  "scp_wiki_wikidot/models.py",
  "scp_wiki_wikidot/posts.py",
  "scp_wiki_wikidot/resolve.py",
  "scp_wiki_wikidot/storage.py",
  "scp_wiki_wikidot/transport.py",
  "scp_wiki_wikidot/wikidot_py.py",
  "uv.lock",
]);

const PRIVATE_CAPSULES = new WeakMap();
const PREPARED_RUNTIMES = new WeakMap();
const PREPARED_SOURCES = new WeakMap();

export class WikidotXmlrpcPrivateCapsuleError extends Error {
  constructor(code) {
    super(`Wikidot XML-RPC private capsule ${code}`);
    this.code = code;
    this.name = "WikidotXmlrpcPrivateCapsuleError";
    this.stack = `${this.name}: ${this.message}`;
  }
}

function fail(code) {
  throw new WikidotXmlrpcPrivateCapsuleError(code);
}

function dataObject(value, expectedKeys, code) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    fail(code);
  }
  let keys;
  let prototype;
  let descriptors;
  try {
    keys = Reflect.ownKeys(value);
    prototype = Reflect.getPrototypeOf(value);
    descriptors = keys.map((key) =>
      Reflect.getOwnPropertyDescriptor(value, key),
    );
  } catch {
    fail(code);
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some((key) => typeof key !== "string") ||
    stableStringify([...keys].sort()) !== stableStringify(expectedKeys)
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

function assertAbsoluteHostPath(value, code) {
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

function assertRelativePath(value, code) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > 4096 ||
    value.startsWith("/") ||
    value.includes("\\") ||
    value.includes("\0")
  ) {
    fail(code);
  }
  const segments = value.split("/");
  if (
    segments.some(
      (segment) =>
        segment.length === 0 ||
        segment === "." ||
        segment === ".." ||
        /[\u0000-\u001f\u007f]/u.test(segment),
    )
  ) {
    fail(code);
  }
  return value;
}

function sameSnapshot(left, right) {
  return (
    left.dev === right.dev &&
    left.ino === right.ino &&
    left.size === right.size &&
    left.mtimeNs === right.mtimeNs &&
    left.mode === right.mode &&
    left.uid === right.uid &&
    left.gid === right.gid
  );
}

function utf8Compare(left, right) {
  return Buffer.compare(Buffer.from(left, "utf8"), Buffer.from(right, "utf8"));
}

async function privateDirectory(value, code) {
  const raw = assertAbsoluteHostPath(value, code);
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

async function ensurePrivateDirectory(value, code) {
  try {
    await fs.mkdir(value, { mode: PRIVATE_DIRECTORY_MODE });
    await fs.chmod(value, PRIVATE_DIRECTORY_MODE);
  } catch (error) {
    if (error?.code === "EEXIST") return privateDirectory(value, code);
    fail(code);
  }
  return privateDirectory(value, code);
}

function safeJoin(root, relative, code) {
  const normalized = assertRelativePath(relative, code);
  const candidate = path.resolve(root, ...normalized.split("/"));
  if (candidate === root || !candidate.startsWith(`${root}${path.sep}`)) {
    fail(code);
  }
  return candidate;
}

async function readRegularFile(filePath, expected = undefined) {
  let handle;
  try {
    handle = await fs.open(
      filePath,
      fsConstants.O_RDONLY | fsConstants.O_NOFOLLOW | fsConstants.O_NONBLOCK,
    );
  } catch {
    fail("runtime_file_open_failed");
  }
  try {
    const before = await handle.stat({ bigint: true });
    if (
      !before.isFile() ||
      before.uid !== BigInt(process.geteuid()) ||
      before.size > 1024n * 1024n * 1024n
    ) {
      fail("runtime_file_invalid");
    }
    const executable = (before.mode & 0o111n) !== 0n;
    if (
      expected !== undefined &&
      (before.size !== BigInt(expected.bytes) ||
        executable !== expected.executable)
    ) {
      fail("runtime_file_identity_mismatch");
    }
    const size = Number(before.size);
    const buffer = Buffer.allocUnsafe(Math.min(1024 * 1024, Math.max(size, 1)));
    const chunks = [];
    const hash = crypto.createHash("sha256");
    let offset = 0;
    while (offset < size) {
      const { bytesRead } = await handle.read(
        buffer,
        0,
        Math.min(buffer.byteLength, size - offset),
        offset,
      );
      if (bytesRead === 0) fail("runtime_file_changed");
      const bytes = Buffer.from(buffer.subarray(0, bytesRead));
      chunks.push(bytes);
      hash.update(bytes);
      offset += bytesRead;
    }
    const after = await handle.stat({ bigint: true });
    if (offset !== size || !sameSnapshot(before, after)) {
      fail("runtime_file_changed");
    }
    const sha256 = hash.digest("hex");
    if (expected !== undefined && sha256 !== expected.sha256) {
      fail("runtime_file_identity_mismatch");
    }
    return Object.freeze({
      bytes: size,
      contents: Buffer.concat(chunks, size),
      executable,
      sha256,
    });
  } finally {
    await handle.close().catch(() => {});
  }
}

async function listRuntimeFiles(root) {
  const files = [];
  async function walk(directory, prefix) {
    let entries;
    try {
      entries = await fs.readdir(directory, { withFileTypes: true });
    } catch {
      fail("runtime_directory_read_failed");
    }
    entries.sort((left, right) => utf8Compare(left.name, right.name));
    for (const entry of entries) {
      const relative =
        prefix.length === 0 ? entry.name : `${prefix}/${entry.name}`;
      const candidate = safeJoin(root, relative, "runtime_path_invalid");
      let stat;
      try {
        stat = await fs.lstat(candidate, { bigint: true });
      } catch {
        fail("runtime_path_invalid");
      }
      if (stat.isSymbolicLink()) fail("runtime_symlink_rejected");
      if (stat.isDirectory()) {
        if (
          stat.uid !== BigInt(process.geteuid()) ||
          (stat.mode & 0o777n) !== BigInt(PRIVATE_DIRECTORY_MODE)
        ) {
          fail("runtime_directory_invalid");
        }
        await walk(candidate, relative);
      } else if (stat.isFile()) {
        const file = await readRegularFile(candidate);
        files.push(
          Object.freeze({
            bytes: file.bytes,
            executable: file.executable,
            path: relative,
            sha256: file.sha256,
          }),
        );
      } else {
        fail("runtime_special_file_rejected");
      }
    }
  }
  await walk(root, "");
  return Object.freeze(files);
}

function sourceIdentity(files, sourcePath) {
  const file = files.find((candidate) => candidate.path === sourcePath);
  if (file === undefined) fail("source_closure_incomplete");
  return Object.freeze({
    blobOid: file.blobOid,
    bytes: file.byteLength,
    path: sourcePath,
    sha256: file.sha256,
  });
}

function sourceDescriptor(tree) {
  const files = Object.freeze(
    tree.files.map((file) =>
      Object.freeze({
        blobOid: file.blobOid,
        bytes: file.byteLength,
        path: file.path,
        sha256: file.sha256,
      }),
    ),
  );
  const recipe = sourceIdentity(files, "pyproject.toml");
  const lock = sourceIdentity(files, "uv.lock");
  const worker = sourceIdentity(files, WORKER_PATH);
  return Object.freeze({
    dependencyLockBlobOid: lock.blobOid,
    dependencyLockFileSha256: lock.sha256,
    dependencyRecipeBlobOid: recipe.blobOid,
    dependencyRecipeSha256: recipe.sha256,
    files,
    workerBlobOid: worker.blobOid,
    workerFileSha256: worker.sha256,
    workerRepositoryCommit: tree.commitOid,
    workerRepositoryTree: tree.treeOid,
  });
}

function assertPreparedRuntime(value) {
  const privateValue = PREPARED_RUNTIMES.get(value);
  if (privateValue === undefined) fail("runtime_capability_invalid");
  return privateValue;
}

function assertPreparedSource(value) {
  const privateValue = PREPARED_SOURCES.get(value);
  if (privateValue === undefined) fail("source_capability_invalid");
  return privateValue;
}

async function makeParentDirectories(root, relative) {
  const segments = assertRelativePath(relative, "capsule_path_invalid").split(
    "/",
  );
  segments.pop();
  let current = root;
  for (const segment of segments) {
    current = path.join(current, segment);
    await ensurePrivateDirectory(current, "capsule_directory_create_failed");
  }
}

async function writePrivateFile(filePath, bytes, executable) {
  let handle;
  try {
    handle = await fs.open(
      filePath,
      fsConstants.O_WRONLY |
        fsConstants.O_CREAT |
        fsConstants.O_EXCL |
        fsConstants.O_NOFOLLOW,
      executable ? PRIVATE_EXECUTABLE_MODE : PRIVATE_FILE_MODE,
    );
    let offset = 0;
    while (offset < bytes.byteLength) {
      const { bytesWritten } = await handle.write(bytes, offset);
      if (bytesWritten === 0) fail("capsule_file_write_failed");
      offset += bytesWritten;
    }
    await handle.sync();
  } catch (error) {
    if (error instanceof WikidotXmlrpcPrivateCapsuleError) throw error;
    fail("capsule_file_write_failed");
  } finally {
    await handle?.close().catch(() => {});
  }
}

async function copyRuntimeFile(sourceRoot, destinationRoot, file) {
  const sourcePath = safeJoin(sourceRoot, file.path, "runtime_path_invalid");
  const destinationPath = safeJoin(
    destinationRoot,
    file.path,
    "capsule_path_invalid",
  );
  await makeParentDirectories(destinationRoot, file.path);
  const contents = await readRegularFile(sourcePath, file);
  await writePrivateFile(destinationPath, contents.contents, file.executable);
  const copied = await readRegularFile(destinationPath, {
    bytes: file.bytes,
    executable: file.executable,
    sha256: file.sha256,
  });
  if (!copied.contents.equals(contents.contents))
    fail("capsule_file_identity_mismatch");
}

async function copySourceFile(destinationRoot, file) {
  const destinationPath = safeJoin(
    destinationRoot,
    file.path,
    "capsule_path_invalid",
  );
  await makeParentDirectories(destinationRoot, file.path);
  const bytes = file.readBytes();
  const actual = crypto.createHash("sha256").update(bytes).digest("hex");
  if (actual !== file.sha256 || bytes.byteLength !== file.byteLength) {
    fail("source_file_identity_mismatch");
  }
  await writePrivateFile(destinationPath, bytes, false);
  const copied = await readRegularFile(destinationPath, {
    bytes: file.byteLength,
    executable: false,
    sha256: file.sha256,
  });
  if (!copied.contents.equals(bytes)) fail("source_file_identity_mismatch");
}

function isInside(root, value) {
  if (typeof value !== "string" || !path.isAbsolute(value)) return false;
  const resolved = path.resolve(value);
  return resolved === root || resolved.startsWith(`${root}${path.sep}`);
}

function probeProgram() {
  return [
    "import importlib.util, json, sys, sysconfig",
    "from pathlib import Path",
    "worker = Path(sys.argv[1]).resolve()",
    "spec = importlib.util.spec_from_file_location('wikidot_xmlrpc_capsule_probe', worker)",
    "if spec is None or spec.loader is None: raise RuntimeError('worker spec')",
    "module = importlib.util.module_from_spec(spec)",
    "sys.modules[spec.name] = module",
    "spec.loader.exec_module(module)",
    `names = ${JSON.stringify(SOURCE_PROBE_MODULES)}`,
    "modules = {name: str(Path(importlib.import_module(name).__file__).resolve()) for name in names}",
    "record = {'base_prefix': sys.base_prefix, 'executable': sys.executable, 'modules': modules, 'prefix': sys.prefix, 'stdlib': sysconfig.get_path('stdlib'), 'sys_path': sys.path, 'version': list(sys.version_info[:3])}",
    "print(json.dumps(record, sort_keys=True, separators=(',', ':')), flush=True)",
  ].join("\n");
}

async function runProbe(
  pythonExecutable,
  workerEntrypoint,
  capsuleRoot,
  version,
) {
  const output = await new Promise((resolve, reject) => {
    let child;
    let settled = false;
    let bytes = 0;
    const chunks = [];
    const finish = (callback) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      callback();
    };
    const terminate = () => {
      if (child?.pid === undefined) return;
      try {
        process.kill(-child.pid, "SIGKILL");
      } catch {
        child.kill("SIGKILL");
      }
    };
    const timer = setTimeout(() => {
      terminate();
      finish(() =>
        reject(new WikidotXmlrpcPrivateCapsuleError("runtime_probe_failed")),
      );
    }, PROBE_TIMEOUT_MS);
    try {
      child = spawn(
        pythonExecutable,
        ["-I", "-B", "-c", probeProgram(), workerEntrypoint],
        {
          cwd: "/",
          detached: true,
          env: { LANG: "C.UTF-8", LC_ALL: "C.UTF-8" },
          shell: false,
          stdio: ["ignore", "pipe", "ignore"],
        },
      );
    } catch {
      finish(() =>
        reject(new WikidotXmlrpcPrivateCapsuleError("runtime_probe_failed")),
      );
      return;
    }
    child.once("error", () =>
      finish(() =>
        reject(new WikidotXmlrpcPrivateCapsuleError("runtime_probe_failed")),
      ),
    );
    child.stdout.on("data", (chunk) => {
      if (settled) return;
      bytes += chunk.byteLength;
      if (bytes > PROBE_MAX_BYTES) {
        terminate();
        finish(() =>
          reject(new WikidotXmlrpcPrivateCapsuleError("runtime_probe_failed")),
        );
      } else {
        chunks.push(Buffer.from(chunk));
      }
    });
    child.once("close", (code, signal) => {
      if (settled) return;
      if (code !== 0 || signal !== null) {
        finish(() =>
          reject(new WikidotXmlrpcPrivateCapsuleError("runtime_probe_failed")),
        );
      } else {
        finish(() => resolve(Buffer.concat(chunks, bytes)));
      }
    });
  });
  let record;
  try {
    record = JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(output),
    );
  } catch {
    fail("runtime_probe_failed");
  }
  const expectedVersion = version.split(".").map((part) => Number(part));
  const input = dataObject(
    record,
    [
      "base_prefix",
      "executable",
      "modules",
      "prefix",
      "stdlib",
      "sys_path",
      "version",
    ],
    "runtime_probe_failed",
  );
  if (
    !Array.isArray(input.sys_path) ||
    !Array.isArray(input.version) ||
    input.version.length !== 3 ||
    !input.version.every(Number.isSafeInteger) ||
    stableStringify(input.version) !== stableStringify(expectedVersion) ||
    ![input.base_prefix, input.executable, input.prefix, input.stdlib].every(
      (value) => isInside(capsuleRoot, value),
    ) ||
    input.sys_path.length === 0 ||
    !input.sys_path.every((value) => isInside(capsuleRoot, value))
  ) {
    fail("runtime_probe_failed");
  }
  const modules = dataObject(
    input.modules,
    SOURCE_PROBE_MODULES,
    "runtime_probe_failed",
  );
  if (!Object.values(modules).every((value) => isInside(capsuleRoot, value))) {
    fail("runtime_probe_failed");
  }
}

function createPrivateCapsule(root, pythonExecutable, workerEntrypoint) {
  let child = null;
  let closed = false;
  let disposed = false;
  const capsule = Object.freeze({
    async dispose() {
      if (disposed) return;
      if (child !== null && !closed) fail("worker_still_running");
      try {
        await fs.rm(root, { force: false, maxRetries: 2, recursive: true });
      } catch {
        fail("capsule_cleanup_failed");
      }
      disposed = true;
    },
    spawn() {
      if (disposed || child !== null) fail("capsule_spawn_invalid");
      try {
        child = spawn(pythonExecutable, ["-I", "-B", workerEntrypoint], {
          cwd: "/",
          detached: true,
          env: { LANG: "C.UTF-8", LC_ALL: "C.UTF-8" },
          shell: false,
          stdio: ["pipe", "pipe", "ignore"],
        });
      } catch {
        fail("capsule_spawn_failed");
      }
      child.once("close", () => {
        closed = true;
      });
      return Object.freeze({
        child,
        signalProcessGroup(signal) {
          if (child.pid === undefined) return;
          try {
            process.kill(-child.pid, signal);
          } catch (error) {
            if (error.code !== "ESRCH") throw error;
          }
        },
      });
    },
  });
  PRIVATE_CAPSULES.set(capsule, true);
  return capsule;
}

export async function prepareWikidotXmlrpcRuntime(value) {
  const input = dataObject(value, RUNTIME_OPTIONS, "runtime_options_invalid");
  const root = await privateDirectory(input.root, "runtime_root_invalid");
  const pythonExecutablePath = assertRelativePath(
    input.pythonExecutablePath,
    "runtime_role_invalid",
  );
  const venvConfigPath = assertRelativePath(
    input.venvConfigPath,
    "runtime_role_invalid",
  );
  const files = await listRuntimeFiles(root);
  let manifest;
  try {
    manifest = buildWikidotXmlrpcInstalledEnvironmentManifest({
      files,
      pythonExecutablePath,
      pythonImplementation: "cpython",
      pythonVersion: input.pythonVersion,
      venvConfigPath,
    });
  } catch {
    fail("runtime_manifest_invalid");
  }
  const runtime = Object.freeze({ manifest });
  PREPARED_RUNTIMES.set(runtime, Object.freeze({ root }));
  return runtime;
}

export async function prepareWikidotXmlrpcWorkerSource(value) {
  const input = dataObject(value, SOURCE_OPTIONS, "source_options_invalid");
  let tree;
  try {
    tree = await readExactGitTreeFiles(
      { gitDirectory: input.gitDirectory },
      { commitOid: input.commitOid, treeOid: input.treeOid },
      WIKIDOT_XMLRPC_WORKER_SOURCE_PATHS,
      {
        maxBytesPerFile: 8 * 1024 * 1024,
        maxFiles: WIKIDOT_XMLRPC_WORKER_SOURCE_PATHS.length,
        maxTotalBytes: 64 * 1024 * 1024,
      },
    );
  } catch {
    fail("source_materialization_invalid");
  }
  const source = sourceDescriptor(tree);
  PREPARED_SOURCES.set(source, tree);
  return source;
}

export async function materializeWikidotXmlrpcPrivateCapsule(value) {
  const input = dataObject(value, CAPSULE_OPTIONS, "capsule_options_invalid");
  const runtime = assertPreparedRuntime(input.runtime);
  const sourceTree = assertPreparedSource(input.source);
  const parent = await privateDirectory(
    input.capsuleParent,
    "capsule_parent_invalid",
  );
  let root;
  try {
    root = await fs.mkdtemp(path.join(parent, "wikidot-xmlrpc-"));
    await fs.chmod(root, PRIVATE_DIRECTORY_MODE);
    root = await privateDirectory(root, "capsule_directory_create_failed");
    const applicationRoot = await ensurePrivateDirectory(
      path.join(root, "application"),
      "capsule_directory_create_failed",
    );
    const runtimeRoot = await ensurePrivateDirectory(
      path.join(root, "runtime"),
      "capsule_directory_create_failed",
    );
    for (const file of input.runtime.manifest.files) {
      await copyRuntimeFile(runtime.root, runtimeRoot, file);
    }
    for (const file of sourceTree.files) {
      await copySourceFile(applicationRoot, file);
    }
    const workerEntrypoint = safeJoin(
      applicationRoot,
      WORKER_PATH,
      "capsule_path_invalid",
    );
    const pythonExecutable = safeJoin(
      runtimeRoot,
      input.runtime.manifest.python_executable_path,
      "capsule_path_invalid",
    );
    await runProbe(
      pythonExecutable,
      workerEntrypoint,
      root,
      input.runtime.manifest.python_version,
    );
    return createPrivateCapsule(root, pythonExecutable, workerEntrypoint);
  } catch (error) {
    await fs
      .rm(root, { force: true, maxRetries: 2, recursive: true })
      .catch(() => {});
    if (error instanceof WikidotXmlrpcPrivateCapsuleError) throw error;
    fail("capsule_materialization_failed");
  }
}

export function isWikidotXmlrpcPrivateCapsule(value) {
  return PRIVATE_CAPSULES.has(value);
}
