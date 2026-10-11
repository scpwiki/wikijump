#!/usr/bin/env node
// Offline, read-only diagnostic. This is NOT a browser-parity admission gate.
import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import path from "node:path";

const RECORDS_SCHEMA = "wikijump_full_parity.browser_rendering_record.v1";
const CACHE_SCHEMA = "wikijump_full_parity.browser_response_cache_store.v1";

function httpUrl(value) {
  if (typeof value !== "string")
    throw new Error("a failed request or barrier URL is malformed");
  let result;
  try {
    result = new URL(value);
  } catch {
    throw new Error("a failed request or barrier URL is malformed");
  }
  if (
    !["http:", "https:"].includes(result.protocol) ||
    result.username ||
    result.password
  ) {
    throw new Error(
      "a failed request or barrier URL is not an anonymous HTTP(S) URL",
    );
  }
  return result;
}

export function inspectBrowserCaptureBarriers(
  records,
  cacheManifest,
  lock = null,
  gateConfig = null,
) {
  if (
    records?.schema !== "wikijump_full_parity.browser_rendering_evidence.v1"
  ) {
    throw new Error("unsupported browser rendering evidence schema");
  }
  if (
    !Array.isArray(records.evidence) ||
    records.evidence.length === 0 ||
    records.evidence.some(
      (row) =>
        row?.schema !== RECORDS_SCHEMA ||
        !Array.isArray(row.source_failed_requests),
    )
  ) {
    throw new Error("capture evidence rows are malformed");
  }
  if (
    cacheManifest?.schema !== CACHE_SCHEMA ||
    !Array.isArray(cacheManifest.entries) ||
    (cacheManifest.acquisition_barriers !== undefined &&
      !Array.isArray(cacheManifest.acquisition_barriers))
  ) {
    throw new Error(
      "persistent cache manifest lacks a valid acquisition-barrier inventory",
    );
  }
  if (
    typeof cacheManifest.identity !== "string" ||
    records.capture?.source_response_cache?.persistent_identity !==
      cacheManifest.identity
  ) {
    throw new Error("capture/cache identities differ or are absent");
  }
  const barriers = new Set(
    (cacheManifest.acquisition_barriers ?? []).map(
      (value) => httpUrl(value).href,
    ),
  );
  const sourceFailed = records.evidence.flatMap(
    (row) => row.source_failed_requests,
  );
  const matches = sourceFailed.filter((failed) =>
    barriers.has(httpUrl(failed.url).href),
  );
  const matchingHosts = [
    ...new Set(matches.map((failed) => httpUrl(failed.url).hostname)),
  ].sort();
  const captureBarriers =
    records.capture?.source_response_cache?.acquisition_barriers;
  if (!Number.isSafeInteger(captureBarriers) || captureBarriers < 0) {
    throw new Error("capture's barrier count is missing or malformed");
  }
  const gateFailed = records.capture?.request_gate?.enforcement_failed;
  if (typeof gateFailed !== "boolean")
    throw new Error("capture request gate result is missing");
  if ((lock === null) !== (gateConfig === null)) {
    throw new Error("lock and request-gate config must be inspected together");
  }
  if (lock !== null) {
    const owner = gateConfig?.lock?.owner;
    if (
      lock.schema !== "wikijump_full_parity.browser_capture_lock.v1" ||
      gateConfig?.schema !==
        "wikijump_full_parity.browser_request_gate_config.v1" ||
      !["pending", "sealed"].includes(lock.state_confirmation) ||
      !owner ||
      owner.run_id !== gateConfig.run_id ||
      !["hostname", "pid", "process_start_ticks", "run_id"].every(
        (field) => lock[field] === owner[field],
      )
    ) {
      throw new Error(
        "capture lock does not match the recorded run identity or state",
      );
    }
  }
  return {
    schema: "wikijump.test_quality.browser_capture_barrier_inspection.v1",
    diagnostic_only: true,
    compatible_oracle_proven: false,
    cache_identity_match: true,
    source_failed_requests: sourceFailed.length,
    source_failed_requests_matching_current_barriers: matches.length,
    matched_hosts: matchingHosts,
    current_cache_acquisition_barriers: barriers.size,
    captured_acquisition_barrier_count: captureBarriers,
    barrier_counts_disagree: barriers.size !== captureBarriers,
    gate_enforcement_failed: gateFailed,
    capture_error_rows: records.evidence.filter(
      (row) => row.capture_errors?.length > 0,
    ).length,
    ...(lock === null
      ? {}
      : { lock_state_confirmation: lock.state_confirmation }),
    attribution_limit:
      "Current cache state is not a time-frozen proof of the cause of earlier failures; matches are diagnostic correlation only. This does not authorize reacquisition or lock release.",
  };
}

async function main(argv) {
  if (argv.length !== 4 && argv.length !== 8) {
    throw new Error(
      "Usage: node inspect-browser-capture-barriers.mjs --records RECORDS --cache-manifest MANIFEST [--lock LOCK --request-gate-config CONFIG]",
    );
  }
  const options = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    if (
      ![
        "--records",
        "--cache-manifest",
        "--lock",
        "--request-gate-config",
      ].includes(argv[index]) ||
      options.has(argv[index])
    ) {
      throw new Error("invalid or repeated CLI argument");
    }
    options.set(argv[index], argv[index + 1]);
  }
  if (
    !options.has("--records") ||
    !options.has("--cache-manifest") ||
    options.has("--lock") !== options.has("--request-gate-config") ||
    (argv.length === 8 && !options.has("--lock"))
  )
    throw new Error("missing required CLI argument");
  const read = async (flag) => {
    let contents;
    try {
      contents = await readFile(path.resolve(options.get(flag)), "utf8");
    } catch {
      throw new Error(`${flag} file could not be read`);
    }
    try {
      return JSON.parse(contents);
    } catch {
      throw new Error(`${flag} file is not valid JSON`);
    }
  };
  const report = inspectBrowserCaptureBarriers(
    await read("--records"),
    await read("--cache-manifest"),
    options.has("--lock") ? await read("--lock") : null,
    options.has("--request-gate-config")
      ? await read("--request-gate-config")
      : null,
  );
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
  main(process.argv.slice(2)).catch((error) => {
    // Do not print raw file content, request URLs or other captured secrets.
    console.error(
      `offline browser capture inspection failed: ${error.message}`,
    );
    process.exitCode = 1;
  });
}
