import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";

import {
  main as runInventoryCli,
  parseArguments as parseInventoryArguments,
  usage as inventoryCliUsage,
} from "../scripts/build-reference-acquisition-inventory.mjs";
import {
  inventoryFixtureInputs,
  SOURCE_ORIGIN,
  TWO_REFERENCE_ROWS,
} from "./support/reference-acquisition-inventory-fixture.mjs";

const CLI_PATH = fileURLToPath(
  new URL("../scripts/build-reference-acquisition-inventory.mjs", import.meta.url),
);

test("reference inventory CLI validates all identity inputs before publication", async () => {
  const argv = [
    "--manifest", "manifest.jsonl",
    "--summary", "summary.json",
    "--output", "inventory.json",
    "--family", "EN",
    "--source-origin", SOURCE_ORIGIN,
    "--shards", "2",
    "--expected-count", "1",
    "--expected-manifest-sha256", "a".repeat(64),
    "--expected-summary-sha256", "b".repeat(64),
  ];
  assert.equal(parseInventoryArguments(argv).family, "EN");
  assert.deepEqual(parseInventoryArguments(["--help"]), {help: true});
  assert.match(inventoryCliUsage(), /expected-manifest-sha256/u);
  const calls = [];
  const output = [];
  const code = await runInventoryCli(argv, {
    readFile: (filePath) => Buffer.from(filePath),
    buildInventory: (options) => {
      calls.push(options);
      return {identity: {sha256: "c".repeat(64)}, rows: [{}], schema: "inventory.v1"};
    },
    serializeInventory: () => Buffer.from("inventory"),
    publish: async () => "created",
    stdout: (line) => output.push(JSON.parse(line)),
  });
  assert.equal(code, 0);
  assert.equal(calls[0].expectedCount, 1);
  assert.equal(output[0].row_count, 1);
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
    "--manifest", manifest,
    "--summary", summary,
    "--output", output,
    "--family", "EN",
    "--source-origin", SOURCE_ORIGIN,
    "--shards", "64",
    "--expected-count", "2",
    "--expected-manifest-sha256", expectedManifestSha256,
    "--expected-summary-sha256", expectedSummarySha256,
  ];
}

test("CLI output is path-independent, atomic, and never overwrites", () => {
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), "reference-inventory-"));
  const fixture = inventoryFixtureInputs(TWO_REFERENCE_ROWS);
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
      {encoding: "utf8"},
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
    {encoding: "utf8"},
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
    {encoding: "utf8"},
  );
  assert.equal(failedRun.status, 1);
  assert.equal(fs.existsSync(missingOutput), false);
  assert.deepEqual(
    fs.readdirSync(path.dirname(existingOutput)).filter((name) => name.includes(".tmp")),
    [],
  );
});
