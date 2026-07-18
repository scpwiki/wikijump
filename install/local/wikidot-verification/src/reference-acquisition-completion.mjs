import { publishBytesNoReplaceAt } from "./atomic-no-replace.mjs";
import { stableStringify } from "./corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
  readReferenceAcquisitionAttempt,
  validateReferenceAcquisitionContext,
} from "./reference-acquisition-attempt.mjs";
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
const REFERENCE_COMPLETION_POINTER_MAX_BYTES = 1024;
const COMPLETION_INDEX_DESCRIPTOR = Object.freeze({
  digest_encoding: "lowercase-hex",
  hash_algorithm: "sha256",
  pointer_encoding: "stable-json-v1-jsonl",
  pointer_path_template: "sha256/{prefix2}/{work_identity_sha256}",
  pointer_schema:
    "https://wikijump.org/schemas/reference-acquisition-completion-pointer-v1.schema.json",
  schema: "wikijump_full_parity.reference_acquisition_completion_index.v1",
});
const COMPLETION_INDEX_DESCRIPTOR_BYTES = Buffer.from(
  `${stableStringify(COMPLETION_INDEX_DESCRIPTOR)}\n`,
);

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

export class ReferenceAcquisitionCompletionConflictError extends Error {
  constructor(existing, attempted) {
    super("a different complete attempt already owns this work identity");
    this.code = "REFERENCE_ACQUISITION_COMPLETION_CONFLICT";
    this.existing = existing;
    this.attempted = attempted;
  }
}

function assertCompletionDigest(value) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error("work identity must be a lowercase SHA-256 digest");
  }
}

class ReferenceCompletionIndex {
  #handles;

  constructor(handles) {
    this.#handles = handles;
  }

  async #assertBindings() {
    const handles = this.#handles;
    if (handles === null) {
      throw new Error("reference completion index is closed");
    }
    await handles.assertStoreBindings();
    await handles.assertDirectoryBinding(
      handles.root,
      "completions",
      handles.completions,
      "completions",
    );
    await handles.assertDirectoryBinding(
      handles.completions,
      "sha256",
      handles.sha256,
      "completion sha256",
    );
    return handles;
  }

  async #openPrefix(handles, digest, create) {
    const name = digest.slice(0, 2);
    try {
      return await handles.openDirectoryAt(handles.sha256, name, {
        create,
        label: `completion prefix ${name}`,
      });
    } catch (error) {
      if (!create && error.code === "ENOENT") {
        await this.#assertBindings();
        return null;
      }
      throw error;
    }
  }

  #readVisible(handles, prefix, digest, allowMissing = false) {
    return handles.readAndHashFileAt(prefix, digest, {
      allowMissing,
      collect: true,
      expectedMode: 0o400,
      maxBytes: REFERENCE_COMPLETION_POINTER_MAX_BYTES,
      sizeMismatchMessage: `completion ${digest} exceeds its byte limit`,
    });
  }

  async read(digest) {
    assertCompletionDigest(digest);
    const handles = await this.#assertBindings();
    const prefix = await this.#openPrefix(handles, digest, false);
    if (prefix === null) return null;
    try {
      const visible = await this.#readVisible(handles, prefix, digest, true);
      await handles.assertDirectoryBinding(
        handles.sha256,
        digest.slice(0, 2),
        prefix,
        `completion prefix ${digest.slice(0, 2)}`,
      );
      await this.#assertBindings();
      return visible?.contents ?? null;
    } finally {
      await prefix.close();
    }
  }

  async publish(digest, value) {
    assertCompletionDigest(digest);
    const bytes = Buffer.from(value);
    if (bytes.byteLength > REFERENCE_COMPLETION_POINTER_MAX_BYTES) {
      throw new Error("completion pointer exceeds its byte limit");
    }
    const handles = await this.#assertBindings();
    const prefix = await this.#openPrefix(handles, digest, true);
    try {
      const disposition = await publishBytesNoReplaceAt(prefix, digest, bytes, {
        mode: 0o400,
      });
      if (disposition === "exists") await prefix.sync();
      const visible = await this.#readVisible(handles, prefix, digest);
      await handles.assertDirectoryBinding(
        handles.sha256,
        digest.slice(0, 2),
        prefix,
        `completion prefix ${digest.slice(0, 2)}`,
      );
      await this.#assertBindings();
      return Object.freeze({ bytes: visible.contents, disposition });
    } finally {
      await prefix.close();
    }
  }

  async close() {
    const handles = this.#handles;
    this.#handles = null;
    if (handles === null) return;
    await handles.sha256.close();
    await handles.completions.close();
  }
}

async function prepareReferenceCompletionIndex(root, options) {
  await options.assertStoreBindings();
  let completions;
  let sha256;
  try {
    completions = await options.openDirectoryAt(root, "completions", {
      create: options.create,
      label: "completions",
    });
    sha256 = await options.openDirectoryAt(completions, "sha256", {
      create: options.create,
      label: "completion sha256",
    });
    if (options.create) {
      await publishBytesNoReplaceAt(
        completions,
        "index.json",
        COMPLETION_INDEX_DESCRIPTOR_BYTES,
        { mode: 0o400 },
      );
    }
    const descriptor = await options.readAndHashFileAt(
      completions,
      "index.json",
      {
        collect: true,
        expectedMode: 0o400,
        maxBytes: COMPLETION_INDEX_DESCRIPTOR_BYTES.byteLength,
        sizeMismatchMessage: "completion index descriptor is not canonical",
      },
    );
    if (!descriptor.contents.equals(COMPLETION_INDEX_DESCRIPTOR_BYTES)) {
      throw new Error("completion index descriptor is not canonical");
    }
    await completions.sync();
    await options.assertDirectoryBinding(
      root,
      "completions",
      completions,
      "completions",
    );
    await options.assertDirectoryBinding(
      completions,
      "sha256",
      sha256,
      "completion sha256",
    );
    await options.assertStoreBindings();
    return new ReferenceCompletionIndex({
      ...options,
      completions,
      root,
      sha256,
    });
  } catch (error) {
    await sha256?.close().catch(() => {});
    await completions?.close().catch(() => {});
    throw error;
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

  async #resolveTarget(target) {
    const bytes = await this.#index.read(target.work_identity.sha256);
    if (bytes === null) return null;
    const pointer = parseReferenceAcquisitionCompletionPointer(
      bytes,
      target.work_identity,
    );
    const attempt = await readReferenceAcquisitionAttempt(
      this.#store,
      pointer.attempt,
      this.#context,
    );
    assertAttemptMatchesTarget(attempt, target);
    return Object.freeze({
      attempt,
      attempt_reference: pointer.attempt,
      target,
    });
  }

  async resolve(request) {
    return this.#resolveTarget(this.#target(request));
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
