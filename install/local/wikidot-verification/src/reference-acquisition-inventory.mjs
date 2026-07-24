import {sha256Hex, stableStringify} from "./canonical-json.mjs";
import {normalizeAcquisitionAttachment} from "./reference-acquisition-attachment.mjs";
import {
  assertCanonicalFullname,
  assertReferenceAcquisitionNonEmptyString,
  assertReferenceAcquisitionNonNegativeSafeInteger,
  assertReferenceAcquisitionObject,
  assertReferenceAcquisitionSha256,
  assertTimestamp,
  codePointCompare,
  parseReferenceAcquisitionManifest,
  parseReferenceAcquisitionSummary,
  REFERENCE_ACQUISITION_FAMILY_RE,
  REFERENCE_ACQUISITION_REQUESTED_LAYERS,
  REFERENCE_ACQUISITION_UUID_RE,
  validateOrigin,
} from "./reference-acquisition-inventory-source.mjs";
import {validateAcquisitionSummary} from "./reference-acquisition-summary.mjs";

export {
  assertCanonicalFullname,
  assertTimestamp,
  codePointCompare,
  validateOrigin,
} from "./reference-acquisition-inventory-source.mjs";

function normalizeRow(record, context) {
  const { input: row, inputLineSha256, lineNumber } = record;
  const rowLabel = `manifest line ${lineNumber}`;
  assertReferenceAcquisitionObject(row, rowLabel);
  assertCanonicalFullname(row.fullname, `${rowLabel}.fullname`);
  assertReferenceAcquisitionNonEmptyString(row.source_entity_id, `${rowLabel}.source_entity_id`);
  if (!REFERENCE_ACQUISITION_UUID_RE.test(row.source_entity_id)) {
    throw new Error(`${rowLabel}.source_entity_id must be a lowercase UUID`);
  }
  if (
    row.source_site !== context.sourceSite ||
    row.source_branch !== context.sourceBranch
  ) {
    throw new Error(
      `${rowLabel} source site or branch does not match the requested acquisition`,
    );
  }
  assertReferenceAcquisitionSha256(row.source_sha256, `${rowLabel}.source_sha256`);
  assertReferenceAcquisitionSha256(row.meta_sha256, `${rowLabel}.meta_sha256`);
  assertTimestamp(row.updated_at, `${rowLabel}.updated_at`);
  assertReferenceAcquisitionNonNegativeSafeInteger(row.revisions, `${rowLabel}.revisions`);
  if (row.attachments !== undefined && !Array.isArray(row.attachments)) {
    throw new Error(`${rowLabel}.attachments must be an array when present`);
  }
  const attachments = (row.attachments ?? [])
    .map((attachment) =>
      normalizeAcquisitionAttachment(
        attachment,
        rowLabel,
        context.seenAttachmentUrls,
      ),
    )
    .sort((left, right) =>
      codePointCompare(stableStringify(left), stableStringify(right)),
    );
  const baseline = {
    meta_sha256: row.meta_sha256,
    revisions: row.revisions,
    source_sha256: row.source_sha256,
    updated_at: row.updated_at,
  };
  const fixtureId = `${context.family}:${row.fullname}`;
  const sourceUrl = new URL(context.origin);
  sourceUrl.pathname = `/${row.fullname}`;
  if (context.seenSourceUrls.has(sourceUrl.href)) {
    throw new Error(
      `${rowLabel}.fullname collides at source URL ${sourceUrl.href}`,
    );
  }
  context.seenSourceUrls.add(sourceUrl.href);
  const semanticRow = {
    attachments,
    baseline,
    fullname: row.fullname,
    source_branch: row.source_branch,
    source_entity_id: row.source_entity_id,
    source_site: row.source_site,
  };
  return {
    attachment_count: attachments.length,
    attachment_inventory_sha256: sha256Hex(stableStringify(attachments)),
    baseline,
    family: context.family,
    fixture_id: fixtureId,
    fullname: row.fullname,
    input_line_sha256: inputLineSha256,
    ordinal: lineNumber - 1,
    requested_layers: [...REFERENCE_ACQUISITION_REQUESTED_LAYERS],
    semantic_row_sha256: sha256Hex(stableStringify(semanticRow)),
    slug: row.fullname,
    source_entity_id: row.source_entity_id,
    source_url: sourceUrl.href,
  };
}

function buildShards(rows, family, shardCount) {
  const width = Math.max(4, String(shardCount - 1).length);
  const fixtureIds = Array.from({ length: shardCount }, () => []);
  for (const row of rows) {
    const digest = sha256Hex(row.fixture_id);
    const index = Number(BigInt(`0x${digest}`) % BigInt(shardCount));
    fixtureIds[index].push(row.fixture_id);
  }
  return fixtureIds.map((ids, index) => ({
    count: ids.length,
    fixture_ids: ids,
    fixture_set_sha256: sha256Hex([...ids].sort(codePointCompare).join("\n")),
    shard_id: `${family.toLowerCase()}-${String(index).padStart(width, "0")}`,
  }));
}

export function computeReferenceAcquisitionInventoryIdentity(inventory) {
  assertReferenceAcquisitionObject(inventory, "inventory");
  const body = {...inventory};
  delete body.identity;
  return sha256Hex(stableStringify(body));
}

export function serializeReferenceAcquisitionInventory(inventory) {
  assertReferenceAcquisitionObject(inventory.identity, "inventory.identity");
  if (
    stableStringify(Object.keys(inventory.identity).sort()) !==
      stableStringify(["algorithm", "canonicalization", "sha256"]) ||
    inventory.identity.algorithm !== "sha256" ||
    inventory.identity.canonicalization !== "stable-json-v1"
  ) {
    throw new Error(
      "inventory.identity must use the exact stable-json-v1 SHA-256 envelope",
    );
  }
  assertReferenceAcquisitionSha256(inventory.identity.sha256, "inventory.identity.sha256");
  const computed = computeReferenceAcquisitionInventoryIdentity(inventory);
  if (inventory.identity.sha256 !== computed) {
    throw new Error(
      `inventory identity mismatch: expected ${computed}, got ${inventory.identity.sha256}`,
    );
  }
  return `${stableStringify(inventory)}\n`;
}

export function buildReferenceAcquisitionInventory({
  manifestBytes,
  summaryBytes,
  family,
  sourceOrigin,
  shardCount,
  expectedCount,
  expectedManifestSha256,
  expectedSummarySha256,
}) {
  if (!REFERENCE_ACQUISITION_FAMILY_RE.test(family ?? "")) {
    throw new Error("family must be an uppercase branch identifier");
  }
  if (
    !Number.isSafeInteger(shardCount) ||
    shardCount < 1 ||
    shardCount > 4096
  ) {
    throw new Error("shardCount must be an integer from 1 through 4096");
  }
  if (!Number.isSafeInteger(expectedCount) || expectedCount < 1) {
    throw new Error("expectedCount must be a positive safe integer");
  }
  const { origin, sourceSite } = validateOrigin(sourceOrigin);
  const sourceBranch = family.toLowerCase();
  const manifest = parseReferenceAcquisitionManifest(
    manifestBytes,
    expectedManifestSha256,
    expectedCount,
  );
  const parsedSummary = parseReferenceAcquisitionSummary(summaryBytes, expectedSummarySha256);
  const seenEntityIds = new Set();
  const context = {
    family,
    origin,
    seenAttachmentUrls: new Set(),
    seenSourceUrls: new Set(),
    sourceBranch,
    sourceSite,
  };
  const rows = manifest.rows.map((record) => normalizeRow(record, context));
  for (let index = 0; index < rows.length; index += 1) {
    const row = rows[index];
    if (
      index > 0 &&
      codePointCompare(rows[index - 1].fullname, row.fullname) >= 0
    ) {
      throw new Error(
        `manifest fullnames must be strictly sorted and unique at ${row.fullname}`,
      );
    }
    if (seenEntityIds.has(row.source_entity_id)) {
      throw new Error(`duplicate source_entity_id ${row.source_entity_id}`);
    }
    seenEntityIds.add(row.source_entity_id);
  }
  const attachmentCount = rows.reduce(
    (total, row) => total + row.attachment_count,
    0,
  );
  const attachmentPageCount = rows.filter(
    (row) => row.attachment_count > 0,
  ).length;
  validateAcquisitionSummary(parsedSummary.summary, {
    attachmentCount,
    attachmentPageCount,
    inputRows: manifest.rows.map((record) => record.input),
    manifestSha256: manifest.sha256,
    rows,
    sourceBranch,
    sourceSite,
  });
  const body = {
    family,
    host_paths_included: false,
    requested_layers: [...REFERENCE_ACQUISITION_REQUESTED_LAYERS],
    rows,
    schema: "wikijump_full_parity.reference_acquisition_inventory.v1",
    shard_count: shardCount,
    shards: buildShards(rows, family, shardCount),
    source_manifest: {
      attachment_count: attachmentCount,
      attachment_page_count: attachmentPageCount,
      first_fullname: rows[0].fullname,
      last_fullname: rows.at(-1).fullname,
      row_count: rows.length,
      sha256: manifest.sha256,
      summary_sha256: parsedSummary.sha256,
    },
    source_origin: origin,
  };
  return {
    ...body,
    identity: {
      algorithm: "sha256",
      canonicalization: "stable-json-v1",
      sha256: sha256Hex(stableStringify(body)),
    },
  };
}
