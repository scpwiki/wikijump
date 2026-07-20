import assert from "node:assert/strict";
import test from "node:test";

import { STANDING_BROWSER_CANARIES } from "../src/standing-browser-canaries.mjs";
import {
  buildCandidateParityReceipt,
  validateCandidateParityIdentity,
  validateCandidateParityReceipt,
} from "../src/standing-browser-parity-receipt.mjs";
import { sha256Value } from "../src/standing-browser-parity-util.mjs";
import { STANDING_BROWSER_EXECUTION_MODULES } from "../src/standing-browser-execution-identity.mjs";

const hash = (character) => character.repeat(64);
const git = (character) => character.repeat(40);

function identity() {
  const endpoint = {
    scheme: "https",
    host: "scp-wiki.wikijump.localhost",
    port: 18443,
    resolved_addresses: ["127.0.0.1"],
    allowed_origin_set: [
      "https://scp-wiki.wikijump.localhost:18443",
      "https://scp-wiki.wjfiles.localhost:18443",
    ],
    local_connect_address: "127.0.0.1",
  };
  return {
    schema: "wikijump.standing_candidate_parity_identity.v1",
    status: "sealed",
    artifact_key: hash("a"),
    build: {
      seal_sha256: hash("b"),
      verdict_sha256: hash("c"),
      final_images_sha256: hash("d"),
    },
    candidate: {
      owner: "standing-parity-fixture",
      expires_at: "2099-07-20T00:00:00.000Z",
      compose_project: "wikijump-candidate-fixture",
      port_443_published: false,
      wikijump_commit: git("a"),
      wikijump_tree: git("b"),
      ftml_sha: git("c"),
      profile: "production-build",
      source_clean: true,
      images: { caddy: `sha256:${hash("e")}` },
      config: {
        isolated_overlay_sha256: hash("f"),
        promotion_base_manifest_sha256: hash("0"),
        effective_runtime_services_sha256: sha256Value([
          { role: "caddy", effective_configuration_sha256: hash("e") },
        ]),
      },
      endpoint,
    },
    evidence: {
      status: "sealed",
      manifest_sha256: hash("1"),
      seal_sha256: hash("2"),
    },
  };
}

function artifactHashes() {
  return {
    local_domcontentloaded_immediate_png: hash("3"),
    local_settled_viewport_png: hash("4"),
    local_settled_full_page_png: hash("5"),
    live_domcontentloaded_immediate_png: hash("6"),
    live_settled_viewport_png: hash("7"),
    live_settled_full_page_png: hash("8"),
  };
}

function requestGate() {
  return {
    schema: "wikijump_full_parity.browser_request_gate.v1",
    interval_ms: 4_000,
    next_admissible_at_epoch_ms: 0,
    retry_after_until_epoch_ms: 0,
    enforcement_failed: false,
    grants: [],
    public_requests: 0,
    local_exempt_requests: 6,
    unsupported_requests_blocked: 0,
    websocket_connections_blocked: 0,
    retry_after_honored: 0,
    retry_after_invalid: 0,
    config_sha256: hash("a"),
  };
}

function runtimeIdentity(candidateIdentity) {
  const candidate = candidateIdentity.candidate;
  return {
    schema: "wikijump.standing_candidate_runtime_observation.v1",
    status: "bound",
    observed_at: "2026-07-20T00:00:00.000Z",
    candidate_identity_sha256: hash("9"),
    candidate: {
      compose_project: candidate.compose_project,
      wikijump_commit: candidate.wikijump_commit,
      wikijump_tree: candidate.wikijump_tree,
      ftml_sha: candidate.ftml_sha,
      artifact_key: candidateIdentity.artifact_key,
      profile: candidate.profile,
      config_sha256: candidate.config.isolated_overlay_sha256,
      effective_runtime_services_sha256:
        candidate.config.effective_runtime_services_sha256,
    },
    services: [
      {
        role: "caddy",
        container_id: hash("e"),
        image_id: candidate.images.caddy,
        state: { running: true, status: "running", health: "healthy" },
        labels: {
          "com.docker.compose.project": candidate.compose_project,
          "com.rokurolize.wikijump.owner": candidate.owner,
          "com.rokurolize.wikijump.sha": candidate.wikijump_commit,
          "com.rokurolize.wikijump.tree": candidate.wikijump_tree,
          "com.rokurolize.wikijump.ftml_sha": candidate.ftml_sha,
          "com.rokurolize.wikijump.artifact_key":
            candidateIdentity.artifact_key,
          "com.rokurolize.wikijump.config_sha256":
            candidate.config.isolated_overlay_sha256,
          "com.rokurolize.wikijump.runtime_config_sha256":
            candidate.config.effective_runtime_services_sha256,
          "com.rokurolize.wikijump.profile": candidate.profile,
          "com.rokurolize.wikijump.expires_at": candidate.expires_at,
          "com.rokurolize.wikijump.role": "caddy",
        },
        https_binding: {
          container_port: "443/tcp",
          host_address: candidate.endpoint.local_connect_address,
          host_port: candidate.endpoint.port,
        },
        effective_configuration_sha256: hash("e"),
      },
    ],
  };
}

function executionIdentity(candidateIdentity) {
  const modules = [...STANDING_BROWSER_EXECUTION_MODULES]
    .sort()
    .map((filePath) => ({ path: filePath, sha256: hash("f") }));
  return {
    schema: "wikijump.standing_browser_execution_identity.v1",
    source_clean: true,
    wikijump_commit: candidateIdentity.candidate.wikijump_commit,
    wikijump_tree: candidateIdentity.candidate.wikijump_tree,
    ftml_sha: candidateIdentity.candidate.ftml_sha,
    modules,
    module_manifest_sha256: sha256Value(modules),
  };
}

function comparison(canary) {
  const geometry = canary.geometry_selectors.map((selector) => ({
    selector,
    status: "pass",
  }));
  const properties = Object.keys(canary.first_paint_custom_properties).map(
    (property) => ({
      property,
      status: "pass",
    }),
  );
  const probes = canary.presence_probes.map((probe) => ({
    id: probe.id,
    status: "pass",
  }));
  return {
    status: "pass",
    anomalies: [],
    geometry,
    domcontentloaded_immediate_geometry: geometry,
    domcontentloaded_immediate_custom_properties: properties,
    domcontentloaded_immediate_probes: probes,
    settled_probes: probes,
  };
}

function parityRecords() {
  return STANDING_BROWSER_CANARIES.map((canary) => ({
    input: {
      local_url: `https://scp-wiki.wikijump.localhost:18443/${canary.slug}`,
      live_url: `https://scp-wiki.wikidot.com/${canary.slug}`,
    },
    comparison: comparison(canary),
    artifact_hashes: artifactHashes(),
  }));
}

function passingReceipt() {
  const candidateIdentity = identity();
  return buildCandidateParityReceipt({
    identity: candidateIdentity,
    identitySha256: hash("9"),
    parity: {
      schema: "wikijump_local_lab.standing_browser_parity_run.v2",
      summary: {
        pairs_total: STANDING_BROWSER_CANARIES.length,
        pairs_failed: 0,
      },
      viewport: { width: 1366, height: 900 },
      local_capture_config_sha256: hash("a"),
      request_gate: requestGate(),
      records: parityRecords(),
    },
    parityLedgerSha256: hash("b"),
    liveReference: {
      sha256: hash("c"),
      generated_at: "2026-07-20T00:00:00.000Z",
      policy_version: "2026-07-20.1",
      policy_sha256: hash("d"),
      canary_contract_sha256: hash("e"),
    },
    browserEnvironment: {
      engine: "chromium",
      version: "fixture",
      executable_sha256: hash("f"),
    },
    runtimeIdentity: runtimeIdentity(candidateIdentity),
    executionIdentity: executionIdentity(candidateIdentity),
    runnerSha256: hash("0"),
    observationSha256: hash("1"),
    generatedAt: "2026-07-20T00:00:00.000Z",
  }).receipt;
}

test("candidate identity rejects mutable image tags and a standing project", () => {
  const mutable = identity();
  mutable.candidate.images.caddy = "wikijump:candidate";
  assert.throws(
    () => validateCandidateParityIdentity(mutable),
    /immutable sha256 image id/u,
  );
  const standing = identity();
  standing.candidate.compose_project = "wikijump-standing";
  assert.throws(
    () => validateCandidateParityIdentity(standing),
    /must not be wikijump-standing/u,
  );
});

test("candidate parity receipt binds all six canaries, immediate observation, and screenshot hashes", () => {
  const receipt = passingReceipt();
  assert.equal(validateCandidateParityReceipt(receipt).status, "pass");

  const missingScreenshot = structuredClone(receipt);
  delete missingScreenshot.parity.records[0].artifact_hashes
    .local_settled_full_page_png;
  assert.throws(
    () => validateCandidateParityReceipt(missingScreenshot),
    /every required screenshot artifact/u,
  );

  const postSettleOnly = structuredClone(receipt);
  postSettleOnly.parity.capture_phase = "settled";
  assert.throws(
    () => validateCandidateParityReceipt(postSettleOnly),
    /required DOMContentLoaded observation/u,
  );

  const omittedSubtitleProbe = structuredClone(receipt);
  omittedSubtitleProbe.parity.records[0].comparison.domcontentloaded_immediate_probes =
    omittedSubtitleProbe.parity.records[0].comparison.domcontentloaded_immediate_probes.filter(
      (probe) => probe.id !== "header_subtitle",
    );
  assert.throws(
    () => validateCandidateParityReceipt(omittedSubtitleProbe),
    /lacks a passing DOMContentLoaded probe: header_subtitle/u,
  );
});

test("a nominal pass cannot conceal an anomaly or a noncanonical canary URL", () => {
  const anomalous = structuredClone(passingReceipt());
  anomalous.parity.records[0].comparison.anomalies.push({ code: "hidden" });
  assert.throws(
    () => validateCandidateParityReceipt(anomalous),
    /cannot contain anomalies/u,
  );

  const query = structuredClone(passingReceipt());
  query.parity.records[0].input.live_url += "?drift=1";
  assert.throws(
    () => validateCandidateParityReceipt(query),
    /query strings or fragments/u,
  );
});

test("a candidate receipt rejects a pre-closure or substituted request-gate snapshot", () => {
  const preClosure = structuredClone(passingReceipt());
  preClosure.parity.request_gate.enforcement_failed = true;
  assert.throws(
    () => validateCandidateParityReceipt(preClosure),
    /request gate enforcement was not clean at closure/u,
  );

  const substituted = structuredClone(passingReceipt());
  substituted.parity.request_gate_final_sha256 = hash("f");
  assert.throws(
    () => validateCandidateParityReceipt(substituted),
    /request gate final snapshot hash is invalid/u,
  );
});

test("a candidate receipt refuses a claimed runtime identity that does not bind the running container", () => {
  const claimed = structuredClone(passingReceipt());
  claimed.parity.runtime_identity.services[0].image_id = `sha256:${hash("0")}`;
  assert.throws(
    () => validateCandidateParityReceipt(claimed),
    /image does not bind/u,
  );
});
