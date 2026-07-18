import crypto from "node:crypto";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

import { assertDescriptorTraversalSupport } from "./corpus-file-reader.mjs";

function toBuffer(contents) {
  if (typeof contents === "string") return Buffer.from(contents, "utf8");
  if (contents instanceof Uint8Array) return Buffer.from(contents);
  throw new Error("contents must be a string or Uint8Array");
}

function assertMode(mode) {
  if (!Number.isInteger(mode) || mode < 0 || mode > 0o777) {
    throw new Error(
      "mode must be a Unix permission mode from 0000 through 0777",
    );
  }
}

function assertLeafName(name) {
  if (
    typeof name !== "string" ||
    name.length === 0 ||
    name === "." ||
    name === ".." ||
    name.includes("/") ||
    name.includes("\\") ||
    name.includes("\0")
  ) {
    throw new Error("destination name must be one safe path component");
  }
}

async function removeTemporaryFile(temporaryPath) {
  try {
    await fs.unlink(temporaryPath);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
}

export class AtomicPublicationAmbiguousError extends Error {
  constructor(destination, operation, cause, { durable }) {
    super(
      `destination ${destination} is visible, but ${operation} failed; verify it before retrying`,
      { cause },
    );
    this.code = "ATOMIC_PUBLICATION_AMBIGUOUS";
    this.destination = destination;
    this.durable = durable;
    this.published = true;
  }
}

async function publish({
  contents,
  destination,
  mode,
  syncDirectory,
  temporaryPath,
}) {
  const bytes = toBuffer(contents);
  let handle;
  try {
    handle = await fs.open(
      temporaryPath,
      fsConstants.O_WRONLY |
        fsConstants.O_CREAT |
        fsConstants.O_EXCL |
        (fsConstants.O_NOFOLLOW ?? 0),
      0o600,
    );
    await handle.writeFile(bytes);
    await handle.chmod(mode);
    await handle.sync();
    await handle.close();
    handle = undefined;
  } catch (error) {
    await handle?.close().catch(() => {});
    await removeTemporaryFile(temporaryPath).catch(() => {});
    throw error;
  }

  try {
    await fs.link(temporaryPath, destination);
  } catch (error) {
    if (error.code === "EEXIST") {
      await removeTemporaryFile(temporaryPath);
      return "exists";
    }
    await removeTemporaryFile(temporaryPath).catch(() => {});
    throw error;
  }

  try {
    await syncDirectory();
  } catch (error) {
    throw new AtomicPublicationAmbiguousError(
      destination,
      "commit directory fsync",
      error,
      { durable: false },
    );
  }
  try {
    await removeTemporaryFile(temporaryPath);
  } catch (error) {
    throw new AtomicPublicationAmbiguousError(
      destination,
      "temporary-file cleanup",
      error,
      { durable: true },
    );
  }
  try {
    await syncDirectory();
  } catch (error) {
    throw new AtomicPublicationAmbiguousError(
      destination,
      "cleanup directory fsync",
      error,
      { durable: true },
    );
  }
  return "created";
}

export async function publishBytesNoReplaceAt(
  directoryHandle,
  name,
  contents,
  { mode = 0o600 } = {},
) {
  await assertDescriptorTraversalSupport();
  assertLeafName(name);
  assertMode(mode);
  if (!Number.isInteger(directoryHandle?.fd) || directoryHandle.fd < 0) {
    throw new Error("directoryHandle must be an open directory file handle");
  }
  const directoryPath = `/proc/self/fd/${directoryHandle.fd}`;
  const temporaryPath = `${directoryPath}/.${name}.${process.pid}.${crypto.randomUUID()}.tmp`;
  return publish({
    contents,
    destination: `${directoryPath}/${name}`,
    mode,
    syncDirectory: () => directoryHandle.sync(),
    temporaryPath,
  });
}

export async function publishBytesNoReplace(
  destination,
  contents,
  { mode = 0o600 } = {},
) {
  assertMode(mode);
  if (process.platform === "win32" && (mode & 0o200) === 0) {
    throw new Error("read-only publication modes are unsupported on Windows");
  }
  const absoluteDestination = path.resolve(destination);
  const directory = path.dirname(absoluteDestination);
  const name = path.basename(absoluteDestination);
  assertLeafName(name);
  const temporaryPath = path.join(
    directory,
    `.${name}.${process.pid}.${crypto.randomUUID()}.tmp`,
  );
  const syncDirectory = async () => {
    if (process.platform === "win32") return;
    const handle = await fs.open(
      directory,
      fsConstants.O_RDONLY |
        (fsConstants.O_DIRECTORY ?? 0) |
        (fsConstants.O_NOFOLLOW ?? 0),
    );
    try {
      await handle.sync();
    } finally {
      await handle.close();
    }
  };
  return publish({
    contents,
    destination: absoluteDestination,
    mode,
    syncDirectory,
    temporaryPath,
  });
}
