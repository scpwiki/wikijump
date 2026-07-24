import {types as utilTypes} from "node:util";

import {sha256Hex, stableStringify} from "./canonical-json.mjs";
import {validateReferenceAcquisitionInventory} from "./reference-acquisition-inventory-validation.mjs";
import {validateReferenceObject} from "./reference-object-store.mjs";

const IDENTIFIER_RE = /^[a-z][a-z0-9_.-]{0,127}$/u;

class ReferenceAcquisitionContext {
  constructor(inventory, {expectedIdentitySha256} = {}) {
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
          baseline: Object.freeze({...row.baseline}),
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

export function normalizeReferenceAcquisitionProducer(producer) {
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

function assertContext(context) {
  if (!(context instanceof ReferenceAcquisitionContext)) {
    throw new Error("reference acquisition context is required");
  }
}

export function referenceAcquisitionInventoryBinding(context, ordinal) {
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

export function computeReferenceAcquisitionWorkIdentity(
  context,
  binding,
  layer,
  producer,
) {
  if (!context.rows[binding.ordinal].layers.includes(layer)) {
    throw new Error("attempt layer is not requested by its inventory row");
  }
  const sha256 = sha256Hex(
    stableStringify({inventory: binding, layer, producer}),
  );
  return Object.freeze({
    algorithm: "sha256",
    canonicalization: "stable-json-v1",
    sha256,
  });
}

function buildWorkTarget(context, layer, ordinal, producer) {
  const inventory = referenceAcquisitionInventoryBinding(context, ordinal);
  return Object.freeze({
    inventory,
    layer,
    producer,
    work_identity: computeReferenceAcquisitionWorkIdentity(
      context,
      inventory,
      layer,
      producer,
    ),
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
  return buildWorkTarget(
    context,
    layer,
    ordinal,
    normalizeReferenceAcquisitionProducer(producer),
  );
}

export function listReferenceAcquisitionWorkTargets(options) {
  const {context, layers, producer} = snapshotWorkTargetListOptions(options);
  validateReferenceAcquisitionContext(context);
  const normalizedProducer = normalizeReferenceAcquisitionProducer(producer);
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

export function createReferenceAcquisitionContext(inventory, options) {
  return new ReferenceAcquisitionContext(inventory, options);
}

export function validateReferenceAcquisitionContext(context) {
  assertContext(context);
  return context;
}
