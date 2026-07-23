import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

const DEFAULT_MAX_BYTES = 1024 * 1024;
const READ_CHUNK_BYTES = 64 * 1024;
let descriptorTraversalSupportPromise;

export function isPathInside(root, filePath) {
  const relativePath = path.relative(root, filePath);
  return (
    relativePath === "" ||
    (relativePath !== ".." &&
      !relativePath.startsWith(`..${path.sep}`) &&
      !path.isAbsolute(relativePath))
  );
}

function assertMaxBytes(maxBytes) {
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 0) {
    throw new Error("Corpus file byte limit must be a non-negative safe integer");
  }
}

function sameFileIdentity(left, right) {
  return left.dev === right.dev && left.ino === right.ino;
}

function sameFileSnapshot(left, right) {
  return (
    sameFileIdentity(left, right) &&
    left.size === right.size &&
    left.mtimeNs === right.mtimeNs &&
    left.ctimeNs === right.ctimeNs
  );
}

function notRegularFileError(filePath) {
  return new Error(`Corpus path must be a regular file: ${filePath}`);
}

function changedPathError(filePath) {
  return new Error(`Corpus path changed while being read: ${filePath}`);
}

function oversizedFileError(filePath, maxBytes) {
  return new Error(`Corpus file exceeds ${maxBytes} byte limit: ${filePath}`);
}

function unsupportedDescriptorTraversalError() {
  return new Error("Corpus file reads require Linux descriptor-relative traversal support");
}

function intermediateSymlinkError(filePath) {
  return new Error(`Corpus path must not contain symbolic links: ${filePath}`);
}

function relativeCorpusPath(corpusRoot, filePath) {
  const resolvedRoot = path.resolve(corpusRoot);
  const resolvedFile = path.resolve(filePath);
  if (!isPathInside(resolvedRoot, resolvedFile) || resolvedRoot === resolvedFile) {
    throw new Error(`Corpus file escapes corpus root: ${filePath}`);
  }
  return path.relative(resolvedRoot, resolvedFile);
}

async function closeDirectoryHandles(directoryHandles) {
  for (const directoryHandle of directoryHandles.reverse()) {
    await directoryHandle.close();
  }
}

export async function assertDescriptorTraversalSupport() {
  if (
    process.platform !== "linux" ||
    !Number.isInteger(fsConstants.O_NOFOLLOW) ||
    !Number.isInteger(fsConstants.O_DIRECTORY) ||
    !Number.isInteger(fsConstants.O_NONBLOCK)
  ) {
    throw unsupportedDescriptorTraversalError();
  }

  descriptorTraversalSupportPromise ??= fs
    .stat("/proc/self/fd")
    .then((stats) => {
      if (!stats.isDirectory()) throw unsupportedDescriptorTraversalError();
    })
    .catch(() => {
      throw unsupportedDescriptorTraversalError();
    });
  await descriptorTraversalSupportPromise;
}

export async function openCorpusFileNoSymlinks(corpusRoot, filePath, flags, {closeHandles = closeDirectoryHandles} = {}) {
  await assertDescriptorTraversalSupport();

  const relativePath = relativeCorpusPath(corpusRoot, filePath);
  const components = relativePath.split(path.sep);
  const fileName = components.pop();
  const realCorpusRoot = await fs.realpath(corpusRoot);
  const directoryHandles = [];
  let fileHandle;
  let openError = null;

  try {
    let directoryHandle = await fs.open(
      realCorpusRoot,
      fsConstants.O_RDONLY | fsConstants.O_DIRECTORY | fsConstants.O_NOFOLLOW,
    );
    directoryHandles.push(directoryHandle);

    for (const component of components) {
      let nextDirectoryHandle;
      try {
        nextDirectoryHandle = await fs.open(
          `/proc/self/fd/${directoryHandle.fd}/${component}`,
          fsConstants.O_RDONLY | fsConstants.O_DIRECTORY | fsConstants.O_NOFOLLOW,
        );
      } catch (error) {
        if (["ELOOP", "EMLINK", "ENOTDIR"].includes(error.code)) {
          throw intermediateSymlinkError(filePath);
        }
        throw error;
      }

      directoryHandles.push(nextDirectoryHandle);
      const directoryStats = await nextDirectoryHandle.stat();
      if (!directoryStats.isDirectory()) {
        throw intermediateSymlinkError(filePath);
      }
      directoryHandle = nextDirectoryHandle;
    }

    fileHandle = await fs.open(`/proc/self/fd/${directoryHandle.fd}/${fileName}`, flags);
  } catch (error) {
    openError = error;
  }

  let directoryCloseError = null;
  try {
    await closeHandles(directoryHandles);
  } catch (error) {
    directoryCloseError = error;
  }
  if (directoryCloseError !== null) {
    let fileCloseError = null;
    try {
      await fileHandle?.close();
    } catch (error) {
      fileCloseError = error;
    }
    const errors = [openError, directoryCloseError, fileCloseError].filter((error) => error !== null);
    if (errors.length === 1) throw errors[0];
    throw new AggregateError(errors, "corpus file open and descriptor cleanup failed");
  }
  if (openError !== null) throw openError;

  return fileHandle;
}

async function verifyOpenedPath(corpusRoot, filePath, openedStats) {
  let pathStats;
  try {
    const pathHandle = await openCorpusFileNoSymlinks(
      corpusRoot,
      filePath,
      fsConstants.O_RDONLY | fsConstants.O_NONBLOCK | fsConstants.O_NOFOLLOW,
    );
    try {
      pathStats = await pathHandle.stat({ bigint: true });
    } finally {
      await pathHandle.close();
    }
  } catch {
    throw changedPathError(filePath);
  }

  if (!pathStats.isFile() || !sameFileIdentity(openedStats, pathStats)) {
    throw changedPathError(filePath);
  }
}

async function readBoundedUtf8(fileHandle, filePath, maxBytes) {
  const chunks = [];
  let totalBytes = 0;

  while (totalBytes <= maxBytes) {
    const remaining = maxBytes + 1 - totalBytes;
    const buffer = Buffer.allocUnsafe(Math.min(READ_CHUNK_BYTES, remaining));
    const { bytesRead } = await fileHandle.read(buffer, 0, buffer.length, totalBytes);
    if (bytesRead === 0) break;
    chunks.push(buffer.subarray(0, bytesRead));
    totalBytes += bytesRead;
  }

  if (totalBytes > maxBytes) {
    throw oversizedFileError(filePath, maxBytes);
  }

  return Buffer.concat(chunks, totalBytes).toString("utf8");
}

export function createCorpusFileReader({ openFile = openCorpusFileNoSymlinks } = {}) {
  return async function readCorpusFile(
    corpusRoot,
    filePath,
    { optional = false, maxBytes = DEFAULT_MAX_BYTES } = {},
  ) {
    assertMaxBytes(maxBytes);
    let fileHandle;
    try {
      fileHandle = await openFile(
        corpusRoot,
        filePath,
        fsConstants.O_RDONLY | fsConstants.O_NONBLOCK | fsConstants.O_NOFOLLOW,
      );
    } catch (error) {
      if (optional && error.code === "ENOENT") return null;
      if (error.code === "ELOOP" || error.code === "EMLINK") {
        throw notRegularFileError(filePath);
      }
      throw error;
    }

    try {
      const openedStats = await fileHandle.stat({ bigint: true });
      if (!openedStats.isFile()) {
        throw notRegularFileError(filePath);
      }
      if (openedStats.size > BigInt(maxBytes)) {
        throw oversizedFileError(filePath, maxBytes);
      }

      await verifyOpenedPath(corpusRoot, filePath, openedStats);
      const text = await readBoundedUtf8(fileHandle, filePath, maxBytes);

      const finalStats = await fileHandle.stat({ bigint: true });
      if (!sameFileSnapshot(openedStats, finalStats)) {
        throw changedPathError(filePath);
      }
      await verifyOpenedPath(corpusRoot, filePath, finalStats);

      return text;
    } finally {
      await fileHandle.close();
    }
  };
}

export const readCorpusFile = createCorpusFileReader();
