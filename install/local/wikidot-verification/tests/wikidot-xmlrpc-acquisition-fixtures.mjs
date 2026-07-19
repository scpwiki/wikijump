import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import {
  buildReferenceAcquisitionAttempt,
  createReferenceAcquisitionContext,
  putReferenceAcquisitionAttempt,
} from "../src/reference-acquisition-attempt.mjs";
import { sha256Hex, stableStringify } from "../src/corpus-import-manifest.mjs";
import { buildReferenceAcquisitionInventory } from "../src/reference-acquisition-inventory.mjs";
import {
  buildWikidotXmlrpcCampaign,
  putWikidotXmlrpcCampaign,
} from "../src/reference-acquisition-xmlrpc-campaign.mjs";
import { initializeWikidotXmlrpcCompletions } from "../src/reference-acquisition-xmlrpc-completion.mjs";
import {
  buildWikidotXmlrpcImplementation,
  putWikidotXmlrpcImplementation,
} from "../src/reference-acquisition-xmlrpc-implementation.mjs";
import {
  buildWikidotXmlrpcObservation,
  serializeWikidotXmlrpcObservation,
  serializeWikidotXmlrpcResponse,
} from "../src/reference-acquisition-xmlrpc-observation.mjs";
import { initializeReferenceObjectStore } from "../src/reference-object-store.mjs";

export const CAMPAIGN_NONCE = "00000000-0000-4000-8000-000000000001";
export const PRINCIPAL_ID = 5700026;

export const COORDINATOR_IDENTITY = Object.freeze({
  coordinatorFileSha256: "a".repeat(64),
  nodeVersion: "v26.4.0",
  wikijumpCommit: "1".repeat(40),
  wikijumpTree: "2".repeat(40),
});

export const WORKER_AUTHORITY = Object.freeze({
  dependencyLockFileSha256: "b".repeat(64),
  pythonVersion: "3.13.13",
  workerFileSha256: "c".repeat(64),
  workerPath: "/fixture/worker-repo/scripts/wikidot_xmlrpc_capture_worker.py",
  workerPython: "/fixture/python",
  workerPythonEntry: "/fixture/worker-repo/.venv/bin/python",
  workerRepositoryCommit: "3".repeat(40),
  workerRepositoryRoot: "/fixture/worker-repo",
  workerRepositoryTree: "4".repeat(40),
});

export function buildInventory(count = 2) {
  const rows = Array.from({ length: count }, (_, index) => ({
    attachments: [],
    fullname: `scp-${173 + index}`,
    meta_sha256: "a".repeat(64),
    parent_fullname: null,
    revisions: 42,
    source_branch: "en",
    source_entity_id: `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
    source_sha256: sha256Hex(`source-${index}\n`),
    source_site: "scp-wiki",
    updated_at: "2026-07-18T12:34:56+00:00",
  }));
  const manifestBytes = Buffer.from(
    `${rows.map((row) => stableStringify(row)).join("\n")}\n`,
  );
  const summaryBytes = Buffer.from(
    `${stableStringify({
      attachment_count: 0,
      attachment_page_count: 0,
      first_fullname: rows[0].fullname,
      last_fullname: rows.at(-1).fullname,
      manifest_sha256: sha256Hex(manifestBytes),
      parent_count: 0,
      required_browser_count: 0,
      row_count: rows.length,
      source_browser_visibility_counts: {},
      source_branches: ["en"],
      source_required_actor_count: 0,
      source_sites: ["scp-wiki"],
    })}\n`,
  );
  return buildReferenceAcquisitionInventory({
    expectedCount: count,
    expectedManifestSha256: sha256Hex(manifestBytes),
    expectedSummarySha256: sha256Hex(summaryBytes),
    family: "EN",
    manifestBytes,
    shardCount: 2,
    sourceOrigin: "https://scp-wiki.wikidot.com",
    summaryBytes,
  });
}

export async function createAcquisitionFixture(t, count = 2) {
  const parent = await fs.mkdtemp(
    path.join(os.tmpdir(), "xmlrpc-acquisition-"),
  );
  const inventory = buildInventory(count);
  const state = {
    context: createReferenceAcquisitionContext(inventory, {
      expectedIdentitySha256: inventory.identity.sha256,
    }),
    inventory,
    receiptDirectory: path.join(parent, "receipts"),
    root: path.join(parent, "store"),
    semantic: undefined,
    store: undefined,
  };
  await fs.mkdir(state.receiptDirectory, { mode: 0o700 });
  state.store = await initializeReferenceObjectStore(state.root);
  t.after(async () => {
    await state.semantic?.close().catch(() => {});
    await state.store?.close().catch(() => {});
    await fs.rm(parent, { force: true, recursive: true });
  });
  return state;
}

export async function createXmlrpcCampaignFixture(state) {
  const implementation = await putWikidotXmlrpcImplementation(
    state.store,
    buildWikidotXmlrpcImplementation({
      coordinatorFileSha256: COORDINATOR_IDENTITY.coordinatorFileSha256,
      dependencyLockFileSha256: WORKER_AUTHORITY.dependencyLockFileSha256,
      nodeVersion: COORDINATOR_IDENTITY.nodeVersion,
      pythonVersion: WORKER_AUTHORITY.pythonVersion,
      wikijumpCommit: COORDINATOR_IDENTITY.wikijumpCommit,
      wikijumpTree: COORDINATOR_IDENTITY.wikijumpTree,
      workerFileSha256: WORKER_AUTHORITY.workerFileSha256,
      workerRepositoryCommit: WORKER_AUTHORITY.workerRepositoryCommit,
      workerRepositoryTree: WORKER_AUTHORITY.workerRepositoryTree,
    }),
  );
  const campaign = await putWikidotXmlrpcCampaign(
    state.store,
    buildWikidotXmlrpcCampaign({
      campaignNonce: CAMPAIGN_NONCE,
      implementation: implementation.object,
      inventorySha256: state.inventory.identity.sha256,
      principalId: PRINCIPAL_ID,
    }),
  );
  state.semantic = await initializeWikidotXmlrpcCompletions(
    state.store,
    state.context,
    campaign.reference,
  );
  return Object.freeze({ campaign, implementation, semantic: state.semantic });
}

export async function completeXmlrpcOrdinal(state, campaign, ordinal) {
  const response = responseFor(state, ordinal);
  const responseReference = (
    await state.store.putBytes(
      serializeWikidotXmlrpcResponse(response, response.fullname),
    )
  ).object;
  const startedAt = "2026-07-19T00:00:00.000Z";
  const finishedAt = "2026-07-19T00:00:01.000Z";
  const observationInput = {
    context: state.context,
    finishedAt,
    ordinal,
    producer: campaign.producer,
    response,
    responseReference,
    startedAt,
  };
  const observation = buildWikidotXmlrpcObservation(observationInput);
  const observationReference = (
    await state.store.putBytes(
      serializeWikidotXmlrpcObservation(observation, observationInput),
    )
  ).object;
  const attempt = await putReferenceAcquisitionAttempt(
    state.store,
    buildReferenceAcquisitionAttempt({
      attemptId: `00000000-0000-4000-8000-${String(ordinal + 1).padStart(12, "0")}`,
      context: state.context,
      finishedAt,
      layer: "xmlrpc_page",
      objects: [
        {
          media_type: "application/json",
          object: observationReference,
          role: "observation",
        },
        {
          media_type: "application/json",
          object: responseReference,
          role: "response",
        },
      ],
      ordinal,
      outcome: "complete",
      producer: campaign.producer,
      startedAt,
    }),
    state.context,
  );
  return state.semantic.publish(attempt.object, { ordinal });
}

export function responseFor(state, ordinal, suffix = "") {
  const row = state.context.rows[ordinal];
  return {
    content: `source-${ordinal}${suffix}\n`,
    fullname: row.fullname,
    html: `<p>source-${ordinal}${suffix}</p>`,
    revisions: suffix === "" ? 42 : 43,
    updated_at: suffix === "" ? "2026-07-18T12:34:56Z" : "2026-07-18T13:00:00Z",
  };
}

export function advancingClock() {
  let offset = 0;
  return () => new Date(Date.UTC(2026, 6, 19, 0, 0, offset++)).toISOString();
}

export function nextAttemptId() {
  let counter = 0;
  return () => `00000000-0000-4000-8000-${String(++counter).padStart(12, "0")}`;
}
