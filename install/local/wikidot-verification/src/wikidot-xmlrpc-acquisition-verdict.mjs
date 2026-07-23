import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import { types as utilTypes } from "node:util";

import {
  AtomicPublicationAmbiguousError,
  publishBytesNoReplace,
} from "./atomic-no-replace.mjs";
import { stableStringify } from "./canonical-json.mjs";
import { referenceAcquisitionInventorySha256 } from "./reference-acquisition-attempt.mjs";
import { openWikidotXmlrpcCampaign } from "./reference-acquisition-xmlrpc-campaign.mjs";
import { openWikidotXmlrpcCompletions } from "./reference-acquisition-xmlrpc-completion.mjs";
import { validateReferenceObject } from "./reference-object-store.mjs";

export const WIKIDOT_XMLRPC_ACQUISITION_VERDICT_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_acquisition_verdict.v1";

const MAX_BYTES = 16 * 1024;
const FILE_FLAGS = fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0);
const VERDICT_KEYS = Object.freeze([
  "campaign",
  "completed",
  "implementation",
  "schema",
  "status",
]);

function validateExactDataRecord(value, expectedKeys, label) {
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    utilTypes.isProxy(value)
  ) {
    throw new Error(`${label} must be a data object`);
  }
  let keys;
  let prototype;
  try {
    keys = Reflect.ownKeys(value);
    prototype = Reflect.getPrototypeOf(value);
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
  for (const key of keys) {
    const descriptor = Reflect.getOwnPropertyDescriptor(value, key);
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

function snapshotReference(value, label) {
  return validateReferenceObject(
    validateExactDataRecord(value, ["algorithm", "bytes", "sha256"], label),
  );
}

function normalizeFinalVerdict(value) {
  const verdict = validateExactDataRecord(
    value,
    VERDICT_KEYS,
    "XML-RPC acquisition verdict",
  );
  if (
    verdict.schema !== WIKIDOT_XMLRPC_ACQUISITION_VERDICT_SCHEMA ||
    verdict.status !== "pass" ||
    !Number.isSafeInteger(verdict.completed) ||
    verdict.completed < 0
  ) {
    throw new Error("XML-RPC acquisition verdict is invalid");
  }
  return Object.freeze({
    campaign: snapshotReference(verdict.campaign, "XML-RPC verdict campaign"),
    completed: verdict.completed,
    implementation: snapshotReference(
      verdict.implementation,
      "XML-RPC verdict implementation",
    ),
    schema: WIKIDOT_XMLRPC_ACQUISITION_VERDICT_SCHEMA,
    status: "pass",
  });
}

function finalVerdictBytes(value) {
  const bytes = Buffer.from(
    `${stableStringify(normalizeFinalVerdict(value))}\n`,
  );
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC acquisition verdict exceeds its byte limit");
  }
  return bytes;
}

async function readPublishedVerdict(output) {
  const handle = await fs.open(output, FILE_FLAGS);
  try {
    const before = await handle.stat();
    if (
      !before.isFile() ||
      before.uid !== process.geteuid() ||
      (before.mode & 0o777) !== 0o400 ||
      before.size > MAX_BYTES
    ) {
      throw new Error("XML-RPC acquisition verdict output is invalid");
    }
    const bytes = await handle.readFile();
    const after = await handle.stat();
    if (bytes.byteLength !== before.size || after.size !== before.size) {
      throw new Error("XML-RPC acquisition verdict output changed while read");
    }
    return bytes;
  } finally {
    await handle.close();
  }
}

function publicationOptions(value) {
  const options = validateExactDataRecord(
    value,
    ["campaignReference", "context", "store"],
    "XML-RPC final verdict publication options",
  );
  return Object.freeze({
    campaignReference: snapshotReference(
      options.campaignReference,
      "XML-RPC final verdict campaign reference",
    ),
    context: options.context,
    store: options.store,
  });
}

async function deriveFinalVerdict(value) {
  const options = publicationOptions(value);
  const campaign = await openWikidotXmlrpcCampaign(
    options.store,
    options.campaignReference,
    {
      expectedInventorySha256: referenceAcquisitionInventorySha256(
        options.context,
      ),
    },
  );
  const completions = await openWikidotXmlrpcCompletions(
    options.store,
    options.context,
    campaign.reference,
  );
  try {
    const plan = await completions.planResume();
    if (plan.pending.length !== 0) {
      throw new Error(
        "XML-RPC final verdict requires every campaign target to be semantically complete",
      );
    }
    // Completion means a target has one verified XML-RPC reference state. A
    // deleted tombstone is a state observation, never an empty page response.
    return Object.freeze({
      campaign: campaign.reference,
      completed: plan.complete.length,
      implementation: campaign.descriptor.implementation,
      schema: WIKIDOT_XMLRPC_ACQUISITION_VERDICT_SCHEMA,
      status: "pass",
    });
  } finally {
    await completions.close();
  }
}

export async function publishWikidotXmlrpcAcquisitionVerdict(output, options) {
  const bytes = finalVerdictBytes(await deriveFinalVerdict(options));
  let disposition;
  try {
    disposition = await publishBytesNoReplace(output, bytes, { mode: 0o400 });
  } catch (error) {
    if (
      !(error instanceof AtomicPublicationAmbiguousError) ||
      !error.published
    ) {
      throw error;
    }
    if (!(await readPublishedVerdict(output)).equals(bytes)) {
      throw new Error(
        "ambiguous XML-RPC verdict publication could not be verified",
        {
          cause: error,
        },
      );
    }
    disposition = "ambiguous_verified";
  }
  if (
    disposition === "exists" &&
    !(await readPublishedVerdict(output)).equals(bytes)
  ) {
    throw new Error(
      "XML-RPC acquisition verdict conflicts with existing output",
    );
  }
  return Object.freeze({ bytes, disposition });
}

export function parseWikidotXmlrpcAcquisitionVerdict(value) {
  const bytes = Buffer.from(value);
  if (bytes.byteLength > MAX_BYTES) {
    throw new Error("XML-RPC acquisition verdict exceeds its byte limit");
  }
  let parsed;
  try {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    if (
      !text.endsWith("\n") ||
      text.slice(0, -1).includes("\n") ||
      text.includes("\r")
    ) {
      throw new Error();
    }
    parsed = JSON.parse(text);
  } catch {
    throw new Error(
      "XML-RPC acquisition verdict must contain one canonical UTF-8 JSON line",
    );
  }
  const verdict = normalizeFinalVerdict(parsed);
  if (!finalVerdictBytes(verdict).equals(bytes)) {
    throw new Error("XML-RPC acquisition verdict bytes are not canonical");
  }
  return verdict;
}
