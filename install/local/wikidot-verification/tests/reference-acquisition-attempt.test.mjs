import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { sha256Hex, stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionAttempt,
  buildReferenceAcquisitionWorkTarget,
  createReferenceAcquisitionContext,
  parseReferenceAcquisitionAttempt,
  putReferenceAcquisitionAttempt,
  readReferenceAcquisitionAttempt,
  serializeReferenceAcquisitionAttempt,
  validateReferenceAcquisitionAttempt,
  validateReferenceAcquisitionContext,
} from "../src/reference-acquisition-attempt.mjs";
import { buildReferenceAcquisitionInventory } from "../src/reference-acquisition-inventory.mjs";
import {
  initializeReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";

const ATTEMPT_ID = "00000000-0000-4000-8000-000000000001";
const STARTED_AT = "2026-07-18T17:30:00.000Z";
const FINISHED_AT = "2026-07-18T17:30:01.000Z";
const FIXTURES = "../fixtures/reference-acquisition-attempt-v1/";

test("attempt schema parses", async () => {
  const url = new URL(
    "../schemas/reference-acquisition-attempt-v1.schema.json",
    import.meta.url,
  );
  JSON.parse(await fs.readFile(url, "utf8"));
});

function buildInventory(fullname = "theme:雪-space") {
  const row = {
    attachments: [],
    fullname,
    meta_sha256: "a".repeat(64),
    parent_fullname: null,
    revisions: 42,
    source_branch: "en",
    source_entity_id: "00000000-0000-4000-8000-000000000173",
    source_sha256: "b".repeat(64),
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
    shardCount: 2,
    sourceOrigin: "https://scp-wiki.wikidot.com",
    summaryBytes,
  });
}

function createContext(inventory, expected = inventory.identity.sha256) {
  return createReferenceAcquisitionContext(inventory, {
    expectedIdentitySha256: expected,
  });
}

test("work targets are derived only from a pinned inventory context", () => {
  const inventory = buildInventory();
  const context = createContext(inventory);
  const producer = {
    contract: "wikijump_full_parity.wikidot_xmlrpc_acquirer.v1",
    identity: { algorithm: "sha256", bytes: 17, sha256: "c".repeat(64) },
  };
  assert.equal(validateReferenceAcquisitionContext(context), context);
  assert.throws(() => validateReferenceAcquisitionContext({}), /context/u);
  const target = buildReferenceAcquisitionWorkTarget({
    context,
    layer: "xmlrpc_page",
    ordinal: 0,
    producer,
  });
  assert.equal(
    target.work_identity.sha256,
    sha256Hex(
      stableStringify({
        inventory: target.inventory,
        layer: target.layer,
        producer: target.producer,
      }),
    ),
  );
  assert(Object.isFrozen(target));
  assert.throws(
    () =>
      buildReferenceAcquisitionWorkTarget({
        context,
        layer: "unrequested_layer",
        ordinal: 0,
        producer,
      }),
    /not requested/u,
  );
});

function golden(name) {
  return fs.readFile(new URL(`${FIXTURES}${name}`, import.meta.url));
}

async function temporaryStore(t) {
  const temporaryRoot = await fs.mkdtemp(
    path.join(os.tmpdir(), "reference-attempt-"),
  );
  const root = path.join(temporaryRoot, "store");
  const store = await initializeReferenceObjectStore(root);
  t.after(async () => {
    await store.close();
    await fs.rm(temporaryRoot, { force: true, recursive: true });
  });
  return { root, store };
}

async function producerFor(store) {
  return {
    contract: "wikijump_full_parity.wikidot_xmlrpc_acquirer.v1",
    identity: (await store.putBytes(Buffer.from("producer contract"))).object,
  };
}

function attemptInput(context, producer, objects, overrides = {}) {
  return {
    attemptId: ATTEMPT_ID,
    context,
    finishedAt: FINISHED_AT,
    layer: "xmlrpc_page",
    objects,
    ordinal: 0,
    outcome: "complete",
    producer,
    startedAt: STARTED_AT,
    ...overrides,
  };
}

test("canonical complete attempts round-trip through the content store", async (t) => {
  const inventory = buildInventory();
  const alternate = buildInventory("theme:雨-space");
  assert.throws(
    () => createReferenceAcquisitionContext(inventory),
    /expected/u,
  );
  assert.throws(
    () => createContext(alternate, inventory.identity.sha256),
    /expected authority/u,
  );
  const context = createContext(inventory);
  const fixture = await temporaryStore(t);
  const producer = await producerFor(fixture.store);
  const response = (
    await fixture.store.putBytes(Buffer.from("<xml>response</xml>"))
  ).object;
  const metadata = (await fixture.store.putBytes(Buffer.from("{}\n"))).object;
  const attempt = buildReferenceAcquisitionAttempt(
    attemptInput(context, producer, [
      { media_type: "application/json", object: metadata, role: "metadata" },
      { media_type: "application/xml", object: response, role: "response" },
    ]),
  );
  const unsorted = structuredClone(attempt);
  unsorted.objects.reverse();
  assert.throws(
    () => validateReferenceAcquisitionAttempt(unsorted, context),
    /canonical order/u,
  );
  const bytes = serializeReferenceAcquisitionAttempt(attempt, context);
  assert.deepEqual(bytes, await golden("complete.jsonl"));
  assert.deepEqual(parseReferenceAcquisitionAttempt(bytes, context), attempt);
  const put = () =>
    putReferenceAcquisitionAttempt(fixture.store, attempt, context);
  const first = await put();
  assert.deepEqual(
    await readReferenceAcquisitionAttempt(fixture.store, first.object, context),
    attempt,
  );
});

test("failed attempts are non-completing and receipts fail closed", async (t) => {
  const inventory = buildInventory();
  const context = createContext(inventory);
  const fixture = await temporaryStore(t);
  const producer = await producerFor(fixture.store);
  const failed = buildReferenceAcquisitionAttempt(
    attemptInput(context, producer, [], {
      failure: { code: "transport_timeout", retryable: true },
      outcome: "failed",
    }),
  );
  for (const timestamp of [
    "2026-07-18T17:30:00Z",
    "2026-07-18T18:30:00.000+01:00",
    "2026-02-30T17:30:00.000Z",
    "+010000-01-01T00:00:00.000Z",
    "-000001-01-01T00:00:00.000Z",
  ]) {
    const noncanonical = structuredClone(failed);
    noncanonical.started_at = timestamp;
    assert.throws(
      () => validateReferenceAcquisitionAttempt(noncanonical, context),
      /canonical UTC timestamp/u,
    );
  }
  const invalid = (overrides, objects = []) =>
    buildReferenceAcquisitionAttempt(
      attemptInput(context, producer, objects, overrides),
    );
  assert.throws(() => invalid({ outcome: "complete" }), /require/u);
  assert.throws(() => invalid({}, [null]), /must be an object/u);
  assert.throws(() => invalid({ outcome: "failed" }), /attempt.failure/u);
  assert.throws(
    () =>
      invalid({
        failure: { code: "transport_timeout", retryable: true },
        finishedAt: STARTED_AT,
        startedAt: FINISHED_AT,
        outcome: "failed",
      }),
    /finished before/u,
  );
  const canonical = serializeReferenceAcquisitionAttempt(failed, context);
  assert.deepEqual(canonical, await golden("failed.jsonl"));
  assert.throws(
    () =>
      parseReferenceAcquisitionAttempt(
        Buffer.concat([canonical, Buffer.from("\n")]),
        context,
      ),
    /canonical JSON line/u,
  );
  const changed = structuredClone(failed);
  changed.inventory.semantic_row_sha256 = "0".repeat(64);
  assert.throws(
    () => validateReferenceAcquisitionAttempt(changed, context),
    /does not match/u,
  );
  const missingProducer = structuredClone(failed);
  missingProducer.producer.identity.sha256 = crypto
    .createHash("sha256")
    .update("absent producer")
    .digest("hex");
  missingProducer.work_identity.sha256 = sha256Hex(
    stableStringify({
      inventory: missingProducer.inventory,
      layer: missingProducer.layer,
      producer: missingProducer.producer,
    }),
  );
  await assert.rejects(
    putReferenceAcquisitionAttempt(fixture.store, missingProducer, context),
    { code: "ENOENT" },
  );
});

test("transitive object corruption invalidates a stored attempt", async (t) => {
  const inventory = buildInventory();
  const context = createContext(inventory);
  const fixture = await temporaryStore(t);
  const producer = await producerFor(fixture.store);
  const response = (await fixture.store.putBytes(Buffer.from("response")))
    .object;
  const attempt = buildReferenceAcquisitionAttempt(
    attemptInput(context, producer, [
      { media_type: "application/xml", object: response, role: "response" },
    ]),
  );
  const stored = await putReferenceAcquisitionAttempt(
    fixture.store,
    attempt,
    context,
  );
  const responsePath = path.join(
    fixture.root,
    ...referenceObjectRelativePath(response.sha256).split("/"),
  );
  await fs.chmod(responsePath, 0o600);
  await fs.writeFile(responsePath, "tampered");
  await fs.chmod(responsePath, 0o400);
  await assert.rejects(
    readReferenceAcquisitionAttempt(fixture.store, stored.object, context),
    /corrupt/u,
  );
});
