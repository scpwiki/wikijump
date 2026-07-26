import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

import {
  DISPOSITION_POLICY_SCHEMA,
  classifySyntaxDispositions,
  validateDispositionPolicy,
} from "../src/syntax-differential-dispositions.mjs";
import {parseArgs} from "../scripts/check-syntax-differential-dispositions.mjs";

const hash = (digit) => digit.repeat(64);
const comparison = (caseId, status, sourceHash = hash("a")) => ({
  schema: "wikijump_syntax_differential.syntax_comparison.v1",
  case_id: caseId,
  status,
  ...(status === "not-applicable" ? {} : {identities: {source_sha256: sourceHash}}),
});
const verdict = (...comparisons) => ({
  schema: "wikijump_syntax_differential.verdict.v1",
  comparisons,
});
const policy = (...cases) => ({
  schema: DISPOSITION_POLICY_SCHEMA,
  cases,
});
const entry = (caseId, disposition, sourceHash = hash("a")) => ({
  case_id: caseId,
  source_sha256: sourceHash,
  disposition,
  reason: "Reviewed boundary.",
});

test("checked-in policy binds the reviewed nine cases to source identities", () => {
  const checkedIn = validateDispositionPolicy(
    JSON.parse(
      fs.readFileSync(
        new URL("../fixtures/syntax-differential/disposition-policy.json", import.meta.url),
        "utf8",
      ),
    ),
  );

  assert.deepEqual(
    checkedIn.cases.map(({case_id, source_sha256}) => [case_id, source_sha256.slice(0, 24)]),
    [
      ["record--041d27d62e8796931e5bf45e", "041d27d62e8796931e5bf45e"],
      ["record--1508f615cacfeb5911b05a8f", "1508f615cacfeb5911b05a8f"],
      ["record--1aa38fcab11f0786a323ec84", "1aa38fcab11f0786a323ec84"],
      ["record--4b790c0731c4d8e0d29b3b2e", "4b790c0731c4d8e0d29b3b2e"],
      ["record--b41cb7d94f78e297ed4b91b9", "b41cb7d94f78e297ed4b91b9"],
      ["record--b70d8e87e7200f8e3ce2abd4", "b70d8e87e7200f8e3ce2abd4"],
      ["record--cbe1eb71be3bc18ae6477ce7", "cbe1eb71be3bc18ae6477ce7"],
      ["record--d4b2e038dd0a3ec266412748", "d4b2e038dd0a3ec266412748"],
      ["record--d40a0fe016a2ae0e02de10aa", "d40a0fe016a2ae0e02de10aa"],
    ],
  );
});

test("classifies exact mismatch identities and leaves runtime-tier cases not applicable", () => {
  const result = classifySyntaxDispositions(
    verdict(
      comparison("security", "mismatch"),
      comparison("runtime", "mismatch", hash("b")),
      comparison("resource", "mismatch", hash("c")),
      comparison("runtime-tier", "not-applicable"),
    ),
    policy(
      entry("security", "intentional-security-boundary"),
      entry("runtime", "wikijump-runtime-boundary", hash("b")),
      entry("resource", "live-observation-resource-failure", hash("c")),
    ),
  );

  assert.equal(result.status, "accepted");
  assert.deepEqual(
    result.accepted.map(({case_id, disposition}) => [case_id, disposition]),
    [
      ["security", "intentional-security-boundary"],
      ["runtime", "wikijump-runtime-boundary"],
      ["resource", "live-observation-resource-failure"],
    ],
  );
  assert.deepEqual(result.failures, []);
});

test("fails unknown and source-changed mismatches", () => {
  const result = classifySyntaxDispositions(
    verdict(
      comparison("unknown", "mismatch"),
      comparison("changed", "mismatch", hash("b")),
    ),
    policy(entry("changed", "wikijump-runtime-boundary")),
  );

  assert.equal(result.status, "failed");
  assert.deepEqual(result.failures.map(({kind}) => kind), [
    "unknown-mismatch",
    "stale-source-hash",
  ]);
});

test("fails resolved, inapplicable, and missing policy entries", () => {
  const result = classifySyntaxDispositions(
    verdict(
      comparison("resolved", "match"),
      comparison("runtime-tier", "not-applicable"),
    ),
    policy(
      entry("resolved", "live-observation-resource-failure"),
      entry("runtime-tier", "wikijump-runtime-boundary"),
      entry("gone", "intentional-security-boundary"),
    ),
  );

  assert.equal(result.status, "failed");
  assert.deepEqual(result.failures.map(({kind}) => kind), [
    "resolved-policy-entry",
    "inapplicable-policy-entry",
    "missing-policy-case",
  ]);
});

test("runner errors always fail even without a policy entry", () => {
  const result = classifySyntaxDispositions(
    verdict(comparison("broken-runner", "runner-error")),
    policy(),
  );

  assert.equal(result.status, "failed");
  assert.deepEqual(result.failures, [
    {case_id: "broken-runner", kind: "runner-error"},
  ]);
});

test("rejects duplicate or unsupported policy entries", () => {
  const duplicate = entry("same", "intentional-security-boundary");
  assert.throws(
    () => classifySyntaxDispositions(verdict(), policy(duplicate, duplicate)),
    /invalid or duplicated/u,
  );
  assert.throws(
    () => classifySyntaxDispositions(
      verdict(),
      policy(entry("case", "accepted")),
    ),
    /invalid or duplicated/u,
  );
});

test("CLI requires explicit verdict and policy paths", () => {
  assert.deepEqual(parseArgs(["--", "--verdict", "verdict.json", "--policy", "policy.json"]), {
    verdict: "verdict.json",
    policy: "policy.json",
  });
  assert.deepEqual(parseArgs(["--help"]), {help: true});
  assert.throws(() => parseArgs(["--verdict", "verdict.json"]), /required/u);
});
