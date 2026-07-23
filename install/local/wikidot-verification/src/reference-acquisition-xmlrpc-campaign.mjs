import { types as utilTypes } from "node:util";

import { sha256Hex, stableStringify } from "./canonical-json.mjs";
import {
  openWikidotXmlrpcImplementation,
  WIKIDOT_XMLRPC_IMPLEMENTATION_SCHEMA,
} from "./reference-acquisition-xmlrpc-implementation.mjs";
import {
  WIKIDOT_XMLRPC_OBSERVATION_SCHEMA,
  WIKIDOT_XMLRPC_PRODUCER_CONTRACT,
} from "./reference-acquisition-xmlrpc-observation.mjs";
import {
  isReferenceObjectStore,
  validateReferenceObject,
} from "./reference-object-store.mjs";

export const WIKIDOT_XMLRPC_CAMPAIGN_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_campaign.v1";

const BUILD_KEYS = Object.freeze([
  "campaignNonce",
  "implementation",
  "inventorySha256",
  "principalId",
]);
const CAMPAIGN_KEYS = Object.freeze([
  "campaign_id",
  "campaign_nonce",
  "endpoint",
  "fallback_used",
  "implementation",
  "inventory_sha256",
  "layer",
  "method",
  "observation_schema",
  "principal_id",
  "producer_contract",
  "raw_wire_captured",
  "read_only",
  "response_canonicalization",
  "schema",
  "site",
]);
const FIXED_AUTHORITY = Object.freeze({
  endpoint: "https://www.wikidot.com/xml-rpc-api.php",
  fallback_used: false,
  layer: "xmlrpc_page",
  method: "pages.get_one",
  observation_schema: WIKIDOT_XMLRPC_OBSERVATION_SCHEMA,
  producer_contract: WIKIDOT_XMLRPC_PRODUCER_CONTRACT,
  raw_wire_captured: false,
  read_only: true,
  response_canonicalization: "stable-json-v1-jsonl",
  schema: WIKIDOT_XMLRPC_CAMPAIGN_SCHEMA,
  site: "scp-wiki",
});
const FATAL_UTF8_DECODER = new TextDecoder("utf-8", { fatal: true });
const MAX_BYTES = 16 * 1024;
const SHA256_RE = /^[0-9a-f]{64}$/u;
const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;

function dataObject(value, expectedKeys, label) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error(`${label} must be a data object`);
  }
  let prototype;
  let keys;
  let descriptors;
  try {
    prototype = Reflect.getPrototypeOf(value);
    keys = Reflect.ownKeys(value);
    descriptors = keys.map((key) =>
      Reflect.getOwnPropertyDescriptor(value, key),
    );
  } catch {
    throw new Error(`${label} must be a data object`);
  }
  if (
    ![Object.prototype, null].includes(prototype) ||
    keys.some((key) => typeof key !== "string") ||
    stableStringify([...keys].sort()) !== stableStringify(expectedKeys)
  ) {
    throw new Error(`${label} has unexpected fields or prototype`);
  }
  const snapshot = {};
  for (const [index, key] of keys.entries()) {
    const descriptor = descriptors[index];
    if (
      descriptor === undefined ||
      !descriptor.enumerable ||
      !("value" in descriptor)
    ) {
      throw new Error(`${label} must contain only enumerable data fields`);
    }
    Object.defineProperty(snapshot, key, {
      enumerable: true,
      value: descriptor.value,
    });
  }
  return Object.freeze(snapshot);
}

function assertSha256(value, label) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
}

function snapshotReference(value, label) {
  return validateReferenceObject(
    dataObject(value, ["algorithm", "bytes", "sha256"], label),
  );
}

function campaignBody(value) {
  const implementation = snapshotReference(
    value.implementation,
    "campaign implementation reference",
  );
  assertSha256(value.inventory_sha256, "campaign inventory_sha256");
  if (
    typeof value.campaign_nonce !== "string" ||
    !UUID_RE.test(value.campaign_nonce) ||
    !Number.isSafeInteger(value.principal_id) ||
    value.principal_id < 1 ||
    Object.entries(FIXED_AUTHORITY).some(
      ([field, expected]) => value[field] !== expected,
    )
  ) {
    throw new Error("XML-RPC campaign authority fields are invalid");
  }
  return Object.freeze({
    campaign_nonce: value.campaign_nonce,
    endpoint: FIXED_AUTHORITY.endpoint,
    fallback_used: FIXED_AUTHORITY.fallback_used,
    implementation,
    inventory_sha256: value.inventory_sha256,
    layer: FIXED_AUTHORITY.layer,
    method: FIXED_AUTHORITY.method,
    observation_schema: FIXED_AUTHORITY.observation_schema,
    principal_id: value.principal_id,
    producer_contract: FIXED_AUTHORITY.producer_contract,
    raw_wire_captured: FIXED_AUTHORITY.raw_wire_captured,
    read_only: FIXED_AUTHORITY.read_only,
    response_canonicalization: FIXED_AUTHORITY.response_canonicalization,
    schema: FIXED_AUTHORITY.schema,
    site: FIXED_AUTHORITY.site,
  });
}

function normalizeCampaign(value) {
  const input = dataObject(value, CAMPAIGN_KEYS, "XML-RPC campaign");
  const body = campaignBody(input);
  const campaignId = sha256Hex(stableStringify(body));
  if (input.campaign_id !== campaignId) {
    throw new Error("XML-RPC campaign ID does not match its authority fields");
  }
  return Object.freeze({ campaign_id: campaignId, ...body });
}

function canonicalBytes(value) {
  const bytes = Buffer.from(`${stableStringify(normalizeCampaign(value))}\n`);
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC campaign exceeds its byte limit");
  }
  return bytes;
}

function inputBytes(value) {
  if (
    value === null ||
    typeof value !== "object" ||
    utilTypes.isProxy(value) ||
    !(value instanceof Uint8Array)
  ) {
    throw new Error("XML-RPC campaign input must be bytes");
  }
  try {
    return Buffer.from(value);
  } catch {
    throw new Error("XML-RPC campaign input must be bytes");
  }
}

function assertStore(store) {
  if (!isReferenceObjectStore(store)) {
    throw new Error("reference object store is required");
  }
}

function campaignProducer(reference) {
  return Object.freeze({
    contract: WIKIDOT_XMLRPC_PRODUCER_CONTRACT,
    identity: snapshotReference(reference, "campaign producer reference"),
  });
}

export function buildWikidotXmlrpcCampaign(options) {
  const input = dataObject(options, BUILD_KEYS, "campaign build options");
  const body = campaignBody({
    campaign_nonce: input.campaignNonce,
    ...FIXED_AUTHORITY,
    implementation: input.implementation,
    inventory_sha256: input.inventorySha256,
    principal_id: input.principalId,
  });
  return normalizeCampaign({
    campaign_id: sha256Hex(stableStringify(body)),
    ...body,
  });
}

export function serializeWikidotXmlrpcCampaign(value) {
  return canonicalBytes(value);
}

export function parseWikidotXmlrpcCampaign(value) {
  const bytes = inputBytes(value);
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC campaign exceeds its byte limit");
  }
  let parsed;
  try {
    const text = FATAL_UTF8_DECODER.decode(bytes);
    if (
      !text.endsWith("\n") ||
      text.slice(0, -1).includes("\n") ||
      text.includes("\r")
    ) {
      throw new Error();
    }
    parsed = JSON.parse(text);
  } catch {
    throw new Error("XML-RPC campaign must contain one UTF-8 JSON line");
  }
  const normalized = normalizeCampaign(parsed);
  if (!canonicalBytes(normalized).equals(bytes)) {
    throw new Error("XML-RPC campaign bytes are not canonical");
  }
  return normalized;
}

function campaignRecord(descriptor, reference, implementation, disposition) {
  const object = snapshotReference(reference, "campaign reference");
  return Object.freeze({
    descriptor,
    disposition,
    implementation: implementation.descriptor,
    producer: campaignProducer(object),
    reference: object,
  });
}

export async function putWikidotXmlrpcCampaign(store, value) {
  assertStore(store);
  const descriptor = normalizeCampaign(value);
  const implementation = await openWikidotXmlrpcImplementation(
    store,
    descriptor.implementation,
  );
  if (
    implementation.descriptor.schema !== WIKIDOT_XMLRPC_IMPLEMENTATION_SCHEMA
  ) {
    throw new Error("XML-RPC implementation schema is invalid");
  }
  const result = await store.putBytes(canonicalBytes(descriptor));
  return campaignRecord(
    descriptor,
    result.object,
    implementation,
    result.disposition,
  );
}

export async function openWikidotXmlrpcCampaign(store, reference, options) {
  assertStore(store);
  const input = dataObject(
    options,
    ["expectedInventorySha256"],
    "campaign open options",
  );
  assertSha256(input.expectedInventorySha256, "expected inventory identity");
  const object = snapshotReference(reference, "campaign reference");
  const descriptor = parseWikidotXmlrpcCampaign(
    await store.readObject(object, { maxBytes: MAX_BYTES }),
  );
  if (descriptor.inventory_sha256 !== input.expectedInventorySha256) {
    throw new Error("XML-RPC campaign has the wrong inventory identity");
  }
  const implementation = await openWikidotXmlrpcImplementation(
    store,
    descriptor.implementation,
  );
  return campaignRecord(descriptor, object, implementation, "opened");
}
