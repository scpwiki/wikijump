import assert from "node:assert/strict";
import test from "node:test";

import {reconcileIssueClosure} from "../src/closure-reconciler.mjs";

function issue(number, state, ownerComments = []) {
  return {number, state, owner_comments: ownerComments.map((body) => ({body}))};
}

test("detects historical closed-parent acceptance contradictions", () => {
  const result = reconcileIssueClosure({
    issue: issue(6, "CLOSED", ["Owner note: parent acceptance remained open after child closure."]),
    proofBundles: [{status: "accepted"}],
  });

  assert.equal(result.classification, "STATE_CONTRADICTION");
  assert.match(result.proposed_comment, /controller review/i);
});

test("detects issue 8 style inventory-versus-render contradiction", () => {
  const result = reconcileIssueClosure({
    issue: issue(8, "CLOSED", ["Inventory PR did not satisfy render, resource serving, and browser proof."]),
    proofBundles: [{status: "accepted"}],
  });

  assert.equal(result.classification, "STATE_CONTRADICTION");
});

test("classifies closed issues without proof as unreconciled", () => {
  const result = reconcileIssueClosure({
    issue: issue(41, "CLOSED"),
    proofBundles: [],
  });

  assert.equal(result.classification, "CLOSED_BUT_UNRECONCILED");
});

test("classifies open issues with accepted proof as ready for closure", () => {
  const result = reconcileIssueClosure({
    issue: issue(42, "OPEN"),
    proofBundles: [{status: "accepted"}],
  });

  assert.equal(result.classification, "OPEN_BUT_ACCEPTED");
});

test("classifies unresolved post-merge findings as follow-up work", () => {
  const result = reconcileIssueClosure({
    issue: issue(45, "OPEN"),
    proofBundles: [{status: "accepted"}],
    postMergeFindings: [{id: "finding-1", needs_followup: true}],
  });

  assert.equal(result.classification, "NEEDS_FOLLOWUP_ISSUE");
});

test("classifies unverified post-merge monitor findings as follow-up work", () => {
  const result = reconcileIssueClosure({
    issue: issue(46, "CLOSED"),
    proofBundles: [{status: "accepted"}],
    postMergeFindings: [
      {
        source: "review",
        source_id: "101",
        source_state: "CHANGES_REQUESTED",
        type: "changes_requested",
        controller_disposition: "unverified",
      },
    ],
  });

  assert.equal(result.classification, "NEEDS_FOLLOWUP_ISSUE");
});

test("classifies closed issues with proof and disposed gaps as accepted closed", () => {
  const result = reconcileIssueClosure({
    issue: issue(47, "CLOSED"),
    proofBundles: [{status: "accepted"}],
    gapLedger: [{id: "gap-1", disposition: "accepted_out_of_scope"}],
  });

  assert.equal(result.classification, "ACCEPTED_CLOSED");
});

test("treats null evidence collections as empty", () => {
  const result = reconcileIssueClosure({
    issue: issue(99, "OPEN"),
    children: null,
    proofBundles: null,
    gapLedger: null,
    postMergeFindings: null,
  });

  assert.equal(result.classification, "NEEDS_PROOF");
});

test("treats null top-level input as empty evidence", () => {
  const result = reconcileIssueClosure(null);

  assert.equal(result.issue_number, null);
  assert.equal(result.classification, "NEEDS_PROOF");
});
