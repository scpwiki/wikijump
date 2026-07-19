import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { sha256Hex, stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionAttempt,
  buildReferenceAcquisitionWorkTarget,
  createReferenceAcquisitionContext,
  putReferenceAcquisitionAttempt,
} from "../src/reference-acquisition-attempt.mjs";
import {
  openReferenceAcquisitionCompletions,
  referenceAcquisitionCompletionRelativePath,
} from "../src/reference-acquisition-completion.mjs";
import { buildReferenceAcquisitionInventory } from "../src/reference-acquisition-inventory.mjs";
import {
  buildWikidotXmlrpcCampaign,
  putWikidotXmlrpcCampaign,
} from "../src/reference-acquisition-xmlrpc-campaign.mjs";
import {
  initializeWikidotXmlrpcCompletions,
  openWikidotXmlrpcCompletions,
} from "../src/reference-acquisition-xmlrpc-completion.mjs";
import {
  buildWikidotXmlrpcImplementation,
  putWikidotXmlrpcImplementation,
} from "../src/reference-acquisition-xmlrpc-implementation.mjs";
import {
  buildWikidotXmlrpcObservation,
  serializeWikidotXmlrpcObservation,
  serializeWikidotXmlrpcResponse,
  WIKIDOT_XMLRPC_OBSERVATION_MAX_BYTES,
} from "../src/reference-acquisition-xmlrpc-observation.mjs";
import {
  initializeReferenceObjectStore,
  openReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";

const STARTED_AT = "2026-07-19T00:00:00.000Z";
const FINISHED_AT = "2026-07-19T00:00:01.000Z";
const ATTEMPT_ID = "00000000-0000-4000-8000-000000000001";

function inventory(count = 2) {
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

function implementation() {
  return buildWikidotXmlrpcImplementation({
    coordinatorFileSha256: "a".repeat(64),
    dependencyLockFileSha256: "b".repeat(64),
    nodeVersion: "v26.4.0",
    pythonVersion: "3.14.0",
    wikijumpCommit: "1".repeat(40),
    wikijumpTree: "2".repeat(40),
    workerFileSha256: "c".repeat(64),
    workerRepositoryCommit: "3".repeat(40),
    workerRepositoryTree: "4".repeat(40),
  });
}

async function fixture(t, count = 2) {
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "xmlrpc-completion-"));
  const currentInventory = inventory(count);
  const state = {
    context: createReferenceAcquisitionContext(currentInventory, {
      expectedIdentitySha256: currentInventory.identity.sha256,
    }),
    root: path.join(parent, "store"),
    semantic: undefined,
    store: undefined,
  };
  state.store = await initializeReferenceObjectStore(state.root);
  state.implementation = await putWikidotXmlrpcImplementation(
    state.store,
    implementation(),
  );
  state.campaign = await putWikidotXmlrpcCampaign(
    state.store,
    buildWikidotXmlrpcCampaign({
      campaignNonce: "00000000-0000-4000-8000-000000000001",
      implementation: state.implementation.object,
      inventorySha256: currentInventory.identity.sha256,
      principalId: 5700026,
    }),
  );
  state.semantic = await initializeWikidotXmlrpcCompletions(
    state.store,
    state.context,
    state.campaign.reference,
  );
  t.after(async () => {
    await state.semantic?.close().catch(() => {});
    await state.store?.close().catch(() => {});
    await fs.rm(parent, { force: true, recursive: true });
  });
  return state;
}

async function captureAssets(state, ordinal = 0, suffix = "") {
  const fullname = `scp-${173 + ordinal}`;
  const response = {
    content: `source-${ordinal}${suffix}\n`,
    fullname,
    html: `<p>source-${ordinal}${suffix}</p>`,
    revisions: suffix === "" ? 42 : 43,
    updated_at: suffix === "" ? "2026-07-18T12:34:56Z" : "2026-07-18T13:00:00Z",
  };
  const responseBytes = serializeWikidotXmlrpcResponse(response, fullname);
  const responseReference = (await state.store.putBytes(responseBytes)).object;
  const input = {
    context: state.context,
    finishedAt: FINISHED_AT,
    ordinal,
    producer: state.campaign.producer,
    response,
    responseReference,
    startedAt: STARTED_AT,
  };
  const observation = buildWikidotXmlrpcObservation(input);
  const observationReference = (
    await state.store.putBytes(
      serializeWikidotXmlrpcObservation(observation, input),
    )
  ).object;
  return {
    observation,
    observationBinding: {
      media_type: "application/json",
      object: observationReference,
      role: "observation",
    },
    response,
    responseBinding: {
      media_type: "application/json",
      object: responseReference,
      role: "response",
    },
  };
}

async function storedAttempt(state, objects, ordinal = 0, timestamps = {}) {
  const attempt = buildReferenceAcquisitionAttempt({
    attemptId: ATTEMPT_ID,
    context: state.context,
    finishedAt: timestamps.finishedAt ?? FINISHED_AT,
    layer: "xmlrpc_page",
    objects,
    ordinal,
    outcome: "complete",
    producer: state.campaign.producer,
    startedAt: timestamps.startedAt ?? STARTED_AT,
  });
  return putReferenceAcquisitionAttempt(state.store, attempt, state.context);
}

test("semantic XML-RPC completions publish, reopen, and resume exactly", async (t) => {
  const state = await fixture(t);
  const assets = await captureAssets(state);
  const stored = await storedAttempt(state, [
    assets.responseBinding,
    assets.observationBinding,
  ]);
  const created = await state.semantic.publish(stored.object, { ordinal: 0 });
  assert.equal(created.disposition, "created");
  assert.deepEqual(created.response, assets.response);
  assert.deepEqual(created.observation, assets.observation);
  assert.equal(
    (await state.semantic.publish(stored.object, { ordinal: 0 })).disposition,
    "exists",
  );
  assert.deepEqual(
    (await state.semantic.resolve({ ordinal: 0 })).response,
    assets.response,
  );
  let plan = await state.semantic.planResume();
  assert.deepEqual(
    plan.complete.map((item) => item.target.inventory.ordinal),
    [0],
  );
  assert.deepEqual(
    plan.pending.map((item) => item.inventory.ordinal),
    [1],
  );
  await state.semantic.close();
  state.semantic = undefined;
  await state.store.close();
  state.store = await openReferenceObjectStore(state.root);
  state.semantic = await openWikidotXmlrpcCompletions(
    state.store,
    state.context,
    state.campaign.reference,
  );
  plan = await state.semantic.planResume();
  assert.equal(plan.complete.length, 1);
  assert.equal(plan.pending.length, 1);
});

test("generic but semantically invalid completions remain terminal corruption", async (t) => {
  for (const variant of [
    "missing",
    "media",
    "extra",
    "crosswired",
    "timestamps",
  ]) {
    await t.test(variant, async (t) => {
      const state = await fixture(t, 1);
      const first = await captureAssets(state);
      let objects = [first.observationBinding, first.responseBinding];
      if (variant === "missing") objects = [first.responseBinding];
      if (variant === "media") {
        objects = objects.map((binding) => ({
          ...binding,
          media_type: "application/xml",
        }));
      }
      if (variant === "extra") {
        const trace = (await state.store.putBytes(Buffer.from("trace"))).object;
        objects = [
          ...objects,
          { media_type: "application/json", object: trace, role: "trace" },
        ];
      }
      if (variant === "crosswired") {
        const second = await captureAssets(state, 0, "-changed");
        objects = [first.observationBinding, second.responseBinding];
      }
      const timestamps =
        variant === "timestamps"
          ? {
              finishedAt: "2026-07-19T00:00:03.000Z",
              startedAt: "2026-07-19T00:00:02.000Z",
            }
          : {};
      const stored = await storedAttempt(state, objects, 0, timestamps);
      const generic = await openReferenceAcquisitionCompletions(
        state.store,
        state.context,
      );
      await generic.publish(stored.object, {
        layer: "xmlrpc_page",
        ordinal: 0,
        producer: state.campaign.producer,
      });
      await assert.rejects(state.semantic.resolve({ ordinal: 0 }), {
        code: "WIKIDOT_XMLRPC_SEMANTIC_COMPLETION_INVALID",
      });
      await assert.rejects(state.semantic.planResume(), {
        code: "WIKIDOT_XMLRPC_SEMANTIC_COMPLETION_INVALID",
      });
      if (variant === "missing") {
        const valid = await captureAssets(state);
        const validAttempt = await storedAttempt(state, [
          valid.observationBinding,
          valid.responseBinding,
        ]);
        await assert.rejects(
          state.semantic.publish(validAttempt.object, { ordinal: 0 }),
          { code: "WIKIDOT_XMLRPC_SEMANTIC_COMPLETION_INVALID" },
        );
      }
      await generic.close();
    });
  }
});

test("semantic validation is bounded, layer-isolated, and transitively fresh", async (t) => {
  const state = await fixture(t, 1);
  const httpTarget = buildReferenceAcquisitionWorkTarget({
    context: state.context,
    layer: "http_document",
    ordinal: 0,
    producer: state.campaign.producer,
  });
  const httpLeaf = path.join(
    state.root,
    ...referenceAcquisitionCompletionRelativePath(httpTarget).split("/"),
  );
  await fs.mkdir(path.dirname(httpLeaf), { mode: 0o700, recursive: true });
  await fs.writeFile(httpLeaf, "{}\n", { mode: 0o400 });
  assert.equal((await state.semantic.planResume()).pending.length, 1);
  await fs.unlink(httpLeaf);

  const assets = await captureAssets(state);
  const oversized = (
    await state.store.putBytes(
      Buffer.alloc(WIKIDOT_XMLRPC_OBSERVATION_MAX_BYTES + 1, 0x20),
    )
  ).object;
  const attempt = buildReferenceAcquisitionAttempt({
    attemptId: ATTEMPT_ID,
    context: state.context,
    finishedAt: FINISHED_AT,
    layer: "xmlrpc_page",
    objects: [
      { ...assets.observationBinding, object: oversized },
      assets.responseBinding,
    ],
    ordinal: 0,
    outcome: "complete",
    producer: state.campaign.producer,
    startedAt: STARTED_AT,
  });
  const attemptReference = (
    await putReferenceAcquisitionAttempt(state.store, attempt, state.context)
  ).object;
  const generic = await openReferenceAcquisitionCompletions(
    state.store,
    state.context,
  );
  assert.equal(generic.publishReceipt, undefined);
  await generic.publish(attemptReference, {
    layer: "xmlrpc_page",
    ordinal: 0,
    producer: state.campaign.producer,
  });
  await assert.rejects(state.semantic.resolve({ ordinal: 0 }), {
    code: "WIKIDOT_XMLRPC_SEMANTIC_COMPLETION_INVALID",
  });
  await generic.close();

  const implementationPath = path.join(
    state.root,
    ...referenceObjectRelativePath(state.implementation.object.sha256).split(
      "/",
    ),
  );
  await fs.chmod(implementationPath, 0o600);
  await fs.writeFile(
    implementationPath,
    Buffer.alloc(state.implementation.object.bytes, 0x20),
  );
  await fs.chmod(implementationPath, 0o400);
  await assert.rejects(state.semantic.planResume(), /corrupt/u);
});

test("semantic completion public inputs fail without leaking hostile values", async (t) => {
  const state = await fixture(t, 1);
  const secret = "sentinel-semantic-secret";
  const accessor = {};
  Object.defineProperty(accessor, "ordinal", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const proxy = new Proxy(
    { ordinal: 0 },
    {
      ownKeys: () => {
        throw new Error(secret);
      },
    },
  );
  const symbolic = { ordinal: 0, [Symbol("secret")]: secret };
  for (const request of [accessor, proxy, symbolic]) {
    await assert.rejects(
      state.semantic.resolve(request),
      (error) => !error.message.includes(secret),
    );
  }
  await assert.rejects(
    state.semantic.planResume({ secret }),
    (error) => !error.message.includes(secret),
  );
  const reference = { ...state.campaign.reference };
  Object.defineProperty(reference, "algorithm", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  await assert.rejects(
    state.semantic.publish(reference, { ordinal: 0 }),
    (error) => !error.message.includes(secret),
  );
});
