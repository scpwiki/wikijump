import assert from "node:assert/strict";
import test from "node:test";

import {
  assertReferenceObjectBytes,
  assertReferenceObjectSha256,
  referenceObjectRelativePath,
  referenceObjectStoreDescriptorBytes,
  validateReferenceObject,
} from "../src/reference-object-descriptor.mjs";

test("reference object descriptor validates canonical identity and path layout", () => {
  const digest = "a".repeat(64);
  assert.doesNotThrow(() => assertReferenceObjectSha256(digest));
  assert.doesNotThrow(() => assertReferenceObjectBytes(0));
  assert.throws(() => assertReferenceObjectSha256("A".repeat(64)), /lowercase SHA-256/u);
  assert.throws(() => assertReferenceObjectBytes(-1), /non-negative safe integer/u);
  assert.equal(
    referenceObjectRelativePath(digest),
    `objects/sha256/aa/${digest}`,
  );
  assert.deepEqual(
    validateReferenceObject({algorithm: "sha256", bytes: 3, sha256: digest}),
    {algorithm: "sha256", bytes: 3, sha256: digest},
  );
  assert.equal(Object.isFrozen(validateReferenceObject({algorithm: "sha256", bytes: 3, sha256: digest})), true);
  assert.match(referenceObjectStoreDescriptorBytes().toString("utf8"), /reference_object_store\.v1/u);
});
