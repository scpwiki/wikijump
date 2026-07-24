import crypto from "node:crypto";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

import { publishBytesNoReplaceAt } from "./atomic-no-replace.mjs";
import { assertDescriptorTraversalSupport } from "./corpus-file-reader.mjs";
import {
  assertReferenceObjectBytes as assertBytes,
  assertReferenceObjectSha256 as assertSha256,
  REFERENCE_OBJECT_STORE_DESCRIPTOR_BYTES as STORE_DESCRIPTOR_BYTES,
  REFERENCE_OBJECT_STORE_DESCRIPTOR_MISMATCH as STORE_DESCRIPTOR_MISMATCH,
  validateReferenceObject,
} from "./reference-object-descriptor.mjs";

export {
  referenceObjectRelativePath,
  referenceObjectStoreDescriptorBytes,
  validateReferenceObject,
} from "./reference-object-descriptor.mjs";

const DIRECTORY_FLAGS =
  fsConstants.O_RDONLY | fsConstants.O_DIRECTORY | fsConstants.O_NOFOLLOW;
const FILE_FLAGS =
  fsConstants.O_RDONLY | fsConstants.O_NONBLOCK | fsConstants.O_NOFOLLOW;
const REFERENCE_OBJECT_STORES = new WeakSet();
function assertComponent(name) {
  if (
    typeof name !== "string" ||
    name.length === 0 ||
    name === "." ||
    name === ".." ||
    name.includes("/") ||
    name.includes("\\") ||
    name.includes("\0")
  ) {
    throw new Error("reference store path component is unsafe");
  }
}

function toBuffer(value) {
  if (!(value instanceof Uint8Array)) {
    throw new Error("object bytes must be a Uint8Array");
  }
  return Buffer.from(value);
}

function procPath(directoryHandle, name) {
  return `/proc/self/fd/${directoryHandle.fd}/${name}`;
}

function sameIdentity(left, right) {
  return left.dev === right.dev && left.ino === right.ino;
}

function sameSnapshot(left, right) {
  // Publishing uses a temporary hard link to the final inode. Removing that
  // link legitimately changes ctime while leaving the immutable object intact.
  return (
    sameIdentity(left, right) &&
    left.size === right.size &&
    left.mtimeNs === right.mtimeNs &&
    left.mode === right.mode &&
    left.uid === right.uid &&
    left.gid === right.gid
  );
}

function assertOwnedMode(stat, expectedMode, label) {
  if (stat.uid !== BigInt(process.geteuid())) {
    throw new Error(`${label} must be owned by the current user`);
  }
  if ((stat.mode & 0o777n) !== BigInt(expectedMode)) {
    throw new Error(`${label} must have mode ${expectedMode.toString(8)}`);
  }
}

async function assertDirectoryHandle(handle, label) {
  const stat = await handle.stat({ bigint: true });
  if (!stat.isDirectory()) throw new Error(`${label} must be a directory`);
  assertOwnedMode(stat, 0o700, label);
  return stat;
}

async function openDirectoryAt(parentHandle, name, { create, label }) {
  assertComponent(name);
  const componentPath = procPath(parentHandle, name);
  let handle;
  try {
    handle = await fs.open(componentPath, DIRECTORY_FLAGS);
  } catch (error) {
    if (error.code !== "ENOENT" || !create) {
      if (["ELOOP", "EMLINK", "ENOTDIR"].includes(error.code)) {
        throw new Error(`${label} must not be a symbolic link`, {
          cause: error,
        });
      }
      throw error;
    }
    try {
      await fs.mkdir(componentPath, { mode: 0o700 });
    } catch (mkdirError) {
      if (mkdirError.code !== "EEXIST") throw mkdirError;
    }
    await parentHandle.sync();
    handle = await fs.open(componentPath, DIRECTORY_FLAGS);
  }
  try {
    await assertDirectoryHandle(handle, label);
    return handle;
  } catch (error) {
    await handle.close().catch(() => {});
    throw error;
  }
}

async function assertDirectoryBinding(
  parentHandle,
  name,
  expectedHandle,
  label,
) {
  const actualHandle = await openDirectoryAt(parentHandle, name, {
    create: false,
    label,
  });
  try {
    const [actual, expected] = await Promise.all([
      actualHandle.stat({ bigint: true }),
      expectedHandle.stat({ bigint: true }),
    ]);
    if (!sameIdentity(actual, expected)) throw new Error(`${label} changed`);
  } finally {
    await actualHandle.close();
  }
}

async function readAndHashFileAt(
  directoryHandle,
  name,
  {
    allowMissing = false,
    collect,
    expectedBytes,
    expectedMode,
    maxBytes,
    sizeMismatchMessage,
  },
) {
  assertComponent(name);
  const exactSize = expectedBytes !== undefined;
  if (exactSize === (maxBytes !== undefined)) {
    throw new Error("exactly one of expectedBytes or maxBytes is required");
  }
  const sizeLimit = exactSize ? expectedBytes : maxBytes;
  assertBytes(sizeLimit, exactSize ? "expectedBytes" : "maxBytes");
  const filePath = procPath(directoryHandle, name);
  let handle;
  try {
    handle = await fs.open(filePath, FILE_FLAGS);
  } catch (error) {
    if (allowMissing && error.code === "ENOENT") return null;
    throw error;
  }
  try {
    const before = await handle.stat({ bigint: true });
    if (!before.isFile()) throw new Error(`${name} must be a regular file`);
    assertOwnedMode(before, expectedMode, name);
    const invalidSize = exactSize
      ? before.size !== BigInt(sizeLimit)
      : before.size > BigInt(sizeLimit);
    if (invalidSize) {
      throw new Error(sizeMismatchMessage);
    }
    const size = Number(before.size);
    const buffer = Buffer.allocUnsafe(Math.min(1024 * 1024, Math.max(size, 1)));
    const chunks = collect ? [] : null;
    const hash = crypto.createHash("sha256");
    let offset = 0;
    while (offset < size) {
      const { bytesRead } = await handle.read(
        buffer,
        0,
        Math.min(buffer.length, size - offset),
        offset,
      );
      if (bytesRead === 0) break;
      const bytes = buffer.subarray(0, bytesRead);
      hash.update(bytes);
      if (chunks !== null) chunks.push(Buffer.from(bytes));
      offset += bytesRead;
    }
    const after = await handle.stat({ bigint: true });
    if (offset !== size || !sameSnapshot(before, after)) {
      throw new Error(`${name} changed while it was being verified`);
    }
    assertOwnedMode(after, expectedMode, name);
    const namedHandle = await fs.open(filePath, FILE_FLAGS);
    try {
      const named = await namedHandle.stat({ bigint: true });
      if (!named.isFile() || !sameSnapshot(after, named)) {
        throw new Error(`${name} changed while it was being verified`);
      }
      assertOwnedMode(named, expectedMode, name);
    } finally {
      await namedHandle.close();
    }
    return {
      bytes: offset,
      contents: chunks === null ? null : Buffer.concat(chunks, offset),
      sha256: hash.digest("hex"),
    };
  } finally {
    await handle.close();
  }
}

async function verifyDescriptor(rootHandle) {
  const actual = await readAndHashFileAt(rootHandle, "store.json", {
    collect: true,
    expectedBytes: STORE_DESCRIPTOR_BYTES.byteLength,
    expectedMode: 0o400,
    sizeMismatchMessage: STORE_DESCRIPTOR_MISMATCH,
  });
  if (!actual.contents.equals(STORE_DESCRIPTOR_BYTES)) {
    throw new Error(STORE_DESCRIPTOR_MISMATCH);
  }
}

async function openRoot(root, create) {
  await assertDescriptorTraversalSupport();
  const absoluteRoot = path.resolve(root);
  const parsed = path.parse(absoluteRoot);
  if (absoluteRoot === parsed.root)
    throw new Error("store root must not be a filesystem root");
  const parentPath = await fs.realpath(path.dirname(absoluteRoot));
  const parentHandle = await fs.open(parentPath, DIRECTORY_FLAGS);
  try {
    const rootHandle = await openDirectoryAt(
      parentHandle,
      path.basename(absoluteRoot),
      {
        create,
        label: "reference object store root",
      },
    );
    return { parentHandle, rootHandle, rootName: path.basename(absoluteRoot) };
  } catch (error) {
    await parentHandle.close();
    throw error;
  }
}

export function isReferenceObjectStore(value) {
  return REFERENCE_OBJECT_STORES.has(value);
}

async function assertStoreHandleBindings(handles) {
  await assertDirectoryBinding(
    handles.parent,
    handles.rootName,
    handles.root,
    "reference object store root",
  );
  await assertDirectoryBinding(
    handles.root,
    "objects",
    handles.objects,
    "objects",
  );
  await assertDirectoryBinding(
    handles.objects,
    "sha256",
    handles.sha256,
    "sha256",
  );
}

class ReferenceObjectStore {
  #handles;

  constructor(handles) {
    this.#handles = handles;
  }

  async #assertBindings() {
    const handles = this.#handles;
    if (handles === null) throw new Error("reference object store is closed");
    await assertStoreHandleBindings(handles);
    return handles;
  }

  async putBytes(value, { expectedBytes, expectedSha256 } = {}) {
    const bytes = toBuffer(value);
    const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
    const object = { algorithm: "sha256", bytes: bytes.byteLength, sha256 };
    if (expectedBytes !== undefined) {
      assertBytes(expectedBytes, "expectedBytes");
      if (expectedBytes !== object.bytes)
        throw new Error("object byte length mismatch");
    }
    if (expectedSha256 !== undefined) {
      assertSha256(expectedSha256, "expectedSha256");
      if (expectedSha256 !== object.sha256)
        throw new Error("object SHA-256 mismatch");
    }
    const handles = await this.#assertBindings();
    const prefixName = sha256.slice(0, 2);
    const prefix = await openDirectoryAt(handles.sha256, prefixName, {
      create: true,
      label: `object prefix ${prefixName}`,
    });
    try {
      const disposition = await publishBytesNoReplaceAt(prefix, sha256, bytes, {
        mode: 0o400,
      });
      const corruptionMessage = `reference object ${object.sha256} is corrupt`;
      const actual = await readAndHashFileAt(prefix, sha256, {
        collect: false,
        expectedBytes: object.bytes,
        expectedMode: 0o400,
        sizeMismatchMessage: corruptionMessage,
      });
      if (actual.bytes !== object.bytes || actual.sha256 !== object.sha256) {
        throw new Error(corruptionMessage);
      }
      if (disposition === "exists") await prefix.sync();
      await assertDirectoryBinding(
        handles.sha256,
        prefixName,
        prefix,
        `object prefix ${prefixName}`,
      );
      await this.#assertBindings();
      return { disposition, object: Object.freeze(object) };
    } finally {
      await prefix.close();
    }
  }

  async verifyObject(value) {
    const object = validateReferenceObject(value);
    const handles = await this.#assertBindings();
    const prefixName = object.sha256.slice(0, 2);
    const prefix = await openDirectoryAt(handles.sha256, prefixName, {
      create: false,
      label: `object prefix ${prefixName}`,
    });
    try {
      const corruptionMessage = `reference object ${object.sha256} is corrupt`;
      const actual = await readAndHashFileAt(prefix, object.sha256, {
        collect: false,
        expectedBytes: object.bytes,
        expectedMode: 0o400,
        sizeMismatchMessage: corruptionMessage,
      });
      if (actual.bytes !== object.bytes || actual.sha256 !== object.sha256) {
        throw new Error(corruptionMessage);
      }
      await assertDirectoryBinding(
        handles.sha256,
        prefixName,
        prefix,
        `object prefix ${prefixName}`,
      );
      await this.#assertBindings();
      return object;
    } finally {
      await prefix.close();
    }
  }

  async readObject(value, { maxBytes } = {}) {
    const object = validateReferenceObject(value);
    assertBytes(maxBytes, "maxBytes");
    if (object.bytes > maxBytes) {
      throw new Error(`reference object exceeds maxBytes ${maxBytes}`);
    }
    const handles = await this.#assertBindings();
    const prefixName = object.sha256.slice(0, 2);
    const prefix = await openDirectoryAt(handles.sha256, prefixName, {
      create: false,
      label: `object prefix ${prefixName}`,
    });
    try {
      const corruptionMessage = `reference object ${object.sha256} is corrupt`;
      const actual = await readAndHashFileAt(prefix, object.sha256, {
        collect: true,
        expectedBytes: object.bytes,
        expectedMode: 0o400,
        sizeMismatchMessage: corruptionMessage,
      });
      if (actual.sha256 !== object.sha256) throw new Error(corruptionMessage);
      await assertDirectoryBinding(
        handles.sha256,
        prefixName,
        prefix,
        `object prefix ${prefixName}`,
      );
      await this.#assertBindings();
      return actual.contents;
    } finally {
      await prefix.close();
    }
  }

  async close() {
    const handles = this.#handles;
    this.#handles = null;
    if (handles === null) return;
    for (const handle of [
      handles.sha256,
      handles.objects,
      handles.root,
      handles.parent,
    ]) {
      await handle.close();
    }
  }
}

async function prepareStore(root, create) {
  const { parentHandle, rootHandle, rootName } = await openRoot(root, create);
  let objectsHandle;
  let sha256Handle;
  try {
    objectsHandle = await openDirectoryAt(rootHandle, "objects", {
      create,
      label: "objects",
    });
    sha256Handle = await openDirectoryAt(objectsHandle, "sha256", {
      create,
      label: "sha256",
    });
    if (create) {
      await publishBytesNoReplaceAt(
        rootHandle,
        "store.json",
        STORE_DESCRIPTOR_BYTES,
        {
          mode: 0o400,
        },
      );
    }
    await verifyDescriptor(rootHandle);
    await rootHandle.sync();
    const handles = {
      objects: objectsHandle,
      parent: parentHandle,
      root: rootHandle,
      rootName,
      sha256: sha256Handle,
    };
    const store = new ReferenceObjectStore(handles);
    REFERENCE_OBJECT_STORES.add(store);
    const { registerReferenceAcquisitionCompletionStore } =
      await import("./reference-acquisition-completion.mjs");
    registerReferenceAcquisitionCompletionStore(store, {
      assertDirectoryBinding,
      assertStoreBindings: () => assertStoreHandleBindings(handles),
      openDirectoryAt,
      readAndHashFileAt,
      root: handles.root,
    });
    return store;
  } catch (error) {
    await sha256Handle?.close().catch(() => {});
    await objectsHandle?.close().catch(() => {});
    await rootHandle.close().catch(() => {});
    await parentHandle.close().catch(() => {});
    throw error;
  }
}

export async function initializeReferenceObjectStore(root) {
  return await prepareStore(root, true);
}

export async function openReferenceObjectStore(root) {
  return await prepareStore(root, false);
}
