#!/usr/bin/env node

import path from "node:path";

import {
  classifyListPagesPreviewDifferential,
  writeListPagesPreviewClassification,
} from "../src/listpages-preview-classification.mjs";

function nextValue(argv, index, option) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`missing value for ${option}`);
  }
  return value;
}

export function parseArgs(argv) {
  const args = { verdict: null, references: null, output: null };
  for (let index = 2; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--verdict") {
      args.verdict = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--references") {
      args.references = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--output") {
      args.output = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--help" || option === "-h") {
      return { help: true };
    } else {
      throw new Error(`unknown argument: ${option}`);
    }
  }
  if (!args.verdict) throw new Error("--verdict is required");
  if (!args.references) throw new Error("--references is required");
  if (!args.output) throw new Error("--output is required");
  return args;
}

function printHelp() {
  console.log("Usage: node install/local/wikidot-verification/scripts/classify-listpages-preview-differential.mjs --verdict FILE --references FILE --output FILE");
}

export async function main(argv = process.argv) {
  const args = parseArgs(argv);
  if (args.help) {
    printHelp();
    return;
  }
  const classification = await classifyListPagesPreviewDifferential({
    verdictPath: args.verdict,
    referencesPath: args.references,
  });
  await writeListPagesPreviewClassification(classification, args.output);
  console.log(JSON.stringify({
    output: args.output,
    summary: classification.summary,
  }));
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
