import assert from "node:assert/strict";
import test from "node:test";

import {validateAcquisitionSummary} from "../src/reference-acquisition-summary.mjs";

function context() {
  return {
    attachmentCount: 2,
    attachmentPageCount: 1,
    inputRows: [
      {parent_fullname: null, required_browser: true, source_browser_visibility: "public", source_required_actor: null},
      {parent_fullname: "alpha", required_browser: false, source_browser_visibility: "private", source_required_actor: "member"},
    ],
    manifestSha256: "a".repeat(64),
    rows: [{fullname: "alpha"}, {fullname: "beta"}],
    sourceBranch: "en",
    sourceSite: "scp-wiki",
  };
}

function summary() {
  return {
    attachment_count: 2,
    attachment_page_count: 1,
    first_fullname: "alpha",
    last_fullname: "beta",
    manifest_sha256: "a".repeat(64),
    parent_count: 1,
    required_browser_count: 1,
    row_count: 2,
    source_branches: ["en"],
    source_browser_visibility_counts: {private: 1, public: 1},
    source_required_actor_count: 1,
    source_sites: ["scp-wiki"],
  };
}

test("acquisition summary must bind every manifest-derived count and identity", () => {
  assert.doesNotThrow(() => validateAcquisitionSummary(summary(), context()));
  const wrong = summary();
  wrong.source_sites = ["scp-jp"];
  assert.throws(() => validateAcquisitionSummary(wrong, context()), /source_sites does not match/u);
  wrong.source_sites = ["scp-wiki"];
  wrong.row_count = 3;
  assert.throws(() => validateAcquisitionSummary(wrong, context()), /summary\.row_count mismatch/u);
});
