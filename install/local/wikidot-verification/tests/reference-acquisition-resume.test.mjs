import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

import {
  buildReferenceAcquisitionAttempt,
  putReferenceAcquisitionAttempt,
} from "../src/reference-acquisition-attempt.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
} from "../src/reference-acquisition-work-target.mjs";
import {
  initializeReferenceAcquisitionCompletions,
  openReferenceAcquisitionCompletions,
  ReferenceAcquisitionCompletionConflictError,
  referenceAcquisitionCompletionRelativePath,
  serializeReferenceAcquisitionCompletionPointer,
} from "../src/reference-acquisition-completion.mjs";
import {
  openReferenceObjectStore,
  referenceObjectRelativePath,
} from "../src/reference-object-store.mjs";
import {
  attemptInput,
  buildInventory,
  completeReceipt,
  createContext,
  producerFor,
  temporaryStore,
} from "./support/reference-acquisition-attempt-fixture.mjs";

test("completion index contracts and deterministic resume are canonical", async (t) => {
  JSON.parse(
    await fs.readFile(
      new URL(
        "../schemas/reference-acquisition-completion-pointer-v1.schema.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const inventory = buildInventory();
  const context = createContext(inventory);
  const fixture = await temporaryStore(t);
  const producer = await producerFor(fixture.store);
  producer.contract = "wikijump_full_parity.prefix_collision_100.v1";
  const stored = await completeReceipt(fixture.store, context, producer);
  await assert.rejects(
    initializeReferenceAcquisitionCompletions(fixture.store, {}),
    /context/u,
  );
  await assert.rejects(fs.access(path.join(fixture.root, "completions")));
  const completions = await initializeReferenceAcquisitionCompletions(
    fixture.store,
    context,
  );
  t.after(() => completions.close());
  const request = { layer: "xmlrpc_page", ordinal: 0, producer };
  const target = buildReferenceAcquisitionWorkTarget({ context, ...request });
  const samePrefixPending = buildReferenceAcquisitionWorkTarget({
    context,
    layer: "http_document",
    ordinal: 0,
    producer,
  });
  assert.equal(
    target.work_identity.sha256.slice(0, 2),
    samePrefixPending.work_identity.sha256.slice(0, 2),
  );
  assert.equal(await completions.resolve(request), null);
  const created = await completions.publish(stored.reference, request);
  assert.equal(created.disposition, "created");
  assert.deepEqual(created.attempt, stored.attempt);
  const existing = await completions.publish(stored.reference, request);
  assert.equal(existing.disposition, "exists");
  assert.deepEqual(await completions.resolve(request), {
    attempt: stored.attempt,
    attempt_reference: stored.reference,
    target,
  });
  const pointerPath = path.join(
    fixture.root,
    ...referenceAcquisitionCompletionRelativePath(target).split("/"),
  );
  assert.equal((await fs.stat(pointerPath)).mode & 0o777, 0o400);
  assert.deepEqual(
    await fs.readFile(path.join(fixture.root, "completions", "index.json")),
    await fs.readFile(
      new URL(
        "../fixtures/reference-acquisition-completion-v1/index.json",
        import.meta.url,
      ),
    ),
  );
  await completions.close();
  const reopenedStore = await openReferenceObjectStore(fixture.root);
  t.after(() => reopenedStore.close());
  const reopenedCompletions = await openReferenceAcquisitionCompletions(
    reopenedStore,
    context,
  );
  t.after(() => reopenedCompletions.close());
  assert.deepEqual(await reopenedCompletions.resolve(request), {
    attempt: stored.attempt,
    attempt_reference: stored.reference,
    target,
  });
  const absentProducer = structuredClone(producer);
  absentProducer.identity.sha256 = "d".repeat(64);
  await assert.rejects(
    reopenedCompletions.planResume({ producer: absentProducer }),
    { code: "ENOENT" },
  );
  const plan = await reopenedCompletions.planResume({ producer });
  assert.equal(plan.complete.length, 1);
  assert.deepEqual(
    plan.pending.map((item) => item.layer),
    inventory.rows[0].requested_layers.filter(
      (layer) => layer !== "xmlrpc_page",
    ),
  );
  await fs.chmod(
    path.join(
      fixture.root,
      ...referenceObjectRelativePath(stored.response.sha256).split("/"),
    ),
    0o600,
  );
  await assert.rejects(reopenedCompletions.resolve(request), /mode 400/u);
  await assert.rejects(
    reopenedCompletions.planResume({ producer }),
    /mode 400/u,
  );
});

test("resume snapshots one producer and partitions multiple rows deterministically", async (t) => {
  const inventory = buildInventory(["theme:alpha", "theme:omega"]);
  const context = createContext(inventory);
  const fixture = await temporaryStore(t);
  const producer = await producerFor(fixture.store);
  const rowZeroHttp = await completeReceipt(fixture.store, context, producer, {
    layer: "http_document",
  });
  const rowOneXmlrpc = await completeReceipt(fixture.store, context, producer, {
    attemptId: "00000000-0000-4000-8000-000000000002",
    body: "row one response",
    ordinal: 1,
  });
  const completions = await initializeReferenceAcquisitionCompletions(
    fixture.store,
    context,
  );
  t.after(() => completions.close());
  await completions.publish(rowZeroHttp.reference, {
    layer: "http_document",
    ordinal: 0,
    producer,
  });
  await completions.publish(rowOneXmlrpc.reference, {
    layer: "xmlrpc_page",
    ordinal: 1,
    producer,
  });
  const reads = {};
  const counted = (label, value) =>
    new Proxy(value, {
      get(target, key) {
        if (typeof key === "string" && Object.hasOwn(target, key)) {
          const name = `${label}.${key}`;
          reads[name] = (reads[name] ?? 0) + 1;
        }
        return target[key];
      },
    });
  const accessorProducer = counted("producer", {
    contract: producer.contract,
    identity: counted("identity", producer.identity),
  });
  const plan = await completions.planResume({ producer: accessorProducer });
  assert.deepEqual(reads, {
    "identity.algorithm": 1,
    "identity.bytes": 1,
    "identity.sha256": 1,
    "producer.contract": 1,
    "producer.identity": 1,
  });
  const key = (target) => [target.inventory.ordinal, target.layer];
  assert.deepEqual(
    plan.complete.map((item) => key(item.target)),
    [
      [0, "http_document"],
      [1, "xmlrpc_page"],
    ],
  );
  assert.deepEqual(plan.pending.map(key), [
    [0, "xmlrpc_page"],
    [0, "browser_document"],
    [1, "http_document"],
    [1, "browser_document"],
  ]);
  const plannedTargets = [
    ...plan.complete.map((item) => item.target),
    ...plan.pending,
  ];
  assert(
    plannedTargets.every(
      (target) => target.producer === plannedTargets[0].producer,
    ),
  );
});

test("failed attempts stay pending and distinct complete retries conflict", async (t) => {
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
  const failedReference = (
    await putReferenceAcquisitionAttempt(fixture.store, failed, context)
  ).object;
  const completions = await initializeReferenceAcquisitionCompletions(
    fixture.store,
    context,
  );
  t.after(() => completions.close());
  const request = { layer: "xmlrpc_page", ordinal: 0, producer };
  await assert.rejects(
    completions.publish(failedReference, request),
    /only complete/u,
  );
  assert.equal(await completions.resolve(request), null);
  const target = buildReferenceAcquisitionWorkTarget({ context, ...request });
  const failedLeaf = path.join(
    fixture.root,
    ...referenceAcquisitionCompletionRelativePath(target).split("/"),
  );
  await fs.mkdir(path.dirname(failedLeaf), { mode: 0o700, recursive: true });
  await fs.writeFile(
    failedLeaf,
    serializeReferenceAcquisitionCompletionPointer(
      {
        attempt: failedReference,
        schema:
          "wikijump_full_parity.reference_acquisition_completion_pointer.v1",
        work_identity: target.work_identity,
      },
      target.work_identity,
    ),
    { mode: 0o400 },
  );
  await assert.rejects(completions.resolve(request), /only complete/u);
  await assert.rejects(completions.planResume({ producer }), /only complete/u);
  await fs.unlink(failedLeaf);
  const wrongLayer = await completeReceipt(fixture.store, context, producer, {
    layer: "http_document",
  });
  await assert.rejects(
    completions.publish(wrongLayer.reference, request),
    /wrong layer/u,
  );
  const first = await completeReceipt(fixture.store, context, producer);
  const second = await completeReceipt(fixture.store, context, producer, {
    attemptId: "00000000-0000-4000-8000-000000000002",
    body: "retry response",
  });
  const results = await Promise.allSettled([
    completions.publish(first.reference, request),
    completions.publish(second.reference, request),
  ]);
  assert.equal(
    results.filter((result) => result.status === "fulfilled").length,
    1,
  );
  const rejected = results.find((result) => result.status === "rejected");
  assert(
    rejected.reason instanceof ReferenceAcquisitionCompletionConflictError,
  );
  const winner = results.find((result) => result.status === "fulfilled").value;
  assert.deepEqual(
    (await completions.resolve(request)).attempt_reference,
    winner.attempt_reference,
  );
});

test("resume rejects poisoned completion prefixes instead of treating them as pending", async (t) => {
  const inventory = buildInventory();
  const context = createContext(inventory);
  const fixture = await temporaryStore(t);
  const producer = await producerFor(fixture.store);
  const completions = await initializeReferenceAcquisitionCompletions(
    fixture.store,
    context,
  );
  t.after(() => completions.close());
  const target = buildReferenceAcquisitionWorkTarget({
    context,
    layer: "browser_document",
    ordinal: 0,
    producer,
  });
  const leaf = path.join(
    fixture.root,
    ...referenceAcquisitionCompletionRelativePath(target).split("/"),
  );
  const prefix = path.dirname(leaf);
  const outside = path.join(path.dirname(fixture.root), "outside-prefix");
  await fs.mkdir(outside, { mode: 0o700 });
  await fs.symlink(outside, prefix, "dir");
  await assert.rejects(completions.planResume({ producer }), /symbolic link/u);
  await fs.unlink(prefix);
  await fs.writeFile(prefix, "not a directory", { mode: 0o400 });
  await assert.rejects(completions.planResume({ producer }));
  await fs.unlink(prefix);
  await fs.mkdir(prefix, { mode: 0o755 });
  await fs.chmod(prefix, 0o755);
  await assert.rejects(completions.planResume({ producer }), /mode 700/u);
});

test("resume rejects poisoned pointer leaves and rebound completion directories", async (t) => {
  const inventory = buildInventory();
  const context = createContext(inventory);
  const fixture = await temporaryStore(t);
  const producer = await producerFor(fixture.store);
  const completions = await initializeReferenceAcquisitionCompletions(
    fixture.store,
    context,
  );
  t.after(() => completions.close());
  const xmlRequest = { layer: "xmlrpc_page", ordinal: 0, producer };
  const xmlTarget = buildReferenceAcquisitionWorkTarget({
    context,
    ...xmlRequest,
  });
  const httpRequest = { layer: "http_document", ordinal: 0, producer };
  const httpTarget = buildReferenceAcquisitionWorkTarget({
    context,
    ...httpRequest,
  });
  const http = await completeReceipt(fixture.store, context, producer, {
    layer: "http_document",
  });
  const pointerFor = (target, attempt = http.reference) => ({
    attempt,
    schema: "wikijump_full_parity.reference_acquisition_completion_pointer.v1",
    work_identity: target.work_identity,
  });
  const leafFor = (target) =>
    path.join(
      fixture.root,
      ...referenceAcquisitionCompletionRelativePath(target).split("/"),
    );
  const xmlLeaf = leafFor(xmlTarget);
  await fs.mkdir(path.dirname(xmlLeaf), { mode: 0o700, recursive: true });
  await fs.writeFile(
    xmlLeaf,
    serializeReferenceAcquisitionCompletionPointer(
      pointerFor(xmlTarget),
      xmlTarget.work_identity,
    ),
    { mode: 0o400 },
  );
  const httpLeaf = leafFor(httpTarget);
  await fs.mkdir(path.dirname(httpLeaf), { mode: 0o700, recursive: true });
  await fs.writeFile(httpLeaf, `${JSON.stringify(pointerFor(httpTarget))} \n`, {
    mode: 0o400,
  });
  await assert.rejects(completions.resolve(xmlRequest), /wrong layer/u);
  await assert.rejects(completions.planResume({ producer }), /wrong layer/u);
  await fs.unlink(xmlLeaf);

  await assert.rejects(completions.resolve(httpRequest), /not canonical/u);
  await assert.rejects(completions.planResume({ producer }), /not canonical/u);
  await fs.unlink(httpLeaf);
  await fs.writeFile(httpLeaf, "", { mode: 0o600 });
  await fs.truncate(httpLeaf, 4 * 1024 * 1024 * 1024);
  await fs.chmod(httpLeaf, 0o400);
  await assert.rejects(completions.resolve(httpRequest), /byte limit/u);
  await assert.rejects(completions.planResume({ producer }), /byte limit/u);
  await fs.unlink(httpLeaf);
  await fs.writeFile(httpLeaf, "{}\n", { mode: 0o400 });
  await assert.rejects(
    completions.planResume({ producer }),
    /unexpected fields/u,
  );
  await fs.unlink(httpLeaf);
  const danglingAttempt = {
    algorithm: "sha256",
    bytes: 123,
    sha256: "f".repeat(64),
  };
  await fs.writeFile(
    httpLeaf,
    serializeReferenceAcquisitionCompletionPointer(
      pointerFor(httpTarget, danglingAttempt),
      httpTarget.work_identity,
    ),
    { mode: 0o400 },
  );
  await assert.rejects(completions.planResume({ producer }), {
    code: "ENOENT",
  });
  await fs.unlink(httpLeaf);

  const browserRequest = { layer: "browser_document", ordinal: 0, producer };
  const browserTarget = buildReferenceAcquisitionWorkTarget({
    context,
    ...browserRequest,
  });
  const browserLeaf = leafFor(browserTarget);
  const outside = path.join(path.dirname(fixture.root), "outside");
  await fs.mkdir(path.dirname(browserLeaf), { mode: 0o700, recursive: true });
  await fs.mkdir(browserLeaf, { mode: 0o700 });
  await assert.rejects(
    completions.planResume({ producer }),
    /must be a regular file/u,
  );
  await fs.rmdir(browserLeaf);
  await fs.writeFile(outside, "outside", { mode: 0o400 });
  await fs.symlink(outside, browserLeaf);
  await assert.rejects(completions.planResume({ producer }), { code: "ELOOP" });
  assert.equal(await fs.readFile(outside, "utf8"), "outside");

  await fs.unlink(browserLeaf);
  const xml = await completeReceipt(fixture.store, context, producer, {
    attemptId: "00000000-0000-4000-8000-000000000003",
    body: "valid XML response",
  });
  await completions.publish(xml.reference, xmlRequest);
  assert.deepEqual(
    (await completions.resolve(xmlRequest)).attempt_reference,
    xml.reference,
  );
  const completionRoot = path.join(fixture.root, "completions");
  await fs.rename(completionRoot, `${completionRoot}-original`);
  await fs.mkdir(completionRoot, { mode: 0o700 });
  await assert.rejects(completions.resolve(xmlRequest), /completions changed/u);
  assert.deepEqual(await fs.readdir(completionRoot), []);
});
