import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

const resolveFixture = (relativePath) =>
  fileURLToPath(new URL(relativePath, import.meta.url));

test("versioned store descriptor, reference schema, and golden vectors agree", () => {
  const descriptorBytes = fs.readFileSync(
    resolveFixture("../fixtures/reference-object-store-v1/store.json"),
  );
  const descriptor = JSON.parse(descriptorBytes);
  const storeSchema = JSON.parse(
    fs.readFileSync(
      resolveFixture("../schemas/reference-object-store-v1.schema.json"),
      "utf8",
    ),
  );
  const referenceSchema = JSON.parse(
    fs.readFileSync(
      resolveFixture("../schemas/reference-object-v1.schema.json"),
      "utf8",
    ),
  );
  const vectorFixture = JSON.parse(
    fs.readFileSync(
      resolveFixture("../fixtures/reference-object-store-v1/vectors.json"),
      "utf8",
    ),
  );

  assert.equal(descriptorBytes.length, 285);
  assert.equal(
    crypto.createHash("sha256").update(descriptorBytes).digest("hex"),
    "dfc3db9423713751f1f8bda474b934632fa969232f6a44dabb28e765a6288f79",
  );
  assert.deepEqual(
    Object.keys(descriptor).sort(),
    [...storeSchema.required].sort(),
  );
  for (const [key, value] of Object.entries(descriptor)) {
    assert.equal(storeSchema.properties[key].const, value);
  }
  assert.equal(descriptor.reference_schema, referenceSchema.$id);
  assert.deepEqual(referenceSchema.required.sort(), [
    "algorithm",
    "bytes",
    "sha256",
  ]);

  for (const vector of vectorFixture.vectors) {
    const bytes = Buffer.from(vector.input_hex, "hex");
    assert.equal(bytes.length, vector.bytes);
    assert.equal(
      crypto.createHash("sha256").update(bytes).digest("hex"),
      vector.sha256,
    );
    assert.equal(
      vector.relative_path,
      `objects/sha256/${vector.sha256.slice(0, 2)}/${vector.sha256}`,
    );
  }
});
