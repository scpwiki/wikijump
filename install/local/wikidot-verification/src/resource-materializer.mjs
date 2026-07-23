import {createHash, randomUUID} from "node:crypto";
import {
  lstat,
  mkdir,
  readFile,
  realpath,
  rename,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import path from "node:path";

import {
  assertFixtureResourceManifestEntry,
  assertResolvedPathWithin,
  resolveFixtureResourcePath,
} from "./resource-manifest.mjs";

export const DEFAULT_MAX_RESOURCE_BYTES = 64 * 1024 * 1024;
const DEFAULT_HTTP_TIMEOUT_MS = 30_000;

function assertPositiveInteger(value, name) {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new TypeError(`${name} must be a positive safe integer`);
  }
}

function asResourceBuffer(value) {
  if (Buffer.isBuffer(value)) {
    return value;
  }
  if (value instanceof Uint8Array) {
    return Buffer.from(value.buffer, value.byteOffset, value.byteLength);
  }
  if (value instanceof ArrayBuffer) {
    return Buffer.from(value);
  }
  throw new TypeError("resource loader must return Buffer, Uint8Array, or ArrayBuffer bytes");
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function readResponseBody({
  response,
  controller,
  maxResourceBytes,
  originalUrl,
}) {
  if (response.body && typeof response.body.getReader === "function") {
    const reader = response.body.getReader();
    const chunks = [];
    let totalBytes = 0;
    try {
      while (true) {
        const {done, value} = await reader.read();
        if (done) {
          break;
        }

        const chunk = asResourceBuffer(value);
        totalBytes += chunk.byteLength;
        if (totalBytes > maxResourceBytes) {
          controller.abort();
          throw new Error(
            `resource body exceeds maxResourceBytes (${maxResourceBytes}): ${originalUrl}`,
          );
        }
        chunks.push(chunk);
      }
    } finally {
      reader.releaseLock();
    }
    return Buffer.concat(chunks, totalBytes);
  }

  throw new TypeError(
    `fetch response must provide a streaming body to enforce maxResourceBytes: ${originalUrl}`,
  );
}

async function ensureSafeDirectoryTree(realRoot, directoryPath) {
  assertResolvedPathWithin(realRoot, directoryPath, "resource directory path");
  const relativePath = path.relative(realRoot, directoryPath);
  let currentPath = realRoot;

  for (const segment of relativePath.split(path.sep).filter(Boolean)) {
    currentPath = path.join(currentPath, segment);
    let metadata;
    try {
      metadata = await lstat(currentPath);
    } catch (error) {
      if (error?.code !== "ENOENT") {
        throw error;
      }
      try {
        await mkdir(currentPath, {mode: 0o755});
      } catch (mkdirError) {
        if (mkdirError?.code !== "EEXIST") {
          throw mkdirError;
        }
      }
      metadata = await lstat(currentPath);
    }

    if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
      throw new Error(`resource directory component is not a real directory: ${currentPath}`);
    }
  }

  const realDirectory = await realpath(directoryPath);
  assertResolvedPathWithin(realRoot, realDirectory, "resource directory path");
  return realDirectory;
}

async function writeResourceAtomically({bytes, realRoot, targetPath}) {
  const parentPath = path.dirname(targetPath);
  const realParent = await ensureSafeDirectoryTree(realRoot, parentPath);

  const temporaryPath = path.join(
    realParent,
    `.${path.basename(targetPath)}.${process.pid}.${randomUUID()}.tmp`,
  );

  try {
    await writeFile(temporaryPath, bytes, {flag: "wx", mode: 0o644});
    await rename(temporaryPath, targetPath);
  } catch (error) {
    await rm(temporaryPath, {force: true}).catch(() => {});
    throw error;
  }

  const writtenBytes = await readFile(targetPath);
  if (!writtenBytes.equals(bytes)) {
    throw new Error(`written resource bytes differ from loader bytes: ${targetPath}`);
  }
  return writtenBytes;
}

export async function materializeFixtureResourceManifest({
  manifest,
  outputRoot,
  loadResource,
  maxResourceBytes = DEFAULT_MAX_RESOURCE_BYTES,
}) {
  if (!Array.isArray(manifest)) {
    throw new TypeError("manifest must be an array");
  }
  if (typeof loadResource !== "function") {
    throw new TypeError("loadResource must be a function");
  }
  assertPositiveInteger(maxResourceBytes, "maxResourceBytes");
  if (typeof outputRoot !== "string" || outputRoot.length === 0) {
    throw new TypeError("outputRoot must be a non-empty string");
  }

  const resolvedRoot = path.resolve(outputRoot);
  await mkdir(resolvedRoot, {recursive: true});
  const realRoot = await realpath(resolvedRoot);

  const validated = manifest.map((entry) => {
    assertFixtureResourceManifestEntry(entry);
    return Object.freeze({...entry});
  });
  const targetOwners = new Map();
  const targetPaths = validated.map((entry) => {
    const targetPath = resolveFixtureResourcePath(
      realRoot,
      entry.local_target_path,
    );
    const existingOwner = targetOwners.get(targetPath);
    if (existingOwner !== undefined) {
      throw new Error(
        `manifest entries resolve to the same file: ${existingOwner} and ${entry.original_url}`,
      );
    }
    targetOwners.set(targetPath, entry.original_url);
    return targetPath;
  });

  const materialized = [];
  for (const [index, entry] of validated.entries()) {
    const loaded = await loadResource(entry);
    const bytes = asResourceBuffer(loaded);
    if (bytes.byteLength > maxResourceBytes) {
      throw new Error(
        `resource exceeds maxResourceBytes (${maxResourceBytes}): ${entry.original_url}`,
      );
    }

    const writtenBytes = await writeResourceAtomically({
      bytes,
      realRoot,
      targetPath: targetPaths[index],
    });
    materialized.push({...entry, sha256: sha256(writtenBytes)});
  }

  return materialized;
}

export function createLocalFixtureResourceLoader({
  sourceRoot,
  maxResourceBytes = DEFAULT_MAX_RESOURCE_BYTES,
}) {
  if (typeof sourceRoot !== "string" || sourceRoot.length === 0) {
    throw new TypeError("sourceRoot must be a non-empty string");
  }
  assertPositiveInteger(maxResourceBytes, "maxResourceBytes");

  const resolvedSourceRoot = path.resolve(sourceRoot);
  return async (entry) => {
    assertFixtureResourceManifestEntry(entry);
    const sourcePath = resolveFixtureResourcePath(
      resolvedSourceRoot,
      entry.local_target_path,
    );
    const [realSourceRoot, realSourcePath] = await Promise.all([
      realpath(resolvedSourceRoot),
      realpath(sourcePath),
    ]);
    assertResolvedPathWithin(realSourceRoot, realSourcePath, "resource source path");

    const sourceMetadata = await stat(realSourcePath);
    if (!sourceMetadata.isFile()) {
      throw new Error(`resource source is not a regular file: ${realSourcePath}`);
    }
    if (sourceMetadata.size > maxResourceBytes) {
      throw new Error(
        `resource source exceeds maxResourceBytes (${maxResourceBytes}): ${entry.original_url}`,
      );
    }

    const bytes = await readFile(realSourcePath);
    if (bytes.byteLength > maxResourceBytes) {
      throw new Error(
        `resource source exceeds maxResourceBytes (${maxResourceBytes}): ${entry.original_url}`,
      );
    }
    return bytes;
  };
}

export function createHttpFixtureResourceLoader({
  fetchImpl = globalThis.fetch,
  maxResourceBytes = DEFAULT_MAX_RESOURCE_BYTES,
  timeoutMs = DEFAULT_HTTP_TIMEOUT_MS,
  redirect = "error",
} = {}) {
  if (typeof fetchImpl !== "function") {
    throw new TypeError("fetchImpl must be a function");
  }
  assertPositiveInteger(maxResourceBytes, "maxResourceBytes");
  assertPositiveInteger(timeoutMs, "timeoutMs");
  if (!["error", "follow", "manual"].includes(redirect)) {
    throw new TypeError("redirect must be error, follow, or manual");
  }

  return async (entry) => {
    assertFixtureResourceManifestEntry(entry);

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const response = await fetchImpl(entry.original_url, {
        method: "GET",
        redirect,
        signal: controller.signal,
      });

      if (!response || response.ok !== true) {
        const status = response?.status ?? "unknown";
        throw new Error(
          `resource request failed with status ${status}: ${entry.original_url}`,
        );
      }

      const contentLengthText = response.headers?.get?.("content-length");
      if (contentLengthText !== null && contentLengthText !== undefined) {
        const contentLength = Number(contentLengthText);
        if (Number.isFinite(contentLength) && contentLength > maxResourceBytes) {
          controller.abort();
          throw new Error(
            `resource content-length exceeds maxResourceBytes (${maxResourceBytes}): ${entry.original_url}`,
          );
        }
      }

      return await readResponseBody({
        response,
        controller,
        maxResourceBytes,
        originalUrl: entry.original_url,
      });
    } finally {
      clearTimeout(timeout);
    }
  };
}
