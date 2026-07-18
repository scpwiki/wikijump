import assert from "node:assert/strict";
import test from "node:test";

import { sha256Hex, stableStringify } from "../src/corpus-import-manifest.mjs";
import { createReferenceAcquisitionContext } from "../src/reference-acquisition-attempt.mjs";
import { buildReferenceAcquisitionInventory } from "../src/reference-acquisition-inventory.mjs";
import {
  buildWikidotXmlrpcObservation,
  parseWikidotXmlrpcObservation,
  parseWikidotXmlrpcResponse,
  serializeWikidotXmlrpcObservation,
  serializeWikidotXmlrpcResponse,
  WIKIDOT_XMLRPC_PRODUCER_CONTRACT,
} from "../src/reference-acquisition-xmlrpc-observation.mjs";

const FROZEN_SOURCE = "frozen source\n";
const STARTED_AT = "2026-07-19T00:00:00.000Z";
const FINISHED_AT = "2026-07-19T00:00:01.000Z";

function inventory() {
  const row = {
    attachments: [],
    fullname: "scp-173",
    meta_sha256: "a".repeat(64),
    parent_fullname: null,
    revisions: 42,
    source_branch: "en",
    source_entity_id: "00000000-0000-4000-8000-000000000173",
    source_sha256: sha256Hex(FROZEN_SOURCE),
    source_site: "scp-wiki",
    updated_at: "2026-07-18T12:34:56+00:00",
  };
  const manifestBytes = Buffer.from(`${stableStringify(row)}\n`);
  const summary = {
    attachment_count: 0,
    attachment_page_count: 0,
    first_fullname: row.fullname,
    last_fullname: row.fullname,
    manifest_sha256: sha256Hex(manifestBytes),
    parent_count: 0,
    required_browser_count: 0,
    row_count: 1,
    source_browser_visibility_counts: {},
    source_branches: ["en"],
    source_required_actor_count: 0,
    source_sites: ["scp-wiki"],
  };
  const summaryBytes = Buffer.from(`${stableStringify(summary)}\n`);
  return buildReferenceAcquisitionInventory({
    expectedCount: 1,
    expectedManifestSha256: sha256Hex(manifestBytes),
    expectedSummarySha256: sha256Hex(summaryBytes),
    family: "EN",
    manifestBytes,
    shardCount: 1,
    sourceOrigin: "https://scp-wiki.wikidot.com",
    summaryBytes,
  });
}

function fixture() {
  const currentInventory = inventory();
  return {
    context: createReferenceAcquisitionContext(currentInventory, {
      expectedIdentitySha256: currentInventory.identity.sha256,
    }),
    producer: {
      contract: WIKIDOT_XMLRPC_PRODUCER_CONTRACT,
      identity: { algorithm: "sha256", bytes: 26, sha256: "c".repeat(64) },
    },
  };
}

function response(overrides = {}) {
  return {
    content: FROZEN_SOURCE,
    fullname: "scp-173",
    html: "<p>frozen source</p>",
    revisions: 42,
    updated_at: "2026-07-18T12:34:56Z",
    unknown_future_field: { preserved: true },
    ...overrides,
  };
}

function observationInput(current, capturedResponse) {
  const responseBytes = serializeWikidotXmlrpcResponse(
    capturedResponse,
    "scp-173",
  );
  return {
    context: current.context,
    finishedAt: FINISHED_AT,
    ordinal: 0,
    producer: current.producer,
    response: capturedResponse,
    responseReference: {
      algorithm: "sha256",
      bytes: responseBytes.byteLength,
      sha256: sha256Hex(responseBytes),
    },
    startedAt: STARTED_AT,
  };
}

test("exact decoded XML-RPC observations round-trip canonically", () => {
  const current = fixture();
  const capturedResponse = response();
  const input = observationInput(current, capturedResponse);
  const observation = buildWikidotXmlrpcObservation(input);
  assert.deepEqual(observation.baseline_relation, {
    classification: "exact",
    mismatched_fields: [],
  });
  assert.equal(observation.raw_wire_captured, false);
  assert.equal(observation.fallback_used, false);
  const responseBytes = serializeWikidotXmlrpcResponse(
    capturedResponse,
    "scp-173",
  );
  assert.deepEqual(
    parseWikidotXmlrpcResponse(responseBytes, "scp-173"),
    capturedResponse,
  );
  assert.deepEqual(
    parseWikidotXmlrpcObservation(
      serializeWikidotXmlrpcObservation(observation, input),
      input,
    ),
    observation,
  );
});

test("advanced live revisions remain captures with explicit baseline drift", () => {
  const current = fixture();
  const capturedResponse = response({
    content: "new source\n",
    revisions: 43,
    updated_at: "2026-07-18T13:00:00Z",
  });
  assert.deepEqual(
    buildWikidotXmlrpcObservation(observationInput(current, capturedResponse))
      .baseline_relation,
    {
      classification: "advanced_changed_source",
      mismatched_fields: ["revisions", "source_sha256", "updated_at"],
    },
  );
  const incoherent = response({
    content: "unexpected\n",
    updated_at: "2026-07-18T13:00:00Z",
  });
  assert.equal(
    buildWikidotXmlrpcObservation(observationInput(current, incoherent))
      .baseline_relation.classification,
    "identity_discontinuity_or_regression",
  );
});

test("wrong targets and non-JSON response values fail closed", () => {
  const current = fixture();
  assert.throws(
    () => observationInput(current, response({ fullname: "scp-682" })),
    /fullname/u,
  );
  assert.throws(
    () => observationInput(current, response({ rating: Number.NaN })),
    /non-finite/u,
  );
  assert.throws(
    () => observationInput(current, response({ content: undefined })),
    /non-JSON/u,
  );
  const exact = observationInput(current, response());
  const observation = buildWikidotXmlrpcObservation(exact);
  const changed = structuredClone(observation);
  changed.raw_wire_captured = true;
  assert.throws(
    () =>
      parseWikidotXmlrpcObservation(
        serializeWikidotXmlrpcObservation(changed, exact),
        exact,
      ),
    /does not match/u,
  );
});

test("XML-RPC inputs are snapshotted and observations are immutable", () => {
  const current = fixture();
  let contractReads = 0;
  const producer = new Proxy(current.producer, {
    get(target, property, receiver) {
      if (property === "contract") {
        contractReads += 1;
        return contractReads === 1
          ? WIKIDOT_XMLRPC_PRODUCER_CONTRACT
          : "another.contract";
      }
      return Reflect.get(target, property, receiver);
    },
  });
  const capturedResponse = response();
  const input = observationInput({ ...current, producer }, capturedResponse);
  const observation = buildWikidotXmlrpcObservation(input);
  assert.equal(contractReads, 1);
  assert(Object.isFrozen(observation));
  assert(Object.isFrozen(observation.observed));
  assert.throws(() => {
    observation.observed.revisions = 99;
  }, TypeError);

  const accessorResponse = response();
  Object.defineProperty(accessorResponse, "fullname", {
    enumerable: true,
    get: () => "scp-173",
  });
  assert.throws(
    () => serializeWikidotXmlrpcResponse(accessorResponse, "scp-173"),
    /non-JSON object/u,
  );
});

test("XML-RPC strings must encode to UTF-8 without replacement", () => {
  assert.throws(
    () =>
      serializeWikidotXmlrpcResponse(
        response({ content: "\ud800" }),
        "scp-173",
      ),
    /ill-formed Unicode/u,
  );
  const invalidKey = response();
  invalidKey["\udfff"] = true;
  assert.throws(
    () => serializeWikidotXmlrpcResponse(invalidKey, "scp-173"),
    /ill-formed Unicode/u,
  );
});
