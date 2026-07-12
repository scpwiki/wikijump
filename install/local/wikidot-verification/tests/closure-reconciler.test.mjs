import assert from "node:assert/strict";
import test from "node:test";

import {reconcileIssueClosure} from "../src/closure-reconciler.mjs";
import {collectPostMergeFindings} from "../src/post-merge-monitor.mjs";

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

test("reconciles mixed and verified monitor output end to end", async () => {
  const report = await collectPostMergeFindings({
    pr: {number: 52, merged_at: "2026-07-10T10:00:00Z"},
    reviews: [{id: 1001, state: "CHANGES_REQUESTED", submitted_at: "2026-07-10T10:05:00Z", body: "review"}],
    comments: [{id: 1002, created_at: "2026-07-10T10:06:00Z", body: "comment"}],
  });
  const base = {issue: issue(52, "CLOSED"), proofBundles: [{status: "accepted"}]};
  const verified = report.findings.map((finding) => ({...finding, controller_disposition: "verified"}));

  assert.equal(reconcileIssueClosure({...base, postMergeFindings: report.findings}).classification, "NEEDS_FOLLOWUP_ISSUE");
  assert.equal(reconcileIssueClosure({...base, postMergeFindings: verified}).classification, "ACCEPTED_CLOSED");
  assert.equal(
    reconcileIssueClosure({...base, postMergeFindings: [verified[0], report.findings[1]]}).classification,
    "NEEDS_FOLLOWUP_ISSUE",
  );
});

test("fails closed on malformed monitor dispositions but accepts a null collection", async () => {
  const report = await collectPostMergeFindings({
    pr: {number: 53, merged_at: "2026-07-10T10:00:00Z"},
    comments: [{id: 1003, created_at: "2026-07-10T10:01:00Z", body: "follow-up"}],
  });
  const base = {issue: issue(53, "CLOSED"), proofBundles: [{status: "accepted"}]};

  for (const disposition of [null, "", "resolved", 7]) {
    const finding = {...report.findings[0], controller_disposition: disposition};
    assert.equal(
      reconcileIssueClosure({...base, postMergeFindings: [finding]}).classification,
      "NEEDS_FOLLOWUP_ISSUE",
    );
  }
  assert.equal(reconcileIssueClosure({...base, postMergeFindings: [null]}).classification, "NEEDS_FOLLOWUP_ISSUE");
  assert.equal(reconcileIssueClosure({...base, postMergeFindings: null}).classification, "ACCEPTED_CLOSED");
});

test("fails closed on malformed monitor identities and finding fields", () => {
  const base = {issue: issue(54, "CLOSED"), proofBundles: [{status: "accepted"}]};
  const malformedFindings = [
    [],
    {source: "review", source_id: 1004, controller_disposition: "verified"},
    {source: "review", source_id: "", controller_disposition: "verified"},
    {source: "review", source_id: "  ", controller_disposition: "verified"},
    {source: "review", controller_disposition: "verified"},
    {source: "unknown", source_id: "1004", controller_disposition: "verified"},
    {source_id: "1004", controller_disposition: "verified"},
    {id: "legacy", needs_followup: "false"},
    {id: "legacy", disposition: null},
  ];

  for (const finding of malformedFindings) {
    assert.equal(
      reconcileIssueClosure({...base, postMergeFindings: [finding]}).classification,
      "NEEDS_FOLLOWUP_ISSUE",
    );
  }
});

test("preserves legacy finding compatibility when no monitor schema fields are present", () => {
  const result = reconcileIssueClosure({
    issue: issue(55, "CLOSED"),
    proofBundles: [{status: "accepted"}],
    postMergeFindings: [{id: "legacy", needs_followup: false, disposition: "resolved"}],
  });

  assert.equal(result.classification, "ACCEPTED_CLOSED");
});

test("does not accept inherited monitor verification fields", () => {
  const inheritedFinding = Object.create({
    source: "review",
    source_id: "1005",
    controller_disposition: "verified",
  });
  const result = reconcileIssueClosure({
    issue: issue(56, "CLOSED"),
    proofBundles: [{status: "accepted"}],
    postMergeFindings: [inheritedFinding],
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
