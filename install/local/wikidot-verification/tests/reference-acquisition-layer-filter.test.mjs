import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { sha256Hex, stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionWorkTarget,
  createReferenceAcquisitionContext,
  listReferenceAcquisitionWorkTargets,
  referenceAcquisitionInventorySha256,
} from "../src/reference-acquisition-work-target.mjs";
import {
  initializeReferenceAcquisitionCompletions,
  referenceAcquisitionCompletionRelativePath,
} from "../src/reference-acquisition-completion.mjs";
import { buildReferenceAcquisitionInventory } from "../src/reference-acquisition-inventory.mjs";
import { initializeReferenceObjectStore } from "../src/reference-object-store.mjs";

function buildInventory(fullnames = ["theme:alpha", "theme:omega"]) {
  const rows = fullnames.map((fullname, index) => ({
    attachments: [],
    fullname,
    meta_sha256: "a".repeat(64),
    parent_fullname: null,
    revisions: 42,
    source_branch: "en",
    source_entity_id: `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
    source_sha256: "b".repeat(64),
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
    expectedCount: rows.length,
    expectedManifestSha256: sha256Hex(manifestBytes),
    expectedSummarySha256: sha256Hex(summaryBytes),
    family: "EN",
    manifestBytes,
    shardCount: 2,
    sourceOrigin: "https://scp-wiki.wikidot.com",
    summaryBytes,
  });
}

function contextFor(inventory) {
  return createReferenceAcquisitionContext(inventory, {
    expectedIdentitySha256: inventory.identity.sha256,
  });
}

function referenceFor(bytes) {
  return {
    algorithm: "sha256",
    bytes: bytes.byteLength,
    sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
  };
}

test("layer filters preserve canonical row and layer order", () => {
  const inventory = buildInventory();
  const context = contextFor(inventory);
  const producer = {
    contract: "wikijump_full_parity.wikidot_xmlrpc_acquirer.v1",
    identity: { algorithm: "sha256", bytes: 1, sha256: "c".repeat(64) },
  };
  const key = (target) => [target.inventory.ordinal, target.layer];
  assert.equal(
    referenceAcquisitionInventorySha256(context),
    inventory.identity.sha256,
  );
  assert.deepEqual(
    listReferenceAcquisitionWorkTargets({ context, producer }).map(key),
    [
      [0, "xmlrpc_page"],
      [0, "http_document"],
      [0, "browser_document"],
      [1, "xmlrpc_page"],
      [1, "http_document"],
      [1, "browser_document"],
    ],
  );
  assert.deepEqual(
    listReferenceAcquisitionWorkTargets({
      context,
      layers: ["browser_document", "xmlrpc_page"],
      producer,
    }).map(key),
    [
      [0, "xmlrpc_page"],
      [0, "browser_document"],
      [1, "xmlrpc_page"],
      [1, "browser_document"],
    ],
  );
});

test("layer filters reject hostile, sparse, duplicate, and unknown inputs", () => {
  const context = contextFor(buildInventory());
  const producer = {
    contract: "wikijump_full_parity.wikidot_xmlrpc_acquirer.v1",
    identity: { algorithm: "sha256", bytes: 1, sha256: "c".repeat(64) },
  };
  const secret = "sentinel-layer-secret";
  const accessor = [];
  Object.defineProperty(accessor, "0", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const proxy = new Proxy(["xmlrpc_page"], {
    ownKeys() {
      throw new Error(secret);
    },
  });
  const symbolic = ["xmlrpc_page"];
  symbolic[Symbol("secret")] = secret;
  for (const layers of [
    [],
    new Array(1),
    accessor,
    proxy,
    symbolic,
    ["xmlrpc_page", "xmlrpc_page"],
    ["unknown_layer"],
  ]) {
    assert.throws(
      () => listReferenceAcquisitionWorkTargets({ context, layers, producer }),
      (error) => !error.message.includes(secret),
    );
  }
  const accessorOptions = { context, producer };
  Object.defineProperty(accessorOptions, "layers", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const proxyOptions = new Proxy(
    { context, layers: ["xmlrpc_page"], producer },
    {
      ownKeys() {
        throw new Error(secret);
      },
    },
  );
  const symbolicOptions = { context, producer, [Symbol("secret")]: secret };
  for (const options of [accessorOptions, proxyOptions, symbolicOptions]) {
    assert.throws(
      () => listReferenceAcquisitionWorkTargets(options),
      (error) => !error.message.includes(secret),
    );
  }
});

test("XML-only resume ignores unrelated poisoned completion leaves", async (t) => {
  const inventory = buildInventory(["scp-173"]);
  const context = contextFor(inventory);
  const parent = await fs.mkdtemp(path.join(os.tmpdir(), "layer-filter-"));
  const root = path.join(parent, "store");
  const store = await initializeReferenceObjectStore(root);
  t.after(async () => {
    await store.close();
    await fs.rm(parent, { force: true, recursive: true });
  });
  let producer;
  let producerBytes;
  let xmlTarget;
  let httpTarget;
  for (let index = 0; index < 4096; index += 1) {
    const bytes = Buffer.from(`producer-${index}`);
    const candidate = {
      contract: "wikijump_full_parity.wikidot_xmlrpc_acquirer.v1",
      identity: referenceFor(bytes),
    };
    const xml = buildReferenceAcquisitionWorkTarget({
      context,
      layer: "xmlrpc_page",
      ordinal: 0,
      producer: candidate,
    });
    const http = buildReferenceAcquisitionWorkTarget({
      context,
      layer: "http_document",
      ordinal: 0,
      producer: candidate,
    });
    if (
      xml.work_identity.sha256.slice(0, 2) ===
      http.work_identity.sha256.slice(0, 2)
    ) {
      ({ producer, producerBytes, xmlTarget, httpTarget } = {
        producer: candidate,
        producerBytes: bytes,
        xmlTarget: xml,
        httpTarget: http,
      });
      break;
    }
  }
  assert(producer, "test requires a same-prefix producer fixture");
  assert.deepEqual(
    (await store.putBytes(producerBytes)).object,
    producer.identity,
  );
  const completions = await initializeReferenceAcquisitionCompletions(
    store,
    context,
  );
  t.after(() => completions.close());
  const secret = "sentinel-resume-options-secret";
  const accessorOptions = { producer };
  Object.defineProperty(accessorOptions, "layers", {
    enumerable: true,
    get() {
      throw new Error(secret);
    },
  });
  const proxyOptions = new Proxy(
    { layers: ["xmlrpc_page"], producer },
    {
      ownKeys() {
        throw new Error(secret);
      },
    },
  );
  const symbolicOptions = { producer, [Symbol("secret")]: secret };
  for (const options of [accessorOptions, proxyOptions, symbolicOptions]) {
    await assert.rejects(
      completions.planResume(options),
      (error) => !error.message.includes(secret),
    );
  }
  const httpLeaf = path.join(
    root,
    ...referenceAcquisitionCompletionRelativePath(httpTarget).split("/"),
  );
  await fs.mkdir(path.dirname(httpLeaf), { mode: 0o700, recursive: true });
  await fs.writeFile(httpLeaf, "{}\n", { mode: 0o400 });
  const xmlPlan = await completions.planResume({
    layers: ["xmlrpc_page"],
    producer,
  });
  assert.deepEqual(xmlPlan.complete, []);
  assert.deepEqual(xmlPlan.pending, [xmlTarget]);
  await assert.rejects(
    completions.planResume({ producer }),
    /unexpected fields/u,
  );
  await fs.unlink(httpLeaf);
  const xmlLeaf = path.join(
    root,
    ...referenceAcquisitionCompletionRelativePath(xmlTarget).split("/"),
  );
  await fs.writeFile(xmlLeaf, "{}\n", { mode: 0o400 });
  await assert.rejects(
    completions.planResume({ layers: ["xmlrpc_page"], producer }),
    /unexpected fields/u,
  );
});
