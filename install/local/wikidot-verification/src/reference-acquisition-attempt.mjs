import {stableStringify} from "./canonical-json.mjs";
import {validateReferenceObject} from "./reference-object-store.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
  computeReferenceAcquisitionWorkIdentity,
  normalizeReferenceAcquisitionProducer,
  referenceAcquisitionInventoryBinding,
} from "./reference-acquisition-work-target.mjs";

export {
  buildReferenceAcquisitionWorkTarget,
  createReferenceAcquisitionContext,
  listReferenceAcquisitionWorkTargets,
  referenceAcquisitionInventoryRow,
  referenceAcquisitionInventorySha256,
  validateReferenceAcquisitionContext,
} from "./reference-acquisition-work-target.mjs";

export const REFERENCE_ACQUISITION_ATTEMPT_SCHEMA =
  "wikijump_full_parity.reference_acquisition_attempt.v1";
export const REFERENCE_ACQUISITION_ATTEMPT_MAX_BYTES = 1024 * 1024;

const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;
const TIMESTAMP_RE =
  /^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}Z$/u;
const ROLE_RE = /^[a-z][a-z0-9_]{0,63}$/u;
const MEDIA_TYPE_RE =
  /^[a-z0-9][a-z0-9!#$&^_.+-]{0,63}\/[a-z0-9][a-z0-9!#$&^_.+-]{0,63}$/u;
const ATTEMPT_KEYS = Object.freeze([
  "attempt_id",
  "failure",
  "finished_at",
  "inventory",
  "layer",
  "objects",
  "outcome",
  "producer",
  "schema",
  "started_at",
  "work_identity",
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

function assertTimestamp(value, label) {
  if (typeof value !== "string" || !TIMESTAMP_RE.test(value)) {
    throw new Error(`${label} must be a canonical UTC timestamp`);
  }
  let canonical;
  try {
    canonical = new Date(value).toISOString();
  } catch {
    canonical = null;
  }
  if (canonical !== value) {
    throw new Error(`${label} must be a canonical UTC timestamp`);
  }
}

function normalizeFailure(failure, outcome) {
  if (outcome === "complete") {
    if (failure !== null)
      throw new Error("complete attempt failure must be null");
    return null;
  }
  assertExactKeys(failure, ["code", "retryable"], "attempt.failure");
  if (
    !/^[a-z][a-z0-9_.-]{0,127}$/u.test(failure.code) ||
    typeof failure.retryable !== "boolean"
  ) {
    throw new Error("attempt.failure is invalid");
  }
  return Object.freeze({code: failure.code, retryable: failure.retryable});
}

function normalizeObjects(objects, outcome) {
  if (
    !Array.isArray(objects) ||
    objects.length > 1024 ||
    (outcome === "complete" && objects.length === 0)
  ) {
    throw new Error(
      "attempt objects must be an array of at most 1024 items; complete attempts require at least one object",
    );
  }
  const roles = new Set();
  const normalized = objects.map((binding, index) => {
    const label = `attempt.objects[${index}]`;
    assertExactKeys(binding, ["media_type", "object", "role"], label);
    if (!ROLE_RE.test(binding.role) || roles.has(binding.role)) {
      throw new Error(`${label}.role is invalid or duplicated`);
    }
    if (!MEDIA_TYPE_RE.test(binding.media_type)) {
      throw new Error(`${label}.media_type must be a normalized media type`);
    }
    roles.add(binding.role);
    return Object.freeze({
      media_type: binding.media_type,
      object: validateReferenceObject(binding.object),
      role: binding.role,
    });
  });
  normalized.sort((left, right) =>
    left.role < right.role ? -1 : left.role > right.role ? 1 : 0,
  );
  return Object.freeze(normalized);
}

function normalizeAttempt(attempt, context) {
  assertExactKeys(attempt, ATTEMPT_KEYS, "attempt");
  if (attempt.schema !== REFERENCE_ACQUISITION_ATTEMPT_SCHEMA) {
    throw new Error("attempt schema is invalid");
  }
  if (!UUID_RE.test(attempt.attempt_id)) {
    throw new Error("attempt.attempt_id must be a lowercase UUID");
  }
  if (attempt.outcome !== "complete" && attempt.outcome !== "failed") {
    throw new Error("attempt.outcome must be complete or failed");
  }
  assertTimestamp(attempt.started_at, "attempt.started_at");
  assertTimestamp(attempt.finished_at, "attempt.finished_at");
  if (attempt.finished_at < attempt.started_at) {
    throw new Error("attempt finished before it started");
  }
  assertExactKeys(
    attempt.inventory,
    ["fixture_id", "ordinal", "semantic_row_sha256", "sha256"],
    "attempt.inventory",
  );
  const binding = referenceAcquisitionInventoryBinding(
    context,
    attempt.inventory.ordinal,
  );
  if (stableStringify(attempt.inventory) !== stableStringify(binding)) {
    throw new Error("attempt does not match its inventory row");
  }
  const producer = normalizeReferenceAcquisitionProducer(attempt.producer);
  const workIdentity = computeReferenceAcquisitionWorkIdentity(
    context,
    binding,
    attempt.layer,
    producer,
  );
  assertExactKeys(
    attempt.work_identity,
    ["algorithm", "canonicalization", "sha256"],
    "attempt.work_identity",
  );
  if (
    stableStringify(attempt.work_identity) !== stableStringify(workIdentity)
  ) {
    throw new Error("attempt work identity is invalid");
  }
  const normalized = Object.freeze({
    attempt_id: attempt.attempt_id,
    failure: normalizeFailure(attempt.failure, attempt.outcome),
    finished_at: attempt.finished_at,
    inventory: binding,
    layer: attempt.layer,
    objects: normalizeObjects(attempt.objects, attempt.outcome),
    outcome: attempt.outcome,
    producer,
    schema: REFERENCE_ACQUISITION_ATTEMPT_SCHEMA,
    started_at: attempt.started_at,
    work_identity: workIdentity,
  });
  if (stableStringify(normalized) !== stableStringify(attempt)) {
    throw new Error("attempt fields are not in canonical order or form");
  }
  return normalized;
}

export function buildReferenceAcquisitionAttempt({
  attemptId,
  context,
  failure = null,
  finishedAt,
  layer,
  objects,
  ordinal,
  outcome,
  producer,
  startedAt,
}) {
  const target = buildReferenceAcquisitionWorkTarget({
    context,
    layer,
    ordinal,
    producer,
  });
  return normalizeAttempt(
    {
      attempt_id: attemptId,
      failure,
      finished_at: finishedAt,
      inventory: target.inventory,
      layer,
      objects: normalizeObjects(objects, outcome),
      outcome,
      producer: target.producer,
      schema: REFERENCE_ACQUISITION_ATTEMPT_SCHEMA,
      started_at: startedAt,
      work_identity: target.work_identity,
    },
    context,
  );
}

export function validateReferenceAcquisitionAttempt(attempt, context) {
  return normalizeAttempt(attempt, context);
}

export function serializeReferenceAcquisitionAttempt(attempt, context) {
  const normalized = normalizeAttempt(attempt, context);
  return Buffer.from(`${stableStringify(normalized)}\n`, "utf8");
}

export function parseReferenceAcquisitionAttempt(value, context) {
  const bytes = Buffer.from(value);
  if (bytes.length > REFERENCE_ACQUISITION_ATTEMPT_MAX_BYTES) {
    throw new Error("attempt receipt exceeds its byte limit");
  }
  let text;
  try {
    text = FATAL_UTF8_DECODER.decode(bytes);
  } catch {
    throw new Error("attempt receipt must be valid UTF-8");
  }
  if (
    !text.endsWith("\n") ||
    text.slice(0, -1).includes("\n") ||
    text.includes("\r")
  ) {
    throw new Error("attempt receipt must contain one canonical JSON line");
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("attempt receipt must contain valid JSON");
  }
  const normalized = normalizeAttempt(parsed, context);
  if (
    !serializeReferenceAcquisitionAttempt(normalized, context).equals(bytes)
  ) {
    throw new Error("attempt receipt bytes are not canonical");
  }
  return normalized;
}

async function verifyReferencedObjects(store, attempt) {
  await store.verifyObject(attempt.producer.identity);
  for (const binding of attempt.objects)
    await store.verifyObject(binding.object);
}

export async function putReferenceAcquisitionAttempt(store, attempt, context) {
  const normalized = normalizeAttempt(attempt, context);
  await verifyReferencedObjects(store, normalized);
  const result = await store.putBytes(
    serializeReferenceAcquisitionAttempt(normalized, context),
  );
  return Object.freeze({ ...result, attempt: normalized });
}

export async function readReferenceAcquisitionAttempt(
  store,
  reference,
  context,
) {
  const bytes = await store.readObject(reference, {
    maxBytes: REFERENCE_ACQUISITION_ATTEMPT_MAX_BYTES,
  });
  const attempt = parseReferenceAcquisitionAttempt(bytes, context);
  await verifyReferencedObjects(store, attempt);
  return attempt;
}

export async function readReferenceAcquisitionAttemptReceipt(
  store,
  reference,
  context,
) {
  const bytes = await store.readObject(reference, {
    maxBytes: REFERENCE_ACQUISITION_ATTEMPT_MAX_BYTES,
  });
  return parseReferenceAcquisitionAttempt(bytes, context);
}
