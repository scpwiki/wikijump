import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import {sha256Hex, stableStringify} from "../../src/corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionAttempt,
  putReferenceAcquisitionAttempt,
} from "../../src/reference-acquisition-attempt.mjs";
import {buildReferenceAcquisitionInventory} from "../../src/reference-acquisition-inventory.mjs";
import {initializeReferenceObjectStore} from "../../src/reference-object-store.mjs";
import {createReferenceAcquisitionContext} from "../../src/reference-acquisition-work-target.mjs";

export const ATTEMPT_ID = "00000000-0000-4000-8000-000000000001";
export const STARTED_AT = "2026-07-18T17:30:00.000Z";
export const FINISHED_AT = "2026-07-18T17:30:01.000Z";
export const FIXTURES = "../../fixtures/reference-acquisition-attempt-v1/";

export function buildInventory(fullname = "theme:雪-space") {
  const fullnames = Array.isArray(fullname) ? fullname : [fullname];
  const rows = fullnames.map((currentFullname, index) => ({
    attachments: [],
    fullname: currentFullname,
    meta_sha256: "a".repeat(64),
    parent_fullname: null,
    revisions: 42,
    source_branch: "en",
    source_entity_id: `00000000-0000-4000-8000-${String(173 + index).padStart(12, "0")}`,
    source_sha256: "b".repeat(64),
    source_site: "scp-wiki",
    updated_at: "2026-07-18T12:34:56+00:00",
  }));
  const manifestBytes = Buffer.from(
    `${rows.map((row) => stableStringify(row)).join("\n")}\n`,
  );
  const summary = {
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
  };
  const summaryBytes = Buffer.from(`${stableStringify(summary)}\n`);
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

export function createContext(inventory, expected = inventory.identity.sha256) {
  return createReferenceAcquisitionContext(inventory, {
    expectedIdentitySha256: expected,
  });
}

export function golden(name) {
  return fs.readFile(new URL(`${FIXTURES}${name}`, import.meta.url));
}

export async function temporaryStore(t) {
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

export async function producerFor(store) {
  return {
    contract: "wikijump_full_parity.wikidot_xmlrpc_acquirer.v1",
    identity: (await store.putBytes(Buffer.from("producer contract"))).object,
  };
}

export async function completeReceipt(
  store,
  context,
  producer,
  {
    attemptId = ATTEMPT_ID,
    body = "response",
    layer = "xmlrpc_page",
    ordinal = 0,
  } = {},
) {
  const response = (await store.putBytes(Buffer.from(body))).object;
  const attempt = buildReferenceAcquisitionAttempt(
    attemptInput(
      context,
      producer,
      [{ media_type: "application/xml", object: response, role: "response" }],
      { attemptId, layer, ordinal },
    ),
  );
  const stored = await putReferenceAcquisitionAttempt(store, attempt, context);
  return { attempt, reference: stored.object, response };
}

export function attemptInput(context, producer, objects, overrides = {}) {
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
