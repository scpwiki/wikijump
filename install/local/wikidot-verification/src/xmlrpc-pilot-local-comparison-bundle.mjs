import {constants as fsConstants} from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

import {openCorpusFileNoSymlinks} from "./corpus-file-reader.mjs";
import {
  createReferenceAcquisitionContext,
  referenceAcquisitionInventoryRow,
} from "./reference-acquisition-attempt.mjs";
import {openWikidotXmlrpcCampaign} from "./reference-acquisition-xmlrpc-campaign.mjs";
import {openWikidotXmlrpcCompletions} from "./reference-acquisition-xmlrpc-completion.mjs";
import {openReferenceObjectStore} from "./reference-object-store.mjs";
import {parseWikidotXmlrpcAcquisitionVerdict} from "./wikidot-xmlrpc-acquisition-verdict.mjs";
import {
  assertDesignatedInputReceipts,
  assertDesignatedXmlrpcPilotSource,
  bytesIdentity,
  canonicalJsonEqual,
  designatedXmlrpcPilotSource,
  exactOrdinalSet,
  jsonl,
  manifestRecord,
  validateRunReceipt,
  validateThrottleReceipt,
  XMLRPC_EN_128_DESIGNATED_SOURCE,
  XMLRPC_PILOT_MAX_RECEIPT_BYTES,
} from "./xmlrpc-pilot-source-contract.mjs";

export {
  XMLRPC_EN_128_DESIGNATED_SOURCE,
  XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA,
} from "./xmlrpc-pilot-source-contract.mjs";

async function readOpenRegularFile(handle, label, maxBytes) {
  const before = await handle.stat();
  if (!before.isFile() || before.size > maxBytes) {
    throw new Error(`${label} is not a bounded regular file`);
  }
  const bytes = await handle.readFile();
  const after = await handle.stat();
  if (
    bytes.byteLength !== before.size ||
    after.size !== before.size ||
    after.dev !== before.dev ||
    after.ino !== before.ino
  ) {
    throw new Error(`${label} changed while being read`);
  }
  return bytes;
}

async function readPilotReceipt(root, relativePath, label) {
  const handle = await openCorpusFileNoSymlinks(
    root,
    path.join(root, relativePath),
    fsConstants.O_RDONLY |
      (fsConstants.O_NONBLOCK ?? 0) |
      (fsConstants.O_NOFOLLOW ?? 0),
  );
  try {
    return await readOpenRegularFile(handle, label, XMLRPC_PILOT_MAX_RECEIPT_BYTES);
  } finally {
    await handle.close();
  }
}

function parseJson(bytes, label) {
  try {
    return JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
  } catch {
    throw new Error(`${label} is not valid UTF-8 JSON`);
  }
}

export async function openVerifiedXmlrpcPilotBundle({
  pilotRoot,
  sourceExpectation = XMLRPC_EN_128_DESIGNATED_SOURCE,
} = {}) {
  if (typeof pilotRoot !== "string" || !path.isAbsolute(pilotRoot)) {
    throw new Error("pilotRoot must be an absolute path");
  }
  const designation = designatedXmlrpcPilotSource(sourceExpectation);
  const root = await fs.realpath(pilotRoot);
  const [inventoryBytes, resultBytes, verdictBytes, throttleBytes] =
    await Promise.all([
      readPilotReceipt(
        root,
        "receipts/inventory.json",
        "XML-RPC inventory receipt",
      ),
      readPilotReceipt(root, "receipts/result.json", "XML-RPC result receipt"),
      readPilotReceipt(
        root,
        "receipts/verdict.json",
        "XML-RPC verdict receipt",
      ),
      readPilotReceipt(
        root,
        "receipts/throttle.json",
        "XML-RPC throttle receipt",
      ),
    ]);
  const inputReceipts = Object.freeze({
    inventory: bytesIdentity(inventoryBytes),
    result: bytesIdentity(resultBytes),
    throttle: bytesIdentity(throttleBytes),
    verdict: bytesIdentity(verdictBytes),
  });
  assertDesignatedInputReceipts(designation, inputReceipts);
  const inventory = parseJson(inventoryBytes, "XML-RPC inventory receipt");
  const context = createReferenceAcquisitionContext(inventory, {
    expectedIdentitySha256: inventory?.identity?.sha256,
  });
  const result = validateRunReceipt(
    parseJson(resultBytes, "XML-RPC result receipt"),
    context.rows.length,
  );
  assertDesignatedXmlrpcPilotSource(designation, result, context);
  const finalVerdict = parseWikidotXmlrpcAcquisitionVerdict(verdictBytes);
  if (
    finalVerdict.completed !== context.rows.length ||
    !canonicalJsonEqual(finalVerdict.campaign, result.campaign) ||
    !canonicalJsonEqual(finalVerdict.implementation, result.implementation) ||
    !canonicalJsonEqual(result.verdict, bytesIdentity(verdictBytes)) ||
    result.inventory.sha256 !== context.inventorySha256
  ) {
    throw new Error(
      "XML-RPC final verdict does not bind the supplied inventory and run",
    );
  }
  const throttle = validateThrottleReceipt(
    parseJson(throttleBytes, "XML-RPC throttle receipt"),
    result,
    context.inventorySha256,
  );
  const storePath = path.join(root, "store");
  const storeInfo = await fs.lstat(storePath);
  if (!storeInfo.isDirectory() || storeInfo.isSymbolicLink()) {
    throw new Error("XML-RPC object store must be a non-symlink directory");
  }
  const store = await openReferenceObjectStore(storePath);
  let completions;
  try {
    await store.readObject(throttle.throttle_config, {
      maxBytes: XMLRPC_PILOT_MAX_RECEIPT_BYTES,
    });
    const campaign = await openWikidotXmlrpcCampaign(store, result.campaign, {
      expectedInventorySha256: context.inventorySha256,
    });
    if (!canonicalJsonEqual(campaign.descriptor.implementation, result.implementation)) {
      throw new Error("XML-RPC campaign implementation does not bind the run");
    }
    completions = await openWikidotXmlrpcCompletions(
      store,
      context,
      campaign.reference,
    );
    const plan = await completions.planResume();
    if (plan.pending.length !== 0) {
      throw new Error("XML-RPC pilot has incomplete semantic completions");
    }
    exactOrdinalSet(
      plan.complete.map((item) => item.target.inventory.ordinal),
      context.rows.length,
      "XML-RPC pilot completion set",
    );
    const rows = [];
    for (let ordinal = 0; ordinal < context.rows.length; ordinal += 1) {
      const semantic = await completions.resolve({ ordinal });
      if (semantic === null || semantic.target.inventory.ordinal !== ordinal) {
        throw new Error(
          "XML-RPC pilot completion disappeared during materialization",
        );
      }
      const inventoryRow = referenceAcquisitionInventoryRow(context, ordinal);
      const manifest = manifestRecord(semantic, inventoryRow);
      rows.push(
        Object.freeze({
          manifest,
          reference:
            semantic.kind === "live"
              ? Object.freeze({
                  kind: "live",
                  content: semantic.response.content,
                  html: semantic.response.html,
                })
              : Object.freeze({ kind: "wikidot_deleted" }),
        }),
      );
    }
    const manifests = rows.map((row) => row.manifest);
    const manifestBytes = jsonl(manifests);
    const manifestIdentity = bytesIdentity(manifestBytes);
    if (!canonicalJsonEqual(designation.verified_pilot_manifest, manifestIdentity)) {
      throw new Error(
        "XML-RPC pilot manifest does not match the designated source",
      );
    }
    return Object.freeze({
      manifest_bytes: manifestBytes,
      manifest_identity: manifestIdentity,
      rows: Object.freeze(rows),
      source: Object.freeze({
        acquisition_artifact_key: result.artifact_key,
        campaign: result.campaign,
        designation,
        implementation: result.implementation,
        input_receipts: inputReceipts,
        inventory_sha256: context.inventorySha256,
        throttle_config: throttle.throttle_config,
      }),
    });
  } finally {
    await completions?.close().catch(() => {});
    await store.close().catch(() => {});
  }
}
