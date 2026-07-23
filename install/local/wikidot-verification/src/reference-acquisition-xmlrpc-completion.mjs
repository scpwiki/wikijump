import { types as utilTypes } from "node:util";

import { stableStringify } from "./canonical-json.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
  readReferenceAcquisitionAttemptReceipt,
  referenceAcquisitionInventoryRow,
  referenceAcquisitionInventorySha256,
} from "./reference-acquisition-attempt.mjs";
import {
  initializeReferenceAcquisitionCompletions,
  openReferenceAcquisitionCompletions,
  ReferenceAcquisitionCompletionConflictError,
} from "./reference-acquisition-completion.mjs";
import { openWikidotXmlrpcCampaign } from "./reference-acquisition-xmlrpc-campaign.mjs";
import {
  parseWikidotXmlrpcDeletedTombstone,
  parseWikidotXmlrpcObservation,
  parseWikidotXmlrpcResponse,
  WIKIDOT_XMLRPC_DELETED_TOMBSTONE_MAX_BYTES,
  WIKIDOT_XMLRPC_DELETED_TOMBSTONE_ROLE,
  WIKIDOT_XMLRPC_OBSERVATION_MAX_BYTES,
  WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES,
} from "./reference-acquisition-xmlrpc-observation.mjs";
import {
  isReferenceObjectStore,
  validateReferenceObject,
} from "./reference-object-store.mjs";

const LAYER = "xmlrpc_page";
const MEDIA_TYPE = "application/json";
const SEMANTIC_VALIDATION_BATCH_SIZE = 4;

function dataObject(value, expectedKeys, label) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error(`${label} must be a data object`);
  }
  let prototype;
  let keys;
  try {
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
  } catch {
    throw new Error(`${label} must be a data object`);
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some((key) => typeof key !== "string") ||
    stableStringify([...keys].sort()) !== stableStringify(expectedKeys)
  ) {
    throw new Error(`${label} has unexpected fields or prototype`);
  }
  const snapshot = {};
  for (const key of keys) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error(`${label} must contain only enumerable data fields`);
    }
    Object.defineProperty(snapshot, key, {
      enumerable: true,
      value: descriptor.value,
    });
  }
  return Object.freeze(snapshot);
}

function snapshotReference(value, label) {
  return validateReferenceObject(
    dataObject(value, ["algorithm", "bytes", "sha256"], label),
  );
}

function ordinalRequest(value) {
  const request = dataObject(value, ["ordinal"], "XML-RPC completion request");
  if (!Number.isSafeInteger(request.ordinal) || request.ordinal < 0) {
    throw new Error("XML-RPC completion ordinal must be a safe integer");
  }
  return request.ordinal;
}

function assertAttemptMatchesTarget(attempt, target) {
  if (attempt.outcome !== "complete" || attempt.failure !== null) {
    throw new Error("XML-RPC semantic completion requires a complete attempt");
  }
  for (const key of ["inventory", "layer", "producer", "work_identity"]) {
    if (stableStringify(attempt[key]) !== stableStringify(target[key])) {
      throw new Error("XML-RPC semantic completion has the wrong target");
    }
  }
}

export class WikidotXmlrpcSemanticCompletionError extends Error {
  constructor(ordinal) {
    super("Wikidot XML-RPC semantic completion is invalid");
    this.code = "WIKIDOT_XMLRPC_SEMANTIC_COMPLETION_INVALID";
    this.ordinal = ordinal;
  }
}

class WikidotXmlrpcCompletions {
  #campaignReference;
  #completions;
  #context;
  #inventorySha256;
  #store;

  constructor(store, context, campaignReference, completions) {
    this.#campaignReference = campaignReference;
    this.#completions = completions;
    this.#context = context;
    this.#inventorySha256 = referenceAcquisitionInventorySha256(context);
    this.#store = store;
  }

  #openCampaign() {
    return openWikidotXmlrpcCampaign(this.#store, this.#campaignReference, {
      expectedInventorySha256: this.#inventorySha256,
    });
  }

  #target(campaign, ordinal) {
    referenceAcquisitionInventoryRow(this.#context, ordinal);
    const request = Object.freeze({
      layer: LAYER,
      ordinal,
      producer: campaign.producer,
    });
    return Object.freeze({
      request,
      target: buildReferenceAcquisitionWorkTarget({
        context: this.#context,
        ...request,
      }),
    });
  }

  async #capture(resolved, target, campaign) {
    const ordinal = target.inventory.ordinal;
    try {
      assertAttemptMatchesTarget(resolved.attempt, target);
      if (
        stableStringify(resolved.target) !== stableStringify(target) ||
        resolved.attempt_reference === undefined
      ) {
        throw new Error("XML-RPC semantic completion shape is invalid");
      }
      const row = referenceAcquisitionInventoryRow(this.#context, ordinal);
      const tombstoneInput = {
        context: this.#context,
        finishedAt: resolved.attempt.finished_at,
        ordinal,
        producer: campaign.producer,
        startedAt: resolved.attempt.started_at,
      };
      if (resolved.attempt.objects.length === 1) {
        const [tombstoneBinding] = resolved.attempt.objects;
        if (
          tombstoneBinding.role !== WIKIDOT_XMLRPC_DELETED_TOMBSTONE_ROLE ||
          tombstoneBinding.media_type !== MEDIA_TYPE
        ) {
          throw new Error("XML-RPC semantic completion roles are invalid");
        }
        const tombstone = parseWikidotXmlrpcDeletedTombstone(
          await this.#store.readObject(tombstoneBinding.object, {
            maxBytes: WIKIDOT_XMLRPC_DELETED_TOMBSTONE_MAX_BYTES,
          }),
          tombstoneInput,
        );
        return Object.freeze({ kind: "deleted", tombstone });
      }
      if (resolved.attempt.objects.length !== 2) {
        throw new Error("XML-RPC semantic completion shape is invalid");
      }
      const [observationBinding, responseBinding] = resolved.attempt.objects;
      if (
        observationBinding.role !== "observation" ||
        observationBinding.media_type !== MEDIA_TYPE ||
        responseBinding.role !== "response" ||
        responseBinding.media_type !== MEDIA_TYPE
      ) {
        throw new Error("XML-RPC semantic completion roles are invalid");
      }
      const response = parseWikidotXmlrpcResponse(
        await this.#store.readObject(responseBinding.object, {
          maxBytes: WIKIDOT_XMLRPC_RESPONSE_MAX_BYTES,
        }),
        row.fullname,
      );
      const observation = parseWikidotXmlrpcObservation(
        await this.#store.readObject(observationBinding.object, {
          maxBytes: WIKIDOT_XMLRPC_OBSERVATION_MAX_BYTES,
        }),
        {
          context: this.#context,
          finishedAt: resolved.attempt.finished_at,
          ordinal,
          producer: campaign.producer,
          response,
          responseReference: responseBinding.object,
          startedAt: resolved.attempt.started_at,
        },
      );
      return Object.freeze({ kind: "live", observation, response });
    } catch {
      throw new WikidotXmlrpcSemanticCompletionError(ordinal);
    }
  }

  async #semanticRecord(resolved, target, campaign) {
    const capture = await this.#capture(resolved, target, campaign);
    return Object.freeze({ ...resolved, ...capture });
  }

  async #resolveOrdinal(ordinal) {
    const campaign = await this.#openCampaign();
    const {request, target} = this.#target(campaign, ordinal);
    const resolved = await this.#completions.resolveAttemptReceipt(request);
    if (resolved === null) return null;
    return this.#semanticRecord(resolved, target, campaign);
  }

  async resolve(value) {
    return this.#resolveOrdinal(ordinalRequest(value));
  }

  async publish(attemptReference, value) {
    const reference = snapshotReference(
      attemptReference,
      "XML-RPC completion attempt reference",
    );
    const ordinal = ordinalRequest(value);
    const campaign = await this.#openCampaign();
    const {target} = this.#target(campaign, ordinal);
    const attempt = await readReferenceAcquisitionAttemptReceipt(
      this.#store,
      reference,
      this.#context,
    );
    await this.#semanticRecord(
      Object.freeze({
        attempt,
        attempt_reference: reference,
        target,
      }),
      target,
      campaign,
    );
    const currentCampaign = await this.#openCampaign();
    const current = this.#target(currentCampaign, ordinal);
    let publication;
    try {
      publication = await this.#completions.publish(reference, current.request);
    } catch (error) {
      if (error instanceof ReferenceAcquisitionCompletionConflictError) {
        await this.#resolveOrdinal(ordinal);
      }
      throw error;
    }
    const visible = await this.#resolveOrdinal(ordinal);
    if (visible === null) {
      throw new WikidotXmlrpcSemanticCompletionError(ordinal);
    }
    return Object.freeze({
      ...visible,
      disposition: publication.disposition,
    });
  }

  async planResume(...args) {
    if (args.length !== 0) {
      throw new Error("XML-RPC completion resume accepts no options");
    }
    const campaign = await this.#openCampaign();
    const plan = await this.#completions.planResumeAttemptReceipts({
      layers: [LAYER],
      producer: campaign.producer,
    });
    for (
      let offset = 0;
      offset < plan.complete.length;
      offset += SEMANTIC_VALIDATION_BATCH_SIZE
    ) {
      const batch = plan.complete.slice(
        offset,
        offset + SEMANTIC_VALIDATION_BATCH_SIZE,
      );
      const results = await Promise.allSettled(
        batch.map((resolved) =>
          this.#semanticRecord(resolved, resolved.target, campaign),
        ),
      );
      for (const result of results) {
        if (result.status === "rejected") throw result.reason;
      }
    }
    return Object.freeze({
      complete: plan.complete,
      pending: plan.pending,
    });
  }

  close() {
    return this.#completions.close();
  }
}

async function prepare(store, context, campaignReference, create) {
  if (!isReferenceObjectStore(store)) {
    throw new Error("reference object store is required");
  }
  const inventorySha256 = referenceAcquisitionInventorySha256(context);
  const campaign = await openWikidotXmlrpcCampaign(store, campaignReference, {
    expectedInventorySha256: inventorySha256,
  });
  const completions = await (create
    ? initializeReferenceAcquisitionCompletions(store, context)
    : openReferenceAcquisitionCompletions(store, context));
  return new WikidotXmlrpcCompletions(
    store,
    context,
    campaign.reference,
    completions,
  );
}

export function initializeWikidotXmlrpcCompletions(
  store,
  context,
  campaignReference,
) {
  return prepare(store, context, campaignReference, true);
}

export function openWikidotXmlrpcCompletions(
  store,
  context,
  campaignReference,
) {
  return prepare(store, context, campaignReference, false);
}
