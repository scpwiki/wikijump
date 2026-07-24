import assert from "node:assert/strict";
import test from "node:test";

import {sha256Hex} from "../src/canonical-json.mjs";
import {
  assertCanonicalFullname,
  assertTimestamp,
  parseReferenceAcquisitionManifest,
  parseReferenceAcquisitionSummary,
  validateOrigin,
} from "../src/reference-acquisition-inventory-source.mjs";

test("reference acquisition source validation rejects ambiguous identity inputs", () => {
  assert.deepEqual(validateOrigin("https://scp-wiki.wikidot.com"), {
    origin: "https://scp-wiki.wikidot.com",
    sourceSite: "scp-wiki",
  });
  assert.doesNotThrow(() => assertCanonicalFullname("theme:雪 space", "fullname"));
  assert.throws(() => assertCanonicalFullname("../unsafe", "fullname"), /unsafe path/u);
  assert.doesNotThrow(() => assertTimestamp("2026-07-24T00:00:00.000Z", "updated_at"));
  assert.throws(() => assertTimestamp("2026-02-30T00:00:00Z", "updated_at"), /RFC 3339/u);
});

test("reference acquisition source parsing binds bytes and expected digests", () => {
  const manifest = '{"fullname":"alpha"}\n';
  const parsed = parseReferenceAcquisitionManifest(
    manifest,
    sha256Hex(manifest),
    1,
  );
  assert.equal(parsed.rows[0].input.fullname, "alpha");
  const summary = '{"row_count":1}\n';
  assert.deepEqual(
    parseReferenceAcquisitionSummary(summary, sha256Hex(summary)).summary,
    {row_count: 1},
  );
  assert.throws(
    () => parseReferenceAcquisitionManifest(manifest, "0".repeat(64), 1),
    /manifest SHA-256 mismatch/u,
  );
});
