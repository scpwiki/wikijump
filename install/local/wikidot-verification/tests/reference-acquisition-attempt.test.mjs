import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {sha256Hex, stableStringify} from "../src/corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionAttempt,
  parseReferenceAcquisitionAttempt,
  putReferenceAcquisitionAttempt,
  readReferenceAcquisitionAttempt,
  serializeReferenceAcquisitionAttempt,
  validateReferenceAcquisitionAttempt,
} from "../src/reference-acquisition-attempt.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
  createReferenceAcquisitionContext,
  validateReferenceAcquisitionContext,
} from "../src/reference-acquisition-work-target.mjs";
import {referenceObjectRelativePath} from "../src/reference-object-store.mjs";
import {
  FINISHED_AT,
  STARTED_AT,
  attemptInput,
  buildInventory,
  createContext,
  golden,
  producerFor,
  temporaryStore,
} from "./support/reference-acquisition-attempt-fixture.mjs";

test("attempt schema parses", async () => {
  const url = new URL(
    "../schemas/reference-acquisition-attempt-v1.schema.json",
    import.meta.url,
  );
  JSON.parse(await fs.readFile(url, "utf8"));
});

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
