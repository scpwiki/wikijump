import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  REQUIRED_SURFACES,
  validateMarkerContractFixtures,
} from "../scripts/run-ftml-marker-contract-canary.mjs";
import { collectCandidateSourceExecutionIdentity } from "./candidate-source-execution-identity.mjs";
import { candidateSitePageOrigin } from "./standing-browser-parity-receipt.mjs";
import {
  aggregateCompareVerdict,
  comparePair,
} from "./render-compare.mjs";
import {
  readJsonObject,
  sha256File,
} from "./standing-browser-parity-util.mjs";

const REPOSITORY_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../..",
);
const FIXTURE_INDEX =
  "install/local/wikidot-verification/fixtures/ftml-marker-contract/fixtures.json";
export const RETAINED_CANARY =
  "install/local/wikidot-verification/fixtures/ftml-marker-contract-canary/canary-summary-e82b6909.json";
const CASE_ID = "F1380_FTML_MARKER_CONTRACT";
const SOURCE_FILES = Object.freeze([
  "deepwell/Cargo.lock",
  "install/local/wikidot-verification/fixtures/ftml-marker-contract/fixtures.json",
  "install/local/wikidot-verification/fixtures/ftml-marker-contract-canary/canary-summary-e82b6909.json",
  "install/local/wikidot-verification/scripts/run-candidate-cases.mjs",
  "install/local/wikidot-verification/scripts/run-ftml-marker-contract-canary.mjs",
  "install/local/wikidot-verification/src/atomic-no-replace.mjs",
  "install/local/wikidot-verification/src/browser-request-gate.mjs",
  "install/local/wikidot-verification/src/candidate-browser-contexts.mjs",
  "install/local/wikidot-verification/src/candidate-case-command.mjs",
  "install/local/wikidot-verification/src/candidate-case-runner.mjs",
  "install/local/wikidot-verification/src/candidate-source-execution-identity.mjs",
  "install/local/wikidot-verification/src/compatibility-candidate-input-producer.mjs",
  "install/local/wikidot-verification/src/capture-egress-proxy.mjs",
  "install/local/wikidot-verification/src/cli-entry.mjs",
  "install/local/wikidot-verification/src/corpus-file-reader.mjs",
  "install/local/wikidot-verification/src/ftml-marker-contract-candidate-case-set.mjs",
  "install/local/wikidot-verification/src/oracle-fixtures.mjs",
  "install/local/wikidot-verification/src/render-compare.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-browser-session.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-contract.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-observation.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-receipt.mjs",
  "install/local/wikidot-verification/src/standing-browser-parity-util.mjs",
  "install/local/wikidot-verification/src/standing-browser-pseudo-layout.mjs",
  "install/local/wikidot-verification/src/standing-browser-runtime-identity.mjs",
  "install/local/wikidot-verification/src/standing-browser-screenshot.mjs",
  "install/local/wikidot-verification/package.json",
  "install/local/wikidot-verification/pnpm-lock.yaml",
]);

function absolute(relativePath) {
  return path.join(REPOSITORY_ROOT, relativePath);
}

function expectedRecords(summary, fixtures) {
  assert.equal(summary.status, "pass", "retained marker canary must have passed");
  assert.deepEqual(summary.required_surfaces, REQUIRED_SURFACES);
  const records = summary.baseline?.records;
  assert.equal(records?.schema, "wikijump_full_parity.browser_rendering_evidence.v1");
  assert.equal(records?.stage, "baseline");
  assert.equal(records?.selected_count, fixtures.fixtures.length);
  assert.ok(Array.isArray(records.evidence), "retained marker records are missing");
  const byId = new Map(records.evidence.map((record) => [record.fixture_id, record]));
  assert.equal(byId.size, records.evidence.length, "retained marker records must be unique");
  return fixtures.fixtures.map((fixture) => {
    const record = byId.get(fixture.fixture_id);
    assert.ok(record, `retained marker record is missing: ${fixture.fixture_id}`);
    assert.equal(record.slug, fixture.slug);
    assert.equal(record.source_status, 200);
    assert.equal(record.local_status, 200);
    assert.equal(typeof record.local_visible_text, "string");
    assert.equal(Number.isSafeInteger(record.marker_count), true);
    return Object.freeze({ fixture, record });
  });
}

function cargoLockSha256(executionIdentity) {
  const lock = executionIdentity.modules.find(
    ({ path: sourcePath }) => sourcePath === "deepwell/Cargo.lock",
  );
  assert.ok(lock, "candidate source identity must hash deepwell/Cargo.lock");
  return lock.sha256;
}

async function observedPage(page) {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    try {
      return await page.evaluate(() => {
        const root = document.querySelector("#page-content");
        if (!root) return null;
        return {
          visible_text: (root.innerText ?? "").replace(/\s+/gu, " ").trim(),
          marker_count: root.querySelectorAll(
            "h1,h2,h3,h4,h5,h6,hr,div,span,[style*='text-align']",
          ).length,
        };
      });
    } catch (error) {
      if (
        attempt === 2 ||
        !/Execution context was destroyed|Cannot find context with specified id/iu.test(
          error?.message ?? "",
        )
      ) {
        throw error;
      }
      await page.waitForLoadState("domcontentloaded", { timeout: 30_000 });
    }
  }
  throw new Error("FTML marker browser observation did not reach a stable document");
}

function verifyCleanup(proof, resources) {
  assert.deepEqual(resources, []);
  assert.deepEqual(proof, {
    public_absence_verified: true,
    public_mutations: 0,
    scope: "read-only-browser-capture",
  });
  return proof;
}

export const FTML_MARKER_CONTRACT_CASE_IDS = Object.freeze([CASE_ID]);

export function createFtmlMarkerContractCandidateCaseSet() {
  return Object.freeze({
    id: "ftml-marker-contract",
    caseIds: FTML_MARKER_CONTRACT_CASE_IDS,
    async prepareRun({ candidateIdentity, candidateBrowserContexts }) {
      if (candidateIdentity.candidate.port_443_published !== false) {
        throw new Error(
          "FTML marker cases require the exact non-standing scp-wiki candidate",
        );
      }
      const pageOrigin = candidateSitePageOrigin(candidateIdentity, "scp-wiki");

      const executionIdentity = await collectCandidateSourceExecutionIdentity(
        candidateIdentity,
        SOURCE_FILES,
      );
      const fixtures = validateMarkerContractFixtures(
        await readJsonObject(absolute(FIXTURE_INDEX), "FTML marker fixture index"),
      );
      const retained = await readJsonObject(
        absolute(RETAINED_CANARY),
        "retained FTML marker canary",
      );
      assert.equal(retained.candidate_ftml, executionIdentity.ftml_sha);
      assert.match(
        retained.run_id ?? "",
        new RegExp(
          `^ftml-marker-${retained.candidate_ftml.slice(0, 8)}-`,
          "u",
        ),
        "retained marker canary run_id must identify its own candidate revision",
      );
      const retainedFixtures = expectedRecords(retained, fixtures);
      const fixtureSha256 = await sha256File(absolute(FIXTURE_INDEX));
      const retainedSha256 = await sha256File(absolute(RETAINED_CANARY));
      const plan = Object.freeze({
        schema: "wikijump.ftml_marker_contract_candidate_plan.v1",
        site_slug: fixtures.site_slug,
        layout: fixtures.layout,
        surfaces: REQUIRED_SURFACES,
        source_identity: {
          wikijump_commit: executionIdentity.wikijump_commit,
          wikijump_tree: executionIdentity.wikijump_tree,
          ftml_sha: executionIdentity.ftml_sha,
          cargo_lock_sha256: cargoLockSha256(executionIdentity),
        },
        fixture_index: { path: FIXTURE_INDEX, sha256: fixtureSha256 },
        retained_canary: { path: RETAINED_CANARY, sha256: retainedSha256 },
        comparison: "existing-render-compare-visible-text-and-marker-count",
      });

      return Object.freeze({
        sourceFiles: SOURCE_FILES,
        runtimeBindings: [],
        privateInputIdentity: { mode: "no-private-input" },
        plan,
        async execute() {
          candidateBrowserContexts.setActiveFixture(CASE_ID);
          const { context } = await candidateBrowserContexts.newCandidateContext();
          const observations = [];
          try {
            for (const [index, { fixture }] of retainedFixtures.entries()) {
              const page = await context.newPage();
              try {
                const capture = await candidateBrowserContexts.captureCandidateObservation({
                  context,
                  page,
                  url: `${pageOrigin}/${fixture.slug}`,
                  label: CASE_ID,
                  index,
                  contract: {
                    slug: fixture.slug,
                    theme_family: "ftml-marker-contract",
                  },
                  viewport: { width: 1280, height: 900 },
                  timeoutMs: 30_000,
                  settleMs: 0,
                });
                const pageObservation = await observedPage(page);
                if (pageObservation === null) {
                  throw new Error(`marker fixture has no #page-content: ${fixture.fixture_id}`);
                }
                observations.push({
                  fixture_id: fixture.fixture_id,
                  slug: fixture.slug,
                  surface: fixture.surface,
                  capture,
                  page: pageObservation,
                });
              } finally {
                await page.close({ runBeforeUnload: false });
              }
            }
          } finally {
            await context.close();
          }
          return [{ case_id: CASE_ID, observations: { fixtures: observations } }];
        },
        async cleanup() {
          return {
            public_absence_verified: true,
            public_mutations: 0,
            scope: "read-only-browser-capture",
          };
        },
        verifyCase(caseId, observations) {
          assert.equal(caseId, CASE_ID);
          assert.equal(observations.fixtures.length, retainedFixtures.length);
          const pairs = retainedFixtures.map(({ fixture, record }, index) => {
            const observed = observations.fixtures[index];
            assert.equal(observed.fixture_id, fixture.fixture_id);
            assert.equal(observed.slug, fixture.slug);
            assert.equal(observed.surface, fixture.surface);
            assert.equal(observed.capture.navigation_status, 200);
            assert.deepEqual(observed.capture.failures, []);
            assert.equal(observed.page.marker_count, record.marker_count);
            return comparePair({
              fixtureId: fixture.fixture_id,
              sourceVisibleText: record.local_visible_text,
              localVisibleText: observed.page.visible_text,
              sourceUrl: record.local_url,
              localUrl: observed.capture.input_url,
              sourceArtifact: record.local_browser_artifact,
              localArtifact: observed.capture.settled_viewport_screenshot?.path,
            });
          });
          const comparison = aggregateCompareVerdict({
            runId: `candidate-marker-${plan.source_identity.ftml_sha.slice(0, 12)}`,
            pairs,
          });
          assert.deepEqual(comparison.verdict.aggregate.regressions, []);
          return {
            verified: true,
            comparison: comparison.verdict,
            source_identity: plan.source_identity,
            fixture_index: plan.fixture_index,
            retained_canary: plan.retained_canary,
          };
        },
        verifyCleanup,
      });
    },
  });
}
