#!/usr/bin/env node

import crypto from "node:crypto";
import { constants as fsConstants, createReadStream } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { publishBytesNoReplace } from "./atomic-no-replace.mjs";
import { readExactGitTreeFiles } from "./exact-git-blob.mjs";
import {
  buildReferenceAcquisitionAttempt,
  createReferenceAcquisitionContext,
  putReferenceAcquisitionAttempt,
  referenceAcquisitionInventoryRow,
} from "./reference-acquisition-attempt.mjs";
import {
  buildWikidotXmlrpcCampaign,
  putWikidotXmlrpcCampaign,
} from "./reference-acquisition-xmlrpc-campaign.mjs";
import { initializeWikidotXmlrpcCompletions } from "./reference-acquisition-xmlrpc-completion.mjs";
import {
  buildWikidotXmlrpcImplementation,
  putWikidotXmlrpcImplementation,
} from "./reference-acquisition-xmlrpc-implementation.mjs";
import {
  buildWikidotXmlrpcDeletedTombstone,
  buildWikidotXmlrpcObservation,
  serializeWikidotXmlrpcDeletedTombstone,
  serializeWikidotXmlrpcObservation,
  serializeWikidotXmlrpcResponse,
  WIKIDOT_XMLRPC_DELETED_TOMBSTONE_ROLE,
} from "./reference-acquisition-xmlrpc-observation.mjs";
import {
  buildReferenceAcquisitionInventory,
  serializeReferenceAcquisitionInventory,
} from "./reference-acquisition-inventory.mjs";
import {
  hashWikidotXmlrpcInstalledEnvironmentManifest,
  putWikidotXmlrpcInstalledEnvironmentManifest,
} from "./wikidot-xmlrpc-installed-environment-manifest.mjs";
import { initializeReferenceObjectStore } from "./reference-object-store.mjs";
import {
  assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest,
  assertWikidotXmlrpcPythonEnvironmentMatchesWorkerAuthority,
  buildWikidotXmlrpcPythonEnvironment,
  putWikidotXmlrpcPythonEnvironment,
} from "./wikidot-xmlrpc-python-environment.mjs";
import {
  materializeWikidotXmlrpcPrivateCapsule,
  prepareWikidotXmlrpcRuntime,
  prepareWikidotXmlrpcWorkerSource,
} from "./wikidot-xmlrpc-private-capsule.mjs";
import { publishWikidotXmlrpcAcquisitionVerdict } from "./wikidot-xmlrpc-acquisition-verdict.mjs";
import {
  buildWikidotXmlrpcWorkerAuthority,
  putWikidotXmlrpcWorkerAuthority,
} from "./wikidot-xmlrpc-worker-authority.mjs";
import {
  OperatorSignalError,
  WikidotXmlrpcWorkerClient,
  WorkerProtocolError,
  WorkerTerminatedError,
} from "./wikidot-xmlrpc-worker-client.mjs";
import { sha256Hex, stableStringify } from "./corpus-import-manifest.mjs";

export const WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY = Object.freeze({
  commit: "bfa0ca8c39f54f16610a7267880b6dad01789396",
  file_sha256:
    "184618c4ed86b96ca002e6dfd0351593dd43d14875b6c6d3ab29c593f65c15ab",
  blob: "3eaa311c1b4b76d2711631452ae5ee0a09abffba",
  tree: "0ba1cb193c4aabf34556648a233a73ffa7aedffb",
});
// This duplicates the bootstrap's Node-only source list deliberately. The
// coordinator cannot import the bootstrap before proving its own closure, so it
// holds an exact copy and tests require both declarations to remain identical.
export const WIKIDOT_XMLRPC_CANONICAL_COORDINATOR_SOURCE_PATHS = Object.freeze(
  [
    "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition.mjs",
    "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs",
    "install/local/wikidot-verification/src/atomic-no-replace.mjs",
    "install/local/wikidot-verification/src/corpus-file-reader.mjs",
    "install/local/wikidot-verification/src/corpus-import-manifest.mjs",
    "install/local/wikidot-verification/src/exact-git-blob.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-attachment.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-attempt.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-completion-index.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-completion.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-inventory-validation.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-inventory.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-summary.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-campaign.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-completion.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-implementation.mjs",
    "install/local/wikidot-verification/src/reference-acquisition-xmlrpc-observation.mjs",
    "install/local/wikidot-verification/src/reference-object-store.mjs",
    "install/local/wikidot-verification/src/resource-manifest.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-verdict.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-exact-data-record.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-installed-environment-manifest.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-private-capsule.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-python-environment.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-attestation.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-authority.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-client.mjs",
    "install/local/wikidot-verification/src/wikidot-xmlrpc-worker-session-capability.mjs",
  ].sort(),
);
const OUTPUT_RECEIPT_FIELDS = Object.freeze([
  "inventoryOutput",
  "throttleReceipt",
  "resultReceipt",
  "verdict",
]);
const EN_SOURCE_ORIGIN = "https://scp-wiki.wikidot.com";
const FULLNAME_SELECTION_DOMAIN = "wikijump-full-en-xmlrpc-pilot-v1\0";
const PRIVATE_DIRECTORY_MODE = 0o700;
const PRIVATE_FILE_MODE = 0o400;
const MATERIALIZED_COORDINATOR_PATH =
  "install/local/wikidot-verification/src/wikidot-xmlrpc-acquisition-runner.mjs";
const MATERIALIZED_DESCRIPTOR_FD = 4;
const MATERIALIZED_DESCRIPTOR_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_materialized_launch.v1";
const MATERIALIZED_ENTRYPOINT_PATH =
  "install/local/wikidot-verification/scripts/run-wikidot-xmlrpc-acquisition-materialized.mjs";
const MAX_MATERIALIZED_DESCRIPTOR_BYTES = 64 * 1024;
const MAX_MATERIALIZED_FILE_BYTES = 2 * 1024 * 1024;
const MAX_MATERIALIZED_FILES = 128;
const MAX_MATERIALIZED_TOTAL_BYTES = 32 * 1024 * 1024;
const RESULT_SCHEMA = "wikijump_full_parity.wikidot_xmlrpc_acquisition_run.v1";
const THROTTLE_CONFIG_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_throttle_config.v1";
const THROTTLE_RECEIPT_SCHEMA =
  "wikijump_full_parity.wikidot_xmlrpc_throttle_receipt.v1";
const SHA1_RE = /^[0-9a-f]{40}$/u;
const SHA256_RE = /^[0-9a-f]{64}$/u;
const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/u;

const OPTION_NAMES = Object.freeze([
  "campaign-nonce",
  "capsule-parent",
  "expected-full-inventory-sha256",
  "expected-manifest-sha256",
  "expected-summary-sha256",
  "full-inventory",
  "inventory-output",
  "manifest",
  "principal-id",
  "result-receipt",
  "runtime-python",
  "runtime-root",
  "runtime-venv-config",
  "runtime-version",
  "selection-count",
  "shards",
  "source-commit",
  "source-git-dir",
  "source-tree",
  "store",
  "summary",
  "throttle-receipt",
  "verdict",
  "wikijump-commit",
  "wikijump-git-dir",
  "wikijump-tree",
]);

function fail(code) {
  throw new Error(`XML-RPC acquisition runner ${code}`);
}

function canonicalBytes(value) {
  return Buffer.from(`${stableStringify(value)}\n`, "utf8");
}

function assertSha256(value, code) {
  if (typeof value !== "string" || !SHA256_RE.test(value)) fail(code);
  return value;
}

function assertSha1(value, code) {
  if (typeof value !== "string" || !SHA1_RE.test(value)) fail(code);
  return value;
}

function assertAbsolutePath(value, code) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > 4096 ||
    value.includes("\0") ||
    !path.isAbsolute(value)
  ) {
    fail(code);
  }
  return value;
}

function assertRelativePath(value, code) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > 4096 ||
    value.includes("\0") ||
    value.startsWith("/") ||
    value.includes("\\") ||
    value
      .split("/")
      .some((part) => part.length === 0 || part === "." || part === "..")
  ) {
    fail(code);
  }
  return value;
}

function assertSafeInteger(value, code, maximum = Number.MAX_SAFE_INTEGER) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 1 || parsed > maximum) {
    fail(code);
  }
  return parsed;
}

function parseArguments(argv) {
  if (argv.length === 1 && ["--help", "-h"].includes(argv[0])) {
    return Object.freeze({ help: true });
  }
  const values = {};
  for (let index = 0; index < argv.length; index += 2) {
    const option = argv[index];
    const value = argv[index + 1];
    if (
      typeof option !== "string" ||
      !option.startsWith("--") ||
      value === undefined ||
      typeof value !== "string" ||
      value.startsWith("--")
    ) {
      fail("arguments_invalid");
    }
    const name = option.slice(2);
    if (!OPTION_NAMES.includes(name) || Object.hasOwn(values, name)) {
      fail("arguments_invalid");
    }
    values[name] = value;
  }
  if (OPTION_NAMES.some((name) => !Object.hasOwn(values, name))) {
    fail("arguments_incomplete");
  }
  return Object.freeze(values);
}

export function normalizeRunnerOptions(values) {
  if (values?.help === true) return values;
  if (values === null || typeof values !== "object" || Array.isArray(values)) {
    fail("arguments_invalid");
  }
  const input = values;
  if (
    Object.keys(input).length !== OPTION_NAMES.length ||
    OPTION_NAMES.some((name) => !Object.hasOwn(input, name))
  ) {
    fail("arguments_incomplete");
  }
  const selectionCount = assertSafeInteger(
    input["selection-count"],
    "selection_count_invalid",
  );
  const shards = assertSafeInteger(input.shards, "shards_invalid", 4096);
  const principalId = assertSafeInteger(
    input["principal-id"],
    "principal_id_invalid",
  );
  if (
    typeof input["campaign-nonce"] !== "string" ||
    !UUID_RE.test(input["campaign-nonce"])
  ) {
    fail("campaign_nonce_invalid");
  }
  return Object.freeze({
    campaignNonce: input["campaign-nonce"],
    capsuleParent: assertAbsolutePath(
      input["capsule-parent"],
      "capsule_parent_invalid",
    ),
    expectedFullInventorySha256: assertSha256(
      input["expected-full-inventory-sha256"],
      "full_inventory_identity_invalid",
    ),
    expectedManifestSha256: assertSha256(
      input["expected-manifest-sha256"],
      "manifest_identity_invalid",
    ),
    expectedSummarySha256: assertSha256(
      input["expected-summary-sha256"],
      "summary_identity_invalid",
    ),
    fullInventoryPath: assertAbsolutePath(
      input["full-inventory"],
      "full_inventory_path_invalid",
    ),
    inventoryOutput: assertAbsolutePath(
      input["inventory-output"],
      "inventory_output_invalid",
    ),
    manifestPath: assertAbsolutePath(input.manifest, "manifest_path_invalid"),
    principalId,
    resultReceipt: assertAbsolutePath(
      input["result-receipt"],
      "result_receipt_invalid",
    ),
    runtimePython: assertRelativePath(
      input["runtime-python"],
      "runtime_python_invalid",
    ),
    runtimeRoot: assertAbsolutePath(
      input["runtime-root"],
      "runtime_root_invalid",
    ),
    runtimeVenvConfig: assertRelativePath(
      input["runtime-venv-config"],
      "runtime_venv_config_invalid",
    ),
    runtimeVersion: input["runtime-version"],
    selectionCount,
    shards,
    sourceCommit: assertSha1(input["source-commit"], "source_commit_invalid"),
    sourceGitDirectory: assertAbsolutePath(
      input["source-git-dir"],
      "source_git_directory_invalid",
    ),
    sourceTree: assertSha1(input["source-tree"], "source_tree_invalid"),
    storeRoot: assertAbsolutePath(input.store, "store_root_invalid"),
    summaryPath: assertAbsolutePath(input.summary, "summary_path_invalid"),
    throttleReceipt: assertAbsolutePath(
      input["throttle-receipt"],
      "throttle_receipt_invalid",
    ),
    verdict: assertAbsolutePath(input.verdict, "verdict_invalid"),
    wikijumpCommit: assertSha1(
      input["wikijump-commit"],
      "wikijump_commit_invalid",
    ),
    wikijumpGitDirectory: assertAbsolutePath(
      input["wikijump-git-dir"],
      "wikijump_git_directory_invalid",
    ),
    wikijumpTree: assertSha1(input["wikijump-tree"], "wikijump_tree_invalid"),
  });
}

export function partitionRunnerOptions(options) {
  return Object.freeze({
    campaign: Object.freeze({
      campaignNonce: options.campaignNonce,
      principalId: options.principalId,
    }),
    inventory: Object.freeze({
      expectedFullInventorySha256: options.expectedFullInventorySha256,
      expectedManifestSha256: options.expectedManifestSha256,
      expectedSummarySha256: options.expectedSummarySha256,
      fullInventoryPath: options.fullInventoryPath,
      manifestPath: options.manifestPath,
      selectionCount: options.selectionCount,
      shards: options.shards,
      summaryPath: options.summaryPath,
    }),
    launch: Object.freeze({
      wikijumpCommit: options.wikijumpCommit,
      wikijumpGitDirectory: options.wikijumpGitDirectory,
      wikijumpTree: options.wikijumpTree,
    }),
    outputs: Object.freeze({
      inventoryOutput: options.inventoryOutput,
      resultReceipt: options.resultReceipt,
      throttleReceipt: options.throttleReceipt,
      verdict: options.verdict,
    }),
    runtime: Object.freeze({
      pythonExecutablePath: options.runtimePython,
      pythonVersion: options.runtimeVersion,
      root: options.runtimeRoot,
      venvConfigPath: options.runtimeVenvConfig,
    }),
    source: Object.freeze({
      commitOid: options.sourceCommit,
      gitDirectory: options.sourceGitDirectory,
      treeOid: options.sourceTree,
    }),
    storage: Object.freeze({
      capsuleParent: options.capsuleParent,
      storeRoot: options.storeRoot,
    }),
  });
}

export function assertPinnedPilotWorkerIdentity(options) {
  if (
    options.commitOid !== WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.commit ||
    options.treeOid !== WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.tree
  ) {
    fail("pilot_worker_identity_invalid");
  }
}

async function outputDestinationIdentity(value) {
  const destination = path.resolve(value);
  const parent = path.dirname(destination);
  let canonicalParent;
  try {
    canonicalParent = await fs.realpath(parent);
    const parentStat = await fs.lstat(canonicalParent, { bigint: true });
    if (!parentStat.isDirectory()) fail("output_destination_invalid");
  } catch {
    fail("output_destination_invalid");
  }
  const canonicalDestination = path.join(
    canonicalParent,
    path.basename(destination),
  );
  try {
    const leaf = await fs.lstat(canonicalDestination, { bigint: true });
    if (leaf.isSymbolicLink()) fail("output_destination_invalid");
    return Object.freeze({
      destination: canonicalDestination,
      inode: `${leaf.dev}:${leaf.ino}`,
    });
  } catch (error) {
    if (error?.code === "ENOENT") {
      return Object.freeze({ destination: canonicalDestination, inode: null });
    }
    fail("output_destination_invalid");
  }
}

export async function assertDistinctOutputDestinations(options) {
  const destinations = await Promise.all(
    OUTPUT_RECEIPT_FIELDS.map(async (field) =>
      Object.freeze({
        field,
        ...(await outputDestinationIdentity(options[field])),
      }),
    ),
  );
  const paths = new Set();
  const inodes = new Set();
  for (const destination of destinations) {
    if (paths.has(destination.destination)) fail("output_destinations_alias");
    paths.add(destination.destination);
    if (destination.inode !== null) {
      if (inodes.has(destination.inode)) fail("output_destinations_alias");
      inodes.add(destination.inode);
    }
  }
}

function decodeUtf8(bytes, code) {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    fail(code);
  }
}

function canonicalLines(bytes, code) {
  const text = decodeUtf8(bytes, code);
  if (!text.endsWith("\n") || text.includes("\r")) fail(code);
  const lines = text.slice(0, -1).split("\n");
  if (lines.length === 0 || lines.some((line) => line.length === 0)) fail(code);
  return lines;
}

function selectionRank(fullname) {
  return sha256Hex(
    Buffer.from(`${FULLNAME_SELECTION_DOMAIN}${fullname}`, "utf8"),
  );
}

function subsetSummary(rows, manifestBytes) {
  const visibilityCounts = {};
  for (const row of rows) {
    if (row.source_browser_visibility !== undefined) {
      visibilityCounts[row.source_browser_visibility] =
        (visibilityCounts[row.source_browser_visibility] ?? 0) + 1;
    }
  }
  return {
    attachment_count: rows.reduce(
      (total, row) => total + (row.attachments?.length ?? 0),
      0,
    ),
    attachment_page_count: rows.filter(
      (row) => (row.attachments?.length ?? 0) > 0,
    ).length,
    first_fullname: rows[0].fullname,
    last_fullname: rows.at(-1).fullname,
    manifest_sha256: sha256Hex(manifestBytes),
    parent_count: rows.filter((row) => row.parent_fullname !== null).length,
    required_browser_count: rows.filter((row) => row.required_browser === true)
      .length,
    row_count: rows.length,
    source_branches: ["en"],
    source_browser_visibility_counts: visibilityCounts,
    source_required_actor_count: rows.filter(
      (row) =>
        row.source_required_actor !== undefined &&
        row.source_required_actor !== null,
    ).length,
    source_sites: ["scp-wiki"],
  };
}

function rawRowMatchesInventory(row, expected) {
  return (
    row?.fullname === expected.fullname &&
    row.source_entity_id === expected.sourceEntityId &&
    row.meta_sha256 === expected.baseline.meta_sha256 &&
    row.revisions === expected.baseline.revisions &&
    row.source_sha256 === expected.baseline.source_sha256 &&
    row.updated_at === expected.baseline.updated_at
  );
}

export function derivePilotInventory({
  expectedFullInventorySha256,
  expectedManifestSha256,
  expectedSummarySha256,
  fullInventory,
  manifestBytes,
  selectionCount,
  shardCount,
  summaryBytes,
}) {
  if (sha256Hex(manifestBytes) !== expectedManifestSha256) {
    fail("manifest_identity_mismatch");
  }
  if (sha256Hex(summaryBytes) !== expectedSummarySha256) {
    fail("summary_identity_mismatch");
  }
  let summary;
  try {
    summary = JSON.parse(decodeUtf8(summaryBytes, "summary_invalid"));
  } catch {
    fail("summary_invalid");
  }
  if (summary?.manifest_sha256 !== expectedManifestSha256) {
    fail("summary_manifest_mismatch");
  }
  const context = createReferenceAcquisitionContext(fullInventory, {
    expectedIdentitySha256: expectedFullInventorySha256,
  });
  if (
    fullInventory.family !== "EN" ||
    fullInventory.source_origin !== EN_SOURCE_ORIGIN ||
    fullInventory.source_manifest?.sha256 !== expectedManifestSha256 ||
    fullInventory.source_manifest?.summary_sha256 !== expectedSummarySha256 ||
    selectionCount > context.rows.length
  ) {
    fail("full_inventory_authority_invalid");
  }
  const selected = context.rows
    .map((row) => Object.freeze({ rank: selectionRank(row.fullname), row }))
    .sort((left, right) =>
      left.rank < right.rank
        ? -1
        : left.rank > right.rank
          ? 1
          : left.row.fullname < right.row.fullname
            ? -1
            : left.row.fullname > right.row.fullname
              ? 1
              : 0,
    )
    .slice(0, selectionCount);
  const selectedByFullname = new Map(
    selected.map(({ row }) => [row.fullname, row]),
  );
  const selectedRows = [];
  const selectedLines = [];
  for (const line of canonicalLines(manifestBytes, "manifest_invalid")) {
    let row;
    try {
      row = JSON.parse(line);
    } catch {
      fail("manifest_invalid");
    }
    const expected = selectedByFullname.get(row?.fullname);
    if (expected === undefined) continue;
    if (!rawRowMatchesInventory(row, expected)) {
      fail("manifest_inventory_mismatch");
    }
    selectedByFullname.delete(row.fullname);
    selectedLines.push(line);
    selectedRows.push(row);
  }
  if (selectedByFullname.size !== 0 || selectedRows.length !== selectionCount) {
    fail("manifest_selection_incomplete");
  }
  const selectedManifestBytes = Buffer.from(`${selectedLines.join("\n")}\n`);
  const selectedSummaryBytes = canonicalBytes(
    subsetSummary(selectedRows, selectedManifestBytes),
  );
  const inventory = buildReferenceAcquisitionInventory({
    expectedCount: selectionCount,
    expectedManifestSha256: sha256Hex(selectedManifestBytes),
    expectedSummarySha256: sha256Hex(selectedSummaryBytes),
    family: "EN",
    manifestBytes: selectedManifestBytes,
    shardCount,
    sourceOrigin: EN_SOURCE_ORIGIN,
    summaryBytes: selectedSummaryBytes,
  });
  const fullRows = new Map(
    fullInventory.rows.map((row) => [row.fullname, row]),
  );
  for (const row of inventory.rows) {
    const full = fullRows.get(row.fullname);
    if (
      full === undefined ||
      [
        "attachment_count",
        "attachment_inventory_sha256",
        "baseline",
        "input_line_sha256",
        "semantic_row_sha256",
        "source_entity_id",
      ].some(
        (field) => stableStringify(row[field]) !== stableStringify(full[field]),
      )
    ) {
      fail("pilot_inventory_full_identity_mismatch");
    }
  }
  return Object.freeze({
    inventory,
    inventoryBytes: Buffer.from(
      serializeReferenceAcquisitionInventory(inventory),
    ),
    selection: Object.freeze({
      algorithm: "sha256-fullname-rank-v1",
      full_inventory_sha256: expectedFullInventorySha256,
      full_manifest_sha256: expectedManifestSha256,
      full_summary_sha256: expectedSummarySha256,
      selected_count: selectionCount,
    }),
  });
}

async function readImmutableFile(filePath, maxBytes) {
  let handle;
  try {
    handle = await fs.open(
      filePath,
      fsConstants.O_RDONLY | (fsConstants.O_NOFOLLOW ?? 0),
    );
    const before = await handle.stat({ bigint: true });
    if (
      !before.isFile() ||
      before.uid !== BigInt(process.geteuid()) ||
      (before.mode & 0o777n) !== BigInt(PRIVATE_FILE_MODE) ||
      before.size > BigInt(maxBytes)
    ) {
      fail("receipt_invalid");
    }
    const bytes = await handle.readFile();
    const after = await handle.stat({ bigint: true });
    if (
      after.size !== before.size ||
      bytes.byteLength !== Number(before.size)
    ) {
      fail("receipt_changed");
    }
    return bytes;
  } finally {
    await handle?.close().catch(() => {});
  }
}

async function publishExactReceipt(filePath, bytes) {
  const disposition = await publishBytesNoReplace(filePath, bytes, {
    mode: PRIVATE_FILE_MODE,
  });
  if (disposition === "exists") {
    const existing = await readImmutableFile(filePath, bytes.byteLength);
    if (!existing.equals(bytes)) fail("receipt_conflict");
  }
  return disposition;
}

function roleFile(manifest, role, code) {
  const file = manifest.files.find((candidate) => candidate.path === role);
  if (file === undefined) fail(code);
  return file;
}

async function publishAuthorityArtifacts(
  store,
  runtime,
  source,
  coordinatorIdentity,
) {
  const python = roleFile(
    runtime.manifest,
    runtime.manifest.python_executable_path,
    "runtime_python_role_missing",
  );
  const venv = roleFile(
    runtime.manifest,
    runtime.manifest.venv_config_path,
    "runtime_venv_role_missing",
  );
  const environmentHash = hashWikidotXmlrpcInstalledEnvironmentManifest(
    runtime.manifest,
  );
  const authority = buildWikidotXmlrpcWorkerAuthority({
    dependencyEnvironmentSha256: environmentHash,
    dependencyLockFileSha256: source.dependencyLockFileSha256,
    dependencyRecipeSha256: source.dependencyRecipeSha256,
    pythonExecutableSha256: python.sha256,
    pythonVersion: runtime.manifest.python_version,
    venvConfigSha256: venv.sha256,
    workerBlobOid: source.workerBlobOid,
    workerFileSha256: source.workerFileSha256,
    workerRepositoryCommit: source.workerRepositoryCommit,
    workerRepositoryTree: source.workerRepositoryTree,
  });
  const environment = buildWikidotXmlrpcPythonEnvironment({
    dependencyEnvironmentSha256: environmentHash,
    dependencyLockBlobOid: source.dependencyLockBlobOid,
    dependencyLockFileSha256: source.dependencyLockFileSha256,
    dependencyRecipeBlobOid: source.dependencyRecipeBlobOid,
    dependencyRecipeSha256: source.dependencyRecipeSha256,
    pythonExecutableSha256: python.sha256,
    pythonImplementation: runtime.manifest.python_implementation,
    pythonVersion: runtime.manifest.python_version,
    venvConfigSha256: venv.sha256,
    workerBlobOid: source.workerBlobOid,
    workerFileSha256: source.workerFileSha256,
    workerRepositoryCommit: source.workerRepositoryCommit,
    workerRepositoryTree: source.workerRepositoryTree,
  });
  assertWikidotXmlrpcPythonEnvironmentMatchesWorkerAuthority(
    environment,
    authority,
  );
  assertWikidotXmlrpcPythonEnvironmentMatchesInstalledEnvironmentManifest(
    environment,
    runtime.manifest,
  );
  const [installed, authorityRecord, environmentRecord] = await Promise.all([
    putWikidotXmlrpcInstalledEnvironmentManifest(store, runtime.manifest),
    putWikidotXmlrpcWorkerAuthority(store, authority),
    putWikidotXmlrpcPythonEnvironment(store, environment),
  ]);
  const implementation = await putWikidotXmlrpcImplementation(
    store,
    buildWikidotXmlrpcImplementation({
      coordinatorFileSha256: coordinatorIdentity.fileSha256,
      dependencyLockFileSha256: source.dependencyLockFileSha256,
      nodeVersion: process.version,
      pythonVersion: runtime.manifest.python_version,
      wikijumpCommit: coordinatorIdentity.wikijumpCommit,
      wikijumpTree: coordinatorIdentity.wikijumpTree,
      workerFileSha256: source.workerFileSha256,
      workerRepositoryCommit: source.workerRepositoryCommit,
      workerRepositoryTree: source.workerRepositoryTree,
    }),
  );
  return Object.freeze({
    authority: authorityRecord,
    environment: environmentRecord,
    implementation,
    installed,
  });
}

function materializedDataObject(value, expectedKeys, code) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    fail(code);
  }
  const actual = Object.keys(value).sort();
  if (
    actual.length !== expectedKeys.length ||
    actual.some((key, index) => key !== expectedKeys[index])
  ) {
    fail(code);
  }
  return value;
}

function materializedSourcePath(value, code) {
  const normalized = assertRelativePath(value, code);
  const segments = normalized.split("/");
  if (
    segments.length > 16 ||
    segments.some((segment) => !/^[A-Za-z0-9_][A-Za-z0-9._-]*$/u.test(segment))
  ) {
    fail(code);
  }
  return normalized;
}

function normalizeMaterializedLaunchDescriptor(value) {
  const input = materializedDataObject(
    value,
    [
      "coordinator_path",
      "entrypoint_path",
      "files",
      "materialization_root",
      "schema",
      "wikijump_commit",
      "wikijump_tree",
    ],
    "materialized_launch_descriptor_invalid",
  );
  if (
    input.schema !== MATERIALIZED_DESCRIPTOR_SCHEMA ||
    input.coordinator_path !== MATERIALIZED_COORDINATOR_PATH ||
    input.entrypoint_path !== MATERIALIZED_ENTRYPOINT_PATH ||
    !Array.isArray(input.files) ||
    input.files.length !==
      WIKIDOT_XMLRPC_CANONICAL_COORDINATOR_SOURCE_PATHS.length ||
    input.files.length > MAX_MATERIALIZED_FILES
  ) {
    fail("materialized_launch_descriptor_invalid");
  }
  const root = assertAbsolutePath(
    input.materialization_root,
    "materialized_launch_descriptor_invalid",
  );
  const files = [];
  let previousPath = null;
  for (const value of input.files) {
    const file = materializedDataObject(
      value,
      ["blob_oid", "bytes", "path", "sha256"],
      "materialized_launch_descriptor_invalid",
    );
    const sourcePath = materializedSourcePath(
      file.path,
      "materialized_launch_descriptor_invalid",
    );
    if (
      sourcePath !== file.path ||
      (previousPath !== null && previousPath >= sourcePath) ||
      !Number.isSafeInteger(file.bytes) ||
      file.bytes < 0 ||
      file.bytes > MAX_MATERIALIZED_FILE_BYTES
    ) {
      fail("materialized_launch_descriptor_invalid");
    }
    if (
      sourcePath !==
      WIKIDOT_XMLRPC_CANONICAL_COORDINATOR_SOURCE_PATHS[files.length]
    ) {
      fail("materialized_launch_descriptor_invalid");
    }
    files.push(
      Object.freeze({
        blobOid: assertSha1(
          file.blob_oid,
          "materialized_launch_descriptor_invalid",
        ),
        byteLength: file.bytes,
        path: sourcePath,
        sha256: assertSha256(
          file.sha256,
          "materialized_launch_descriptor_invalid",
        ),
      }),
    );
    previousPath = sourcePath;
  }
  return Object.freeze({
    files: Object.freeze(files),
    materializationRoot: root,
    wikijumpCommit: assertSha1(
      input.wikijump_commit,
      "materialized_launch_descriptor_invalid",
    ),
    wikijumpTree: assertSha1(
      input.wikijump_tree,
      "materialized_launch_descriptor_invalid",
    ),
  });
}

async function readMaterializedLaunchDescriptor() {
  let bytes;
  try {
    bytes = await readBoundedMaterializedDescriptor(MATERIALIZED_DESCRIPTOR_FD);
  } catch (error) {
    if (error?.message === "materialized_descriptor_too_large") {
      fail("materialized_launch_descriptor_invalid");
    }
    fail("materialized_launch_descriptor_unavailable");
  }
  if (
    bytes.byteLength === 0 ||
    bytes.byteLength > MAX_MATERIALIZED_DESCRIPTOR_BYTES
  ) {
    fail("materialized_launch_descriptor_invalid");
  }
  try {
    return normalizeMaterializedLaunchDescriptor(
      JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)),
    );
  } catch (error) {
    if (
      error?.message?.includes("XML-RPC acquisition runner materialized_launch")
    ) {
      throw error;
    }
    fail("materialized_launch_descriptor_invalid");
  }
}

function readBoundedMaterializedDescriptor(fileDescriptor) {
  return new Promise((resolve, reject) => {
    let stream;
    let settled = false;
    let totalBytes = 0;
    const chunks = [];
    const settle = (callback) => {
      if (settled) return;
      settled = true;
      callback();
    };
    try {
      stream = createReadStream(null, {
        autoClose: false,
        emitClose: false,
        fd: fileDescriptor,
        highWaterMark: 4096,
      });
    } catch (error) {
      reject(error);
      return;
    }
    stream.on("data", (chunk) => {
      if (settled) return;
      totalBytes += chunk.byteLength;
      if (totalBytes > MAX_MATERIALIZED_DESCRIPTOR_BYTES) {
        settle(() => {
          stream.destroy();
          reject(new Error("materialized_descriptor_too_large"));
        });
        return;
      }
      chunks.push(chunk);
    });
    stream.once("error", (error) => settle(() => reject(error)));
    stream.once("end", () =>
      settle(() => resolve(Buffer.concat(chunks, totalBytes))),
    );
  });
}

async function verifyPrivateMaterializedRoot(root) {
  let raw;
  let resolved;
  let stat;
  try {
    raw = await fs.lstat(root, { bigint: true });
    if (raw.isSymbolicLink()) fail("materialized_launch_private_root_invalid");
    resolved = await fs.realpath(root);
    stat = await fs.lstat(resolved, { bigint: true });
  } catch (error) {
    if (
      error?.message?.includes("XML-RPC acquisition runner materialized_launch")
    ) {
      throw error;
    }
    fail("materialized_launch_private_root_invalid");
  }
  if (
    resolved !== root ||
    !stat.isDirectory() ||
    stat.uid !== BigInt(process.geteuid()) ||
    (stat.mode & 0o777n) !== BigInt(PRIVATE_DIRECTORY_MODE)
  ) {
    fail("materialized_launch_private_root_invalid");
  }
}

async function verifyPrivateMaterializedFile(root, file, exact) {
  let current = root;
  const segments = file.path.split("/");
  for (const segment of segments.slice(0, -1)) {
    current = path.join(current, segment);
    let stat;
    try {
      stat = await fs.lstat(current, { bigint: true });
    } catch {
      fail("materialized_launch_private_file_invalid");
    }
    if (
      stat.isSymbolicLink() ||
      !stat.isDirectory() ||
      stat.uid !== BigInt(process.geteuid()) ||
      (stat.mode & 0o777n) !== BigInt(PRIVATE_DIRECTORY_MODE)
    ) {
      fail("materialized_launch_private_file_invalid");
    }
  }
  const destination = path.resolve(root, ...segments);
  if (destination === root || !destination.startsWith(`${root}${path.sep}`)) {
    fail("materialized_launch_private_file_invalid");
  }
  const bytes = await readImmutableFile(
    destination,
    MAX_MATERIALIZED_FILE_BYTES,
  ).catch(() => fail("materialized_launch_private_file_invalid"));
  if (
    bytes.byteLength !== file.byteLength ||
    sha256Hex(bytes) !== file.sha256 ||
    !bytes.equals(exact.readBytes())
  ) {
    fail("materialized_launch_private_file_invalid");
  }
  return destination;
}

export async function assertMaterializedCoordinatorLaunch(options) {
  const descriptor = await readMaterializedLaunchDescriptor();
  if (
    descriptor.wikijumpCommit !== options.wikijumpCommit ||
    descriptor.wikijumpTree !== options.wikijumpTree
  ) {
    fail("materialized_launch_identity_mismatch");
  }
  await verifyPrivateMaterializedRoot(descriptor.materializationRoot);
  let exactTree;
  try {
    exactTree = await readExactGitTreeFiles(
      { gitDirectory: options.wikijumpGitDirectory },
      {
        commitOid: descriptor.wikijumpCommit,
        treeOid: descriptor.wikijumpTree,
      },
      descriptor.files.map((file) => file.path),
      {
        maxBytesPerFile: MAX_MATERIALIZED_FILE_BYTES,
        maxFiles: MAX_MATERIALIZED_FILES,
        maxTotalBytes: MAX_MATERIALIZED_TOTAL_BYTES,
      },
    );
  } catch {
    fail("materialized_launch_git_invalid");
  }
  if (exactTree.files.length !== descriptor.files.length) {
    fail("materialized_launch_git_invalid");
  }
  let coordinatorDestination = null;
  for (const [index, file] of descriptor.files.entries()) {
    const exact = exactTree.files[index];
    if (
      exact.path !== file.path ||
      exact.blobOid !== file.blobOid ||
      exact.byteLength !== file.byteLength ||
      exact.sha256 !== file.sha256
    ) {
      fail("materialized_launch_git_invalid");
    }
    const destination = await verifyPrivateMaterializedFile(
      descriptor.materializationRoot,
      file,
      exact,
    );
    if (file.path === MATERIALIZED_COORDINATOR_PATH) {
      coordinatorDestination = destination;
    }
  }
  if (
    coordinatorDestination === null ||
    coordinatorDestination !== fileURLToPath(import.meta.url)
  ) {
    fail("materialized_launch_required");
  }
  const coordinator = descriptor.files.find(
    (file) => file.path === MATERIALIZED_COORDINATOR_PATH,
  );
  if (coordinator === undefined) fail("materialized_launch_descriptor_invalid");
  return Object.freeze({
    fileSha256: coordinator.sha256,
    wikijumpCommit: descriptor.wikijumpCommit,
    wikijumpTree: descriptor.wikijumpTree,
  });
}

function artifactKey(inventory, implementation) {
  return `wikidot-xmlrpc-en-${inventory.rows.length}-${inventory.identity.sha256}-${implementation.object.sha256}`;
}

async function sealThrottleReceipt({
  artifacts,
  campaign,
  inventory,
  throttleReceipt,
  selection,
}) {
  const key = artifactKey(inventory, artifacts.implementation);
  const descriptor = {
    artifact_key: key,
    campaign: campaign.reference,
    environment: artifacts.environment.object,
    implementation: artifacts.implementation.object,
    installed_environment: artifacts.installed.object,
    inventory_sha256: inventory.identity.sha256,
    rate_capacity: 1,
    rate_refill_per_second: 0.25,
    read_only: true,
    retry_max_attempts: 5,
    retry_max_delay_seconds: 120,
    retry_wait_max_seconds: 60,
    schema: THROTTLE_CONFIG_SCHEMA,
    selection,
    status: "sealed",
    worker_authority: artifacts.authority.object,
  };
  const config = await artifacts.store.putBytes(canonicalBytes(descriptor));
  const receipt = {
    artifact_key: key,
    campaign: campaign.reference,
    implementation: artifacts.implementation.object,
    inventory_sha256: inventory.identity.sha256,
    schema: THROTTLE_RECEIPT_SCHEMA,
    status: "sealed",
    throttle_config: config.object,
  };
  await publishExactReceipt(throttleReceipt, canonicalBytes(receipt));
  return Object.freeze({ artifactKey: key, config: config.object, receipt });
}

export function scrubWikidotCredentials(environment = process.env) {
  delete environment.WIKIDOT_APP_NAME;
  delete environment.WIKIDOT_API_KEY;
}

export function takeCredentialsAfterSeal(environment = process.env) {
  const appName = environment.WIKIDOT_APP_NAME;
  const apiKey = environment.WIKIDOT_API_KEY;
  scrubWikidotCredentials(environment);
  if (
    typeof appName !== "string" ||
    appName.length === 0 ||
    typeof apiKey !== "string" ||
    apiKey.length === 0
  ) {
    return null;
  }
  return Object.freeze({ apiKey, appName });
}

export function expectedWorkerExitCode(result) {
  if (result.retryable === true) return 75;
  return result.code === "worker_internal_error" ? 70 : null;
}

export async function capturePending({ completions, context, store, worker }) {
  const plan = await completions.planResume();
  for (const pending of plan.pending) {
    const ordinal = pending.inventory.ordinal;
    const row = referenceAcquisitionInventoryRow(context, ordinal);
    const startedAt = new Date().toISOString();
    const result = await worker.capture(ordinal, row.fullname);
    const finishedAt = new Date().toISOString();
    if (
      result.ok === false &&
      result.code === "wikidot_deleted" &&
      result.retryable === false
    ) {
      const tombstoneInput = {
        context,
        finishedAt,
        ordinal,
        producer: pending.producer,
        startedAt,
      };
      const tombstone = buildWikidotXmlrpcDeletedTombstone(tombstoneInput);
      const tombstoneReference = (
        await store.putBytes(
          serializeWikidotXmlrpcDeletedTombstone(tombstone, tombstoneInput),
        )
      ).object;
      const attempt = await putReferenceAcquisitionAttempt(
        store,
        buildReferenceAcquisitionAttempt({
          attemptId: crypto.randomUUID(),
          context,
          finishedAt,
          layer: "xmlrpc_page",
          objects: [
            {
              media_type: "application/json",
              object: tombstoneReference,
              role: WIKIDOT_XMLRPC_DELETED_TOMBSTONE_ROLE,
            },
          ],
          ordinal,
          outcome: "complete",
          producer: pending.producer,
          startedAt,
        }),
        context,
      );
      await completions.publish(attempt.object, { ordinal });
      continue;
    }
    if (result.ok !== true) {
      const attempt = await putReferenceAcquisitionAttempt(
        store,
        buildReferenceAcquisitionAttempt({
          attemptId: crypto.randomUUID(),
          context,
          failure: { code: result.code, retryable: result.retryable },
          finishedAt,
          layer: "xmlrpc_page",
          objects: [],
          ordinal,
          outcome: "failed",
          producer: pending.producer,
          startedAt,
        }),
        context,
      );
      const exitCode = expectedWorkerExitCode(result);
      if (exitCode !== null) await worker.expectExit(exitCode);
      return Object.freeze({
        failure: Object.freeze({
          attempt: attempt.object,
          code: result.code,
          retryable: result.retryable,
        }),
        status: result.retryable ? "retryable_stop" : "terminal_stop",
        workerExited: exitCode !== null,
      });
    }
    const response = result.response;
    const responseReference = (
      await store.putBytes(
        serializeWikidotXmlrpcResponse(response, row.fullname),
      )
    ).object;
    const observationInput = {
      context,
      finishedAt,
      ordinal,
      producer: pending.producer,
      response,
      responseReference,
      startedAt,
    };
    const observation = buildWikidotXmlrpcObservation(observationInput);
    const observationReference = (
      await store.putBytes(
        serializeWikidotXmlrpcObservation(observation, observationInput),
      )
    ).object;
    const attempt = await putReferenceAcquisitionAttempt(
      store,
      buildReferenceAcquisitionAttempt({
        attemptId: crypto.randomUUID(),
        context,
        finishedAt,
        layer: "xmlrpc_page",
        objects: [
          {
            media_type: "application/json",
            object: observationReference,
            role: "observation",
          },
          {
            media_type: "application/json",
            object: responseReference,
            role: "response",
          },
        ],
        ordinal,
        outcome: "complete",
        producer: pending.producer,
        startedAt,
      }),
      context,
    );
    await completions.publish(attempt.object, { ordinal });
  }
  return Object.freeze({
    failure: null,
    status: "complete",
    workerExited: false,
  });
}

async function stopWorker(worker) {
  if (worker === null) return;
  try {
    await worker.closeClean();
  } catch {
    await worker.terminate("SIGTERM").catch(() => {});
    fail("worker_cleanup_failed");
  }
}

async function closeCapsule(capsule) {
  if (capsule !== null) await capsule.dispose();
}

function safeErrorCode(error) {
  if (error instanceof OperatorSignalError) return "operator_interrupted";
  if (error instanceof WorkerProtocolError) return "worker_protocol_error";
  if (error instanceof WorkerTerminatedError) return "worker_terminated";
  return "coordinator_error";
}

function runReceipt({
  artifactKey: receiptArtifactKey,
  campaign,
  completed,
  failure,
  implementation,
  inventory,
  outcome,
  throttle,
  verdict,
}) {
  return {
    artifact_key: receiptArtifactKey,
    campaign,
    completed,
    failure,
    implementation,
    inventory,
    outcome,
    schema: RESULT_SCHEMA,
    throttle,
    verdict,
  };
}

async function publishRunReceipt(resultReceipt, receipt) {
  await publishExactReceipt(resultReceipt, canonicalBytes(receipt));
}

async function completedCount(completions) {
  if (completions === null) return 0;
  try {
    return (await completions.planResume()).complete.length;
  } catch {
    return 0;
  }
}

class AcquisitionSession {
  constructor() {
    this.capsule = null;
    this.completions = null;
    this.store = null;
    this.worker = null;
  }

  async closeCaptureResources() {
    const failures = [];
    const worker = this.worker;
    this.worker = null;
    await stopWorker(worker).catch((error) => failures.push(error));
    const capsule = this.capsule;
    this.capsule = null;
    await closeCapsule(capsule).catch((error) => failures.push(error));
    if (failures.length > 0) {
      throw new AggregateError(failures, "XML-RPC acquisition capture cleanup failed");
    }
  }

  async closeCompletions() {
    const completions = this.completions;
    this.completions = null;
    await completions?.close();
  }

  async closeStore() {
    const store = this.store;
    this.store = null;
    await store?.close();
  }

  async close() {
    const failures = [];
    await this.closeCaptureResources().catch((error) => failures.push(error));
    await this.closeCompletions().catch((error) => failures.push(error));
    await this.closeStore().catch((error) => failures.push(error));
    if (failures.length > 0) {
      throw new AggregateError(failures, "XML-RPC acquisition session cleanup failed");
    }
  }
}

export async function runAcquisition(input) {
  let options = null;
  let sealed = null;
  let campaign = null;
  let artifacts = null;
  let context = null;
  let inventory = null;
  let lastCompleted = 0;
  const resources = new AcquisitionSession();
  try {
    options = partitionRunnerOptions(normalizeRunnerOptions(input));
    const coordinator = await assertMaterializedCoordinatorLaunch(options.launch);
    assertPinnedPilotWorkerIdentity(options.source);
    await assertDistinctOutputDestinations(options.outputs);
    const [fullInventoryBytes, manifestBytes, summaryBytes] = await Promise.all(
      [
        fs.readFile(options.inventory.fullInventoryPath),
        fs.readFile(options.inventory.manifestPath),
        fs.readFile(options.inventory.summaryPath),
      ],
    );
    let fullInventory;
    try {
      fullInventory = JSON.parse(
        decodeUtf8(fullInventoryBytes, "full_inventory_invalid"),
      );
    } catch {
      fail("full_inventory_invalid");
    }
    const pilot = derivePilotInventory({
      expectedFullInventorySha256: options.inventory.expectedFullInventorySha256,
      expectedManifestSha256: options.inventory.expectedManifestSha256,
      expectedSummarySha256: options.inventory.expectedSummarySha256,
      fullInventory,
      manifestBytes,
      selectionCount: options.inventory.selectionCount,
      shardCount: options.inventory.shards,
      summaryBytes,
    });
    await publishExactReceipt(options.outputs.inventoryOutput, pilot.inventoryBytes);
    inventory = pilot.inventory;
    context = createReferenceAcquisitionContext(inventory, {
      expectedIdentitySha256: inventory.identity.sha256,
    });
    resources.store = await initializeReferenceObjectStore(options.storage.storeRoot);
    const store = resources.store;
    const runtime = await prepareWikidotXmlrpcRuntime(options.runtime);
    const source = await prepareWikidotXmlrpcWorkerSource(options.source);
    if (
      source.workerBlobOid !== WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.blob ||
      source.workerFileSha256 !==
        WIKIDOT_XMLRPC_PILOT_WORKER_IDENTITY.file_sha256
    ) {
      fail("pilot_worker_materialization_invalid");
    }
    artifacts = await publishAuthorityArtifacts(
      store,
      runtime,
      source,
      coordinator,
    );
    artifacts = Object.freeze({ ...artifacts, store });
    campaign = await putWikidotXmlrpcCampaign(
      store,
      buildWikidotXmlrpcCampaign({
        campaignNonce: options.campaign.campaignNonce,
        implementation: artifacts.implementation.object,
        inventorySha256: inventory.identity.sha256,
        principalId: options.campaign.principalId,
      }),
    );
    resources.completions = await initializeWikidotXmlrpcCompletions(
      store,
      context,
      campaign.reference,
    );
    let plan = await resources.completions.planResume();
    if (plan.pending.length !== 0) {
      resources.capsule = await materializeWikidotXmlrpcPrivateCapsule({
        capsuleParent: options.storage.capsuleParent,
        runtime,
        source,
      });
    }
    sealed = await sealThrottleReceipt({
      artifacts,
      campaign,
      inventory,
      throttleReceipt: options.outputs.throttleReceipt,
      selection: pilot.selection,
    });
    let outcome = "pass";
    let failure = null;
    let workerExited = false;
    const credentials = takeCredentialsAfterSeal();
    if (plan.pending.length !== 0) {
      if (credentials === null) {
        outcome = "credentials_unavailable";
      } else {
        resources.worker = new WikidotXmlrpcWorkerClient(resources.capsule.spawn());
        await resources.worker.start(
          options.campaign.principalId,
          artifacts.environment.descriptor,
          credentials,
        );
        const capture = await capturePending({
          completions: resources.completions,
          context,
          store,
          worker: resources.worker,
        });
        outcome = capture.status;
        failure = capture.failure;
        workerExited = capture.workerExited;
      }
    }
    if (workerExited) resources.worker = null;
    await resources.closeCaptureResources();
    plan = await resources.completions.planResume();
    const completed = plan.complete.length;
    lastCompleted = completed;
    if (outcome === "complete" && plan.pending.length === 0) outcome = "pass";
    if (outcome === "pass" && plan.pending.length !== 0) {
      outcome = "coordinator_error";
      failure = { code: "incomplete_after_capture", retryable: false };
    }
    let verdict = null;
    await resources.closeCompletions();
    if (outcome === "pass") {
      const publication = await publishWikidotXmlrpcAcquisitionVerdict(
        options.outputs.verdict,
        { campaignReference: campaign.reference, context, store },
      );
      verdict = Object.freeze({
        bytes: publication.bytes.byteLength,
        sha256: sha256Hex(publication.bytes),
      });
    }
    const receipt = runReceipt({
      artifactKey: sealed.artifactKey,
      campaign: campaign.reference,
      completed,
      failure,
      implementation: artifacts.implementation.object,
      inventory: Object.freeze({
        row_count: inventory.rows.length,
        sha256: inventory.identity.sha256,
      }),
      outcome,
      throttle: sealed.config,
      verdict,
    });
    await publishRunReceipt(options.outputs.resultReceipt, receipt);
    await resources.closeStore();
    return Object.freeze({
      artifactKey: sealed.artifactKey,
      completed,
      exitCode: outcome === "pass" ? 0 : 2,
      outcome,
    });
  } catch (error) {
    let failureCode = safeErrorCode(error);
    await resources.closeCaptureResources().catch(() => {
      failureCode = "coordinator_error";
    });
    if (
      sealed !== null &&
      campaign !== null &&
      artifacts !== null &&
      inventory !== null
    ) {
      const receipt = runReceipt({
        artifactKey: sealed.artifactKey,
        campaign: campaign.reference,
        completed: resources.completions === null
          ? lastCompleted
          : await completedCount(resources.completions),
        failure: { code: failureCode, retryable: false },
        implementation: artifacts.implementation.object,
        inventory: Object.freeze({
          row_count: inventory.rows.length,
          sha256: inventory.identity.sha256,
        }),
        outcome: "coordinator_error",
        throttle: sealed.config,
        verdict: null,
      });
      await resources.closeCompletions();
      await publishRunReceipt(options.outputs.resultReceipt, receipt);
      await resources.closeStore();
      return Object.freeze({
        artifactKey: sealed.artifactKey,
        completed: receipt.completed,
        exitCode: 2,
        outcome: "coordinator_error",
      });
    }
    throw error;
  } finally {
    scrubWikidotCredentials();
    await resources.close();
  }
}

export function usage() {
  return "Usage: run-wikidot-xmlrpc-acquisition.mjs --manifest PATH --summary PATH --full-inventory PATH --expected-manifest-sha256 SHA256 --expected-summary-sha256 SHA256 --expected-full-inventory-sha256 SHA256 --selection-count 128 --shards COUNT --store PATH --inventory-output PATH --throttle-receipt PATH --result-receipt PATH --verdict PATH --capsule-parent PATH --runtime-root PATH --runtime-python RELATIVE --runtime-venv-config RELATIVE --runtime-version VERSION --source-git-dir PATH --source-commit SHA1 --source-tree SHA1 --wikijump-git-dir PATH --wikijump-commit SHA1 --wikijump-tree SHA1 --campaign-nonce UUID --principal-id ID";
}

export async function main(argv) {
  const parsed = parseArguments(argv);
  if (parsed.help === true) {
    process.stdout.write(`${usage()}\n`);
    return 0;
  }
  const result = await runAcquisition(parsed);
  process.stdout.write(
    `${JSON.stringify({
      artifact_key: result.artifactKey,
      completed: result.completed,
      outcome: result.outcome,
    })}\n`,
  );
  return result.exitCode;
}
