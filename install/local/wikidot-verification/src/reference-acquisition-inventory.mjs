import { sha256Hex, stableStringify } from "./corpus-import-manifest.mjs";
import { normalizeAcquisitionAttachment } from "./reference-acquisition-attachment.mjs";
import { validateAcquisitionSummary } from "./reference-acquisition-summary.mjs";

const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const SHA256_RE = /^[0-9a-f]{64}$/u;
const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;
const FAMILY_RE = /^[A-Z][A-Z0-9-]*$/u;
const RFC3339_RE =
  /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(?:Z|[+-](\d{2}):(\d{2}))$/u;
const REQUESTED_LAYERS = Object.freeze([
  "xmlrpc_page",
  "http_document",
  "browser_document",
]);

function assertObject(value, label) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
}

function assertNonEmptyString(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} must be a non-empty string`);
  }
}

function assertSha256(value, label) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

function assertNonNegativeSafeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${label} must be a non-negative safe integer`);
  }
}

export function assertTimestamp(value, label) {
  assertNonEmptyString(value, label);
  const match = RFC3339_RE.exec(value);
  if (match === null) {
    throw new Error(`${label} must be an RFC 3339 date-time`);
  }
  const [
    ,
    yearText,
    monthText,
    dayText,
    hourText,
    minuteText,
    secondText,
    offsetHourText = "0",
    offsetMinuteText = "0",
  ] = match;
  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  const leapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth = [
    0,
    31,
    leapYear ? 29 : 28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31,
  ];
  if (
    month < 1 ||
    month > 12 ||
    day < 1 ||
    day > daysInMonth[month] ||
    Number(hourText) > 23 ||
    Number(minuteText) > 59 ||
    Number(secondText) > 59 ||
    Number(offsetHourText) > 23 ||
    Number(offsetMinuteText) > 59 ||
    Number.isNaN(Date.parse(value))
  ) {
    throw new Error(`${label} must be an RFC 3339 date-time`);
  }
}

export function assertCanonicalFullname(fullname, label) {
  assertNonEmptyString(fullname, label);
  if (
    fullname.includes("/") ||
    fullname.includes("\\") ||
    /[\u0000-\u001f\u007f]/u.test(fullname)
  ) {
    throw new Error(`${label} contains an unsafe path character`);
  }
  const url = new URL("https://example.invalid/");
  url.pathname = `/${fullname}`;
  let roundTrip;
  try {
    roundTrip = decodeURIComponent(url.pathname.slice(1));
  } catch (error) {
    throw new Error(
      `${label} contains invalid percent encoding: ${error.message}`,
    );
  }
  if (roundTrip !== fullname) {
    throw new Error(
      `${label} does not round-trip through a canonical URL path`,
    );
  }
}

function toBuffer(value, label) {
  if (typeof value === "string") {
    return Buffer.from(value, "utf8");
  }
  if (value instanceof Uint8Array) {
    return Buffer.from(value.buffer, value.byteOffset, value.byteLength);
  }
  throw new Error(`${label} must be a string or Uint8Array`);
}

function decodeUtf8(bytes, label) {
  try {
    return FATAL_UTF8_DECODER.decode(bytes);
  } catch (error) {
    throw new Error(`${label} must contain valid UTF-8: ${error.message}`);
  }
}

export function codePointCompare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function validateOrigin(sourceOrigin) {
  let parsed;
  try {
    parsed = new URL(sourceOrigin);
  } catch (error) {
    throw new Error(`sourceOrigin must be an absolute URL: ${error.message}`);
  }
  if (
    parsed.protocol !== "https:" ||
    parsed.port !== "" ||
    parsed.username !== "" ||
    parsed.password !== "" ||
    parsed.pathname !== "/" ||
    parsed.search !== "" ||
    parsed.hash !== ""
  ) {
    throw new Error("sourceOrigin must be a credential-free HTTPS origin");
  }
  const suffix = ".wikidot.com";
  if (
    !parsed.hostname.endsWith(suffix) ||
    parsed.hostname.length === suffix.length
  ) {
    throw new Error("sourceOrigin must identify a Wikidot site hostname");
  }
  return {
    origin: parsed.origin,
    sourceSite: parsed.hostname.slice(0, -suffix.length),
  };
}

function parseManifest(manifestBytes, expectedManifestSha256, expectedCount) {
  const bytes = toBuffer(manifestBytes, "manifestBytes");
  assertSha256(expectedManifestSha256, "expectedManifestSha256");
  const actualSha256 = sha256Hex(bytes);
  if (actualSha256 !== expectedManifestSha256) {
    throw new Error(
      `manifest SHA-256 mismatch: expected ${expectedManifestSha256}, got ${actualSha256}`,
    );
  }
  const text = decodeUtf8(bytes, "manifestBytes");
  if (!text.endsWith("\n") || text.includes("\r")) {
    throw new Error(
      "manifestBytes must use LF lines and end with exactly one LF",
    );
  }
  const lines = text.slice(0, -1).split("\n");
  if (lines.some((line) => line.length === 0)) {
    throw new Error(
      "manifestBytes must not contain blank lines or extra terminal LFs",
    );
  }
  if (lines.length !== expectedCount) {
    throw new Error(
      `manifest row count mismatch: expected ${expectedCount}, got ${lines.length}`,
    );
  }
  return {
    bytes,
    sha256: actualSha256,
    rows: lines.map((line, index) => {
      try {
        return {
          input: JSON.parse(line),
          inputLineSha256: sha256Hex(line),
          lineNumber: index + 1,
        };
      } catch (error) {
        throw new Error(
          `manifest line ${index + 1} is not valid JSON: ${error.message}`,
        );
      }
    }),
  };
}

function parseSummary(summaryBytes, expectedSummarySha256) {
  const bytes = toBuffer(summaryBytes, "summaryBytes");
  assertSha256(expectedSummarySha256, "expectedSummarySha256");
  const actualSha256 = sha256Hex(bytes);
  if (actualSha256 !== expectedSummarySha256) {
    throw new Error(
      `summary SHA-256 mismatch: expected ${expectedSummarySha256}, got ${actualSha256}`,
    );
  }
  let summary;
  try {
    summary = JSON.parse(decodeUtf8(bytes, "summaryBytes"));
  } catch (error) {
    throw new Error(
      `summaryBytes must contain one JSON document: ${error.message}`,
    );
  }
  assertObject(summary, "summary");
  return { summary, sha256: actualSha256 };
}

function normalizeRow(record, context) {
  const { input: row, inputLineSha256, lineNumber } = record;
  const rowLabel = `manifest line ${lineNumber}`;
  assertObject(row, rowLabel);
  assertCanonicalFullname(row.fullname, `${rowLabel}.fullname`);
  assertNonEmptyString(row.source_entity_id, `${rowLabel}.source_entity_id`);
  if (!UUID_RE.test(row.source_entity_id)) {
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
  assertSha256(row.source_sha256, `${rowLabel}.source_sha256`);
  assertSha256(row.meta_sha256, `${rowLabel}.meta_sha256`);
  assertTimestamp(row.updated_at, `${rowLabel}.updated_at`);
  assertNonNegativeSafeInteger(row.revisions, `${rowLabel}.revisions`);
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
    requested_layers: [...REQUESTED_LAYERS],
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
  assertObject(inventory, "inventory");
  const body = {...inventory};
  delete body.identity;
  return sha256Hex(stableStringify(body));
}

export function serializeReferenceAcquisitionInventory(inventory) {
  assertObject(inventory.identity, "inventory.identity");
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
  assertSha256(inventory.identity.sha256, "inventory.identity.sha256");
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
  if (!FAMILY_RE.test(family ?? "")) {
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
  const manifest = parseManifest(
    manifestBytes,
    expectedManifestSha256,
    expectedCount,
  );
  const parsedSummary = parseSummary(summaryBytes, expectedSummarySha256);
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
    requested_layers: [...REQUESTED_LAYERS],
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
