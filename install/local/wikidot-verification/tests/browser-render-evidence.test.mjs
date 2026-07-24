import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {test} from "node:test";

import {browserCaptureFailure} from "../scripts/capture-browser-rendering.mjs";
import {
  buildEvidenceRecord,
  compactVisibleText,
  writeEvidenceArtifacts,
} from "../src/browser-render-evidence.mjs";
import {
  inventoryRows,
  rowLocalUrl,
  rowSourceUrl,
  safePathSegment,
  selectInventoryRows,
} from "../src/browser-render-target.mjs";
import {browserRenderInventory as inventory} from "./support/browser-render-evidence-fixture.mjs";

test("browser capture failure preserves both operation and cleanup errors", () => {
  const captureError = new Error("capture failed");
  const cleanupError = new Error("cleanup failed");
  const combined = browserCaptureFailure(captureError, cleanupError);
  assert(combined instanceof AggregateError);
  assert.deepEqual(combined.errors, [captureError, cleanupError]);
  assert.equal(browserCaptureFailure(captureError, null), captureError);
  assert.equal(browserCaptureFailure(null, cleanupError), cleanupError);
  assert.equal(browserCaptureFailure(null, null), null);
});

test("selectInventoryRows intersects explicit fixture ids with shard membership", () => {
  const rows = inventoryRows(inventory);
  const selected = selectInventoryRows({
    rows,
    fixtureIds: ["EN:alpha", "EN:beta"],
    shardId: "en-0001",
    shardManifest: {
      schema: "wikijump_full_parity.corpus_shard_manifest.v1",
      shards: [{shard_id: "en-0001", fixture_ids: ["EN:beta"]}],
    },
  });

  assert.deepEqual(selected.map((row) => row.fixture_id), ["EN:beta"]);
});

test("selectInventoryRows rejects absent requested fixture ids", () => {
  const rows = inventoryRows(inventory);
  assert.throws(
    () => selectInventoryRows({rows, fixtureIds: ["EN:alpha", "EN:missing"]}),
    /requested fixture IDs were not found: EN:missing/
  );
});

test("inventoryRows rejects duplicate fixture ids", () => {
  assert.throws(
    () => inventoryRows({schema: inventory.schema, rows: [inventory.rows[0], {...inventory.rows[1], fixture_id: "EN:alpha"}]}),
    /inventory\.rows\[1\] duplicates fixture_id: EN:alpha/
  );
});

test("selectInventoryRows rejects shard fixture ids missing from the inventory", () => {
  const rows = inventoryRows(inventory);
  assert.throws(
    () =>
      selectInventoryRows({
        rows,
        shardId: "en-0001",
        shardManifest: {
          schema: "wikijump_full_parity.corpus_shard_manifest.v1",
          shards: [{shard_id: "en-0001", fixture_ids: ["EN:alpha", "EN:missing"]}],
        },
      }),
    /shard en-0001 fixture IDs were not found in inventory: EN:missing/
  );
});

test("buildEvidenceRecord emits fields accepted by the browser rendering validator", () => {
  const record = buildEvidenceRecord({
    row: inventory.rows[0],
    source: {status: 200, finalUrl: "https://scp-wiki.wikidot.com/alpha", visibleText: " Alpha\n page "},
    local: {status: 200, finalUrl: "https://scp-wiki.wikijump.localhost/alpha", visibleText: "Alpha page"},
    sourceArtifact: "/tmp/live.dom.html",
    localArtifact: "/tmp/local.dom.html",
    sourceScreenshot: "/tmp/live.png",
    localScreenshot: "/tmp/local.png",
  });

  assert.equal(record.evidence_type, "browser_rendering");
  assert.equal(record.fixture_id, "EN:alpha");
  assert.equal(record.source_visible_text, "Alpha page");
  assert.equal(record.local_visible_text, "Alpha page");
  assert.equal(record.source_browser_artifact, "/tmp/live.dom.html");
  assert.equal(record.local_browser_artifact, "/tmp/local.dom.html");
  assert.deepEqual(record.capture_errors, []);
});

test("writeEvidenceArtifacts keeps row artifacts under a safe fixture directory", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "wikijump-browser-evidence-"));
  const artifacts = await writeEvidenceArtifacts({
    outputDir: root,
    row: {fixture_id: "EN:../alpha beta"},
    source: {html: "<html>live</html>"},
    local: {html: "<html>local</html>"},
    screenshot: true,
  });

  assert.equal(path.dirname(artifacts.sourceArtifact), path.join(root, safePathSegment("EN:../alpha beta")));
  assert.equal(await fs.readFile(artifacts.sourceArtifact, "utf8"), "<html>live</html>");
  assert.equal(await fs.readFile(artifacts.localArtifact, "utf8"), "<html>local</html>");
  assert.equal(compactVisibleText(" one\n\t two "), "one two");
});

test("safePathSegment keeps colliding fixture IDs distinct", () => {
  assert.notEqual(safePathSegment("EN:a/b"), safePathSegment("EN:a_b"));
  assert.doesNotMatch(safePathSegment("EN:alpha"), /:/);
  assert.doesNotMatch(safePathSegment("EN:alpha."), /\.-[a-f0-9]{12}$/);
  assert.notEqual(
    safePathSegment(`EN:${"a".repeat(180)}1`),
    safePathSegment(`EN:${"a".repeat(180)}2`)
  );
});

test("inventoryRows rejects malformed rows before browser capture starts", () => {
  assert.throws(
    () => inventoryRows({schema: inventory.schema, rows: [{slug: "missing-fixture"}]}),
    /inventory\.rows\[0\] must be an object with a non-empty fixture_id/
  );
  assert.throws(
    () => inventoryRows({schema: inventory.schema, rows: [null]}),
    /inventory\.rows\[0\] must be an object with a non-empty fixture_id/
  );
});

test("row URL helpers skip blank preferred fields before falling back", () => {
  assert.equal(rowSourceUrl({source_url: "", live_url: "https://live.example/page"}), "https://live.example/page");
  assert.equal(
    rowLocalUrl({local_https_url: "", local_http_url: "http://local.example/page"}),
    "http://local.example/page"
  );
});
