import path from "node:path";

import {stableStringify} from "./canonical-json.mjs";

const SHA256_RE = /^[0-9a-f]{64}$/u;
const STORE_DESCRIPTOR = Object.freeze({
  digest_encoding: "lowercase-hex",
  hash_algorithm: "sha256",
  object_encoding: "raw",
  object_path_template: "objects/sha256/{prefix2}/{sha256}",
  reference_schema:
    "https://wikijump.org/schemas/reference-object-v1.schema.json",
  schema: "wikijump_full_parity.reference_object_store.v1",
});

export const REFERENCE_OBJECT_STORE_DESCRIPTOR_BYTES = Buffer.from(
  `${stableStringify(STORE_DESCRIPTOR)}\n`,
  "utf8",
);
export const REFERENCE_OBJECT_STORE_DESCRIPTOR_MISMATCH =
  "store.json does not match the canonical reference store descriptor";

export function assertReferenceObjectSha256(value, label = "sha256") {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

export function assertReferenceObjectBytes(value, label = "bytes") {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${label} must be a non-negative safe integer`);
  }
}

export function validateReferenceObject(object) {
  if (object === null || typeof object !== "object" || Array.isArray(object)) {
    throw new Error("reference object must be an object");
  }
  if (
    stableStringify(Object.keys(object).sort()) !==
    stableStringify(["algorithm", "bytes", "sha256"])
  ) {
    throw new Error(
      "reference object must contain only algorithm, bytes, and sha256",
    );
  }
  const algorithm = object.algorithm;
  const bytes = object.bytes;
  const sha256 = object.sha256;
  if (algorithm !== "sha256") {
    throw new Error("reference object algorithm must be sha256");
  }
  assertReferenceObjectBytes(bytes, "reference object bytes");
  assertReferenceObjectSha256(sha256, "reference object sha256");
  return Object.freeze({algorithm, bytes, sha256});
}

export function referenceObjectRelativePath(sha256) {
  assertReferenceObjectSha256(sha256);
  return path.posix.join("objects", "sha256", sha256.slice(0, 2), sha256);
}

export function referenceObjectStoreDescriptorBytes() {
  return Buffer.from(REFERENCE_OBJECT_STORE_DESCRIPTOR_BYTES);
}
