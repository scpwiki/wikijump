#!/usr/bin/env node

import fs from "node:fs";

import { publishBytesNoReplace } from "../src/atomic-no-replace.mjs";
import {
  buildReferenceAcquisitionInventory,
  serializeReferenceAcquisitionInventory,
} from "../src/reference-acquisition-inventory.mjs";

const OPTIONS = new Set([
  "manifest",
  "summary",
  "output",
  "family",
  "source-origin",
  "shards",
  "expected-count",
  "expected-manifest-sha256",
  "expected-summary-sha256",
]);

function parseArguments(argv) {
  const values = {};
  for (let index = 0; index < argv.length; index += 2) {
    const token = argv[index];
    const value = argv[index + 1];
    if (
      !token?.startsWith("--") ||
      value === undefined ||
      value.startsWith("--")
    ) {
      throw new Error(`expected --option value at argument ${index + 1}`);
    }
    const name = token.slice(2);
    if (!OPTIONS.has(name)) {
      throw new Error(`unknown option --${name}`);
    }
    if (Object.hasOwn(values, name)) {
      throw new Error(`duplicate option --${name}`);
    }
    values[name] = value;
  }
  for (const option of OPTIONS) {
    if (!Object.hasOwn(values, option)) {
      throw new Error(`missing required option --${option}`);
    }
  }
  return values;
}

try {
  const options = parseArguments(process.argv.slice(2));
  const inventory = buildReferenceAcquisitionInventory({
    expectedCount: Number(options["expected-count"]),
    expectedManifestSha256: options["expected-manifest-sha256"],
    expectedSummarySha256: options["expected-summary-sha256"],
    family: options.family,
    manifestBytes: fs.readFileSync(options.manifest),
    shardCount: Number(options.shards),
    sourceOrigin: options["source-origin"],
    summaryBytes: fs.readFileSync(options.summary),
  });
  const publication = await publishBytesNoReplace(
    options.output,
    serializeReferenceAcquisitionInventory(inventory),
    { mode: 0o644 },
  );
  if (publication === "exists") {
    throw new Error(`EEXIST: output already exists: ${options.output}`);
  }
  process.stdout.write(
    `${JSON.stringify({
      identity_sha256: inventory.identity.sha256,
      row_count: inventory.rows.length,
      schema: inventory.schema,
    })}\n`,
  );
} catch (error) {
  process.stderr.write(`${error.stack ?? error.message}\n`);
  process.exitCode = 1;
}
