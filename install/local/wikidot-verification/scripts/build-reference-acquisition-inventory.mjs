#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

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

function publishNoOverwrite(outputPath, contents) {
  const absoluteOutput = path.resolve(outputPath);
  const directory = path.dirname(absoluteOutput);
  const temporaryPath = path.join(
    directory,
    `.${path.basename(absoluteOutput)}.${process.pid}.${crypto.randomUUID()}.tmp`,
  );
  let descriptor;
  try {
    descriptor = fs.openSync(temporaryPath, "wx", 0o644);
    fs.writeFileSync(descriptor, contents);
    fs.fsyncSync(descriptor);
    fs.closeSync(descriptor);
    descriptor = undefined;
    fs.linkSync(temporaryPath, absoluteOutput);
    fs.unlinkSync(temporaryPath);
    if (process.platform !== "win32") {
      const directoryDescriptor = fs.openSync(directory, "r");
      try {
        fs.fsyncSync(directoryDescriptor);
      } finally {
        fs.closeSync(directoryDescriptor);
      }
    }
  } finally {
    if (descriptor !== undefined) {
      fs.closeSync(descriptor);
    }
    try {
      fs.unlinkSync(temporaryPath);
    } catch (error) {
      if (error.code !== "ENOENT") {
        throw error;
      }
    }
  }
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
  publishNoOverwrite(
    options.output,
    serializeReferenceAcquisitionInventory(inventory),
  );
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
