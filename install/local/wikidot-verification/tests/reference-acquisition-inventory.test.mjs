import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import {fileURLToPath} from "node:url";

import {sha256Hex} from "../src/corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionInventory,
  computeReferenceAcquisitionInventoryIdentity,
  serializeReferenceAcquisitionInventory,
} from "../src/reference-acquisition-inventory.mjs";
import {validateReferenceAcquisitionInventory} from "../src/reference-acquisition-inventory-validation.mjs";
import {
  inventoryFixtureInputs,
  referenceAttachment,
  SOURCE_ORIGIN,
  TWO_REFERENCE_ROWS,
} from "./support/reference-acquisition-inventory-fixture.mjs";

function build(rows, overrides = {}) {
  return buildReferenceAcquisitionInventory({
    ...inventoryFixtureInputs(rows),
    family: "EN",
    shardCount: 64,
    sourceOrigin: SOURCE_ORIGIN,
    ...overrides,
  });
}

test("builds a deterministic, portable, exactly sharded acquisition inventory", () => {
  const first = build(TWO_REFERENCE_ROWS);
  const second = build(structuredClone(TWO_REFERENCE_ROWS));
  assert.equal(
    serializeReferenceAcquisitionInventory(first),
    serializeReferenceAcquisitionInventory(second),
  );
  assert.equal(
    first.rows[1].source_url,
    "https://scp-wiki.wikidot.com/theme:%E9%9B%AA%20space",
  );
  assert.deepEqual(first.rows[0].requested_layers, [
    "xmlrpc_page",
    "http_document",
    "browser_document",
  ]);
  assert.equal(first.shards.length, 64);
  assert.deepEqual(
    first.shards.flatMap((shard) => shard.fixture_ids).sort(),
    first.rows.map((value) => value.fixture_id).sort(),
  );
  assert.equal(
    first.shards.reduce((count, shard) => count + shard.count, 0),
    2,
  );
  assert.deepEqual(first.shards[21], {
    count: 1,
    fixture_ids: ["EN:theme:雪 space"],
    fixture_set_sha256: sha256Hex("EN:theme:雪 space"),
    shard_id: "en-0021",
  });
  assert.deepEqual(first.shards[24], {
    count: 1,
    fixture_ids: ["EN:alpha"],
    fixture_set_sha256: sha256Hex("EN:alpha"),
    shard_id: "en-0024",
  });
  assert.equal(
    first.identity.sha256,
    computeReferenceAcquisitionInventoryIdentity(first),
  );
  const serialized = serializeReferenceAcquisitionInventory(first);
  assert.equal(serialized.endsWith("\n"), true);
  assert.doesNotMatch(
    serialized,
    /host\/path|corpus_path|file_path|metadata_path/u,
  );
});

test("validates loaded inventory structure and exact shard ownership", () => {
  const inventory = build(TWO_REFERENCE_ROWS);
  const validated = validateReferenceAcquisitionInventory(inventory, {
    expectedIdentitySha256: inventory.identity.sha256,
  });
  assert.equal(validated, inventory);
  assert.equal(Object.isFrozen(validated.rows[0].baseline), true);
  assert.throws(() => {
    validated.rows[0].fullname = "mutated";
  }, TypeError);
  const shallowFrozen = build(TWO_REFERENCE_ROWS);
  Object.freeze(shallowFrozen.rows);
  validateReferenceAcquisitionInventory(shallowFrozen, {
    expectedIdentitySha256: shallowFrozen.identity.sha256,
  });
  assert.equal(Object.isFrozen(shallowFrozen.rows[0].baseline), true);
  assert.throws(
    () =>
      validateReferenceAcquisitionInventory(structuredClone(inventory), {
        expectedIdentitySha256: "0".repeat(64),
      }),
    /expected authority/u,
  );
  const wrongOrdinal = structuredClone(inventory);
  wrongOrdinal.rows[0].ordinal = 1;
  wrongOrdinal.identity.sha256 =
    computeReferenceAcquisitionInventoryIdentity(wrongOrdinal);
  assert.throws(
    () =>
      validateReferenceAcquisitionInventory(wrongOrdinal, {
        expectedIdentitySha256: wrongOrdinal.identity.sha256,
      }),
    /ordinal is not exact/u,
  );
  const wrongShard = structuredClone(inventory);
  const populatedShard = wrongShard.shards.find(
    (shard) => shard.fixture_ids.length > 0,
  );
  populatedShard.fixture_ids = [];
  populatedShard.count = 0;
  populatedShard.fixture_set_sha256 = sha256Hex("");
  wrongShard.identity.sha256 =
    computeReferenceAcquisitionInventoryIdentity(wrongShard);
  assert.throws(
    () =>
      validateReferenceAcquisitionInventory(wrongShard, {
        expectedIdentitySha256: wrongShard.identity.sha256,
      }),
    /exact fixture set/u,
  );
});

test("loaded validation preserves the builder authority and canonical row rules", () => {
  for (const sourceOrigin of [
    "https://attacker.example",
    "https://wikidot.com",
    "https://scp-wiki.wikidot.com:444",
  ]) {
    const inventory = build(TWO_REFERENCE_ROWS);
    inventory.source_origin = sourceOrigin;
    for (const row of inventory.rows) {
      const sourceUrl = new URL(sourceOrigin);
      sourceUrl.pathname = `/${row.fullname}`;
      row.source_url = sourceUrl.href;
    }
    inventory.identity.sha256 =
      computeReferenceAcquisitionInventoryIdentity(inventory);
    assert.throws(() =>
      validateReferenceAcquisitionInventory(inventory, {
        expectedIdentitySha256: inventory.identity.sha256,
      }),
    );
  }
  for (const fullname of ["a/b", "a\\b", "../admin", "bad%"]) {
    const inventory = build(TWO_REFERENCE_ROWS);
    const row = inventory.rows[0];
    row.fullname = fullname;
    row.slug = fullname;
    row.fixture_id = `EN:${fullname}`;
    const sourceUrl = new URL(inventory.source_origin);
    sourceUrl.pathname = `/${fullname}`;
    row.source_url = sourceUrl.href;
    inventory.identity.sha256 =
      computeReferenceAcquisitionInventoryIdentity(inventory);
    assert.throws(() =>
      validateReferenceAcquisitionInventory(inventory, {
        expectedIdentitySha256: inventory.identity.sha256,
      }),
    );
  }
  const badTimestamp = build(TWO_REFERENCE_ROWS);
  badTimestamp.rows[0].baseline.updated_at = "2026-07-18";
  badTimestamp.identity.sha256 =
    computeReferenceAcquisitionInventoryIdentity(badTimestamp);
  assert.throws(
    () =>
      validateReferenceAcquisitionInventory(badTimestamp, {
        expectedIdentitySha256: badTimestamp.identity.sha256,
      }),
    /RFC 3339/u,
  );
  const reordered = build(TWO_REFERENCE_ROWS);
  reordered.rows.reverse();
  reordered.rows.forEach((row, ordinal) => {
    row.ordinal = ordinal;
  });
  reordered.source_manifest.first_fullname = reordered.rows[0].fullname;
  reordered.source_manifest.last_fullname = reordered.rows.at(-1).fullname;
  reordered.identity.sha256 =
    computeReferenceAcquisitionInventoryIdentity(reordered);
  assert.throws(
    () =>
      validateReferenceAcquisitionInventory(reordered, {
        expectedIdentitySha256: reordered.identity.sha256,
      }),
    /canonical row order/u,
  );
});

test("fails closed on invalid trust anchors, input framing, ordering, and identity", () => {
  const valid = inventoryFixtureInputs(TWO_REFERENCE_ROWS);
  assert.throws(
    () => build(TWO_REFERENCE_ROWS, { expectedManifestSha256: "0".repeat(64) }),
    /SHA-256 mismatch/u,
  );
  assert.throws(
    () => build(TWO_REFERENCE_ROWS, { expectedCount: 3 }),
    /row count mismatch/u,
  );
  assert.throws(
    () => build(TWO_REFERENCE_ROWS, { expectedSummarySha256: "0".repeat(64) }),
    /summary SHA-256 mismatch/u,
  );
  assert.throws(
    () =>
      buildReferenceAcquisitionInventory({
        ...valid,
        family: "EN",
        shardCount: 64,
        sourceOrigin: "http://scp-wiki.wikidot.com",
      }),
    /credential-free HTTPS origin/u,
  );
  assert.throws(
    () =>
      build(TWO_REFERENCE_ROWS, { sourceOrigin: "https://scp-wiki.wikidot.com:8443" }),
    /credential-free HTTPS origin/u,
  );
  assert.throws(() => build([...TWO_REFERENCE_ROWS].reverse()), /strictly sorted/u);
  assert.throws(
    () =>
      build([TWO_REFERENCE_ROWS[0], { ...TWO_REFERENCE_ROWS[1], fullname: TWO_REFERENCE_ROWS[0].fullname }]),
    /collides at source URL|strictly sorted and unique/u,
  );
  assert.throws(
    () => build([{ ...TWO_REFERENCE_ROWS[0], source_entity_id: null }, TWO_REFERENCE_ROWS[1]]),
    /source_entity_id must be a non-empty string/u,
  );
  assert.throws(
    () => build([{ ...TWO_REFERENCE_ROWS[0], attachments: {} }, TWO_REFERENCE_ROWS[1]]),
    /attachments must be an array/u,
  );
  assert.throws(
    () => build([{ ...TWO_REFERENCE_ROWS[0], fullname: "../admin" }, TWO_REFERENCE_ROWS[1]]),
    /unsafe path character/u,
  );
  assert.throws(
    () => build([{ ...TWO_REFERENCE_ROWS[0], updated_at: "2026-07-18" }, TWO_REFERENCE_ROWS[1]]),
    /RFC 3339/u,
  );
  assert.throws(
    () =>
      build([
        TWO_REFERENCE_ROWS[0],
        { ...TWO_REFERENCE_ROWS[1], source_entity_id: TWO_REFERENCE_ROWS[0].source_entity_id },
      ]),
    /duplicate source_entity_id/u,
  );
  const malformed = Buffer.from("{}\r\n");
  assert.throws(
    () =>
      buildReferenceAcquisitionInventory({
        ...valid,
        manifestBytes: malformed,
        expectedCount: 1,
        expectedManifestSha256: sha256Hex(malformed),
        family: "EN",
        shardCount: 1,
        sourceOrigin: SOURCE_ORIGIN,
      }),
    /use LF lines/u,
  );
  const invalidJson = Buffer.from("{not-json}\n");
  assert.throws(
    () =>
      buildReferenceAcquisitionInventory({
        ...valid,
        expectedCount: 1,
        expectedManifestSha256: sha256Hex(invalidJson),
        family: "EN",
        manifestBytes: invalidJson,
        shardCount: 1,
        sourceOrigin: SOURCE_ORIGIN,
      }),
    /not valid JSON/u,
  );
  const wrongSummary = JSON.parse(valid.summaryBytes);
  wrongSummary.attachment_count += 1;
  const wrongSummaryBytes = Buffer.from(JSON.stringify(wrongSummary));
  assert.throws(
    () =>
      build(TWO_REFERENCE_ROWS, {
        expectedSummarySha256: sha256Hex(wrongSummaryBytes),
        summaryBytes: wrongSummaryBytes,
      }),
    /attachment_count mismatch/u,
  );
  const hostileAttachment = {
    ...referenceAttachment(),
    original_url:
      "http://user:secret@127.0.0.1/local--files/alpha/image%20one.png#token",
  };
  assert.throws(
    () =>
      build([
        { ...TWO_REFERENCE_ROWS[0], attachments: [hostileAttachment] },
        TWO_REFERENCE_ROWS[1],
      ]),
    /credentials, a fragment|host is out of scope/u,
  );
  const traversingAttachment = {
    ...referenceAttachment(),
    filename: "secret.png",
    original_url:
      "http://scp-wiki.wdfiles.com/local--files/alpha/%2e%2e/secret.png",
    wikidot_path: "/local--files/alpha/%2e%2e/secret.png",
  };
  assert.throws(
    () =>
      build([
        { ...TWO_REFERENCE_ROWS[0], attachments: [traversingAttachment] },
        TWO_REFERENCE_ROWS[1],
      ]),
    /unsafe segment/u,
  );
  const tampered = structuredClone(build(TWO_REFERENCE_ROWS));
  tampered.rows[0].fullname = "changed";
  assert.throws(
    () => serializeReferenceAcquisitionInventory(tampered),
    /identity mismatch/u,
  );
  const badEnvelope = structuredClone(build(TWO_REFERENCE_ROWS));
  badEnvelope.identity.algorithm = "md5";
  assert.throws(
    () => serializeReferenceAcquisitionInventory(badEnvelope),
    /exact stable-json/u,
  );
  const extraEnvelope = structuredClone(build(TWO_REFERENCE_ROWS));
  extraEnvelope.identity.extra = true;
  assert.throws(
    () => serializeReferenceAcquisitionInventory(extraEnvelope),
    /exact stable-json/u,
  );
});

test("the structural schema remains valid JSON and names the emitted contract", () => {
  const schemaPath = fileURLToPath(
    new URL(
      "../schemas/reference-acquisition-inventory.schema.json",
      import.meta.url,
    ),
  );
  const schema = JSON.parse(fs.readFileSync(schemaPath, "utf8"));
  const inventory = build(TWO_REFERENCE_ROWS);
  assert.equal(schema.properties.schema.const, inventory.schema);
  assert.equal(schema.properties.host_paths_included.const, false);
  assert.deepEqual(Object.keys(inventory).sort(), [...schema.required].sort());
  assert.deepEqual(
    Object.keys(inventory.identity).sort(),
    [...schema.properties.identity.required].sort(),
  );
  assert.deepEqual(
    Object.keys(inventory.source_manifest).sort(),
    [...schema.properties.source_manifest.required].sort(),
  );
  assert.deepEqual(
    Object.keys(inventory.rows[0]).sort(),
    [...schema.properties.rows.items.required].sort(),
  );
  assert.deepEqual(
    Object.keys(inventory.shards[0]).sort(),
    [...schema.properties.shards.items.required].sort(),
  );
});
