#!/usr/bin/env node

import fs from "node:fs";
import process from "node:process";

import {runCliIfMain} from "../src/cli-entry.mjs";
import { publishBytesNoReplace } from "../src/atomic-no-replace.mjs";
import {
  buildReferenceAcquisitionInventory,
  serializeReferenceAcquisitionInventory,
} from "../src/reference-acquisition-inventory.mjs";

export function usage() {
  return 'Usage: build-reference-acquisition-inventory.mjs --manifest FILE --summary FILE --output FILE --family NAME --source-origin ORIGIN --shards N --expected-count N --expected-manifest-sha256 SHA256 --expected-summary-sha256 SHA256';
}

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

export function parseArguments(argv) {
  if (argv.length === 1 && new Set(["--help", "-h"]).has(argv[0])) return {help: true};
  const values = {};
  for (let index = 0; index < argv.length; index += 2) {
    const token = argv[index];
    const value = argv[index + 1];
    if (!token?.startsWith("--") || value === undefined || value.startsWith("--")) {
      throw new Error(`expected --option value at argument ${index + 1}`);
    }
    const name = token.slice(2);
    if (!OPTIONS.has(name)) throw new Error(`unknown option --${name}`);
    if (Object.hasOwn(values, name)) throw new Error(`duplicate option --${name}`);
    values[name] = value;
  }
  for (const option of OPTIONS) {
    if (!Object.hasOwn(values, option)) throw new Error(`missing required option --${option}`);
  }
  return values;
}

export async function main(argv, {
  readFile = fs.readFileSync,
  buildInventory = buildReferenceAcquisitionInventory,
  serializeInventory = serializeReferenceAcquisitionInventory,
  publish = publishBytesNoReplace,
  stdout = (value) => process.stdout.write(value),
  stderr = (value) => process.stderr.write(value),
} = {}) {
  try {
    const options = parseArguments(argv);
    if (options.help) {
      stdout(`${usage()}\n`);
      return 0;
    }
    const inventory = buildInventory({
      expectedCount: Number(options["expected-count"]),
      expectedManifestSha256: options["expected-manifest-sha256"],
      expectedSummarySha256: options["expected-summary-sha256"],
      family: options.family,
      manifestBytes: readFile(options.manifest),
      shardCount: Number(options.shards),
      sourceOrigin: options["source-origin"],
      summaryBytes: readFile(options.summary),
    });
    const publication = await publish(options.output, serializeInventory(inventory), { mode: 0o644 });
    if (publication === "exists") throw new Error(`EEXIST: output already exists: ${options.output}`);
    stdout(`${JSON.stringify({
      identity_sha256: inventory.identity.sha256,
      row_count: inventory.rows.length,
      schema: inventory.schema,
    })}\n`);
    return 0;
  } catch (error) {
    stderr(`${error.stack ?? error.message}\n`);
    return 1;
  }
}

await runCliIfMain(import.meta.url, main);
