import assert from "node:assert/strict";
import test from "node:test";

import {
  openPrivateComparisonOutputDirectory,
  XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES,
} from "../src/xmlrpc-pilot-local-comparison-output.mjs";

test("comparison output contract names every sealed artifact and rejects relative output paths before I/O", async () => {
  assert.deepEqual(XMLRPC_PILOT_LOCAL_COMPARISON_OUTPUT_FILES, {
    clusters: "mismatch-clusters.json",
    manifest: "verified-pilot-manifest.jsonl",
    rows: "local-comparison.jsonl",
    verdict: "xmlrpc-pilot-verdict.json",
  });
  await assert.rejects(
    () => openPrivateComparisonOutputDirectory({outputDir: "relative", pilotRoot: "/tmp/pilot"}),
    /absolute path/u,
  );
});
