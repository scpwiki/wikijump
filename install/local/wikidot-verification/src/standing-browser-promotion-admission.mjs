import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  DEFAULT_THRESHOLDS,
  validateLiveCompletionPolicy,
} from "./standing-browser-parity-contract.mjs";
import {
  candidatePageOrigin,
  assertCandidateIdentityFresh,
  validateCandidateParityIdentity,
  validateCandidateParityReceipt,
} from "./standing-browser-parity-receipt.mjs";
import { defaultCanaryPairs } from "./standing-browser-canaries.mjs";
import { loadSealedLiveReference } from "./standing-browser-parity-reference.mjs";
import { collectCandidateExecutionIdentity } from "./standing-browser-execution-identity.mjs";
import { observationArtifactName } from "./standing-browser-parity-observation.mjs";
import {
  readJsonObject,
  requireSha256,
  sha256File,
  sha256Value,
} from "./standing-browser-parity-util.mjs";

const SOURCE_DIR = path.dirname(fileURLToPath(import.meta.url));
const RUNNER_PATH = path.join(SOURCE_DIR, "standing-browser-parity-runner.mjs");
const OBSERVATION_PATH = path.join(
  SOURCE_DIR,
  "standing-browser-parity-observation.mjs",
);
const DEFAULT_LIVE_ORIGIN = "https://scp-wiki.wikidot.com";

export const STANDING_CANDIDATE_PARITY_ADMISSION_SCHEMA =
  "wikijump.standing_candidate_parity_admission.v1";

function receiptIdentity(receipt) {
  return validateCandidateParityIdentity({
    schema: "wikijump.standing_candidate_parity_identity.v1",
    status: "sealed",
    artifact_key: receipt.artifact_key,
    build: receipt.build,
    candidate: receipt.candidate,
    evidence: receipt.evidence,
  });
}

function sameCanonical(left, right) {
  return sha256Value(left) === sha256Value(right);
}

async function requirePrivateRegularFile(filePath, name) {
  const stat = await fs.lstat(filePath).catch(() => null);
  if (
    !stat?.isFile() ||
    stat.isSymbolicLink() ||
    stat.nlink !== 1 ||
    (stat.mode & 0o077) !== 0
  ) {
    throw new Error(`${name} must be a private regular file`);
  }
  return stat;
}

async function verifyCandidateLedgerAndArtifacts({
  receipt,
  receiptPath,
  pairs,
}) {
  const root = path.dirname(receiptPath);
  const ledgerPath = path.join(root, "standing-browser-parity.json");
  await requirePrivateRegularFile(ledgerPath, "candidate parity ledger");
  if ((await sha256File(ledgerPath)) !== receipt.parity.ledger_sha256) {
    throw new Error(
      "candidate parity receipt ledger digest does not bind the supplied ledger",
    );
  }
  const ledger = await readJsonObject(ledgerPath, "candidate parity ledger");
  if (
    ledger.schema !== receipt.parity.schema ||
    ledger.status !== receipt.status ||
    ledger.candidate_identity_sha256 !==
      receipt.parity.candidate_identity_sha256 ||
    ledger.live_reference_sha256 !== receipt.parity.live_reference_sha256 ||
    ledger.local_capture_config_sha256 !==
      receipt.parity.request_gate_config_sha256 ||
    !sameCanonical(ledger.viewport, receipt.parity.viewport) ||
    !sameCanonical(ledger.request_gate, receipt.parity.request_gate) ||
    !sameCanonical(ledger.records, receipt.parity.records)
  ) {
    throw new Error(
      "candidate parity ledger does not exactly bind the supplied receipt",
    );
  }
  const recordsByLocalUrl = new Map(
    receipt.parity.records.map((record) => [record.input.local_url, record]),
  );
  for (const [index, pair] of pairs.entries()) {
    const record = recordsByLocalUrl.get(pair.local_url);
    if (!record)
      throw new Error(
        `candidate parity receipt lacks local artifacts for ${pair.local_url}`,
      );
    const expected = {
      local_domcontentloaded_immediate_png: observationArtifactName({
        label: "local",
        index,
        url: pair.local_url,
        phase: "domcontentloaded-immediate",
      }),
      local_settled_viewport_png: observationArtifactName({
        label: "local",
        index,
        url: pair.local_url,
        phase: "settled-viewport",
      }),
      local_settled_full_page_png: observationArtifactName({
        label: "local",
        index,
        url: pair.local_url,
        phase: "settled-full-page",
      }),
    };
    for (const [key, name] of Object.entries(expected)) {
      const artifactPath = path.join(root, name);
      await requirePrivateRegularFile(
        artifactPath,
        `candidate parity artifact ${name}`,
      );
      if ((await sha256File(artifactPath)) !== record.artifact_hashes[key]) {
        throw new Error(
          `candidate parity artifact digest does not bind ${name}`,
        );
      }
    }
  }
  return {
    ledger_sha256: receipt.parity.ledger_sha256,
    local_artifacts_verified: pairs.length * 3,
  };
}

async function readPolicy(filePath) {
  const value = validateLiveCompletionPolicy(
    await readJsonObject(filePath, "live completion policy"),
  );
  return { value, sha256: await sha256File(filePath), filePath };
}

export async function verifyStandingCandidateParityAdmission({
  receiptPath,
  candidateIdentityPath,
  liveReferencePath,
  liveCompletionPolicyPath,
  now = new Date(),
  collectExecutionIdentity = collectCandidateExecutionIdentity,
}) {
  const [receiptRaw, identityRaw, policy] = await Promise.all([
    readJsonObject(receiptPath, "candidate parity receipt"),
    readJsonObject(candidateIdentityPath, "candidate parity identity"),
    readPolicy(liveCompletionPolicyPath),
  ]);
  const [
    receiptSha256,
    candidateIdentitySha256,
    runnerSha256,
    observationSha256,
  ] = await Promise.all([
    sha256File(receiptPath),
    sha256File(candidateIdentityPath),
    sha256File(RUNNER_PATH),
    sha256File(OBSERVATION_PATH),
  ]);
  const externalIdentity = assertCandidateIdentityFresh(
    validateCandidateParityIdentity(identityRaw),
    { now },
  );
  const receipt = validateCandidateParityReceipt(receiptRaw, {
    now,
    requirePass: true,
  });
  const embeddedIdentity = receiptIdentity(receipt);
  if (!sameCanonical(externalIdentity, embeddedIdentity)) {
    throw new Error(
      "candidate parity receipt does not bind the supplied sealed candidate identity",
    );
  }
  if (receipt.parity.candidate_identity_sha256 !== candidateIdentitySha256) {
    throw new Error(
      "candidate parity receipt candidate identity digest does not bind the supplied file",
    );
  }
  if (receipt.parity.parity_script_sha256 !== runnerSha256) {
    throw new Error(
      "candidate parity receipt was not produced by this source-owned runner",
    );
  }
  if (receipt.parity.integrity_script_sha256 !== observationSha256) {
    throw new Error(
      "candidate parity receipt was not produced by this source-owned observation module",
    );
  }
  const currentExecutionIdentity =
    await collectExecutionIdentity(externalIdentity);
  if (
    !sameCanonical(currentExecutionIdentity, receipt.parity.execution_identity)
  ) {
    throw new Error(
      "candidate parity receipt was not produced by this clean source execution identity",
    );
  }
  const pairs = defaultCanaryPairs({
    localOrigin: candidatePageOrigin(externalIdentity),
    liveOrigin: DEFAULT_LIVE_ORIGIN,
  });
  const candidateArtifacts = await verifyCandidateLedgerAndArtifacts({
    receipt,
    receiptPath,
    pairs,
  });
  const liveReference = await loadSealedLiveReference({
    filePath: liveReferencePath,
    expectedSha256: receipt.parity.live_reference_sha256,
    pairs,
    viewport: receipt.parity.viewport,
    thresholds: DEFAULT_THRESHOLDS,
    policy: policy.value,
    policySha256: policy.sha256,
    policyFilePath: policy.filePath,
  });
  if (liveReference.sha256 !== receipt.parity.live_reference_sha256) {
    throw new Error(
      "candidate parity receipt live reference digest does not bind the supplied file",
    );
  }
  return Object.freeze({
    schema: STANDING_CANDIDATE_PARITY_ADMISSION_SCHEMA,
    status: "pass",
    verified_at: new Date().toISOString(),
    candidate_parity_receipt_sha256: requireSha256(
      receiptSha256,
      "candidate parity receipt SHA-256",
    ),
    candidate_identity_sha256: requireSha256(
      candidateIdentitySha256,
      "candidate identity SHA-256",
    ),
    live_reference_sha256: liveReference.sha256,
    live_completion_policy_sha256: policy.sha256,
    source_runner_sha256: runnerSha256,
    source_observation_sha256: observationSha256,
    source_execution_identity_sha256: receipt.parity.execution_identity_sha256,
    candidate: {
      compose_project: externalIdentity.candidate.compose_project,
      endpoint: externalIdentity.candidate.endpoint,
      wikijump_commit: externalIdentity.candidate.wikijump_commit,
      wikijump_tree: externalIdentity.candidate.wikijump_tree,
      ftml_sha: externalIdentity.candidate.ftml_sha,
      artifact_key: externalIdentity.artifact_key,
      expires_at: externalIdentity.candidate.expires_at,
    },
    parity: {
      pairs_total: receipt.parity.pairs_total,
      viewport: receipt.parity.viewport,
      request_gate_final_sha256: receipt.parity.request_gate_final_sha256,
      runtime_identity_sha256: receipt.parity.runtime_identity_sha256,
      ledger_sha256: candidateArtifacts.ledger_sha256,
      local_artifacts_verified: candidateArtifacts.local_artifacts_verified,
    },
  });
}
