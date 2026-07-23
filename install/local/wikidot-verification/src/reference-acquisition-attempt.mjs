import { types as utilTypes } from "node:util";

import { sha256Hex, stableStringify } from "./canonical-json.mjs";
import { validateReferenceAcquisitionInventory } from "./reference-acquisition-inventory-validation.mjs";
import { validateReferenceObject } from "./reference-object-store.mjs";

export const REFERENCE_ACQUISITION_ATTEMPT_SCHEMA =
  "wikijump_full_parity.reference_acquisition_attempt.v1";
export const REFERENCE_ACQUISITION_ATTEMPT_MAX_BYTES = 1024 * 1024;

const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;
const TIMESTAMP_RE =
  /^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}Z$/u;
const IDENTIFIER_RE = /^[a-z][a-z0-9_.-]{0,127}$/u;
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

class ReferenceAcquisitionContext {
  constructor(inventory, { expectedIdentitySha256 } = {}) {
    const validated = validateReferenceAcquisitionInventory(inventory, {
      expectedIdentitySha256,
    });
    this.inventorySha256 = validated.identity.sha256;
    this.rows = Object.freeze(
      validated.rows.map((row) =>
        Object.freeze({
          fixtureId: row.fixture_id,
          fullname: row.fullname,
          layers: Object.freeze([...row.requested_layers]),
          ordinal: row.ordinal,
          baseline: Object.freeze({ ...row.baseline }),
          semanticRowSha256: row.semantic_row_sha256,
          sourceEntityId: row.source_entity_id,
          sourceUrl: row.source_url,
        }),
      ),
    );
    Object.freeze(this);
  }
}

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

function normalizeProducer(producer) {
  assertExactKeys(producer, ["contract", "identity"], "attempt.producer");
  const contract = producer.contract;
  const identity = producer.identity;
  if (!IDENTIFIER_RE.test(contract)) {
    throw new Error("attempt.producer.contract must be a stable identifier");
  }
  return Object.freeze({
    contract,
    identity: validateReferenceObject(identity),
  });
}

function normalizeFailure(failure, outcome) {
  if (outcome === "complete") {
    if (failure !== null)
      throw new Error("complete attempt failure must be null");
    return null;
  }
  assertExactKeys(failure, ["code", "retryable"], "attempt.failure");
  if (
    !IDENTIFIER_RE.test(failure.code) ||
    typeof failure.retryable !== "boolean"
  ) {
    throw new Error("attempt.failure is invalid");
  }
  return Object.freeze({ code: failure.code, retryable: failure.retryable });
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

function assertContext(context) {
  if (!(context instanceof ReferenceAcquisitionContext)) {
    throw new Error("reference acquisition context is required");
  }
}

function inventoryBinding(context, ordinal) {
  assertContext(context);
  if (
    !Number.isSafeInteger(ordinal) ||
    ordinal < 0 ||
    ordinal >= context.rows.length
  ) {
    throw new Error("attempt inventory ordinal is outside the inventory");
  }
  const row = context.rows[ordinal];
  return Object.freeze({
    fixture_id: row.fixtureId,
    ordinal,
    semantic_row_sha256: row.semanticRowSha256,
    sha256: context.inventorySha256,
  });
}

function computeWorkIdentity(context, binding, layer, producer) {
  if (!context.rows[binding.ordinal].layers.includes(layer)) {
    throw new Error("attempt layer is not requested by its inventory row");
  }
  const sha256 = sha256Hex(
    stableStringify({ inventory: binding, layer, producer }),
  );
  return Object.freeze({
    algorithm: "sha256",
    canonicalization: "stable-json-v1",
    sha256,
  });
}

function buildWorkTarget(context, layer, ordinal, producer) {
  const inventory = inventoryBinding(context, ordinal);
  return Object.freeze({
    inventory,
    layer,
    producer,
    work_identity: computeWorkIdentity(context, inventory, layer, producer),
  });
}

function normalizeLayerFilter(context, layers) {
  if (layers === undefined) return null;
  const available = new Set(context.rows[0].layers);
  if (!Array.isArray(layers) || utilTypes.isProxy(layers)) {
    throw new Error("acquisition layer filter must be a data array");
  }
  let keys;
  let lengthDescriptor;
  try {
    keys = Reflect.ownKeys(layers);
    lengthDescriptor = Reflect.getOwnPropertyDescriptor(layers, "length");
  } catch {
    throw new Error("acquisition layer filter must be a data array");
  }
  if (
    lengthDescriptor === undefined ||
    !("value" in lengthDescriptor) ||
    !Number.isSafeInteger(lengthDescriptor.value) ||
    lengthDescriptor.value < 1 ||
    lengthDescriptor.value > available.size ||
    keys.length !== lengthDescriptor.value + 1
  ) {
    throw new Error("acquisition layer filter must be a dense data array");
  }
  const selected = new Set();
  for (let index = 0; index < lengthDescriptor.value; index += 1) {
    const descriptor = Reflect.getOwnPropertyDescriptor(layers, String(index));
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor) ||
      typeof descriptor.value !== "string" ||
      !available.has(descriptor.value) ||
      selected.has(descriptor.value)
    ) {
      throw new Error(
        "acquisition layer filter contains an invalid or duplicate layer",
      );
    }
    selected.add(descriptor.value);
  }
  if (
    keys.some(
      (key) =>
        key !== "length" &&
        (typeof key !== "string" ||
          !Number.isSafeInteger(Number(key)) ||
          String(Number(key)) !== key ||
          Number(key) < 0 ||
          Number(key) >= lengthDescriptor.value),
    )
  ) {
    throw new Error("acquisition layer filter has unexpected fields");
  }
  return selected;
}

function snapshotWorkTargetListOptions(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error("work target list options must be a data object");
  }
  let prototype;
  let keys;
  try {
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
  } catch {
    throw new Error("work target list options must be a data object");
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some(
      (key) =>
        typeof key !== "string" ||
        !["context", "layers", "producer"].includes(key),
    ) ||
    !keys.includes("context") ||
    !keys.includes("producer") ||
    keys.length < 2 ||
    keys.length > 3
  ) {
    throw new Error("work target list options have unexpected fields");
  }
  const snapshot = {};
  for (const key of keys) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error("work target list options must contain data fields");
    }
    snapshot[key] = descriptor.value;
  }
  return snapshot;
}

export function buildReferenceAcquisitionWorkTarget({
  context,
  layer,
  ordinal,
  producer,
}) {
  return buildWorkTarget(context, layer, ordinal, normalizeProducer(producer));
}

export function listReferenceAcquisitionWorkTargets(options) {
  const { context, layers, producer } = snapshotWorkTargetListOptions(options);
  validateReferenceAcquisitionContext(context);
  const normalizedProducer = normalizeProducer(producer);
  const layerFilter = normalizeLayerFilter(context, layers);
  return Object.freeze(
    context.rows.flatMap((row) =>
      row.layers
        .filter((layer) => layerFilter === null || layerFilter.has(layer))
        .map((layer) =>
          buildWorkTarget(context, layer, row.ordinal, normalizedProducer),
        ),
    ),
  );
}

export function referenceAcquisitionInventorySha256(context) {
  assertContext(context);
  return context.inventorySha256;
}

export function referenceAcquisitionInventoryRow(context, ordinal) {
  assertContext(context);
  if (
    !Number.isSafeInteger(ordinal) ||
    ordinal < 0 ||
    ordinal >= context.rows.length
  ) {
    throw new Error(
      "reference acquisition row ordinal is outside the inventory",
    );
  }
  return context.rows[ordinal];
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
  const binding = inventoryBinding(context, attempt.inventory.ordinal);
  if (stableStringify(attempt.inventory) !== stableStringify(binding)) {
    throw new Error("attempt does not match its inventory row");
  }
  const producer = normalizeProducer(attempt.producer);
  const workIdentity = computeWorkIdentity(
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

export function createReferenceAcquisitionContext(inventory, options) {
  return new ReferenceAcquisitionContext(inventory, options);
}

export function validateReferenceAcquisitionContext(context) {
  assertContext(context);
  return context;
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
