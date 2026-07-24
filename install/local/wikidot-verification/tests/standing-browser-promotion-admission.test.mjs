import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  STANDING_BROWSER_CANARIES,
  defaultCanaryPairs,
} from "../src/standing-browser-canaries.mjs";
import { main as runAdmissionCli, parseArgs as parseAdmissionArgs, usage as admissionCliUsage } from "../scripts/verify-standing-candidate-parity-admission.mjs";
import { buildCandidateParityReceipt } from "../src/standing-browser-parity-receipt.mjs";
import { buildLiveReferenceLedger } from "../src/standing-browser-parity-reference.mjs";
import { verifyStandingCandidateParityAdmission } from "../src/standing-browser-promotion-admission.mjs";
import { STANDING_BROWSER_EXECUTION_MODULES } from "../src/standing-browser-execution-identity.mjs";
import { observationArtifactName } from "../src/standing-browser-parity-observation.mjs";
import {
  renderedHomeManifestSha256,
  verifyStandingPromotionPrecondition,
} from "../../../standing/scripts/verify-promotion-precondition.mjs";
import {
  canonicalJson,
  sha256File,
  sha256Value,
} from "../src/standing-browser-parity-util.mjs";

test("standing admission CLI binds all evidence paths before sealing", async () => {
  const argv = [
    "--receipt", "receipt.json",
    "--candidate-identity", "identity.json",
    "--live-reference", "reference.json",
    "--live-completion-policy", "policy.json",
    "--output", "admission.json",
  ];
  const parsed = parseAdmissionArgs(argv);
  assert.equal(parsed.receipt, path.resolve("receipt.json"));
  assert.match(admissionCliUsage(), /does not publish port 443/u);
  const calls = [];
  const output = [];
  const code = await runAdmissionCli(argv, {
    verifyAdmission: async (options) => {
      calls.push(["verify", options]);
      return {status: "pass"};
    },
    seal: async (outputPath, admission) => {
      calls.push(["seal", outputPath, admission]);
      return {path: outputPath, sha256: "a".repeat(64)};
    },
    stdout: (line) => output.push(JSON.parse(line)),
  });
  assert.equal(code, 0);
  assert.equal(calls.length, 2);
  assert.equal(output[0].status, "pass");
});

const hash = (value) => createHash("sha256").update(value).digest("hex");
const git = (character) => character.repeat(40);
const image = (character) => `sha256:${character.repeat(64)}`;
const viewport = { width: 1366, height: 900 };
const PROMOTION_ROLES = Object.freeze([
  "cache",
  "caddy",
  "database",
  "deepwell",
  "files",
  "framerail",
  "wws",
]);

function policy() {
  return {
    schema: "wikijump.standing_browser_live_completion_policy.v1",
    status: "sealed",
    policy_version: "2026-07-20.1",
    allowed_external_failures: [],
  };
}

function runtimeServiceConfigurations(images) {
  return Object.keys(images)
    .sort()
    .map((role) => ({
      role,
      effective_configuration_sha256: hash(`fixture-runtime:${role}`),
    }));
}

function candidateIdentity({
  images = { caddy: image("e") },
  build = {
    seal_sha256: "b".repeat(64),
    verdict_sha256: "c".repeat(64),
    final_images_sha256: "d".repeat(64),
  },
  promotionBaseManifestSha256 = "0".repeat(64),
} = {}) {
  const runtimeConfigurations = runtimeServiceConfigurations(images);
  return {
    schema: "wikijump.standing_candidate_parity_identity.v1",
    status: "sealed",
    artifact_key: "a".repeat(64),
    build,
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
      images,
      config: {
        isolated_overlay_sha256: "f".repeat(64),
        promotion_base_manifest_sha256: promotionBaseManifestSha256,
        effective_runtime_services_sha256: sha256Value(runtimeConfigurations),
      },
      endpoint: {
        scheme: "https",
        host: "scp-wiki.wikijump.localhost",
        port: 18443,
        resolved_addresses: ["127.0.0.1"],
        allowed_origin_set: [
          "https://scp-wiki.wikijump.localhost:18443",
          "https://scp-wiki.wjfiles.localhost:18443",
        ],
        local_connect_address: "127.0.0.1",
      },
    },
    evidence: {
      status: "sealed",
      manifest_sha256: "1".repeat(64),
      seal_sha256: "2".repeat(64),
    },
  };
}

function propertiesFor(canary) {
  return Object.fromEntries(
    Object.entries(canary.first_paint_custom_properties).map(
      ([name, expectation]) => [
        name,
        expectation.operator === "contains"
          ? `url(${expectation.value})`
          : expectation.value,
      ],
    ),
  );
}

function probesFor(canary) {
  return canary.presence_probes.map((probe) => ({
    id: probe.id,
    selector: probe.selector,
    pseudo: probe.pseudo ?? null,
    count: probe.minimum_count ?? 1,
    rendered_count: probe.require_rendered ? (probe.minimum_count ?? 1) : 0,
    style: {},
    ...(probe.pseudo_layout
      ? {
          pseudo_layout: {
            status: "captured",
            node_present: true,
            layout_present: true,
            painted_bounds: { x: 0, y: 0, width: 100, height: 20 },
            visible_bounds: { x: 0, y: 0, width: 100, height: 20 },
            visible_area_ratio: 1,
            descendant_text: probe.pseudo_layout.require_descendant_text
              ? "generated"
              : "",
            computed_style: {
              content: probe.pseudo_layout.require_generated_content
                ? '"generated"'
                : "none",
              "background-image": probe.pseudo_layout.require_background_image
                ? "url(https://cdn.example/logo.png)"
                : "none",
            },
          },
        }
      : {}),
  }));
}

async function writeArtifacts(root, prefix) {
  const result = {};
  for (const [key, fullPage] of [
    ["first", false],
    ["viewport", false],
    ["full", true],
  ]) {
    const file = `${prefix}-${key}.png`;
    const bytes = `${prefix}-${key}`;
    await fs.writeFile(path.join(root, file), bytes, { mode: 0o600 });
    result[key] = { path: file, sha256: hash(bytes), full_page: fullPage };
  }
  return result;
}

async function writeCandidateArtifacts(root, index, pair) {
  const result = {};
  for (const [key, phase] of [
    ["local_domcontentloaded_immediate_png", "domcontentloaded-immediate"],
    ["local_settled_viewport_png", "settled-viewport"],
    ["local_settled_full_page_png", "settled-full-page"],
  ]) {
    const file = observationArtifactName({
      label: "local",
      index,
      url: pair.local_url,
      phase,
    });
    const bytes = `${file}-fixture`;
    await fs.writeFile(path.join(root, file), bytes, { mode: 0o600 });
    result[key] = hash(bytes);
  }
  return result;
}

function liveCapture(pair, canary, artifacts) {
  const probes = probesFor(canary);
  return {
    schema: "wikijump_local_lab.standing_browser_parity_capture.v2",
    captured_at: "2026-07-20T00:00:00.000Z",
    input_url: pair.live_url,
    final_url: pair.live_url,
    navigation_status: 200,
    failures: [],
    broken_images: [],
    first_paint: {
      document: {
        phase: "domcontentloaded_immediate_observation",
        custom_properties: propertiesFor(canary),
        presence_probes: probes,
        geometry: {},
      },
      screenshot: artifacts.first,
    },
    document: {
      phase: "settled",
      custom_properties: propertiesFor(canary),
      presence_probes: probes,
      geometry: {},
      resource_completion: {
        status: "complete",
        load_ready_state: "complete",
        font_status: "loaded",
        image_count: 0,
        incomplete_image_count: 0,
      },
    },
    settled_viewport_screenshot: artifacts.viewport,
    screenshot: artifacts.full,
  };
}

function passingComparison(canary) {
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

function runtimeIdentity(identity, identitySha256) {
  const candidate = identity.candidate;
  const configurations = runtimeServiceConfigurations(candidate.images);
  return {
    schema: "wikijump.standing_candidate_runtime_observation.v1",
    status: "bound",
    observed_at: "2026-07-20T00:00:00.000Z",
    candidate_identity_sha256: identitySha256,
    candidate: {
      compose_project: candidate.compose_project,
      wikijump_commit: candidate.wikijump_commit,
      wikijump_tree: candidate.wikijump_tree,
      ftml_sha: candidate.ftml_sha,
      artifact_key: identity.artifact_key,
      profile: candidate.profile,
      config_sha256: candidate.config.isolated_overlay_sha256,
      effective_runtime_services_sha256:
        candidate.config.effective_runtime_services_sha256,
    },
    services: configurations.map((configuration, index) => ({
      role: configuration.role,
      container_id: `${index}`.repeat(64),
      image_id: candidate.images[configuration.role],
      state: { running: true, status: "running", health: "healthy" },
      labels: {
        "com.docker.compose.project": candidate.compose_project,
        "com.rokurolize.wikijump.owner": candidate.owner,
        "com.rokurolize.wikijump.sha": candidate.wikijump_commit,
        "com.rokurolize.wikijump.tree": candidate.wikijump_tree,
        "com.rokurolize.wikijump.ftml_sha": candidate.ftml_sha,
        "com.rokurolize.wikijump.artifact_key": identity.artifact_key,
        "com.rokurolize.wikijump.config_sha256":
          candidate.config.isolated_overlay_sha256,
        "com.rokurolize.wikijump.runtime_config_sha256":
          candidate.config.effective_runtime_services_sha256,
        "com.rokurolize.wikijump.profile": candidate.profile,
        "com.rokurolize.wikijump.expires_at": candidate.expires_at,
        "com.rokurolize.wikijump.role": configuration.role,
      },
      ...(configuration.role === "caddy"
        ? {
            https_binding: {
              container_port: "443/tcp",
              host_address: "127.0.0.1",
              host_port: 18443,
            },
          }
        : {}),
      effective_configuration_sha256:
        configuration.effective_configuration_sha256,
    })),
  };
}

function executionIdentity(identity) {
  const modules = [...STANDING_BROWSER_EXECUTION_MODULES]
    .sort()
    .map((filePath) => ({ path: filePath, sha256: "f".repeat(64) }));
  return {
    schema: "wikijump.standing_browser_execution_identity.v1",
    source_clean: true,
    wikijump_commit: identity.candidate.wikijump_commit,
    wikijump_tree: identity.candidate.wikijump_tree,
    ftml_sha: identity.candidate.ftml_sha,
    modules,
    module_manifest_sha256: sha256Value(modules),
  };
}

async function fixture(root, identity = candidateIdentity()) {
  const policyPath = path.join(root, "policy.json");
  const identityPath = path.join(root, "candidate-identity.json");
  await fs.writeFile(policyPath, canonicalJson(policy()), { mode: 0o600 });
  await fs.writeFile(identityPath, canonicalJson(identity), { mode: 0o600 });
  const policySha256 = await sha256File(policyPath);
  const identitySha256 = await sha256File(identityPath);
  const pairs = defaultCanaryPairs({
    localOrigin: "https://scp-wiki.wikijump.localhost:18443",
    liveOrigin: "https://scp-wiki.wikidot.com",
  });
  const records = [];
  for (const [index, pair] of pairs.entries()) {
    const canary = STANDING_BROWSER_CANARIES[index];
    const artifacts = await writeArtifacts(root, `canary-${index}`);
    records.push({
      input: pair,
      live: liveCapture(pair, canary, artifacts),
      artifacts,
      candidate_artifacts: await writeCandidateArtifacts(root, index, pair),
    });
  }
  const liveReference = buildLiveReferenceLedger({
    records: records.map(({ input, live }) => ({ input, live })),
    viewport,
    thresholds: {
      geometry_position_px: 8,
      geometry_size_px: 12,
      image_count_delta: 0,
      dom_multiset_distance_ratio: 0.15,
    },
    policy: policy(),
    policySha256,
    browserEnvironment: {
      engine: "chromium",
      version: "fixture",
      executable_sha256: "3".repeat(64),
    },
    requestGate: {
      schema: "wikijump_full_parity.browser_request_gate.v1",
      interval_ms: 4_000,
      enforcement_failed: false,
      public_requests: pairs.length,
      local_exempt_requests: 0,
      unsupported_requests_blocked: 0,
      websocket_connections_blocked: 0,
      retry_after_honored: 0,
      retry_after_invalid: 0,
      config_sha256: "4".repeat(64),
    },
    generatedAt: "2026-07-20T00:00:00.000Z",
  });
  const referencePath = path.join(root, "live-reference.json");
  await fs.writeFile(referencePath, canonicalJson(liveReference), {
    mode: 0o600,
  });
  const referenceSha256 = await sha256File(referencePath);
  const runnerPath = path.resolve(
    import.meta.dirname,
    "../src/standing-browser-parity-runner.mjs",
  );
  const observationPath = path.resolve(
    import.meta.dirname,
    "../src/standing-browser-parity-observation.mjs",
  );
  const candidateRecords = records.map(
    ({ input, artifacts, candidate_artifacts }, index) => ({
      input: { local_url: input.local_url, live_url: input.live_url },
      comparison: passingComparison(STANDING_BROWSER_CANARIES[index]),
      artifact_hashes: {
        ...candidate_artifacts,
        live_domcontentloaded_immediate_png: artifacts.first.sha256,
        live_settled_viewport_png: artifacts.viewport.sha256,
        live_settled_full_page_png: artifacts.full.sha256,
      },
    }),
  );
  const parity = {
    schema: "wikijump_local_lab.standing_browser_parity_run.v2",
    status: "pass",
    generated_at: "2026-07-20T00:00:00.000Z",
    capture_phase: "domcontentloaded_immediate_observation",
    viewport,
    candidate_identity_sha256: identitySha256,
    live_reference_sha256: referenceSha256,
    local_capture_config_sha256: "5".repeat(64),
    request_gate: {
      schema: "wikijump_full_parity.browser_request_gate.v1",
      interval_ms: 4_000,
      next_admissible_at_epoch_ms: 0,
      retry_after_until_epoch_ms: 0,
      enforcement_failed: false,
      grants: [],
      public_requests: 0,
      local_exempt_requests: pairs.length,
      unsupported_requests_blocked: 0,
      websocket_connections_blocked: 0,
      retry_after_honored: 0,
      retry_after_invalid: 0,
      config_sha256: "5".repeat(64),
    },
    records: candidateRecords,
    summary: {
      pairs_total: pairs.length,
      pairs_failed: 0,
      pairs_passed: pairs.length,
    },
  };
  const ledgerPath = path.join(root, "standing-browser-parity.json");
  await fs.writeFile(ledgerPath, canonicalJson(parity), { mode: 0o600 });
  const ledgerSha256 = await sha256File(ledgerPath);
  const receipt = buildCandidateParityReceipt({
    identity,
    identitySha256,
    parity,
    parityLedgerSha256: ledgerSha256,
    liveReference: {
      sha256: referenceSha256,
      generated_at: "2026-07-20T00:00:00.000Z",
      policy_version: policy().policy_version,
      policy_sha256: policySha256,
      canary_contract_sha256:
        liveReference.capture_contract.canary_contract_sha256,
    },
    browserEnvironment: {
      engine: "chromium",
      version: "fixture",
      executable_sha256: "3".repeat(64),
    },
    runtimeIdentity: runtimeIdentity(identity, identitySha256),
    executionIdentity: executionIdentity(identity),
    runnerSha256: await sha256File(runnerPath),
    observationSha256: await sha256File(observationPath),
    generatedAt: "2026-07-20T00:00:00.000Z",
  }).receipt;
  const receiptPath = path.join(root, "candidate-receipt.json");
  await fs.writeFile(receiptPath, canonicalJson(receipt), { mode: 0o600 });
  return { receiptPath, identityPath, referencePath, policyPath };
}

async function createPromotionBuildFixture(root) {
  const stagingHomePath = path.join(root, "staging-home");
  const buildEvidencePath = path.join(root, "build-evidence");
  const outputDirectory = path.join(root, "promotion-output");
  await fs.mkdir(path.join(stagingHomePath, "nested"), { recursive: true });
  await fs.mkdir(path.join(buildEvidencePath, "images"), { recursive: true });
  await fs.mkdir(outputDirectory);
  await fs.writeFile(
    path.join(stagingHomePath, ".env"),
    "STANDING_PROJECT_NAME=wikijump-standing\n",
  );
  await fs.writeFile(
    path.join(stagingHomePath, "nested", "compose.yaml"),
    "services: {}\n",
  );
  const stagingManifestSha256 =
    await renderedHomeManifestSha256(stagingHomePath);
  const images = Object.fromEntries(
    PROMOTION_ROLES.map((role, index) => [role, image(`${index}`)]),
  );
  const finalImages = PROMOTION_ROLES.map((role) => ({
    role,
    image_id: images[role],
    os: "linux",
    architecture: "amd64",
  }));
  const imagesPath = path.join(
    buildEvidencePath,
    "images",
    "final-images.json",
  );
  const verdictPath = path.join(buildEvidencePath, "verdict.json");
  await fs.writeFile(imagesPath, canonicalJson(finalImages), { mode: 0o600 });
  await fs.writeFile(
    verdictPath,
    canonicalJson({
      schema: "wikijump.standing_provenance_build.v1",
      status: "pass",
      promotion_eligible: true,
      run_id: "adapter-fixture-build",
      wikijump_commit: git("a"),
      wikijump_tree: git("b"),
      ftml_sha: git("c"),
      final_images: "images/final-images.json",
    }),
    { mode: 0o600 },
  );
  const manifestPath = path.join(buildEvidencePath, "evidence-manifest.sha256");
  const manifestPaths = ["images/final-images.json", "verdict.json"].sort(
    (left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right)),
  );
  const manifest = await Promise.all(
    manifestPaths.map(
      async (relative) =>
        `${await sha256File(path.join(buildEvidencePath, relative))}  ./${relative}\n`,
    ),
  );
  await fs.writeFile(manifestPath, manifest.join(""), { mode: 0o600 });
  const sealPath = path.join(buildEvidencePath, "seal.json");
  await fs.writeFile(
    sealPath,
    canonicalJson({
      schema: "wikijump.standing_provenance_build_seal.v1",
      status: "sealed",
      run_id: "adapter-fixture-build",
      evidence_manifest_verified: true,
      evidence_manifest_exclusions: ["evidence-manifest.sha256", "seal.json"],
      evidence_manifest_sha256: await sha256File(manifestPath),
      verdict_sha256: await sha256File(verdictPath),
    }),
    { mode: 0o600 },
  );
  return {
    stagingHomePath,
    buildEvidencePath,
    outputPath: path.join(outputDirectory, "promotion-precondition.json"),
    images,
    stagingManifestSha256,
    build: {
      seal_sha256: await sha256File(sealPath),
      verdict_sha256: await sha256File(verdictPath),
      final_images_sha256: await sha256File(imagesPath),
    },
  };
}

test("source-owned receipt verifier verifies a complete candidate receipt and its exact source runner", async (context) => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-admission-"),
  );
  context.after(() => fs.rm(root, { recursive: true, force: true }));
  const paths = await fixture(root);
  const admission = await verifyStandingCandidateParityAdmission({
    receiptPath: paths.receiptPath,
    candidateIdentityPath: paths.identityPath,
    liveReferencePath: paths.referencePath,
    liveCompletionPolicyPath: paths.policyPath,
    now: new Date("2026-07-20T00:00:00.000Z"),
    collectExecutionIdentity: async (identity) => executionIdentity(identity),
  });
  assert.equal(admission.status, "pass");
  assert.equal(admission.parity.pairs_total, STANDING_BROWSER_CANARIES.length);
});

test("source-owned receipt verifier rejects an identity file that differs from the receipt", async (context) => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-admission-"),
  );
  context.after(() => fs.rm(root, { recursive: true, force: true }));
  const paths = await fixture(root);
  const altered = candidateIdentity();
  altered.candidate.owner = "replacement-owner";
  await fs.writeFile(paths.identityPath, canonicalJson(altered), {
    mode: 0o600,
  });
  await assert.rejects(
    verifyStandingCandidateParityAdmission({
      receiptPath: paths.receiptPath,
      candidateIdentityPath: paths.identityPath,
      liveReferencePath: paths.referencePath,
      liveCompletionPolicyPath: paths.policyPath,
      now: new Date("2026-07-20T00:00:00.000Z"),
      collectExecutionIdentity: async (identity) => executionIdentity(identity),
    }),
    /does not bind the supplied sealed candidate identity/u,
  );
});

test("source-owned receipt verifier rejects a candidate receipt whose sealed local screenshot bytes changed", async (context) => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-admission-"),
  );
  context.after(() => fs.rm(root, { recursive: true, force: true }));
  const paths = await fixture(root);
  const pair = defaultCanaryPairs({
    localOrigin: "https://scp-wiki.wikijump.localhost:18443",
    liveOrigin: "https://scp-wiki.wikidot.com",
  })[0];
  const screenshot = observationArtifactName({
    label: "local",
    index: 0,
    url: pair.local_url,
    phase: "domcontentloaded-immediate",
  });
  await fs.writeFile(path.join(root, screenshot), "tampered", { mode: 0o600 });
  await assert.rejects(
    verifyStandingCandidateParityAdmission({
      receiptPath: paths.receiptPath,
      candidateIdentityPath: paths.identityPath,
      liveReferencePath: paths.referencePath,
      liveCompletionPolicyPath: paths.policyPath,
      now: new Date("2026-07-20T00:00:00.000Z"),
      collectExecutionIdentity: async (identity) => executionIdentity(identity),
    }),
    /artifact digest does not bind/u,
  );
});

test("source-owned receipt verifier rejects a receipt claiming a different runner", async (context) => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-browser-admission-"),
  );
  context.after(() => fs.rm(root, { recursive: true, force: true }));
  const paths = await fixture(root);
  const receipt = JSON.parse(await fs.readFile(paths.receiptPath, "utf8"));
  receipt.parity.parity_script_sha256 = "0".repeat(64);
  await fs.writeFile(paths.receiptPath, canonicalJson(receipt), {
    mode: 0o600,
  });
  await assert.rejects(
    verifyStandingCandidateParityAdmission({
      receiptPath: paths.receiptPath,
      candidateIdentityPath: paths.identityPath,
      liveReferencePath: paths.referencePath,
      liveCompletionPolicyPath: paths.policyPath,
      now: new Date("2026-07-20T00:00:00.000Z"),
      collectExecutionIdentity: async (identity) => executionIdentity(identity),
    }),
    /not produced by this source-owned runner/u,
  );
});

test("promotion precondition accepts a complete source-admission fixture", async (context) => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-promotion-precondition-"),
  );
  context.after(() => fs.rm(root, { recursive: true, force: true }));
  const promotion = await createPromotionBuildFixture(root);
  const identity = candidateIdentity({
    images: promotion.images,
    build: promotion.build,
    promotionBaseManifestSha256: promotion.stagingManifestSha256,
  });
  const paths = await fixture(root, identity);
  const result = await verifyStandingPromotionPrecondition({
    receiptPath: paths.receiptPath,
    candidateIdentityPath: paths.identityPath,
    liveReferencePath: paths.referencePath,
    liveCompletionPolicyPath: paths.policyPath,
    buildEvidencePath: promotion.buildEvidencePath,
    stagingHomePath: promotion.stagingHomePath,
    outputPath: promotion.outputPath,
    now: new Date("2026-07-20T00:00:00.000Z"),
    verifyAdmission: (argumentsValue) =>
      verifyStandingCandidateParityAdmission({
        ...argumentsValue,
        collectExecutionIdentity: async (candidate) =>
          executionIdentity(candidate),
      }),
  });
  assert.equal(result.status, "pass");
  assert.equal(result.candidate.artifact_key, identity.artifact_key);
  assert.equal(result.build.seal_sha256, promotion.build.seal_sha256);
  assert.equal(
    result.staging_home.manifest_sha256,
    promotion.stagingManifestSha256,
  );
});
