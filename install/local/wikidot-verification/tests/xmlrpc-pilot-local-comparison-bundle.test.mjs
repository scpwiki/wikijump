import assert from "node:assert/strict";
import test from "node:test";

import {openVerifiedXmlrpcPilotBundle} from "../src/xmlrpc-pilot-local-comparison-bundle.mjs";
import {
  designatedXmlrpcPilotSource,
  exactOrdinalSet,
  manifestRecord,
  validateRunReceipt,
  validateThrottleReceipt,
  XMLRPC_EN_128_DESIGNATED_SOURCE,
  XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA,
} from "../src/xmlrpc-pilot-source-contract.mjs";

test("verified XML-RPC pilot bundle exposes its fixed designation and rejects relative roots before reading receipts", async () => {
  assert.equal(XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA, "wikijump_full_parity.xmlrpc_pilot_manifest_record.v1");
  assert.equal(XMLRPC_EN_128_DESIGNATED_SOURCE.row_count, 128);
  assert.equal(Object.isFrozen(XMLRPC_EN_128_DESIGNATED_SOURCE), true);
  await assert.rejects(() => openVerifiedXmlrpcPilotBundle({pilotRoot: "relative"}), /absolute path/u);
});


test("pilot source contract binds run, throttle, designation, and manifest identity", () => {
  const object = (character, bytes = 1) => ({
    algorithm: "sha256",
    bytes,
    sha256: character.repeat(64),
  });
  const run = validateRunReceipt({
    artifact_key: "pilot-run",
    campaign: object("a"),
    completed: 1,
    failure: null,
    implementation: object("b"),
    inventory: {row_count: 1, sha256: "c".repeat(64)},
    outcome: "pass",
    schema: "wikijump_full_parity.wikidot_xmlrpc_acquisition_run.v1",
    throttle: object("d"),
    verdict: {bytes: 2, sha256: "e".repeat(64)},
  }, 1);
  assert.equal(run.completed, 1);
  const throttle = validateThrottleReceipt({
    artifact_key: run.artifact_key,
    campaign: run.campaign,
    implementation: run.implementation,
    inventory_sha256: run.inventory.sha256,
    schema: "wikijump_full_parity.wikidot_xmlrpc_throttle_receipt.v1",
    status: "sealed",
    throttle_config: run.throttle,
  }, run, run.inventory.sha256);
  assert.deepEqual(throttle.throttle_config, run.throttle);
  assert.deepEqual(exactOrdinalSet([0], 1, "pilot"), [0]);

  const designation = designatedXmlrpcPilotSource(XMLRPC_EN_128_DESIGNATED_SOURCE);
  assert.equal(designation.row_count, 128);
  const record = manifestRecord({
    kind: "deleted",
    target: {
      inventory: {sha256: "f".repeat(64)},
      work_identity: {sha256: "1".repeat(64)},
    },
    tombstone: {status: "deleted"},
  }, {
    fixtureId: "EN:alpha",
    fullname: "alpha",
    ordinal: 0,
    semanticRowSha256: "2".repeat(64),
    sourceEntityId: "entity-1",
  });
  assert.equal(record.reference.kind, "wikidot_deleted");
});
