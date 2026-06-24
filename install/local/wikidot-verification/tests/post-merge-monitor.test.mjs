import assert from "node:assert/strict";
import test from "node:test";

import {collectPostMergeFindings} from "../src/post-merge-monitor.mjs";

const pr25 = {
  number: 25,
  merge_commit: "1e127ab37fd79274a0c116a9012a56222bef760d",
  head_sha: "head25",
  merged_at: "2026-06-24T02:00:00Z",
};

const pr36 = {
  number: 36,
  merge_commit: "857f0fa1b206a57d6f4f3c453ee23309183ee75f",
  head_sha: "head36",
  merged_at: "2026-06-24T03:00:00Z",
};

test("captures post-merge review findings after the merge timestamp", async () => {
  const report = await collectPostMergeFindings({
    pr: pr25,
    reviews: [
      {
        id: 100,
        state: "COMMENTED",
        submitted_at: "2026-06-24T01:59:00Z",
        user: {login: "coderabbitai[bot]"},
        body: "pre-merge comment",
      },
      {
        id: 101,
        state: "CHANGES_REQUESTED",
        submitted_at: "2026-06-24T02:06:00Z",
        user: {login: "chatgpt-codex-connector[bot]"},
        body: "private/deleted fragment hazard",
      },
    ],
  });

  assert.equal(report.pr_number, 25);
  assert.equal(report.findings.length, 1);
  assert.equal(report.findings[0].type, "changes_requested");
  assert.equal(report.findings[0].controller_disposition, "unverified");
});

test("does not include events exactly at the merge timestamp", async () => {
  const report = await collectPostMergeFindings({
    pr: pr25,
    reviews: [
      {
        id: 102,
        state: "COMMENTED",
        submitted_at: pr25.merged_at,
        user: {login: "coderabbitai[bot]"},
        body: "same-time event",
      },
    ],
  });

  assert.equal(report.findings.length, 0);
});

test("captures post-merge inline comments and schedules the next hourly watch", async () => {
  const report = await collectPostMergeFindings({
    pr: pr36,
    comments: [
      {
        id: 200,
        path: "install/local/wikidot-verification/src/resource-manifest.mjs",
        line: 42,
        created_at: "2026-06-24T03:06:00Z",
        user: {login: "chatgpt-codex-connector[bot]"},
        body: "path normalization follow-up",
      },
    ],
  });

  assert.equal(report.findings.length, 1);
  assert.equal(report.findings[0].type, "inline_comment");
  assert.equal(report.findings[0].path, "install/local/wikidot-verification/src/resource-manifest.mjs");
  assert.equal(report.delayed_watch_after, "2026-06-24T04:00:00.000Z");
});

test("rejects a malformed merge timestamp", async () => {
  await assert.rejects(
    collectPostMergeFindings({
      pr: {...pr36, merged_at: "not-a-date"},
    }),
    /valid ISO-8601 timestamp/,
  );
});

test("deduplicates findings already seen by an immediate watch", async () => {
  const first = await collectPostMergeFindings({
    pr: pr36,
    comments: [
      {
        id: 201,
        path: "install/local/wikidot-verification/src/resource-materializer.mjs",
        line: 9,
        created_at: "2026-06-24T03:05:00Z",
        user: {login: "coderabbitai[bot]"},
        body: "absolute path concern",
      },
    ],
  });

  const second = await collectPostMergeFindings({
    pr: pr36,
    comments: [
      {
        id: 201,
        path: "install/local/wikidot-verification/src/resource-materializer.mjs",
        line: 9,
        created_at: "2026-06-24T03:05:00Z",
        user: {login: "coderabbitai[bot]"},
        body: "absolute path concern",
      },
    ],
    previousFindingKeys: first.findings.map((finding) => finding.finding_key),
  });

  assert.equal(first.findings.length, 1);
  assert.equal(second.findings.length, 0);
});
