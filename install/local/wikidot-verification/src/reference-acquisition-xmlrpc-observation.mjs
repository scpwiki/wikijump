import { sha256Hex, stableStringify } from "./canonical-json.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
  referenceAcquisitionInventoryRow,
} from "./reference-acquisition-attempt.mjs";
import { assertTimestamp } from "./reference-acquisition-inventory.mjs";
import { validateReferenceObject } from "./reference-object-store.mjs";

export const WIKIDOT_XMLRPC_OBSERVATION_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_observation.v1";
export const WIKIDOT_XMLRPC_DELETED_TOMBSTONE_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_deleted_tombstone.v1";
export const WIKIDOT_XMLRPC_DELETED_TOMBSTONE_ROLE = "deleted_tombstone";
export const WIKIDOT_XMLRPC_PRODUCER_CONTRACT =
  "wikijump_full_parity.wikidot_xmlrpc_acquirer.v1";
export const WIKIDOT_XMLRPC_OBSERVATION_MAX_BYTES = 64 * 1024;
export const WIKIDOT_XMLRPC_DELETED_TOMBSTONE_MAX_BYTES = 16 * 1024;
export const WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES = 32 * 1024 * 1024;

const CAPTURE_TIMESTAMP_RE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/u;
const ENDPOINT = "https://www.wikidot.com/xml-rpc-api.php";
const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const MAX_JSON_DEPTH = 64;
const MAX_JSON_NODES = 1_000_000;
const METHOD = "pages.get_one";
const SITE = "scp-wiki";

function assertObject(value, label) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
}

function assertCaptureTimestamp(value, label) {
  if (
    typeof value !== "string" ||
    !CAPTURE_TIMESTAMP_RE.test(value) ||
    new Date(value).toISOString() !== value
  ) {
    throw new Error(`${label} must be a canonical UTC millisecond timestamp`);
  }
}

function assertWellFormedString(value) {
  for (let index = 0; index < value.length; index += 1) {
    const current = value.charCodeAt(index);
    if (current >= 0xd800 && current <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) {
        throw new Error("XML-RPC response contains ill-formed Unicode");
      }
      index += 1;
    } else if (current >= 0xdc00 && current <= 0xdfff) {
      throw new Error("XML-RPC response contains ill-formed Unicode");
    }
  }
  return value;
}

function snapshotJsonValue(value, state = { nodes: 0 }, depth = 0) {
  state.nodes += 1;
  if (state.nodes > MAX_JSON_NODES || depth > MAX_JSON_DEPTH) {
    throw new Error("XML-RPC response exceeds its structural limit");
  }
  if (value === null || typeof value === "boolean") return value;
  if (typeof value === "string") return assertWellFormedString(value);
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw new Error("XML-RPC response contains a non-finite number");
    }
    return Object.is(value, -0) ? 0 : value;
  }
  if (Array.isArray(value)) {
    const keys = Reflect.ownKeys(value);
    const lengthDescriptor = Reflect.getOwnPropertyDescriptor(value, "length");
    if (
      lengthDescriptor === undefined ||
      !("value" in lengthDescriptor) ||
      !Number.isSafeInteger(lengthDescriptor.value) ||
      lengthDescriptor.value < 0 ||
      lengthDescriptor.value > MAX_JSON_NODES ||
      keys.length !== lengthDescriptor.value + 1
    ) {
      throw new Error("XML-RPC response contains a non-JSON array");
    }
    const remaining = new Set(keys);
    remaining.delete("length");
    const snapshot = [];
    for (let index = 0; index < lengthDescriptor.value; index += 1) {
      const key = String(index);
      const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
      if (
        !remaining.delete(key) ||
        descriptor === undefined ||
        !descriptor.enumerable ||
        !("value" in descriptor)
      ) {
        throw new Error("XML-RPC response contains a non-JSON array");
      }
      snapshot.push(snapshotJsonValue(descriptor.value, state, depth + 1));
    }
    if (remaining.size !== 0) {
      throw new Error("XML-RPC response contains a non-JSON array");
    }
    return Object.freeze(snapshot);
  }
  if (typeof value !== "object") {
    throw new Error("XML-RPC response contains a non-JSON value");
  }
  assertObject(value, "XML-RPC response value");
  const prototype = Object.getPrototypeOf(value);
  if (prototype !== Object.prototype && prototype !== null) {
    throw new Error("XML-RPC response contains a non-JSON object");
  }
  const snapshot = {};
  for (const key of Reflect.ownKeys(value)) {
    if (typeof key !== "string") {
      throw new Error("XML-RPC response contains a non-JSON object");
    }
    assertWellFormedString(key);
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error("XML-RPC response contains a non-JSON object");
    }
    Object.defineProperty(snapshot, key, {
      enumerable: true,
      value: snapshotJsonValue(descriptor.value, state, depth + 1),
    });
  }
  return Object.freeze(snapshot);
}

function canonicalJsonLineFromSnapshot(value, maxBytes, label) {
  const bytes = Buffer.from(`${stableStringify(value)}\n`, "utf8");
  if (bytes.byteLength > maxBytes) {
    throw new Error(`${label} exceeds its byte limit`);
  }
  return bytes;
}

function parseCanonicalJsonLine(value, maxBytes, label) {
  const bytes = Buffer.from(value);
  if (bytes.byteLength > maxBytes)
    throw new Error(`${label} exceeds its byte limit`);
  let text;
  try {
    text = FATAL_UTF8_DECODER.decode(bytes);
  } catch {
    throw new Error(`${label} must be valid UTF-8`);
  }
  if (
    !text.endsWith("\n") ||
    text.slice(0, -1).includes("\n") ||
    text.includes("\r")
  ) {
    throw new Error(`${label} must contain one canonical JSON line`);
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error(`${label} must contain valid JSON`);
  }
  const snapshot = snapshotJsonValue(parsed);
  const canonical = canonicalJsonLineFromSnapshot(snapshot, maxBytes, label);
  if (!canonical.equals(bytes))
    throw new Error(`${label} bytes are not canonical`);
  return snapshot;
}

function timestampParts(value) {
  assertTimestamp(value, "observed updated_at");
  const match =
    /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})(?:\.(\d+))?(Z|[+-]\d{2}:\d{2})$/u.exec(
      value,
    );
  const milliseconds = Date.parse(`${match[1]}${match[3]}`);
  return {
    fraction: (match[2] ?? "").replace(/0+$/u, ""),
    seconds: BigInt(Math.trunc(milliseconds / 1000)),
  };
}

function compareTimestamps(left, right) {
  const leftParts = timestampParts(left);
  const rightParts = timestampParts(right);
  if (leftParts.seconds !== rightParts.seconds) {
    return leftParts.seconds < rightParts.seconds ? -1 : 1;
  }
  const width = Math.max(leftParts.fraction.length, rightParts.fraction.length);
  const leftFraction = leftParts.fraction.padEnd(width, "0");
  const rightFraction = rightParts.fraction.padEnd(width, "0");
  return leftFraction < rightFraction
    ? -1
    : leftFraction > rightFraction
      ? 1
      : 0;
}

function normalizeResponse(response, expectedFullname) {
  const snapshot = snapshotJsonValue(response);
  assertObject(snapshot, "XML-RPC response");
  if (snapshot.fullname !== expectedFullname) {
    throw new Error("XML-RPC response fullname does not match its target");
  }
  if (
    typeof snapshot.content !== "string" ||
    typeof snapshot.html !== "string"
  ) {
    throw new Error("XML-RPC response must contain string content and html");
  }
  if (!Number.isSafeInteger(snapshot.revisions) || snapshot.revisions < 0) {
    throw new Error(
      "XML-RPC response revisions must be a non-negative safe integer",
    );
  }
  assertTimestamp(snapshot.updated_at, "XML-RPC response updated_at");
  return snapshot;
}

function baselineRelation(row, response) {
  const sourceSha256 = sha256Hex(Buffer.from(response.content, "utf8"));
  const mismatchedFields = [];
  if (response.revisions !== row.baseline.revisions)
    mismatchedFields.push("revisions");
  if (sourceSha256 !== row.baseline.source_sha256)
    mismatchedFields.push("source_sha256");
  const timestampOrder = compareTimestamps(
    response.updated_at,
    row.baseline.updated_at,
  );
  if (timestampOrder !== 0) mismatchedFields.push("updated_at");
  let classification = "exact";
  if (mismatchedFields.length > 0) {
    const sameSource = sourceSha256 === row.baseline.source_sha256;
    const advanced =
      timestampOrder >= 0 &&
      (response.revisions > row.baseline.revisions ||
        (sameSource &&
          response.revisions === row.baseline.revisions &&
          timestampOrder > 0));
    classification = advanced
      ? sameSource
        ? "advanced_same_source"
        : "advanced_changed_source"
      : "identity_discontinuity_or_regression";
  }
  return Object.freeze({
    classification,
    mismatched_fields: Object.freeze(mismatchedFields),
  });
}

function expectedObservation({
  target,
  row,
  response,
  responseReference,
  startedAt,
  finishedAt,
}) {
  const normalizedResponse = normalizeResponse(response, row.fullname);
  const normalizedReference = validateReferenceObject(responseReference);
  const responseBytes = canonicalJsonLineFromSnapshot(
    normalizedResponse,
    WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES,
    "XML-RPC response",
  );
  if (
    normalizedReference.bytes !== responseBytes.byteLength ||
    normalizedReference.sha256 !== sha256Hex(responseBytes)
  ) {
    throw new Error(
      "XML-RPC response reference does not match its canonical bytes",
    );
  }
  assertCaptureTimestamp(startedAt, "capture.started_at");
  assertCaptureTimestamp(finishedAt, "capture.finished_at");
  if (finishedAt < startedAt)
    throw new Error("XML-RPC capture finished before it started");
  return Object.freeze({
    baseline: row.baseline,
    baseline_relation: baselineRelation(row, normalizedResponse),
    capture: Object.freeze({
      finished_at: finishedAt,
      started_at: startedAt,
    }),
    endpoint: ENDPOINT,
    fallback_used: false,
    fullname: row.fullname,
    inventory: target.inventory,
    method: METHOD,
    observed: Object.freeze({
      content_sha256: sha256Hex(
        Buffer.from(normalizedResponse.content, "utf8"),
      ),
      html_sha256: sha256Hex(Buffer.from(normalizedResponse.html, "utf8")),
      revisions: normalizedResponse.revisions,
      updated_at: normalizedResponse.updated_at,
    }),
    raw_wire_captured: false,
    read_only: true,
    response: normalizedReference,
    schema: WIKIDOT_XMLRPC_OBSERVATION_SCHEMA,
    site: SITE,
    source_entity_id: row.sourceEntityId,
    source_url: row.sourceUrl,
    work_identity: target.work_identity,
  });
}

function expectedDeletedTombstone({ target, row, startedAt, finishedAt }) {
  assertCaptureTimestamp(startedAt, "capture.started_at");
  assertCaptureTimestamp(finishedAt, "capture.finished_at");
  if (finishedAt < startedAt)
    throw new Error("XML-RPC capture finished before it started");
  return Object.freeze({
    baseline: row.baseline,
    capture: Object.freeze({
      finished_at: finishedAt,
      started_at: startedAt,
    }),
    classification: "wikidot_deleted",
    endpoint: ENDPOINT,
    fallback_used: false,
    fullname: row.fullname,
    inventory: target.inventory,
    method: METHOD,
    raw_wire_captured: false,
    read_only: true,
    schema: WIKIDOT_XMLRPC_DELETED_TOMBSTONE_SCHEMA,
    site: SITE,
    source_entity_id: row.sourceEntityId,
    source_url: row.sourceUrl,
    work_identity: target.work_identity,
  });
}

function xmlrpcTarget(context, ordinal, producer) {
  const target = buildReferenceAcquisitionWorkTarget({
    context,
    layer: "xmlrpc_page",
    ordinal,
    producer,
  });
  if (target.producer.contract !== WIKIDOT_XMLRPC_PRODUCER_CONTRACT) {
    throw new Error("XML-RPC producer contract is invalid");
  }
  return target;
}

export function serializeWikidotXmlrpcResponse(response, expectedFullname) {
  return canonicalJsonLineFromSnapshot(
    normalizeResponse(response, expectedFullname),
    WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES,
    "XML-RPC response",
  );
}

export function parseWikidotXmlrpcResponse(value, expectedFullname) {
  return normalizeResponse(
    parseCanonicalJsonLine(
      value,
      WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES,
      "XML-RPC response",
    ),
    expectedFullname,
  );
}

export function buildWikidotXmlrpcObservation({
  context,
  finishedAt,
  ordinal,
  producer,
  response,
  responseReference,
  startedAt,
}) {
  const target = xmlrpcTarget(context, ordinal, producer);
  return expectedObservation({
    target,
    row: referenceAcquisitionInventoryRow(context, ordinal),
    response,
    responseReference,
    startedAt,
    finishedAt,
  });
}

export function serializeWikidotXmlrpcObservation(observation, input) {
  const snapshot = snapshotJsonValue(observation);
  const expected = buildWikidotXmlrpcObservation(input);
  if (stableStringify(snapshot) !== stableStringify(expected)) {
    throw new Error(
      "XML-RPC observation metadata does not match its response and target",
    );
  }
  return canonicalJsonLineFromSnapshot(
    expected,
    WIKIDOT_XMLRPC_OBSERVATION_MAX_BYTES,
    "XML-RPC observation",
  );
}

export function parseWikidotXmlrpcObservation(value, input) {
  const parsed = parseCanonicalJsonLine(
    value,
    WIKIDOT_XMLRPC_OBSERVATION_MAX_BYTES,
    "XML-RPC observation",
  );
  const expected = buildWikidotXmlrpcObservation(input);
  if (stableStringify(parsed) !== stableStringify(expected)) {
    throw new Error(
      "XML-RPC observation metadata does not match its response and target",
    );
  }
  return expected;
}

export function buildWikidotXmlrpcDeletedTombstone({
  context,
  finishedAt,
  ordinal,
  producer,
  startedAt,
}) {
  const target = xmlrpcTarget(context, ordinal, producer);
  return expectedDeletedTombstone({
    target,
    row: referenceAcquisitionInventoryRow(context, ordinal),
    startedAt,
    finishedAt,
  });
}

export function serializeWikidotXmlrpcDeletedTombstone(tombstone, input) {
  const snapshot = snapshotJsonValue(tombstone);
  const expected = buildWikidotXmlrpcDeletedTombstone(input);
  if (stableStringify(snapshot) !== stableStringify(expected)) {
    throw new Error(
      "XML-RPC deleted tombstone metadata does not match its target",
    );
  }
  return canonicalJsonLineFromSnapshot(
    expected,
    WIKIDOT_XMLRPC_DELETED_TOMBSTONE_MAX_BYTES,
    "XML-RPC deleted tombstone",
  );
}

export function parseWikidotXmlrpcDeletedTombstone(value, input) {
  const parsed = parseCanonicalJsonLine(
    value,
    WIKIDOT_XMLRPC_DELETED_TOMBSTONE_MAX_BYTES,
    "XML-RPC deleted tombstone",
  );
  const expected = buildWikidotXmlrpcDeletedTombstone(input);
  if (stableStringify(parsed) !== stableStringify(expected)) {
    throw new Error(
      "XML-RPC deleted tombstone metadata does not match its target",
    );
  }
  return expected;
}
