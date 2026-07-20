import net from "node:net";

import {
  STANDING_BROWSER_CANARIES,
  canaryForUrl,
} from "./standing-browser-canaries.mjs";
import {
  STANDING_BROWSER_PARITY_SCHEMA,
  STANDING_CANDIDATE_PARITY_IDENTITY_SCHEMA,
  STANDING_CANDIDATE_PARITY_RECEIPT_SCHEMA,
} from "./standing-browser-parity-contract.mjs";
import {
  isPlainObject,
  normalizedUrl,
  requireNonEmptyString,
  requirePlainObject,
  requireSha256,
  sha256Value,
  sortedUniqueStrings,
} from "./standing-browser-parity-util.mjs";
import { validateCandidateRuntimeObservation } from "./standing-browser-runtime-identity.mjs";
import { validateCandidateExecutionIdentity } from "./standing-browser-execution-identity.mjs";

function requireGitObject(value, name) {
  if (typeof value !== "string" || !/^[0-9a-f]{40}$/u.test(value)) {
    throw new Error(`${name} must be a full lowercase Git object id`);
  }
  return value;
}

function requireIsoTimestamp(value, name) {
  const timestamp = requireNonEmptyString(value, name);
  if (!Number.isFinite(Date.parse(timestamp))) {
    throw new Error(`${name} must be an ISO-8601 timestamp`);
  }
  return timestamp;
}

function requireImageMap(value) {
  const images = requirePlainObject(value, "candidate.images");
  const entries = Object.entries(images).sort(([left], [right]) =>
    left.localeCompare(right),
  );
  if (entries.length === 0)
    throw new Error("candidate.images must not be empty");
  for (const [role, image] of entries) {
    if (!/^[a-z][a-z0-9_-]*$/u.test(role)) {
      throw new Error(`candidate.images contains an invalid role: ${role}`);
    }
    if (typeof image !== "string" || !/^sha256:[0-9a-f]{64}$/u.test(image)) {
      throw new Error(
        `candidate.images.${role} must be an immutable sha256 image id`,
      );
    }
  }
  return Object.freeze(Object.fromEntries(entries));
}

function loopbackAddress(value, name) {
  const address = requireNonEmptyString(value, name);
  const family = net.isIP(address);
  if (family === 4 && address.startsWith("127.")) return address;
  if (family === 6 && address === "::1") return address;
  throw new Error(`${name} must be a loopback IP address`);
}

function candidateEndpoint(value) {
  const endpoint = requirePlainObject(value, "candidate.endpoint");
  if (endpoint.scheme !== "https") {
    throw new Error("candidate.endpoint.scheme must be https");
  }
  const host = requireNonEmptyString(
    endpoint.host,
    "candidate.endpoint.host",
  ).toLowerCase();
  const match =
    /^([a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)\.wikijump\.localhost$/u.exec(host);
  if (!match) {
    throw new Error(
      "candidate.endpoint.host must be an exact *.wikijump.localhost host",
    );
  }
  if (
    !Number.isInteger(endpoint.port) ||
    endpoint.port <= 0 ||
    endpoint.port > 65_535
  ) {
    throw new Error("candidate.endpoint.port must be a valid integer port");
  }
  if (endpoint.port === 443) {
    throw new Error(
      "candidate.endpoint.port must not be the canonical standing port 443",
    );
  }
  const expectedPageOrigin = `https://${host}:${endpoint.port}`;
  const expectedFilesOrigin = `https://${match[1]}.wjfiles.localhost:${endpoint.port}`;
  const allowedOriginSet = sortedUniqueStrings(
    endpoint.allowed_origin_set,
    "candidate.endpoint.allowed_origin_set",
  );
  const expectedOriginSet = [expectedFilesOrigin, expectedPageOrigin].sort();
  if (JSON.stringify(allowedOriginSet) !== JSON.stringify(expectedOriginSet)) {
    throw new Error(
      "candidate.endpoint.allowed_origin_set must exactly name page and file origins",
    );
  }
  const addresses = sortedUniqueStrings(
    endpoint.resolved_addresses,
    "candidate.endpoint.resolved_addresses",
  ).map((address, index) =>
    loopbackAddress(address, `candidate.endpoint.resolved_addresses[${index}]`),
  );
  const localConnectAddress = loopbackAddress(
    endpoint.local_connect_address,
    "candidate.endpoint.local_connect_address",
  );
  if (!addresses.includes(localConnectAddress)) {
    throw new Error(
      "candidate.endpoint.local_connect_address must be a declared resolved address",
    );
  }
  return Object.freeze({
    scheme: "https",
    host,
    port: endpoint.port,
    resolved_addresses: Object.freeze(addresses),
    allowed_origin_set: Object.freeze(allowedOriginSet),
    local_connect_address: localConnectAddress,
  });
}

export function candidatePageOrigin(identity) {
  const endpoint = identity.candidate.endpoint;
  return `${endpoint.scheme}://${endpoint.host}:${endpoint.port}`;
}

export function validateCandidateParityIdentity(value) {
  const identity = requirePlainObject(value, "candidate parity identity");
  if (identity.schema !== STANDING_CANDIDATE_PARITY_IDENTITY_SCHEMA) {
    throw new Error(
      `candidate parity identity must use ${STANDING_CANDIDATE_PARITY_IDENTITY_SCHEMA}`,
    );
  }
  if (identity.status !== "sealed") {
    throw new Error("candidate parity identity.status must be sealed");
  }
  const build = requirePlainObject(
    identity.build,
    "candidate parity identity.build",
  );
  const candidate = requirePlainObject(
    identity.candidate,
    "candidate parity identity.candidate",
  );
  const config = requirePlainObject(candidate.config, "candidate.config");
  const evidence = requirePlainObject(
    identity.evidence,
    "candidate parity identity.evidence",
  );
  const normalized = {
    schema: STANDING_CANDIDATE_PARITY_IDENTITY_SCHEMA,
    status: "sealed",
    artifact_key: requireSha256(
      identity.artifact_key,
      "candidate parity identity.artifact_key",
    ),
    build: {
      seal_sha256: requireSha256(build.seal_sha256, "build.seal_sha256"),
      verdict_sha256: requireSha256(
        build.verdict_sha256,
        "build.verdict_sha256",
      ),
      final_images_sha256: requireSha256(
        build.final_images_sha256,
        "build.final_images_sha256",
      ),
    },
    candidate: {
      owner: requireNonEmptyString(candidate.owner, "candidate.owner"),
      expires_at: requireIsoTimestamp(
        candidate.expires_at,
        "candidate.expires_at",
      ),
      compose_project: requireNonEmptyString(
        candidate.compose_project,
        "candidate.compose_project",
      ),
      port_443_published: candidate.port_443_published,
      wikijump_commit: requireGitObject(
        candidate.wikijump_commit,
        "candidate.wikijump_commit",
      ),
      wikijump_tree: requireGitObject(
        candidate.wikijump_tree,
        "candidate.wikijump_tree",
      ),
      ftml_sha: requireGitObject(candidate.ftml_sha, "candidate.ftml_sha"),
      profile: candidate.profile,
      source_clean: candidate.source_clean,
      images: requireImageMap(candidate.images),
      config: {
        isolated_overlay_sha256: requireSha256(
          config.isolated_overlay_sha256,
          "candidate.config.isolated_overlay_sha256",
        ),
        promotion_base_manifest_sha256: requireSha256(
          config.promotion_base_manifest_sha256,
          "candidate.config.promotion_base_manifest_sha256",
        ),
        effective_runtime_services_sha256: requireSha256(
          config.effective_runtime_services_sha256,
          "candidate.config.effective_runtime_services_sha256",
        ),
      },
      endpoint: candidateEndpoint(candidate.endpoint),
    },
    evidence: {
      status: evidence.status,
      manifest_sha256: requireSha256(
        evidence.manifest_sha256,
        "evidence.manifest_sha256",
      ),
      seal_sha256: requireSha256(evidence.seal_sha256, "evidence.seal_sha256"),
    },
  };
  if (normalized.candidate.compose_project === "wikijump-standing") {
    throw new Error("candidate.compose_project must not be wikijump-standing");
  }
  if (normalized.candidate.port_443_published !== false) {
    throw new Error("candidate.port_443_published must be false");
  }
  if (normalized.candidate.profile !== "production-build") {
    throw new Error("candidate.profile must be production-build");
  }
  if (normalized.candidate.source_clean !== true) {
    throw new Error("candidate.source_clean must be true");
  }
  if (normalized.evidence.status !== "sealed") {
    throw new Error("evidence.status must be sealed");
  }
  return Object.freeze(normalized);
}

export function assertCandidateIdentityFresh(
  identity,
  { now = new Date() } = {},
) {
  const nowMs = now instanceof Date ? now.getTime() : new Date(now).getTime();
  if (!Number.isFinite(nowMs))
    throw new Error("receipt freshness requires a valid current time");
  if (Date.parse(identity.candidate.expires_at) <= nowMs) {
    throw new Error("candidate parity identity is expired");
  }
  return identity;
}

export function assertPairBoundToCandidate(pair, identity) {
  const local = normalizedUrl(pair.local_url, "pair.local_url");
  const live = normalizedUrl(pair.live_url, "pair.live_url");
  const candidateOrigin = candidatePageOrigin(identity);
  if (local.origin !== candidateOrigin) {
    throw new Error(`candidate pair local origin must be ${candidateOrigin}`);
  }
  if (local.port === "443" || local.port === "") {
    throw new Error("candidate pair must use an explicit non-443 port");
  }
  if (live.origin !== "https://scp-wiki.wikidot.com") {
    throw new Error(
      "candidate pair live URL must use the canonical SCP Wiki authority",
    );
  }
  if (local.search || local.hash || live.search || live.hash) {
    throw new Error(
      "candidate pair URLs must not contain query strings or fragments",
    );
  }
  const canary = canaryForUrl(live.href);
  if (!canary) {
    throw new Error(
      `candidate pair is not a declared standing canary: ${live.href}`,
    );
  }
  const expectedLocal = new URL(`/${encodeURI(canary.slug)}`, candidateOrigin)
    .href;
  const expectedLive = new URL(
    `/${encodeURI(canary.slug)}`,
    "https://scp-wiki.wikidot.com",
  ).href;
  if (local.href !== expectedLocal || live.href !== expectedLive) {
    throw new Error(
      "candidate pair URLs must exactly match the declared canary",
    );
  }
  return Object.freeze({ local_url: local.href, live_url: live.href });
}

function assertCompleteCanarySet(records, identity) {
  const slugs = records.map((record) => {
    const pair = assertPairBoundToCandidate(record.input, identity);
    return canaryForUrl(pair.live_url).slug;
  });
  const expected = STANDING_BROWSER_CANARIES.map(
    (canary) => canary.slug,
  ).sort();
  const actual = [...slugs].sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      "candidate parity receipt must bind every declared standing canary exactly once",
    );
  }
}

function requireParityInput(value) {
  const parity = requirePlainObject(value, "candidate parity receipt.parity");
  if (parity.schema !== STANDING_BROWSER_PARITY_SCHEMA) {
    throw new Error(`parity.schema must be ${STANDING_BROWSER_PARITY_SCHEMA}`);
  }
  const records = parity.records;
  if (!Array.isArray(records) || records.length === 0) {
    throw new Error(
      "candidate parity receipt.parity.records must be a non-empty array",
    );
  }
  const pairsTotal = parity.summary?.pairs_total;
  const pairsFailed = parity.summary?.pairs_failed;
  if (!Number.isInteger(pairsTotal) || pairsTotal !== records.length) {
    throw new Error(
      "candidate parity receipt.parity summary does not bind all records",
    );
  }
  if (
    !Number.isInteger(pairsFailed) ||
    pairsFailed < 0 ||
    pairsFailed > pairsTotal
  ) {
    throw new Error("candidate parity receipt.parity pairs_failed is invalid");
  }
  return {
    ...parity,
    request_gate: validateFinalRequestGate(parity.request_gate),
  };
}

function validateFinalRequestGate(value) {
  const gate = requirePlainObject(value, "candidate parity request gate");
  if (gate.schema !== "wikijump_full_parity.browser_request_gate.v1") {
    throw new Error("candidate parity request gate has an unsupported schema");
  }
  if (!Number.isInteger(gate.interval_ms) || gate.interval_ms < 4_000) {
    throw new Error(
      "candidate parity request gate must preserve the initial 0.25 req/s throttle",
    );
  }
  if (gate.enforcement_failed !== false) {
    throw new Error(
      "candidate parity request gate enforcement was not clean at closure",
    );
  }
  for (const name of [
    "public_requests",
    "local_exempt_requests",
    "unsupported_requests_blocked",
    "websocket_connections_blocked",
    "retry_after_honored",
    "retry_after_invalid",
  ]) {
    if (!Number.isInteger(gate[name]) || gate[name] < 0) {
      throw new Error(`candidate parity request gate ${name} is invalid`);
    }
  }
  return {
    ...gate,
    config_sha256: requireSha256(
      gate.config_sha256,
      "candidate parity request gate config SHA-256",
    ),
  };
}

const REQUIRED_ARTIFACT_HASHES = Object.freeze([
  "local_domcontentloaded_immediate_png",
  "local_settled_viewport_png",
  "local_settled_full_page_png",
  "live_domcontentloaded_immediate_png",
  "live_settled_viewport_png",
  "live_settled_full_page_png",
]);

function requireStatusRows(value, name) {
  if (!Array.isArray(value)) {
    throw new Error(`${name} must be an array`);
  }
  for (const row of value) {
    if (!isPlainObject(row) || !new Set(["pass", "fail"]).has(row.status)) {
      throw new Error(`${name} contains an invalid status row`);
    }
  }
  return value;
}

function validateViewport(value, name = "candidate parity viewport") {
  const viewport = requirePlainObject(value, name);
  for (const dimension of ["width", "height"]) {
    if (!Number.isInteger(viewport[dimension]) || viewport[dimension] <= 0) {
      throw new Error(`${name}.${dimension} must be a positive integer`);
    }
  }
  return Object.freeze({ width: viewport.width, height: viewport.height });
}

function requireCompletePassRows(rows, expectedValues, key, name) {
  const observed = new Map(rows.map((row) => [row[key], row]));
  for (const expected of expectedValues) {
    const row = observed.get(expected);
    if (!row || row.status !== "pass") {
      throw new Error(
        `a passing parity record lacks a passing ${name}: ${expected}`,
      );
    }
  }
}

function validateComparison(value, contract = null) {
  const comparison = requirePlainObject(value, "parity record comparison");
  if (!new Set(["pass", "fail"]).has(comparison.status)) {
    throw new Error("parity record comparison status is invalid");
  }
  if (!Array.isArray(comparison.anomalies)) {
    throw new Error("parity record comparison anomalies must be an array");
  }
  for (const name of [
    "geometry",
    "domcontentloaded_immediate_geometry",
    "domcontentloaded_immediate_custom_properties",
    "domcontentloaded_immediate_probes",
    "settled_probes",
  ]) {
    requireStatusRows(comparison[name], `parity record comparison.${name}`);
  }
  if (comparison.status === "pass") {
    if (comparison.anomalies.length !== 0) {
      throw new Error("a passing parity record cannot contain anomalies");
    }
    for (const name of [
      "geometry",
      "domcontentloaded_immediate_geometry",
      "domcontentloaded_immediate_custom_properties",
      "domcontentloaded_immediate_probes",
      "settled_probes",
    ]) {
      if (comparison[name].some((row) => row.status !== "pass")) {
        throw new Error(
          `a passing parity record contains a failed ${name} row`,
        );
      }
    }
    if (contract) {
      requireCompletePassRows(
        comparison.geometry,
        contract.geometry_selectors,
        "selector",
        "settled geometry selector",
      );
      requireCompletePassRows(
        comparison.domcontentloaded_immediate_geometry,
        contract.geometry_selectors,
        "selector",
        "DOMContentLoaded geometry selector",
      );
      requireCompletePassRows(
        comparison.domcontentloaded_immediate_custom_properties,
        Object.keys(contract.first_paint_custom_properties ?? {}),
        "property",
        "DOMContentLoaded custom property",
      );
      const probeIds = (contract.presence_probes ?? []).map(
        (probe) => probe.id,
      );
      requireCompletePassRows(
        comparison.domcontentloaded_immediate_probes,
        probeIds,
        "id",
        "DOMContentLoaded probe",
      );
      requireCompletePassRows(
        comparison.settled_probes,
        probeIds,
        "id",
        "settled probe",
      );
    }
  }
  return comparison;
}

function validateArtifactHashes(value) {
  const artifactHashes = requirePlainObject(
    value,
    "parity record artifact_hashes",
  );
  const actualNames = Object.keys(artifactHashes).sort();
  if (
    JSON.stringify(actualNames) !==
    JSON.stringify([...REQUIRED_ARTIFACT_HASHES].sort())
  ) {
    throw new Error(
      "candidate parity receipt must bind every required screenshot artifact",
    );
  }
  for (const name of REQUIRED_ARTIFACT_HASHES) {
    requireSha256(
      artifactHashes[name],
      `candidate parity receipt artifact hash ${name}`,
    );
  }
  return artifactHashes;
}

export function buildCandidateParityReceipt({
  identity,
  identitySha256,
  parity,
  parityLedgerSha256,
  liveReference,
  browserEnvironment,
  runtimeIdentity,
  executionIdentity,
  runnerSha256,
  observationSha256,
  generatedAt = new Date().toISOString(),
}) {
  const candidateIdentity = assertCandidateIdentityFresh(
    validateCandidateParityIdentity(identity),
  );
  const checkedIdentitySha256 = requireSha256(
    identitySha256,
    "candidate parity identity SHA-256",
  );
  requireSha256(parityLedgerSha256, "parity ledger SHA-256");
  requireSha256(runnerSha256, "runner SHA-256");
  requireSha256(observationSha256, "observation module SHA-256");
  const parityInput = requireParityInput(parity);
  const viewport = validateViewport(parityInput.viewport);
  if (
    parityInput.local_capture_config_sha256 !==
    parityInput.request_gate.config_sha256
  ) {
    throw new Error(
      "candidate parity receipt request-gate snapshot does not bind its capture configuration",
    );
  }
  const reference = requirePlainObject(
    liveReference,
    "live reference identity",
  );
  const observedRuntimeIdentity = validateCandidateRuntimeObservation(
    runtimeIdentity,
    candidateIdentity,
    { identitySha256: checkedIdentitySha256 },
  );
  const observedExecutionIdentity = validateCandidateExecutionIdentity(
    executionIdentity,
    candidateIdentity,
  );
  const browser = requirePlainObject(
    browserEnvironment,
    "browser environment identity",
  );
  const records = parityInput.records.map((record) => {
    const input = assertPairBoundToCandidate(record.input, candidateIdentity);
    const contract = canaryForUrl(input.live_url);
    return {
      input,
      comparison: validateComparison(record.comparison, contract),
      artifact_hashes: validateArtifactHashes(record.artifact_hashes),
    };
  });
  assertCompleteCanarySet(records, candidateIdentity);
  const pairsFailed = records.filter(
    (record) => record.comparison.status !== "pass",
  ).length;
  if (pairsFailed !== parityInput.summary.pairs_failed) {
    throw new Error(
      "parity receipt comparison results disagree with parity summary",
    );
  }
  const result = {
    schema: STANDING_CANDIDATE_PARITY_RECEIPT_SCHEMA,
    status: pairsFailed === 0 ? "pass" : "fail",
    generated_at: requireIsoTimestamp(
      generatedAt,
      "candidate receipt generated_at",
    ),
    artifact_key: candidateIdentity.artifact_key,
    build: candidateIdentity.build,
    candidate: candidateIdentity.candidate,
    parity: {
      schema: STANDING_BROWSER_PARITY_SCHEMA,
      ledger_sha256: parityLedgerSha256,
      candidate_identity_sha256: checkedIdentitySha256,
      live_reference_sha256: requireSha256(
        reference.sha256,
        "live reference SHA-256",
      ),
      live_reference_generated_at: requireIsoTimestamp(
        reference.generated_at,
        "live reference generated_at",
      ),
      live_reference_policy_version: requireNonEmptyString(
        reference.policy_version,
        "live reference policy_version",
      ),
      live_reference_policy_sha256: requireSha256(
        reference.policy_sha256,
        "live reference policy SHA-256",
      ),
      canary_contract_sha256: requireSha256(
        reference.canary_contract_sha256,
        "live reference canary contract SHA-256",
      ),
      parity_script_sha256: runnerSha256,
      integrity_script_sha256: observationSha256,
      request_gate_config_sha256: requireSha256(
        parityInput.local_capture_config_sha256,
        "local capture configuration SHA-256",
      ),
      request_gate: parityInput.request_gate,
      request_gate_final_sha256: sha256Value(parityInput.request_gate),
      local_connect_address:
        candidateIdentity.candidate.endpoint.local_connect_address,
      endpoint: candidateIdentity.candidate.endpoint,
      browser_environment: {
        engine: requireNonEmptyString(
          browser.engine,
          "browser environment engine",
        ),
        version: requireNonEmptyString(
          browser.version,
          "browser environment version",
        ),
        executable_sha256: requireSha256(
          browser.executable_sha256,
          "browser environment executable SHA-256",
        ),
      },
      capture_phase: "domcontentloaded_immediate_observation",
      viewport,
      pairs_total: records.length,
      pairs_failed: pairsFailed,
      runtime_identity_bound: true,
      runtime_identity: observedRuntimeIdentity,
      runtime_identity_sha256: sha256Value(observedRuntimeIdentity),
      execution_identity: observedExecutionIdentity,
      execution_identity_sha256: sha256Value(observedExecutionIdentity),
      records,
    },
    evidence: candidateIdentity.evidence,
  };
  return Object.freeze({
    receipt: result,
    receipt_content_sha256: sha256Value(result),
  });
}

export function validateCandidateParityReceipt(
  value,
  { now = new Date(), requirePass = true } = {},
) {
  const receipt = requirePlainObject(value, "candidate parity receipt");
  if (receipt.schema !== STANDING_CANDIDATE_PARITY_RECEIPT_SCHEMA) {
    throw new Error(
      `candidate parity receipt must use ${STANDING_CANDIDATE_PARITY_RECEIPT_SCHEMA}`,
    );
  }
  requireIsoTimestamp(
    receipt.generated_at,
    "candidate parity receipt generated_at",
  );
  const identity = validateCandidateParityIdentity({
    schema: STANDING_CANDIDATE_PARITY_IDENTITY_SCHEMA,
    status: "sealed",
    artifact_key: receipt.artifact_key,
    build: receipt.build,
    candidate: receipt.candidate,
    evidence: receipt.evidence,
  });
  assertCandidateIdentityFresh(identity, { now });
  const parity = requirePlainObject(
    receipt.parity,
    "candidate parity receipt.parity",
  );
  if (parity.schema !== STANDING_BROWSER_PARITY_SCHEMA) {
    throw new Error(
      "candidate parity receipt has an unsupported parity schema",
    );
  }
  for (const key of [
    "ledger_sha256",
    "candidate_identity_sha256",
    "live_reference_sha256",
    "live_reference_policy_sha256",
    "canary_contract_sha256",
    "parity_script_sha256",
    "integrity_script_sha256",
    "request_gate_config_sha256",
    "request_gate_final_sha256",
    "runtime_identity_sha256",
    "execution_identity_sha256",
  ]) {
    requireSha256(parity[key], `candidate parity receipt.parity.${key}`);
  }
  requireNonEmptyString(
    parity.live_reference_policy_version,
    "candidate parity receipt policy version",
  );
  requireIsoTimestamp(
    parity.live_reference_generated_at,
    "candidate parity receipt reference time",
  );
  const requestGate = validateFinalRequestGate(parity.request_gate);
  if (requestGate.config_sha256 !== parity.request_gate_config_sha256) {
    throw new Error(
      "candidate parity receipt request gate does not bind its capture configuration",
    );
  }
  if (sha256Value(requestGate) !== parity.request_gate_final_sha256) {
    throw new Error(
      "candidate parity receipt request gate final snapshot hash is invalid",
    );
  }
  if (parity.runtime_identity_bound !== true) {
    throw new Error("candidate parity receipt runtime identity is not bound");
  }
  const runtimeIdentity = validateCandidateRuntimeObservation(
    parity.runtime_identity,
    identity,
    {
      identitySha256: parity.candidate_identity_sha256,
    },
  );
  if (sha256Value(runtimeIdentity) !== parity.runtime_identity_sha256) {
    throw new Error(
      "candidate parity receipt runtime identity hash is invalid",
    );
  }
  const executionIdentity = validateCandidateExecutionIdentity(
    parity.execution_identity,
    identity,
  );
  if (sha256Value(executionIdentity) !== parity.execution_identity_sha256) {
    throw new Error(
      "candidate parity receipt execution identity hash is invalid",
    );
  }
  if (parity.capture_phase !== "domcontentloaded_immediate_observation") {
    throw new Error(
      "candidate parity receipt did not record the required DOMContentLoaded observation",
    );
  }
  validateViewport(parity.viewport, "candidate parity receipt viewport");
  if (
    parity.endpoint?.port === 443 ||
    sha256Value(parity.endpoint) !== sha256Value(identity.candidate.endpoint)
  ) {
    throw new Error(
      "candidate parity receipt endpoint does not bind the declared non-standing candidate",
    );
  }
  if (
    parity.local_connect_address !==
    identity.candidate.endpoint.local_connect_address
  ) {
    throw new Error(
      "candidate parity receipt local connect address does not bind the candidate",
    );
  }
  if (!Array.isArray(parity.records) || parity.records.length === 0) {
    throw new Error("candidate parity receipt records are missing");
  }
  assertCompleteCanarySet(parity.records, identity);
  const pairsFailed = parity.records.filter(
    (record) => record?.comparison?.status !== "pass",
  ).length;
  if (
    pairsFailed !== parity.pairs_failed ||
    parity.pairs_total !== parity.records.length
  ) {
    throw new Error(
      "candidate parity receipt summary does not bind record outcomes",
    );
  }
  for (const record of parity.records) {
    const input = assertPairBoundToCandidate(record.input, identity);
    validateComparison(record.comparison, canaryForUrl(input.live_url));
    validateArtifactHashes(record.artifact_hashes);
  }
  const browser = requirePlainObject(
    parity.browser_environment,
    "candidate parity browser environment",
  );
  requireNonEmptyString(browser.engine, "candidate parity browser engine");
  requireNonEmptyString(browser.version, "candidate parity browser version");
  requireSha256(
    browser.executable_sha256,
    "candidate parity browser executable hash",
  );
  if (!new Set(["pass", "fail"]).has(receipt.status)) {
    throw new Error("candidate parity receipt status is invalid");
  }
  if ((receipt.status === "pass") !== (pairsFailed === 0)) {
    throw new Error(
      "candidate parity receipt status disagrees with comparison outcomes",
    );
  }
  if (requirePass && receipt.status !== "pass") {
    throw new Error("candidate parity receipt is not passing");
  }
  return receipt;
}
