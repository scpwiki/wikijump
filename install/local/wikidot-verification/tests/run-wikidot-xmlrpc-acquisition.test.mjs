import assert from "node:assert/strict";
import test from "node:test";

import {sha256Hex, stableStringify} from "../src/corpus-import-manifest.mjs";
import {
  assertPinnedPilotWorkerIdentity,
  derivePilotInventory,
  normalizeRunnerOptions,
  partitionRunnerOptions,
  scrubWikidotCredentials,
  takeCredentialsAfterSeal,
  usage,
  WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY,
} from "../src/wikidot-xmlrpc-acquisition-runner.mjs";
import {
  fixture,
  runnerOptions,
  summaryFor,
} from "./support/run-wikidot-xmlrpc-acquisition-fixture.mjs";

test("runner options are partitioned into immutable workflow phase inputs", () => {
  const phases = partitionRunnerOptions(normalizeRunnerOptions(runnerOptions()));

  assert.equal(Object.isFrozen(phases), true);
  assert.equal(Object.isFrozen(phases.inventory), true);
  assert.deepEqual(Object.keys(phases).sort(), [
    "campaign",
    "inventory",
    "launch",
    "outputs",
    "runtime",
    "source",
    "storage",
  ]);
  assert.equal(phases.inventory.selectionCount, 128);
  assert.equal(phases.campaign.principalId, 5700026);
});

test("pilot selection is deterministic, derives a complete inventory, and omits source host paths", () => {
  const state = fixture();
  const options = {
    expectedFullInventorySha256: state.fullInventory.identity.sha256,
    expectedManifestSha256: sha256Hex(state.manifestBytes),
    expectedSummarySha256: sha256Hex(state.summaryBytes),
    fullInventory: state.fullInventory,
    manifestBytes: state.manifestBytes,
    selectionCount: 3,
    shardCount: 2,
    summaryBytes: state.summaryBytes,
  };
  const first = derivePilotInventory(options);
  const repeated = derivePilotInventory(options);

  assert.equal(first.inventory.rows.length, 3);
  assert.equal(
    first.inventory.identity.sha256,
    repeated.inventory.identity.sha256,
  );
  assert.deepEqual(first.inventoryBytes, repeated.inventoryBytes);
  assert.deepEqual(
    first.inventory.rows.map((row) => row.ordinal),
    [0, 1, 2],
  );
  assert.deepEqual(
    first.inventory.rows.map((row) => row.fullname),
    [...first.inventory.rows.map((row) => row.fullname)].sort(),
  );
  assert.equal(
    first.inventoryBytes.includes(Buffer.from("/private/source", "utf8")),
    false,
  );
  assert.equal(first.selection.selected_count, 3);
  assert.equal(
    first.selection.full_inventory_sha256,
    state.fullInventory.identity.sha256,
  );
});

test("pilot selection rejects a raw manifest that no longer matches the verified source capsule", () => {
  const state = fixture();
  const changedRows = state.rows.map((row) => ({
    ...row,
    source_sha256: "f".repeat(64),
  }));
  const changedManifest = Buffer.from(
    `${changedRows.map((row) => stableStringify(row)).join("\n")}\n`,
  );
  const changedSummary = summaryFor(changedRows, changedManifest);
  assert.throws(
    () =>
      derivePilotInventory({
        expectedFullInventorySha256: state.fullInventory.identity.sha256,
        expectedManifestSha256: sha256Hex(changedManifest),
        expectedSummarySha256: sha256Hex(changedSummary),
        fullInventory: state.fullInventory,
        manifestBytes: changedManifest,
        selectionCount: 2,
        shardCount: 2,
        summaryBytes: changedSummary,
      }),
    /full_inventory_authority_invalid/u,
  );
});

test("runner usage declares the sealed-receipt and exact-identity inputs", () => {
  const text = usage();
  for (const option of [
    "--expected-full-inventory-sha256",
    "--throttle-receipt",
    "--result-receipt",
    "--wikijump-commit",
    "--source-commit",
  ]) {
    assert.match(text, new RegExp(option, "u"));
  }
});

test("runner option parsing accepts canonical UUID campaign nonces", () => {
  const parsed = normalizeRunnerOptions(runnerOptions());
  assert.equal(parsed.campaignNonce, "00000000-0000-4000-8000-000000000001");
  assert.throws(
    () =>
      normalizeRunnerOptions(
        runnerOptions({ "campaign-nonce": "00000000-0000-4000-000000000001" }),
      ),
    /campaign_nonce_invalid/u,
  );
});

test("pilot worker identity is pinned before the coordinator can launch", () => {
  const pinned = normalizeRunnerOptions(
    runnerOptions({
      "source-commit": WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.commit,
      "source-tree": WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.tree,
    }),
  );
  assert.doesNotThrow(() => assertPinnedPilotWorkerIdentity(partitionRunnerOptions(pinned).source));
  assert.throws(
    () =>
      assertPinnedPilotWorkerIdentity(
        partitionRunnerOptions(normalizeRunnerOptions(runnerOptions())).source,
      ),
    /pilot_worker_identity_invalid/u,
  );
});

test("credentials are removed at the seal boundary without entering a receipt", () => {
  const environment = Object.assign(Object.create(null), {
    WIKIDOT_API_KEY: "test-api-key",
    WIKIDOT_APP_NAME: "test-application",
  });
  assert.deepEqual(takeCredentialsAfterSeal(environment), {
    apiKey: "test-api-key",
    appName: "test-application",
  });
  assert.equal(Object.hasOwn(environment, "WIKIDOT_API_KEY"), false);
  assert.equal(Object.hasOwn(environment, "WIKIDOT_APP_NAME"), false);
  environment.WIKIDOT_API_KEY = "another-test-api-key";
  environment.WIKIDOT_APP_NAME = "another-test-application";
  scrubWikidotCredentials(environment);
  assert.deepEqual(environment, Object.create(null));
});
