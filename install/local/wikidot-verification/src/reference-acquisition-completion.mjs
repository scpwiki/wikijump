import { types as utilTypes } from "node:util";

import { stableStringify } from "./canonical-json.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
  listReferenceAcquisitionWorkTargets,
  readReferenceAcquisitionAttempt,
  readReferenceAcquisitionAttemptReceipt,
  validateReferenceAcquisitionContext,
} from "./reference-acquisition-attempt.mjs";
import {
  prepareReferenceCompletionIndex,
  REFERENCE_COMPLETION_POINTER_MAX_BYTES,
} from "./reference-acquisition-completion-index.mjs";
import {
  isReferenceObjectStore,
  validateReferenceObject,
} from "./reference-object-store.mjs";

export const REFERENCE_ACQUISITION_COMPLETION_POINTER_SCHEMA =
  "wikijump_full_parity.reference_acquisition_completion_pointer.v1";

const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const SHA256_RE = /^[0-9a-f]{64}$/u;
const POINTER_KEYS = Object.freeze(["attempt", "schema", "work_identity"]);
const STORE_OPENERS = new WeakMap();
const REFERENCE_COMPLETION_PLAN_BATCH_SIZE = 16;

export function registerReferenceAcquisitionCompletionStore(store, options) {
  const required = [
    "assertDirectoryBinding",
    "assertStoreBindings",
    "openDirectoryAt",
    "readAndHashFileAt",
  ];
  if (
    !isReferenceObjectStore(store) ||
    !Number.isInteger(options?.root?.fd) ||
    options.root.fd < 0 ||
    required.some((name) => typeof options[name] !== "function") ||
    STORE_OPENERS.has(store)
  ) {
    throw new Error("reference completion store registration is invalid");
  }
  STORE_OPENERS.set(store, (create) =>
    prepareReferenceCompletionIndex(options.root, { ...options, create }),
  );
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

function normalizeWorkIdentity(value) {
  assertExactKeys(
    value,
    ["algorithm", "canonicalization", "sha256"],
    "completion work identity",
  );
  if (
    value.algorithm !== "sha256" ||
    value.canonicalization !== "stable-json-v1" ||
    !SHA256_RE.test(value.sha256)
  ) {
    throw new Error("completion work identity is invalid");
  }
  return Object.freeze({ ...value });
}

function normalizePointer(pointer, expectedWorkIdentity) {
  assertExactKeys(pointer, POINTER_KEYS, "completion pointer");
  if (pointer.schema !== REFERENCE_ACQUISITION_COMPLETION_POINTER_SCHEMA) {
    throw new Error("completion pointer schema is invalid");
  }
  const workIdentity = normalizeWorkIdentity(pointer.work_identity);
  const expected = normalizeWorkIdentity(expectedWorkIdentity);
  if (stableStringify(workIdentity) !== stableStringify(expected)) {
    throw new Error("completion pointer has the wrong work identity");
  }
  return Object.freeze({
    attempt: validateReferenceObject(pointer.attempt),
    schema: REFERENCE_ACQUISITION_COMPLETION_POINTER_SCHEMA,
    work_identity: workIdentity,
  });
}

export function serializeReferenceAcquisitionCompletionPointer(
  pointer,
  expectedWorkIdentity,
) {
  const normalized = normalizePointer(pointer, expectedWorkIdentity);
  return Buffer.from(`${stableStringify(normalized)}\n`);
}

export function parseReferenceAcquisitionCompletionPointer(
  value,
  expectedWorkIdentity,
) {
  const bytes = Buffer.from(value);
  if (bytes.byteLength > REFERENCE_COMPLETION_POINTER_MAX_BYTES) {
    throw new Error("completion pointer exceeds its byte limit");
  }
  let text;
  try {
    text = FATAL_UTF8_DECODER.decode(bytes);
  } catch {
    throw new Error("completion pointer must be valid UTF-8");
  }
  if (
    !text.endsWith("\n") ||
    text.slice(0, -1).includes("\n") ||
    text.includes("\r")
  ) {
    throw new Error("completion pointer must contain one canonical JSON line");
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("completion pointer must contain valid JSON");
  }
  const normalized = normalizePointer(parsed, expectedWorkIdentity);
  if (
    !serializeReferenceAcquisitionCompletionPointer(
      normalized,
      expectedWorkIdentity,
    ).equals(bytes)
  ) {
    throw new Error("completion pointer bytes are not canonical");
  }
  return normalized;
}

export function referenceAcquisitionCompletionRelativePath(target) {
  const digest = normalizeWorkIdentity(target.work_identity).sha256;
  return `completions/sha256/${digest.slice(0, 2)}/${digest}`;
}

function assertAttemptMatchesTarget(attempt, target) {
  if (attempt.outcome !== "complete" || attempt.failure !== null) {
    throw new Error("only complete acquisition attempts can satisfy work");
  }
  for (const key of ["inventory", "layer", "producer", "work_identity"]) {
    if (stableStringify(attempt[key]) !== stableStringify(target[key])) {
      throw new Error(`completion attempt has the wrong ${key}`);
    }
  }
}

function sameReference(left, right) {
  return stableStringify(left) === stableStringify(right);
}

function snapshotResumeOptions(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error("completion resume options must be a data object");
  }
  let prototype;
  let keys;
  try {
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
  } catch {
    throw new Error("completion resume options must be a data object");
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some(
      (key) => typeof key !== "string" || !["layers", "producer"].includes(key),
    ) ||
    !keys.includes("producer") ||
    keys.length < 1 ||
    keys.length > 2
  ) {
    throw new Error("completion resume options have unexpected fields");
  }
  const snapshot = {};
  for (const key of keys) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error("completion resume options must contain data fields");
    }
    snapshot[key] = descriptor.value;
  }
  return snapshot;
}

export class ReferenceAcquisitionCompletionConflictError extends Error {
  constructor(existing, attempted) {
    super("a different complete attempt already owns this work identity");
    this.code = "REFERENCE_ACQUISITION_COMPLETION_CONFLICT";
    this.existing = existing;
    this.attempted = attempted;
  }
}

class ReferenceAcquisitionCompletions {
  #context;
  #index;
  #store;

  constructor(store, index, context) {
    this.#context = context;
    this.#index = index;
    this.#store = store;
  }

  #target({ layer, ordinal, producer }) {
    return buildReferenceAcquisitionWorkTarget({
      context: this.#context,
      layer,
      ordinal,
      producer,
    });
  }

  #readAttempt(reference, verifyObjects) {
    const reader = verifyObjects
      ? readReferenceAcquisitionAttempt
      : readReferenceAcquisitionAttemptReceipt;
    return reader(this.#store, reference, this.#context);
  }

  async #resolveTargetBytes(target, bytes, verifyObjects) {
    if (bytes === null) return null;
    const pointer = parseReferenceAcquisitionCompletionPointer(
      bytes,
      target.work_identity,
    );
    const attempt = await this.#readAttempt(pointer.attempt, verifyObjects);
    assertAttemptMatchesTarget(attempt, target);
    return Object.freeze({
      attempt,
      attempt_reference: pointer.attempt,
      target,
    });
  }

  async #resolveTarget(target, verifyObjects) {
    return this.#resolveTargetBytes(
      target,
      await this.#index.read(target.work_identity.sha256),
      verifyObjects,
    );
  }

  async resolve(request) {
    return this.#resolveTarget(this.#target(request), true);
  }

  // Typed layer wrappers inspect canonical attempt receipts before reading referenced objects with role-specific byte bounds.
  async resolveAttemptReceipt(request) {
    return this.#resolveTarget(this.#target(request), false);
  }

  async publish(attemptReference, request) {
    const target = this.#target(request);
    const attemptedReference = validateReferenceObject(attemptReference);
    const attempt = await readReferenceAcquisitionAttempt(
      this.#store,
      attemptedReference,
      this.#context,
    );
    assertAttemptMatchesTarget(attempt, target);
    const pointer = {
      attempt: attemptedReference,
      schema: REFERENCE_ACQUISITION_COMPLETION_POINTER_SCHEMA,
      work_identity: target.work_identity,
    };
    const publication = await this.#index.publish(
      target.work_identity.sha256,
      serializeReferenceAcquisitionCompletionPointer(
        pointer,
        target.work_identity,
      ),
    );
    const visible = parseReferenceAcquisitionCompletionPointer(
      publication.bytes,
      target.work_identity,
    );
    const winner = await readReferenceAcquisitionAttempt(
      this.#store,
      visible.attempt,
      this.#context,
    );
    assertAttemptMatchesTarget(winner, target);
    if (!sameReference(visible.attempt, attemptedReference)) {
      throw new ReferenceAcquisitionCompletionConflictError(
        visible.attempt,
        attemptedReference,
      );
    }
    return Object.freeze({
      attempt: winner,
      attempt_reference: visible.attempt,
      disposition: publication.disposition,
      target,
    });
  }

  async #planResume(options, verifyObjects) {
    const { layers, producer } = snapshotResumeOptions(options);
    const targets = listReferenceAcquisitionWorkTargets({
      context: this.#context,
      layers,
      producer,
    });
    if (verifyObjects)
      await this.#store.verifyObject(targets[0].producer.identity);
    const visible = await this.#index.readMany(
      targets.map((target) => target.work_identity.sha256),
    );
    const complete = [];
    const pending = [];
    for (
      let offset = 0;
      offset < targets.length;
      offset += REFERENCE_COMPLETION_PLAN_BATCH_SIZE
    ) {
      const batch = targets.slice(
        offset,
        offset + REFERENCE_COMPLETION_PLAN_BATCH_SIZE,
      );
      const results = await Promise.allSettled(
        batch.map((target, index) => {
          const record = visible[offset + index];
          return "error" in record
            ? Promise.reject(record.error)
            : this.#resolveTargetBytes(target, record.bytes, verifyObjects);
        }),
      );
      for (let index = 0; index < results.length; index += 1) {
        const result = results[index];
        if (result.status === "rejected") throw result.reason;
        if (result.value === null) pending.push(batch[index]);
        else complete.push(result.value);
      }
    }
    return Object.freeze({
      complete: Object.freeze(complete),
      pending: Object.freeze(pending),
    });
  }

  async planResume(options) {
    return this.#planResume(options, true);
  }

  async planResumeAttemptReceipts(options) {
    return this.#planResume(options, false);
  }

  close() {
    return this.#index.close();
  }
}

async function prepare(store, context, create) {
  validateReferenceAcquisitionContext(context);
  const opener = STORE_OPENERS.get(store);
  if (opener === undefined) {
    throw new Error("registered reference object store is required");
  }
  const index = await opener(create);
  return new ReferenceAcquisitionCompletions(store, index, context);
}

export function initializeReferenceAcquisitionCompletions(store, context) {
  return prepare(store, context, true);
}

export function openReferenceAcquisitionCompletions(store, context) {
  return prepare(store, context, false);
}
