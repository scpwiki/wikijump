import { sha256Hex, stableStringify } from "./canonical-json.mjs";
import {serializeReferenceAcquisitionInventory} from "./reference-acquisition-inventory.mjs";
import {
  assertCanonicalFullname,
  assertTimestamp,
  codePointCompare,
  validateOrigin,
} from "./reference-acquisition-inventory-source.mjs";

const SHA256_RE = /^[0-9a-f]{64}$/u;
const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;
const REQUESTED_LAYERS = Object.freeze([
  "xmlrpc_page",
  "http_document",
  "browser_document",
]);
const TOP_KEYS = Object.freeze([
  "family",
  "host_paths_included",
  "identity",
  "requested_layers",
  "rows",
  "schema",
  "shard_count",
  "shards",
  "source_manifest",
  "source_origin",
]);
const ROW_KEYS = Object.freeze([
  "attachment_count",
  "attachment_inventory_sha256",
  "baseline",
  "family",
  "fixture_id",
  "fullname",
  "input_line_sha256",
  "ordinal",
  "requested_layers",
  "semantic_row_sha256",
  "slug",
  "source_entity_id",
  "source_url",
]);

function assertObject(value, label) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
}

function assertExactKeys(value, keys, label) {
  assertObject(value, label);
  if (stableStringify(Object.keys(value).sort()) !== stableStringify(keys)) {
    throw new Error(`${label} has unexpected fields`);
  }
}

function assertSha256(value, label) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

function assertSafeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${label} must be a non-negative safe integer`);
  }
}

function assertLayers(value, label) {
  if (stableStringify(value) !== stableStringify(REQUESTED_LAYERS)) {
    throw new Error(`${label} must contain the exact acquisition layers`);
  }
}

function validateRow(
  row,
  inventory,
  index,
  seenFixtureIds,
  seenEntityIds,
  seenSourceUrls,
) {
  const label = `inventory.rows[${index}]`;
  assertExactKeys(row, ROW_KEYS, label);
  if (row.ordinal !== index) throw new Error(`${label}.ordinal is not exact`);
  if (
    typeof row.fullname !== "string" ||
    row.fullname.length === 0 ||
    row.slug !== row.fullname ||
    row.family !== inventory.family ||
    row.fixture_id !== `${inventory.family}:${row.fullname}`
  ) {
    throw new Error(`${label} page identity is inconsistent`);
  }
  assertCanonicalFullname(row.fullname, `${label}.fullname`);
  if (!UUID_RE.test(row.source_entity_id)) {
    throw new Error(`${label}.source_entity_id must be a lowercase UUID`);
  }
  if (
    seenFixtureIds.has(row.fixture_id) ||
    seenEntityIds.has(row.source_entity_id)
  ) {
    throw new Error(`${label} duplicates an inventory identity`);
  }
  seenFixtureIds.add(row.fixture_id);
  seenEntityIds.add(row.source_entity_id);
  const sourceUrl = new URL(inventory.source_origin);
  sourceUrl.pathname = `/${row.fullname}`;
  if (row.source_url !== sourceUrl.href) {
    throw new Error(`${label}.source_url is not canonical`);
  }
  if (seenSourceUrls.has(row.source_url)) {
    throw new Error(`${label}.source_url duplicates another row`);
  }
  seenSourceUrls.add(row.source_url);
  assertLayers(row.requested_layers, `${label}.requested_layers`);
  assertSha256(row.input_line_sha256, `${label}.input_line_sha256`);
  assertSha256(row.semantic_row_sha256, `${label}.semantic_row_sha256`);
  assertSha256(
    row.attachment_inventory_sha256,
    `${label}.attachment_inventory_sha256`,
  );
  assertSafeInteger(row.attachment_count, `${label}.attachment_count`);
  assertExactKeys(
    row.baseline,
    ["meta_sha256", "revisions", "source_sha256", "updated_at"],
    `${label}.baseline`,
  );
  assertSha256(row.baseline.meta_sha256, `${label}.baseline.meta_sha256`);
  assertSha256(row.baseline.source_sha256, `${label}.baseline.source_sha256`);
  assertSafeInteger(row.baseline.revisions, `${label}.baseline.revisions`);
  assertTimestamp(row.baseline.updated_at, `${label}.baseline.updated_at`);
}

function validateShards(inventory) {
  if (
    !Number.isSafeInteger(inventory.shard_count) ||
    inventory.shard_count < 1 ||
    inventory.shard_count > 4096 ||
    !Array.isArray(inventory.shards) ||
    inventory.shards.length !== inventory.shard_count
  ) {
    throw new Error("inventory shard count is inconsistent");
  }
  const expected = Array.from({ length: inventory.shard_count }, () => []);
  for (const row of inventory.rows) {
    const shard = Number(
      BigInt(`0x${sha256Hex(row.fixture_id)}`) % BigInt(inventory.shard_count),
    );
    expected[shard].push(row.fixture_id);
  }
  const width = Math.max(4, String(inventory.shard_count - 1).length);
  for (let index = 0; index < inventory.shards.length; index += 1) {
    const shard = inventory.shards[index];
    const label = `inventory.shards[${index}]`;
    assertExactKeys(
      shard,
      ["count", "fixture_ids", "fixture_set_sha256", "shard_id"],
      label,
    );
    const fixtureIds = expected[index];
    if (
      shard.shard_id !==
        `${inventory.family.toLowerCase()}-${String(index).padStart(width, "0")}` ||
      shard.count !== fixtureIds.length ||
      stableStringify(shard.fixture_ids) !== stableStringify(fixtureIds) ||
      shard.fixture_set_sha256 !== sha256Hex(fixtureIds.join("\n"))
    ) {
      throw new Error(`${label} does not match its exact fixture set`);
    }
  }
}

function deepFreeze(value, seen = new WeakSet()) {
  if (seen.has(value)) return value;
  seen.add(value);
  for (const child of Object.values(value)) {
    if (child !== null && typeof child === "object") {
      deepFreeze(child, seen);
    }
  }
  return Object.freeze(value);
}

export function validateReferenceAcquisitionInventory(
  inventory,
  { expectedIdentitySha256 } = {},
) {
  assertExactKeys(inventory, TOP_KEYS, "inventory");
  assertSha256(expectedIdentitySha256, "expected inventory identity");
  serializeReferenceAcquisitionInventory(inventory);
  if (inventory.identity.sha256 !== expectedIdentitySha256) {
    throw new Error("inventory identity does not match the expected authority");
  }
  if (
    inventory.schema !==
      "wikijump_full_parity.reference_acquisition_inventory.v1" ||
    !/^[A-Z][A-Z0-9-]*$/u.test(inventory.family) ||
    inventory.host_paths_included !== false
  ) {
    throw new Error("inventory authority fields are invalid");
  }
  assertLayers(inventory.requested_layers, "inventory.requested_layers");
  validateOrigin(inventory.source_origin);
  if (!Array.isArray(inventory.rows) || inventory.rows.length === 0) {
    throw new Error("inventory.rows must be non-empty");
  }
  const seenFixtureIds = new Set();
  const seenEntityIds = new Set();
  const seenSourceUrls = new Set();
  inventory.rows.forEach((row, index) => {
    if (
      index > 0 &&
      codePointCompare(inventory.rows[index - 1].fullname, row.fullname) >= 0
    ) {
      throw new Error(`${row.fullname} is not in strict canonical row order`);
    }
    validateRow(
      row,
      inventory,
      index,
      seenFixtureIds,
      seenEntityIds,
      seenSourceUrls,
    );
  });
  assertExactKeys(
    inventory.source_manifest,
    [
      "attachment_count",
      "attachment_page_count",
      "first_fullname",
      "last_fullname",
      "row_count",
      "sha256",
      "summary_sha256",
    ],
    "inventory.source_manifest",
  );
  const attachmentCount = inventory.rows.reduce(
    (total, row) => total + row.attachment_count,
    0,
  );
  const attachmentPageCount = inventory.rows.filter(
    (row) => row.attachment_count > 0,
  ).length;
  if (
    inventory.source_manifest.row_count !== inventory.rows.length ||
    inventory.source_manifest.attachment_count !== attachmentCount ||
    inventory.source_manifest.attachment_page_count !== attachmentPageCount ||
    inventory.source_manifest.first_fullname !== inventory.rows[0].fullname ||
    inventory.source_manifest.last_fullname !== inventory.rows.at(-1).fullname
  ) {
    throw new Error("inventory.source_manifest does not summarize its rows");
  }
  assertSha256(inventory.source_manifest.sha256, "source manifest SHA-256");
  assertSha256(
    inventory.source_manifest.summary_sha256,
    "source summary SHA-256",
  );
  validateShards(inventory);
  return deepFreeze(inventory);
}

export function referenceAcquisitionLayers() {
  return [...REQUESTED_LAYERS];
}
