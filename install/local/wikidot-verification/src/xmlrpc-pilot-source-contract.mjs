import {sha256Hex, stableStringify} from "./canonical-json.mjs";
import {validateReferenceObject} from "./reference-object-store.mjs";

export const XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA =
  "wikijump_full_parity.xmlrpc_pilot_manifest_record.v1";
export const XMLRPC_EN_128_DESIGNATED_SOURCE = Object.freeze({
  acquisition_artifact_key:
    "wikidot-xmlrpc-en-128-dcea26590485816ead83156d6a4d5a8dff6c502933e1ea1af3d4f55c294ad571-c3b9434925d8f97b1d50e2530d3d216089ae1015bcad039a1df6ce5a673fba20",
  campaign: Object.freeze({
    algorithm: "sha256",
    bytes: 814,
    sha256: "580c104673997d31f9d58751230f9a50e49b5bebefe7472f504463b619d97a7c",
  }),
  implementation: Object.freeze({
    algorithm: "sha256",
    bytes: 1208,
    sha256: "c3b9434925d8f97b1d50e2530d3d216089ae1015bcad039a1df6ce5a673fba20",
  }),
  input_receipts: Object.freeze({
    inventory: Object.freeze({
      bytes: 119874,
      sha256:
        "d502e0899d8773b947974ecad8e1def6e9c7b9027b480bf13d91586bc8035010",
    }),
    result: Object.freeze({
      bytes: 865,
      sha256:
        "dee2835d022ff1423fb3fabed7bbb06369e0608199640c3d649bacd90c75ba11",
    }),
    throttle: Object.freeze({
      bytes: 723,
      sha256:
        "a3cb81dc535e15a9d04bf5307c58b25629ac5998228ca1b4623a844f761a24fe",
    }),
    verdict: Object.freeze({
      bytes: 355,
      sha256:
        "0a02783bd1721b1a342fde902a77074b307f2be0647c749786a41ddd4b931628",
    }),
  }),
  inventory_sha256:
    "dcea26590485816ead83156d6a4d5a8dff6c502933e1ea1af3d4f55c294ad571",
  row_count: 128,
  verified_pilot_manifest: Object.freeze({
    bytes: 107203,
    sha256: "8b07b79acb217d255a5ab22c9857a85f346fa049ff78f8b9308f656746a1448c",
  }),
});

const ACQUISITION_RUN_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_acquisition_run.v1";
const THROTTLE_RECEIPT_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_throttle_receipt.v1";
export const XMLRPC_PILOT_MAX_RECEIPT_BYTES = 2 * 1024 * 1024;
const RUN_KEYS = Object.freeze([
  "artifact_key",
  "campaign",
  "completed",
  "failure",
  "implementation",
  "inventory",
  "outcome",
  "schema",
  "throttle",
  "verdict",
]);
const THROTTLE_KEYS = Object.freeze([
  "artifact_key",
  "campaign",
  "implementation",
  "inventory_sha256",
  "schema",
  "status",
  "throttle_config",
]);

function validateExactDataRecord(value, keys, label) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    Object.getPrototypeOf(value) !== Object.prototype ||
    stableStringify(Object.keys(value).sort()) !== stableStringify(keys)
  ) {
    throw new Error(`${label} has an invalid shape`);
  }
  return value;
}

function nonEmptyIdentifier(value, label) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > 512 ||
    !/^[A-Za-z0-9][A-Za-z0-9._:-]*$/u.test(value)
  ) {
    throw new Error(`${label} is invalid`);
  }
  return value;
}

function sha256(value, label) {
  if (typeof value !== "string" || !/^[a-f0-9]{64}$/u.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256`);
  }
  return value;
}

function safeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${label} must be a non-negative safe integer`);
  }
  return value;
}

function reference(value, label) {
  try {
    return validateReferenceObject(value);
  } catch {
    throw new Error(`${label} is invalid`);
  }
}

function digestReference(value, label) {
  const object = validateExactDataRecord(value, ["bytes", "sha256"], label);
  return Object.freeze({
    bytes: safeInteger(object.bytes, `${label}.bytes`),
    sha256: sha256(object.sha256, `${label}.sha256`),
  });
}

export function canonicalJsonEqual(left, right) {
  return stableStringify(left) === stableStringify(right);
}

export function jsonl(records) {
  return Buffer.from(
    `${records.map((record) => stableStringify(record)).join("\n")}\n`,
    "utf8",
  );
}

export function bytesIdentity(bytes) {
  return Object.freeze({ bytes: bytes.byteLength, sha256: sha256Hex(bytes) });
}

export function validateRunReceipt(value, rowCount) {
  const run = validateExactDataRecord(value, RUN_KEYS, "XML-RPC run receipt");
  if (
    run.schema !== ACQUISITION_RUN_SCHEMA ||
    run.outcome !== "pass" ||
    run.failure !== null ||
    run.completed !== rowCount
  ) {
    throw new Error("XML-RPC run receipt is not a complete pass");
  }
  const inventory = validateExactDataRecord(
    run.inventory,
    ["row_count", "sha256"],
    "XML-RPC run inventory",
  );
  if (inventory.row_count !== rowCount) {
    throw new Error("XML-RPC run receipt has the wrong row count");
  }
  return Object.freeze({
    artifact_key: nonEmptyIdentifier(
      run.artifact_key,
      "XML-RPC run artifact_key",
    ),
    campaign: reference(run.campaign, "XML-RPC run campaign"),
    completed: rowCount,
    implementation: reference(run.implementation, "XML-RPC run implementation"),
    inventory: Object.freeze({
      row_count: rowCount,
      sha256: sha256(inventory.sha256, "XML-RPC run inventory.sha256"),
    }),
    throttle: reference(run.throttle, "XML-RPC run throttle"),
    verdict: digestReference(run.verdict, "XML-RPC run verdict"),
  });
}

export function validateThrottleReceipt(value, run, inventorySha256) {
  const receipt = validateExactDataRecord(value, THROTTLE_KEYS, "XML-RPC throttle receipt");
  if (
    receipt.schema !== THROTTLE_RECEIPT_SCHEMA ||
    receipt.status !== "sealed" ||
    receipt.artifact_key !== run.artifact_key ||
    receipt.inventory_sha256 !== inventorySha256 ||
    !canonicalJsonEqual(receipt.campaign, run.campaign) ||
    !canonicalJsonEqual(receipt.implementation, run.implementation)
  ) {
    throw new Error("XML-RPC throttle receipt does not bind the run");
  }
  const config = reference(receipt.throttle_config, "XML-RPC throttle config");
  if (!canonicalJsonEqual(config, run.throttle)) {
    throw new Error("XML-RPC throttle receipt conflicts with the run receipt");
  }
  return Object.freeze({ throttle_config: config });
}

export function designatedXmlrpcPilotSource(value) {
  const source = validateExactDataRecord(
    value,
    [
      "acquisition_artifact_key",
      "campaign",
      "implementation",
      "input_receipts",
      "inventory_sha256",
      "row_count",
      "verified_pilot_manifest",
    ],
    "designated XML-RPC pilot source",
  );
  const rowCount = safeInteger(
    source.row_count,
    "designated XML-RPC pilot source row_count",
  );
  if (rowCount === 0) {
    throw new Error(
      "designated XML-RPC pilot source must contain at least one row",
    );
  }
  return Object.freeze({
    acquisition_artifact_key: nonEmptyIdentifier(
      source.acquisition_artifact_key,
      "designated XML-RPC pilot source acquisition_artifact_key",
    ),
    campaign: reference(
      source.campaign,
      "designated XML-RPC pilot source campaign",
    ),
    implementation: reference(
      source.implementation,
      "designated XML-RPC pilot source implementation",
    ),
    input_receipts: Object.freeze({
      inventory: digestReference(
        validateExactDataRecord(
          source.input_receipts,
          ["inventory", "result", "throttle", "verdict"],
          "designated XML-RPC pilot source input_receipts",
        ).inventory,
        "designated XML-RPC pilot source input_receipts.inventory",
      ),
      result: digestReference(
        source.input_receipts.result,
        "designated XML-RPC pilot source input_receipts.result",
      ),
      throttle: digestReference(
        source.input_receipts.throttle,
        "designated XML-RPC pilot source input_receipts.throttle",
      ),
      verdict: digestReference(
        source.input_receipts.verdict,
        "designated XML-RPC pilot source input_receipts.verdict",
      ),
    }),
    inventory_sha256: sha256(
      source.inventory_sha256,
      "designated XML-RPC pilot source inventory_sha256",
    ),
    row_count: rowCount,
    verified_pilot_manifest: digestReference(
      source.verified_pilot_manifest,
      "designated XML-RPC pilot source verified_pilot_manifest",
    ),
  });
}

export function assertDesignatedInputReceipts(designation, inputReceipts) {
  if (!canonicalJsonEqual(designation.input_receipts, inputReceipts)) {
    throw new Error(
      "XML-RPC pilot receipts do not match the designated source",
    );
  }
}

export function assertDesignatedXmlrpcPilotSource(designation, result, context) {
  if (
    designation.row_count !== context.rows.length ||
    designation.acquisition_artifact_key !== result.artifact_key ||
    designation.inventory_sha256 !== context.inventorySha256 ||
    !canonicalJsonEqual(designation.campaign, result.campaign) ||
    !canonicalJsonEqual(designation.implementation, result.implementation)
  ) {
    throw new Error("XML-RPC pilot does not match the designated source");
  }
}

export function exactOrdinalSet(values, count, label) {
  if (!Array.isArray(values) || values.length !== count) {
    throw new Error(`${label} does not contain every expected ordinal`);
  }
  const observed = new Set(values);
  if (
    observed.size !== count ||
    [...observed].some(
      (value) => !Number.isSafeInteger(value) || value < 0 || value >= count,
    )
  ) {
    throw new Error(`${label} has duplicate or out-of-range ordinals`);
  }
  return Object.freeze([...observed].sort((left, right) => left - right));
}

export function manifestRecord(semantic, row) {
  const common = {
    fixture_id: row.fixtureId,
    fullname: row.fullname,
    inventory_sha256: semantic.target.inventory.sha256,
    ordinal: row.ordinal,
    schema: XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA,
    semantic_row_sha256: row.semanticRowSha256,
    source_entity_id: row.sourceEntityId,
    work_identity_sha256: semantic.target.work_identity.sha256,
  };
  if (semantic.kind === "deleted") {
    return Object.freeze({
      ...common,
      reference: Object.freeze({
        kind: "wikidot_deleted",
        tombstone_sha256: sha256Hex(stableStringify(semantic.tombstone)),
      }),
    });
  }
  return Object.freeze({
    ...common,
    reference: Object.freeze({
      content_sha256: semantic.observation.observed.content_sha256,
      html_sha256: semantic.observation.observed.html_sha256,
      kind: "live",
      response: semantic.observation.response,
      revisions: semantic.observation.observed.revisions,
      updated_at: semantic.observation.observed.updated_at,
    }),
  });
}
