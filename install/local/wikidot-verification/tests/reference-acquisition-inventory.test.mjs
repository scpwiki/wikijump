import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { sha256Hex, stableStringify } from "../src/corpus-import-manifest.mjs";
import {
  buildReferenceAcquisitionInventory,
  computeReferenceAcquisitionInventoryIdentity,
  serializeReferenceAcquisitionInventory,
} from "../src/reference-acquisition-inventory.mjs";

const CLI_PATH = fileURLToPath(
  new URL(
    "../scripts/build-reference-acquisition-inventory.mjs",
    import.meta.url,
  ),
);
const SOURCE_ORIGIN = "https://scp-wiki.wikidot.com";

function row(fullname, sourceEntityId, digestCharacter, attachments = []) {
  return {
    attachments,
    fullname,
    meta_sha256: digestCharacter.repeat(64),
    parent_fullname: null,
    revisions: 3,
    source_branch: "en",
    source_entity_id: sourceEntityId,
    source_sha256: (digestCharacter === "a" ? "b" : "a").repeat(64),
    source_site: "scp-wiki",
    updated_at: "2026-07-18T12:34:56+00:00",
  };
}

function attachment(fullname = "alpha") {
  return {
    corpus_path: `/ignored/${fullname}`,
    file_path: `/host/path/${fullname}`,
    filename: "image one.png",
    metadata_path: "/host/path/_state.json",
    mime: "image/png",
    original_url: `http://scp-wiki.wdfiles.com/local--files/${fullname}/image%20one.png`,
    sha256: "c".repeat(64),
    size: 123,
    wikidot_path: `/local--files/${fullname}/image%20one.png`,
  };
}

function inputs(rows) {
  const manifestText = `${rows.map((value) => stableStringify(value)).join("\n")}\n`;
  const summary = {
    attachment_count: rows.reduce(
      (count, value) => count + (value.attachments?.length ?? 0),
      0,
    ),
    attachment_page_count: rows.filter(
      (value) => (value.attachments?.length ?? 0) > 0,
    ).length,
    first_fullname: rows[0].fullname,
    last_fullname: rows.at(-1).fullname,
    manifest_sha256: sha256Hex(manifestText),
    parent_count: rows.filter((value) => value.parent_fullname !== null).length,
    required_browser_count: rows.filter(
      (value) => value.required_browser === true,
    ).length,
    row_count: rows.length,
    source_browser_visibility_counts: {},
    source_branches: ["en"],
    source_required_actor_count: 0,
    source_sites: ["scp-wiki"],
  };
  const summaryBytes = Buffer.from(`${stableStringify(summary)}\n`);
  return {
    expectedCount: rows.length,
    expectedManifestSha256: summary.manifest_sha256,
    expectedSummarySha256: sha256Hex(summaryBytes),
    manifestBytes: Buffer.from(manifestText),
    summaryBytes,
  };
}

function build(rows, overrides = {}) {
  return buildReferenceAcquisitionInventory({
    ...inputs(rows),
    family: "EN",
    shardCount: 64,
    sourceOrigin: SOURCE_ORIGIN,
    ...overrides,
  });
}

const TWO_ROWS = [
  row("alpha", "00000000-0000-0000-0000-000000000001", "a", [attachment()]),
  row("theme:雪 space", "00000000-0000-0000-0000-000000000002", "b"),
];

test("builds a deterministic, portable, exactly sharded acquisition inventory", () => {
  const first = build(TWO_ROWS);
  const second = build(structuredClone(TWO_ROWS));
  assert.equal(
    serializeReferenceAcquisitionInventory(first),
    serializeReferenceAcquisitionInventory(second),
  );
  assert.equal(
    first.rows[1].source_url,
    "https://scp-wiki.wikidot.com/theme:%E9%9B%AA%20space",
  );
  assert.deepEqual(first.rows[0].requested_layers, [
    "xmlrpc_page",
    "http_document",
    "browser_document",
  ]);
  assert.equal(first.shards.length, 64);
  assert.deepEqual(
    first.shards.flatMap((shard) => shard.fixture_ids).sort(),
    first.rows.map((value) => value.fixture_id).sort(),
  );
  assert.equal(
    first.shards.reduce((count, shard) => count + shard.count, 0),
    2,
  );
  assert.deepEqual(first.shards[21], {
    count: 1,
    fixture_ids: ["EN:theme:雪 space"],
    fixture_set_sha256: sha256Hex("EN:theme:雪 space"),
    shard_id: "en-0021",
  });
  assert.deepEqual(first.shards[24], {
    count: 1,
    fixture_ids: ["EN:alpha"],
    fixture_set_sha256: sha256Hex("EN:alpha"),
    shard_id: "en-0024",
  });
  assert.equal(
    first.identity.sha256,
    computeReferenceAcquisitionInventoryIdentity(first),
  );
  const serialized = serializeReferenceAcquisitionInventory(first);
  assert.equal(serialized.endsWith("\n"), true);
  assert.doesNotMatch(
    serialized,
    /host\/path|corpus_path|file_path|metadata_path/u,
  );
});

test("fails closed on invalid trust anchors, input framing, ordering, and identity", () => {
  const valid = inputs(TWO_ROWS);
  assert.throws(
    () => build(TWO_ROWS, { expectedManifestSha256: "0".repeat(64) }),
    /SHA-256 mismatch/u,
  );
  assert.throws(
    () => build(TWO_ROWS, { expectedCount: 3 }),
    /row count mismatch/u,
  );
  assert.throws(
    () => build(TWO_ROWS, { expectedSummarySha256: "0".repeat(64) }),
    /summary SHA-256 mismatch/u,
  );
  assert.throws(
    () =>
      buildReferenceAcquisitionInventory({
        ...valid,
        family: "EN",
        shardCount: 64,
        sourceOrigin: "http://scp-wiki.wikidot.com",
      }),
    /credential-free HTTPS origin/u,
  );
  assert.throws(
    () =>
      build(TWO_ROWS, { sourceOrigin: "https://scp-wiki.wikidot.com:8443" }),
    /credential-free HTTPS origin/u,
  );
  assert.throws(() => build([...TWO_ROWS].reverse()), /strictly sorted/u);
  assert.throws(
    () =>
      build([TWO_ROWS[0], { ...TWO_ROWS[1], fullname: TWO_ROWS[0].fullname }]),
    /collides at source URL|strictly sorted and unique/u,
  );
  assert.throws(
    () => build([{ ...TWO_ROWS[0], source_entity_id: null }, TWO_ROWS[1]]),
    /source_entity_id must be a non-empty string/u,
  );
  assert.throws(
    () => build([{ ...TWO_ROWS[0], attachments: {} }, TWO_ROWS[1]]),
    /attachments must be an array/u,
  );
  assert.throws(
    () => build([{ ...TWO_ROWS[0], fullname: "../admin" }, TWO_ROWS[1]]),
    /unsafe path character/u,
  );
  assert.throws(
    () => build([{ ...TWO_ROWS[0], updated_at: "2026-07-18" }, TWO_ROWS[1]]),
    /RFC 3339/u,
  );
  assert.throws(
    () =>
      build([
        TWO_ROWS[0],
        { ...TWO_ROWS[1], source_entity_id: TWO_ROWS[0].source_entity_id },
      ]),
    /duplicate source_entity_id/u,
  );
  const malformed = Buffer.from("{}\r\n");
  assert.throws(
    () =>
      buildReferenceAcquisitionInventory({
        ...valid,
        manifestBytes: malformed,
        expectedCount: 1,
        expectedManifestSha256: sha256Hex(malformed),
        family: "EN",
        shardCount: 1,
        sourceOrigin: SOURCE_ORIGIN,
      }),
    /use LF lines/u,
  );
  const invalidJson = Buffer.from("{not-json}\n");
  assert.throws(
    () =>
      buildReferenceAcquisitionInventory({
        ...valid,
        expectedCount: 1,
        expectedManifestSha256: sha256Hex(invalidJson),
        family: "EN",
        manifestBytes: invalidJson,
        shardCount: 1,
        sourceOrigin: SOURCE_ORIGIN,
      }),
    /not valid JSON/u,
  );
  const wrongSummary = JSON.parse(valid.summaryBytes);
  wrongSummary.attachment_count += 1;
  const wrongSummaryBytes = Buffer.from(JSON.stringify(wrongSummary));
  assert.throws(
    () =>
      build(TWO_ROWS, {
        expectedSummarySha256: sha256Hex(wrongSummaryBytes),
        summaryBytes: wrongSummaryBytes,
      }),
    /attachment_count mismatch/u,
  );
  const hostileAttachment = {
    ...attachment(),
    original_url:
      "http://user:secret@127.0.0.1/local--files/alpha/image%20one.png#token",
  };
  assert.throws(
    () =>
      build([
        { ...TWO_ROWS[0], attachments: [hostileAttachment] },
        TWO_ROWS[1],
      ]),
    /credentials, a fragment|host is out of scope/u,
  );
  const traversingAttachment = {
    ...attachment(),
    filename: "secret.png",
    original_url:
      "http://scp-wiki.wdfiles.com/local--files/alpha/%2e%2e/secret.png",
    wikidot_path: "/local--files/alpha/%2e%2e/secret.png",
  };
  assert.throws(
    () =>
      build([
        { ...TWO_ROWS[0], attachments: [traversingAttachment] },
        TWO_ROWS[1],
      ]),
    /unsafe segment/u,
  );
  const tampered = structuredClone(build(TWO_ROWS));
  tampered.rows[0].fullname = "changed";
  assert.throws(
    () => serializeReferenceAcquisitionInventory(tampered),
    /identity mismatch/u,
  );
  const badEnvelope = structuredClone(build(TWO_ROWS));
  badEnvelope.identity.algorithm = "md5";
  assert.throws(
    () => serializeReferenceAcquisitionInventory(badEnvelope),
    /exact stable-json/u,
  );
  const extraEnvelope = structuredClone(build(TWO_ROWS));
  extraEnvelope.identity.extra = true;
  assert.throws(
    () => serializeReferenceAcquisitionInventory(extraEnvelope),
    /exact stable-json/u,
  );
});

function cliArguments(
  manifest,
  summary,
  output,
  expectedManifestSha256,
  expectedSummarySha256,
) {
  return [
    CLI_PATH,
    "--manifest",
    manifest,
    "--summary",
    summary,
    "--output",
    output,
    "--family",
    "EN",
    "--source-origin",
    SOURCE_ORIGIN,
    "--shards",
    "64",
    "--expected-count",
    "2",
    "--expected-manifest-sha256",
    expectedManifestSha256,
    "--expected-summary-sha256",
    expectedSummarySha256,
  ];
}

test("CLI output is path-independent, atomic, and never overwrites", () => {
  const temporaryRoot = fs.mkdtempSync(
    path.join(os.tmpdir(), "reference-inventory-"),
  );
  const fixture = inputs(TWO_ROWS);
  const outputs = [];
  for (const name of ["one", "two"]) {
    const directory = path.join(temporaryRoot, name);
    fs.mkdirSync(directory);
    const manifest = path.join(directory, "input.jsonl");
    const summary = path.join(directory, "summary.json");
    const output = path.join(directory, "inventory.json");
    fs.writeFileSync(manifest, fixture.manifestBytes);
    fs.writeFileSync(summary, fixture.summaryBytes);
    const result = spawnSync(
      process.execPath,
      cliArguments(
        manifest,
        summary,
        output,
        fixture.expectedManifestSha256,
        fixture.expectedSummarySha256,
      ),
      { encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stderr);
    outputs.push(fs.readFileSync(output));
  }
  assert.deepEqual(outputs[0], outputs[1]);
  const existingOutput = path.join(temporaryRoot, "one", "inventory.json");
  const secondRun = spawnSync(
    process.execPath,
    cliArguments(
      path.join(temporaryRoot, "one", "input.jsonl"),
      path.join(temporaryRoot, "one", "summary.json"),
      existingOutput,
      fixture.expectedManifestSha256,
      fixture.expectedSummarySha256,
    ),
    { encoding: "utf8" },
  );
  assert.equal(secondRun.status, 1);
  assert.match(secondRun.stderr, /EEXIST/u);
  assert.deepEqual(fs.readFileSync(existingOutput), outputs[0]);
  const missingOutput = path.join(temporaryRoot, "failed.json");
  const failedRun = spawnSync(
    process.execPath,
    cliArguments(
      path.join(temporaryRoot, "one", "input.jsonl"),
      path.join(temporaryRoot, "one", "summary.json"),
      missingOutput,
      "0".repeat(64),
      fixture.expectedSummarySha256,
    ),
    { encoding: "utf8" },
  );
  assert.equal(failedRun.status, 1);
  assert.equal(fs.existsSync(missingOutput), false);
  assert.deepEqual(
    fs
      .readdirSync(path.dirname(existingOutput))
      .filter((name) => name.includes(".tmp")),
    [],
  );
});

test("the structural schema remains valid JSON and names the emitted contract", () => {
  const schemaPath = fileURLToPath(
    new URL(
      "../schemas/reference-acquisition-inventory.schema.json",
      import.meta.url,
    ),
  );
  const schema = JSON.parse(fs.readFileSync(schemaPath, "utf8"));
  const inventory = build(TWO_ROWS);
  assert.equal(schema.properties.schema.const, inventory.schema);
  assert.equal(schema.properties.host_paths_included.const, false);
  assert.deepEqual(Object.keys(inventory).sort(), [...schema.required].sort());
  assert.deepEqual(
    Object.keys(inventory.identity).sort(),
    [...schema.properties.identity.required].sort(),
  );
  assert.deepEqual(
    Object.keys(inventory.source_manifest).sort(),
    [...schema.properties.source_manifest.required].sort(),
  );
  assert.deepEqual(
    Object.keys(inventory.rows[0]).sort(),
    [...schema.properties.rows.items.required].sort(),
  );
  assert.deepEqual(
    Object.keys(inventory.shards[0]).sort(),
    [...schema.properties.shards.items.required].sort(),
  );
});
