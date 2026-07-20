import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  renderedHomeManifestSha256,
  STANDING_PROMOTION_PRECONDITION_SCHEMA,
  verifyStandingPromotionPrecondition,
} from "../scripts/verify-promotion-precondition.mjs";
import { sha256File } from "../../local/wikidot-verification/src/standing-browser-parity-util.mjs";

const REQUIRED_ROLES = Object.freeze([
  "cache",
  "caddy",
  "database",
  "deepwell",
  "files",
  "framerail",
  "wws",
]);
const ROLE_CHARACTERS = Object.freeze(["0", "1", "2", "3", "4", "5", "6"]);

const hash = (value) => createHash("sha256").update(value).digest("hex");
const git = (character) => character.repeat(40);
const image = (character) => `sha256:${character.repeat(64)}`;

async function writeJson(filePath, value) {
  await fs.writeFile(filePath, `${JSON.stringify(value)}\n`, { mode: 0o600 });
}

function finalImages() {
  return REQUIRED_ROLES.map((role, index) => ({
    role,
    image_id: image(ROLE_CHARACTERS[index]),
    os: "linux",
    architecture: "amd64",
  }));
}

function imageMap(images) {
  return Object.fromEntries(
    images.map(({ role, image_id: imageId }) => [role, imageId]),
  );
}

async function fixtureFiles(root, relative = "") {
  const entries = await fs.readdir(path.join(root, relative), {
    withFileTypes: true,
  });
  const files = [];
  for (const entry of entries) {
    const next = path.join(relative, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await fixtureFiles(root, next)));
    } else if (entry.isFile()) {
      files.push(next);
    } else {
      throw new Error(`fixture contains unsupported path: ${next}`);
    }
  }
  return files;
}

function sortPaths(paths) {
  return [...paths].sort((left, right) =>
    Buffer.compare(Buffer.from(left), Buffer.from(right)),
  );
}

async function writeEvidenceManifest(buildEvidencePath) {
  const files = sortPaths(
    (await fixtureFiles(buildEvidencePath)).filter(
      (relative) =>
        relative !== "evidence-manifest.sha256" && relative !== "seal.json",
    ),
  );
  const lines = await Promise.all(
    files.map(
      async (relative) =>
        `${await sha256File(path.join(buildEvidencePath, relative))}  ./${relative}\n`,
    ),
  );
  const manifestPath = path.join(buildEvidencePath, "evidence-manifest.sha256");
  await fs.writeFile(manifestPath, lines.join(""), { mode: 0o600 });
  return sha256File(manifestPath);
}

async function sealBuildEvidence(fixture) {
  const manifestSha256 = await writeEvidenceManifest(fixture.buildEvidencePath);
  await writeJson(fixture.sealPath, {
    schema: "wikijump.standing_provenance_build_seal.v1",
    status: "sealed",
    run_id: fixture.runId,
    evidence_manifest_verified: true,
    evidence_manifest_exclusions: ["evidence-manifest.sha256", "seal.json"],
    evidence_manifest_sha256: manifestSha256,
    verdict_sha256: await sha256File(fixture.verdictPath),
  });
}

function passingAdmission(identity, identitySha256) {
  return {
    schema: "wikijump.standing_candidate_parity_admission.v1",
    status: "pass",
    candidate_parity_receipt_sha256: "1".repeat(64),
    candidate_identity_sha256: identitySha256,
    live_reference_sha256: "2".repeat(64),
    live_completion_policy_sha256: "3".repeat(64),
    source_runner_sha256: "4".repeat(64),
    source_observation_sha256: "5".repeat(64),
    source_execution_identity_sha256: "6".repeat(64),
    candidate: {
      compose_project: identity.candidate.compose_project,
      endpoint: identity.candidate.endpoint,
      wikijump_commit: identity.candidate.wikijump_commit,
      wikijump_tree: identity.candidate.wikijump_tree,
      ftml_sha: identity.candidate.ftml_sha,
      artifact_key: identity.artifact_key,
      expires_at: identity.candidate.expires_at,
    },
    parity: {
      pairs_total: 6,
      viewport: { width: 1366, height: 900 },
      request_gate_final_sha256: "7".repeat(64),
      runtime_identity_sha256: "8".repeat(64),
      ledger_sha256: "9".repeat(64),
      local_artifacts_verified: 18,
    },
  };
}

async function createFixture(t) {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "standing-precondition-"),
  );
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const stagingHomePath = path.join(root, "staging-home");
  const buildEvidencePath = path.join(root, "build-evidence");
  const outputDirectory = path.join(root, "output");
  await fs.mkdir(path.join(stagingHomePath, "nested"), { recursive: true });
  await fs.mkdir(path.join(buildEvidencePath, "images"), { recursive: true });
  await fs.mkdir(path.join(buildEvidencePath, "build"), { recursive: true });
  await fs.mkdir(outputDirectory);
  await fs.writeFile(
    path.join(stagingHomePath, ".env"),
    "STANDING_PROJECT_NAME=wikijump-standing\n",
  );
  await fs.writeFile(
    path.join(stagingHomePath, "nested", "compose.yaml"),
    "services: {}\n",
  );
  const fixture = {
    root,
    runId: "fixture-run",
    stagingHomePath,
    buildEvidencePath,
    outputPath: path.join(outputDirectory, "promotion-precondition.json"),
    imagesPath: path.join(buildEvidencePath, "images", "final-images.json"),
    verdictPath: path.join(buildEvidencePath, "verdict.json"),
    sealPath: path.join(buildEvidencePath, "seal.json"),
  };
  const images = finalImages();
  await writeJson(fixture.imagesPath, images);
  await writeJson(path.join(buildEvidencePath, "build", "context.json"), {
    fixture: true,
  });
  await writeJson(fixture.verdictPath, {
    schema: "wikijump.standing_provenance_build.v1",
    status: "pass",
    promotion_eligible: true,
    run_id: fixture.runId,
    wikijump_commit: git("a"),
    wikijump_tree: git("b"),
    ftml_sha: git("c"),
    final_images: "images/final-images.json",
  });
  await sealBuildEvidence(fixture);
  const manifestSha256 = await renderedHomeManifestSha256(stagingHomePath);
  const identity = {
    schema: "wikijump.standing_candidate_parity_identity.v1",
    status: "sealed",
    artifact_key: "d".repeat(64),
    build: {
      seal_sha256: await sha256File(fixture.sealPath),
      verdict_sha256: await sha256File(fixture.verdictPath),
      final_images_sha256: await sha256File(fixture.imagesPath),
    },
    candidate: {
      owner: "standing-precondition-fixture",
      expires_at: "2099-07-20T00:00:00.000Z",
      compose_project: "wikijump-candidate-fixture",
      port_443_published: false,
      wikijump_commit: git("a"),
      wikijump_tree: git("b"),
      ftml_sha: git("c"),
      profile: "production-build",
      source_clean: true,
      images: imageMap(images),
      config: {
        isolated_overlay_sha256: "f".repeat(64),
        promotion_base_manifest_sha256: manifestSha256,
        effective_runtime_services_sha256: "0".repeat(64),
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
  const candidateIdentityPath = path.join(root, "candidate-identity.json");
  await writeJson(candidateIdentityPath, identity);
  const inputPaths = {
    receiptPath: path.join(root, "candidate-receipt.json"),
    candidateIdentityPath,
    liveReferencePath: path.join(root, "live-reference.json"),
    liveCompletionPolicyPath: path.join(root, "live-policy.json"),
  };
  for (const filePath of [
    inputPaths.receiptPath,
    inputPaths.liveReferencePath,
    inputPaths.liveCompletionPolicyPath,
  ]) {
    await fs.writeFile(filePath, "{}\n", { mode: 0o600 });
  }
  return {
    ...fixture,
    ...inputPaths,
    identity,
    identitySha256: await sha256File(candidateIdentityPath),
  };
}

function passingVerifier(fixture, calls) {
  return async (argumentsValue) => {
    calls.push(argumentsValue);
    return passingAdmission(fixture.identity, fixture.identitySha256);
  };
}

async function rewriteFixtureIdentity(fixture) {
  await writeJson(fixture.candidateIdentityPath, fixture.identity);
  fixture.identitySha256 = await sha256File(fixture.candidateIdentityPath);
}

test("binds a verified source admission to the exact sealed build and rendered home", async (t) => {
  const fixture = await createFixture(t);
  const calls = [];
  const now = new Date("2026-07-20T12:00:00.000Z");
  const result = await verifyStandingPromotionPrecondition({
    ...fixture,
    now,
    verifyAdmission: passingVerifier(fixture, calls),
  });

  assert.deepEqual(calls, [
    {
      receiptPath: fixture.receiptPath,
      candidateIdentityPath: fixture.candidateIdentityPath,
      liveReferencePath: fixture.liveReferencePath,
      liveCompletionPolicyPath: fixture.liveCompletionPolicyPath,
      now,
    },
  ]);
  assert.equal(result.schema, STANDING_PROMOTION_PRECONDITION_SCHEMA);
  assert.equal(result.status, "pass");
  assert.equal(
    result.candidate.wikijump_commit,
    fixture.identity.candidate.wikijump_commit,
  );
  assert.equal(
    result.staging_home.manifest_sha256,
    fixture.identity.candidate.config.promotion_base_manifest_sha256,
  );
  assert.equal(result.output.path, "promotion-precondition.json");
  const sealed = JSON.parse(await fs.readFile(fixture.outputPath, "utf8"));
  assert.equal(sealed.status, "pass");
  assert.equal(
    sealed.build.final_images_sha256,
    fixture.identity.build.final_images_sha256,
  );
});

test("rejects a build seal that no longer matches the candidate identity before publishing output", async (t) => {
  const fixture = await createFixture(t);
  const seal = JSON.parse(await fs.readFile(fixture.sealPath, "utf8"));
  await writeJson(fixture.sealPath, { ...seal, changed: true });

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: passingVerifier(fixture, []),
    }),
    /candidate build seal SHA-256 does not match/u,
  );
  await assert.rejects(fs.access(fixture.outputPath));
});

test("rejects an image inventory that differs from the sealed candidate", async (t) => {
  const fixture = await createFixture(t);
  const images = finalImages();
  images[0].image_id = image("a");
  await writeJson(fixture.imagesPath, images);
  await sealBuildEvidence(fixture);
  fixture.identity.build.seal_sha256 = await sha256File(fixture.sealPath);
  fixture.identity.build.verdict_sha256 = await sha256File(fixture.verdictPath);
  await rewriteFixtureIdentity(fixture);

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: passingVerifier(fixture, []),
    }),
    /candidate final image inventory SHA-256 does not match/u,
  );
  await assert.rejects(fs.access(fixture.outputPath));
});

test("rejects a sealed build whose commit does not match the candidate identity", async (t) => {
  const fixture = await createFixture(t);
  const verdict = JSON.parse(await fs.readFile(fixture.verdictPath, "utf8"));
  await writeJson(fixture.verdictPath, {
    ...verdict,
    wikijump_commit: git("d"),
  });
  await sealBuildEvidence(fixture);

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: passingVerifier(fixture, []),
    }),
    /candidate wikijump_commit versus sealed build does not match/u,
  );
  await assert.rejects(fs.access(fixture.outputPath));
});

test("rejects a build evidence manifest with an incomplete runtime image inventory", async (t) => {
  const fixture = await createFixture(t);
  await writeJson(fixture.imagesPath, finalImages().slice(1));
  await sealBuildEvidence(fixture);

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: passingVerifier(fixture, []),
    }),
    /sealed final image inventory must contain every runtime role/u,
  );
  await assert.rejects(fs.access(fixture.outputPath));
});

test("rejects an unmanifested file in the sealed build evidence", async (t) => {
  const fixture = await createFixture(t);
  await fs.writeFile(
    path.join(fixture.buildEvidencePath, "unlisted-proof"),
    "no",
  );

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: passingVerifier(fixture, []),
    }),
    /build evidence file set does not match the sealed manifest/u,
  );
  await assert.rejects(fs.access(fixture.outputPath));
});

test("rejects a rendered home whose manifest no longer matches the candidate", async (t) => {
  const fixture = await createFixture(t);
  await fs.writeFile(
    path.join(fixture.stagingHomePath, "late-override"),
    "not sealed\n",
  );

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: passingVerifier(fixture, []),
    }),
    /candidate promotion-base manifest SHA-256 does not match/u,
  );
  await assert.rejects(fs.access(fixture.outputPath));
});

test("captures the promotion binding only after source admission completes", async (t) => {
  const fixture = await createFixture(t);
  let verifierCalls = 0;

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: async () => {
        verifierCalls += 1;
        await fs.writeFile(
          path.join(fixture.stagingHomePath, "changed-during-admission"),
          "not part of the candidate identity\n",
        );
        return passingAdmission(fixture.identity, fixture.identitySha256);
      },
    }),
    /candidate promotion-base manifest SHA-256 does not match/u,
  );
  assert.equal(verifierCalls, 1);
  await assert.rejects(fs.access(fixture.outputPath));
});

test("rejects receipts inside a hashed input tree or at an input path", async (t) => {
  const fixture = await createFixture(t);
  const cases = [
    {
      name: "rendered staging home",
      outputPath: path.join(fixture.stagingHomePath, "promotion-receipt.json"),
      error: /output must not be within the rendered staging home/u,
    },
    {
      name: "sealed build evidence",
      outputPath: path.join(
        fixture.buildEvidencePath,
        "promotion-receipt.json",
      ),
      error: /output must not be within sealed build evidence/u,
    },
    {
      name: "candidate parity receipt",
      outputPath: fixture.receiptPath,
      error: /output must not equal candidate parity receipt/u,
    },
  ];
  for (const { name, outputPath, error } of cases) {
    const calls = [];
    await assert.rejects(
      verifyStandingPromotionPrecondition({
        ...fixture,
        outputPath,
        verifyAdmission: passingVerifier(fixture, calls),
      }),
      error,
      name,
    );
    assert.equal(calls.length, 0, name);
  }
});

test("publishes nothing when the source verifier rejects any evidence input", async (t) => {
  for (const rejectedInput of [
    "receipt",
    "candidate identity",
    "screenshot",
    "live reference",
    "live completion policy",
  ]) {
    const fixture = await createFixture(t);
    let verifierCalls = 0;
    await assert.rejects(
      verifyStandingPromotionPrecondition({
        ...fixture,
        verifyAdmission: async () => {
          verifierCalls += 1;
          throw new Error(`rejected ${rejectedInput}`);
        },
      }),
      new RegExp(`rejected ${rejectedInput}`, "u"),
    );
    assert.equal(verifierCalls, 1, rejectedInput);
    await assert.rejects(fs.access(fixture.outputPath));
  }
});

test("uses the real source admission verifier and leaves no output for incomplete evidence", async (t) => {
  const fixture = await createFixture(t);

  await assert.rejects(verifyStandingPromotionPrecondition({ ...fixture }));
  await assert.rejects(fs.access(fixture.outputPath));
});

test("refuses to replace a pre-existing precondition receipt with different bytes", async (t) => {
  const fixture = await createFixture(t);
  await fs.writeFile(fixture.outputPath, "different receipt\n", {
    mode: 0o600,
  });

  await assert.rejects(
    verifyStandingPromotionPrecondition({
      ...fixture,
      verifyAdmission: passingVerifier(fixture, []),
    }),
    /already exists with different bytes/u,
  );
});

test("uses the host-compatible sorted rendered-home manifest algorithm", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "standing-manifest-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.mkdir(path.join(root, "z"));
  await fs.writeFile(path.join(root, "b"), "b");
  await fs.writeFile(path.join(root, "z", "a"), "a");
  const lines = [`${hash("b")}  ./b\n`, `${hash("a")}  ./z/a\n`].join("");
  assert.equal(await renderedHomeManifestSha256(root), hash(lines));
});

test("rejects rendered-home paths whose GNU sha256sum encoding is ambiguous", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "standing-manifest-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.writeFile(path.join(root, "backslash\\name"), "x");

  await assert.rejects(
    renderedHomeManifestSha256(root),
    /rendered home contains an unsafe path/u,
  );
});
