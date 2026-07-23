import { constants as fsConstants } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";

import { sha256Hex, stableStringify } from "./canonical-json.mjs";
import {
  LocalPageReadClient,
  LocalPageReadError,
  sameTimestamp,
} from "./local-page-read.mjs";
import { validateRuntimeIdentity } from "./local-browser-console-smoke.mjs";
import { validateLocalDeepwellRpcUrl } from "./theme-localization-deepwell-adapter.mjs";
import {
  openPrivateComparisonOutputDirectory,
  publishXmlrpcPilotLocalComparisonOutputs,
  XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES,
} from "./xmlrpc-pilot-local-comparison-output.mjs";
import {
  openVerifiedXmlrpcPilotBundle,
  XMLRPC_EN_128_DESIGNATED_SOURCE,
} from "./xmlrpc-pilot-local-comparison-bundle.mjs";

export { LocalPageReadClient, LocalPageReadError } from "./local-page-read.mjs";
export {
  openVerifiedXmlrpcPilotBundle,
  XMLRPC_EN_128_DESIGNATED_SOURCE,
  XMLRPC_PILOT_MANIFEST_RECORD_SCHEMA,
} from "./xmlrpc-pilot-local-comparison-bundle.mjs";

export const XMLRPC_PILOT_LOCAL_COMPARISON_VERDICT_SCHEMA =
  "wikijump_full_parity.xmlrpc_pilot_local_comparison_verdict.v1";
export const XMLRPC_PILOT_LOCAL_COMPARISON_RECORD_SCHEMA =
  "wikijump_full_parity.xmlrpc_pilot_local_comparison_record.v1";
export const XMLRPC_PILOT_LOCAL_COMPARISON_CLUSTER_SCHEMA =
  "wikijump_full_parity.xmlrpc_pilot_local_comparison_cluster.v1";

const MAX_RUNTIME_IDENTITY_BYTES = 2 * 1024 * 1024;
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

function same(left, right) {
  return stableStringify(left) === stableStringify(right);
}

function jsonl(records) {
  return Buffer.from(
    `${records.map((record) => stableStringify(record)).join("\n")}\n`,
    "utf8",
  );
}

function json(value) {
  return Buffer.from(`${stableStringify(value)}\n`, "utf8");
}

function bytesIdentity(bytes) {
  return Object.freeze({ bytes: bytes.byteLength, sha256: sha256Hex(bytes) });
}

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

async function readRuntimeIdentity(filePath) {
  const handle = await fs.open(
    filePath,
    fsConstants.O_RDONLY |
      (fsConstants.O_NONBLOCK ?? 0) |
      (fsConstants.O_NOFOLLOW ?? 0),
  );
  try {
    return await readOpenRegularFile(
      handle,
      "local runtime identity",
      MAX_RUNTIME_IDENTITY_BYTES,
    );
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

function validateRuntimeIdentityForComparison(value) {
  const identity = validateRuntimeIdentity(value);
  const artifactKey = nonEmptyIdentifier(
    identity.artifact_key,
    "runtime identity artifact_key",
  );
  const runtimeConfigSha256 = sha256(
    identity.runtime_config_sha256,
    "runtime identity runtime_config_sha256",
  );
  return Object.freeze({
    artifact_key: artifactKey,
    deepwell_binary_or_image_sha256: identity.deepwell_binary_or_image_sha256,
    features: [...identity.features],
    ftml_sha: identity.ftml_sha,
    framerail_assets_sha256: identity.framerail_assets_sha256,
    profile: identity.profile,
    render_run_id: identity.render_run_id,
    runtime_config_sha256: runtimeConfigSha256,
    rustc_vv: identity.rustc_vv,
    schema: identity.schema,
    wikijump_sha: identity.wikijump_sha,
  });
}

function exactOrdinalSet(values, count, label) {
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

function localSummary(page) {
  return Object.freeze({
    compiled_body_html_sha256: sha256Hex(page.compiled_body_html),
    compiled_body_styles_sha256: sha256Hex(
      stableStringify(page.compiled_body_styles),
    ),
    page_revision_count: page.page_revision_count,
    page_updated_at: page.page_updated_at,
    wikitext_sha256: sha256Hex(page.wikitext),
  });
}

function errorCode(error) {
  return error instanceof LocalPageReadError ? error.code : "unexpected";
}

function comparisonRecord(row, page, error) {
  const common = {
    fixture_id: row.manifest.fixture_id,
    fullname: row.manifest.fullname,
    ordinal: row.manifest.ordinal,
    reference: row.manifest.reference,
    schema: XMLRPC_PILOT_LOCAL_COMPARISON_RECORD_SCHEMA,
  };
  if (row.reference.kind === "wikidot_deleted") {
    return Object.freeze({
      ...common,
      status: "reference_deleted",
    });
  }
  if (error !== null) {
    return Object.freeze({
      ...common,
      error_kind: errorCode(error),
      status: "local_error",
    });
  }
  if (page === null) {
    return Object.freeze({ ...common, status: "local_missing" });
  }
  const local = localSummary(page);
  const differences = [];
  if (local.wikitext_sha256 !== row.manifest.reference.content_sha256)
    differences.push("source_content");
  if (local.compiled_body_html_sha256 !== row.manifest.reference.html_sha256)
    differences.push("compiled_html");
  if (local.page_revision_count !== row.manifest.reference.revisions)
    differences.push("revision_count");
  if (!sameTimestamp(local.page_updated_at, row.manifest.reference.updated_at))
    differences.push("updated_at");
  return Object.freeze({
    ...common,
    differences: Object.freeze(differences),
    local,
    status: differences.length === 0 ? "matched" : "mismatched",
  });
}

function clusterRecords(records) {
  const ordinals = new Map();
  const add = (category, ordinal) => {
    const values = ordinals.get(category) ?? [];
    values.push(ordinal);
    ordinals.set(category, values);
  };
  for (const record of records) {
    if (record.status === "mismatched") {
      for (const difference of record.differences)
        add(difference, record.ordinal);
    } else if (
      record.status === "local_missing" ||
      record.status === "local_error"
    ) {
      add(record.status, record.ordinal);
    }
  }
  return Object.freeze(
    [...ordinals.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([category, values]) =>
        Object.freeze({
          category,
          count: values.length,
          ordinals: Object.freeze(
            [...values].sort((left, right) => left - right),
          ),
          schema: XMLRPC_PILOT_LOCAL_COMPARISON_CLUSTER_SCHEMA,
        }),
      ),
  );
}

function terminalSummary(records, expectedRows) {
  const expected = exactOrdinalSet(
    expectedRows.map((row) => row.manifest.ordinal),
    expectedRows.length,
    "comparison expected set",
  );
  const observed = exactOrdinalSet(
    records.map((record) => record.ordinal),
    expected.length,
    "comparison terminal set",
  );
  const statuses = {};
  for (const record of records)
    statuses[record.status] = (statuses[record.status] ?? 0) + 1;
  const accepted = records.filter(
    (record) =>
      record.status === "matched" || record.status === "reference_deleted",
  ).length;
  const errorCount = statuses.local_error ?? 0;
  return Object.freeze({
    accepted_count: accepted,
    error_count: errorCount,
    expected_count: expected.length,
    expected_ordinal_set_sha256: sha256Hex(stableStringify(expected)),
    observed_count: observed.length,
    observed_ordinal_set_sha256: sha256Hex(stableStringify(observed)),
    terminal_set_equal: same(expected, observed),
    status_counts: Object.freeze(statuses),
  });
}

async function compareBundle(bundle, client) {
  const liveRows = bundle.rows.filter((row) => row.reference.kind === "live");
  let siteId;
  let siteError = null;
  if (liveRows.length > 0) {
    try {
      siteId = await client.siteId();
    } catch (error) {
      siteError = error;
    }
  }
  const records = [];
  for (const row of bundle.rows) {
    if (row.reference.kind === "wikidot_deleted") {
      // A deletion tombstone is a source-state observation at capture time.
      // It is neither empty source nor authority to inspect a later local page.
      records.push(comparisonRecord(row, null, null));
      continue;
    }
    if (siteError !== null) {
      records.push(comparisonRecord(row, null, siteError));
      continue;
    }
    try {
      records.push(
        comparisonRecord(
          row,
          await client.pageGet(siteId, row.manifest.fullname),
          null,
        ),
      );
    } catch (error) {
      records.push(comparisonRecord(row, null, error));
    }
  }
  return Object.freeze({
    records: Object.freeze(records),
    site_id: siteId ?? null,
  });
}

export async function runXmlrpcPilotLocalComparison({
  outputDir,
  pilotRoot,
  rpcUrl,
  runtimeIdentityPath,
  sourceExpectation = XMLRPC_EN_128_DESIGNATED_SOURCE,
  timeoutMs = 30_000,
  fetchImpl = globalThis.fetch,
} = {}) {
  const [bundle, runtimeIdentityBytes] = await Promise.all([
    openVerifiedXmlrpcPilotBundle({ pilotRoot, sourceExpectation }),
    readRuntimeIdentity(runtimeIdentityPath),
  ]);
  const runtimeIdentity = validateRuntimeIdentityForComparison(
    parseJson(runtimeIdentityBytes, "local runtime identity"),
  );
  const client = new LocalPageReadClient({ rpcUrl, timeoutMs, fetchImpl });
  const directory = await openPrivateComparisonOutputDirectory({
    outputDir,
    pilotRoot,
  });
  try {
    const compared = await compareBundle(bundle, client);
    const clusters = clusterRecords(compared.records);
    const terminal = terminalSummary(compared.records, bundle.rows);
    const comparisonBytes = jsonl(compared.records);
    const clustersBytes = json(clusters);
    const runtimeIdentityHash = sha256Hex(runtimeIdentityBytes);
    const artifactKey = `xmlrpc-pilot-local-comparison-v1-${sha256Hex(
      stableStringify({
        manifest_sha256: bundle.manifest_identity.sha256,
        runtime_identity_sha256: runtimeIdentityHash,
        source_acquisition_artifact_key: bundle.source.acquisition_artifact_key,
      }),
    )}`;
    const gateStatus =
      terminal.error_count > 0
        ? "error"
        : terminal.accepted_count === terminal.expected_count &&
            terminal.terminal_set_equal
          ? "pass"
          : "fail";
    const verdict = Object.freeze({
      artifact_key: artifactKey,
      candidate: Object.freeze({
        runtime_identity: runtimeIdentity,
        runtime_identity_sha256: runtimeIdentityHash,
      }),
      gate: Object.freeze({
        status: gateStatus,
        ...terminal,
      }),
      outputs: Object.freeze({
        mismatch_clusters: bytesIdentity(clustersBytes),
        rows: bytesIdentity(comparisonBytes),
        verified_pilot_manifest: bundle.manifest_identity,
      }),
      rpc: Object.freeze({
        endpoint: validateLocalDeepwellRpcUrl(rpcUrl),
        local_site_id: compared.site_id,
        methods: Object.freeze(["site_get", "page_get"]),
        timeout_ms: timeoutMs,
      }),
      schema: XMLRPC_PILOT_LOCAL_COMPARISON_VERDICT_SCHEMA,
      source: bundle.source,
    });
    const verdictBytes = json(verdict);
    await publishXmlrpcPilotLocalComparisonOutputs(directory, {
      clusters: clustersBytes,
      manifest: bundle.manifest_bytes,
      rows: comparisonBytes,
      verdict: verdictBytes,
    });
    return Object.freeze({
      exit_code:
        verdict.gate.status === "pass"
          ? 0
          : verdict.gate.status === "fail"
            ? 1
            : 2,
      output: Object.freeze({
        clusters: path.join(
          outputDir,
          XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.clusters,
        ),
        manifest: path.join(
          outputDir,
          XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.manifest,
        ),
        rows: path.join(
          outputDir,
          XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.rows,
        ),
        verdict: path.join(
          outputDir,
          XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES.verdict,
        ),
      }),
      verdict,
    });
  } finally {
    await directory.close();
  }
}
