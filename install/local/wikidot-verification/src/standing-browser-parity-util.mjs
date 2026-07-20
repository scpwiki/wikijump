import { createHash } from "node:crypto";
import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

import { publishBytesNoReplace } from "./atomic-no-replace.mjs";

export function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function requirePlainObject(value, name) {
  if (!isPlainObject(value)) throw new Error(`${name} must be a JSON object`);
  return value;
}

export function requireNonEmptyString(value, name) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${name} must be a non-empty string`);
  }
  return value;
}

export function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/u.test(value);
}

export function requireSha256(value, name) {
  if (!isSha256(value)) throw new Error(`${name} must be a lowercase SHA-256`);
  return value;
}

function stableValue(value) {
  if (Array.isArray(value)) return value.map(stableValue);
  if (isPlainObject(value)) {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, stableValue(value[key])]),
    );
  }
  if (
    value === null ||
    typeof value === "string" ||
    typeof value === "boolean" ||
    (typeof value === "number" && Number.isFinite(value))
  ) {
    return value;
  }
  throw new Error("canonical JSON input contains an unsupported value");
}

export function canonicalJson(value) {
  return `${JSON.stringify(stableValue(value))}\n`;
}

export function sha256Text(value) {
  return createHash("sha256").update(value).digest("hex");
}

export function sha256Value(value) {
  return sha256Text(canonicalJson(value));
}

export async function sha256File(filePath) {
  return createHash("sha256")
    .update(await fs.readFile(filePath))
    .digest("hex");
}

export async function readJsonObject(filePath, name = filePath) {
  const value = JSON.parse(await fs.readFile(filePath, "utf8"));
  return requirePlainObject(value, name);
}

export async function createPrivateEmptyDirectory(directory) {
  try {
    await fs.mkdir(directory, { mode: 0o700 });
  } catch (error) {
    if (error?.code === "EEXIST") {
      throw new Error(`output directory already exists: ${directory}`);
    }
    throw error;
  }
  await fs.chmod(directory, 0o700);
}

export async function sealJsonNoReplace(destination, value) {
  const serialized = canonicalJson(value);
  const result = await publishBytesNoReplace(destination, serialized, {
    mode: 0o600,
  });
  if (result === "exists") {
    const before = await fs.lstat(destination);
    if (
      !before.isFile() ||
      before.isSymbolicLink() ||
      before.nlink !== 1 ||
      (before.mode & 0o077) !== 0
    ) {
      throw new Error(
        `sealed JSON already exists but is not a private regular file: ${destination}`,
      );
    }
    const handle = await fs.open(
      destination,
      fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0),
    );
    let existing;
    try {
      const opened = await handle.stat();
      if (
        !opened.isFile() ||
        opened.isSymbolicLink() ||
        opened.nlink !== 1 ||
        opened.dev !== before.dev ||
        opened.ino !== before.ino
      ) {
        throw new Error(
          `sealed JSON changed while it was being verified: ${destination}`,
        );
      }
      existing = await handle.readFile("utf8");
    } finally {
      await handle.close();
    }
    if (existing !== serialized) {
      throw new Error(
        `sealed JSON already exists with different bytes: ${destination}`,
      );
    }
  }
  return {
    path: path.basename(destination),
    sha256: sha256Text(serialized),
    publication: result,
  };
}

export function normalizedUrl(value, name) {
  const url = new URL(requireNonEmptyString(value, name));
  if (
    !new Set(["http:", "https:"]).has(url.protocol) ||
    url.username ||
    url.password
  ) {
    throw new Error(`${name} must be an unauthenticated HTTP(S) URL`);
  }
  return url;
}

export function sortedUniqueStrings(values, name) {
  if (
    !Array.isArray(values) ||
    values.some((value) => typeof value !== "string" || value === "")
  ) {
    throw new Error(`${name} must be an array of non-empty strings`);
  }
  const result = [...new Set(values)].sort();
  if (
    result.length !== values.length ||
    values.some((value, index) => value !== result[index])
  ) {
    throw new Error(`${name} must be sorted with no duplicates`);
  }
  return result;
}
