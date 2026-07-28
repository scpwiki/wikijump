#!/usr/bin/env node

import path from "node:path";

import {
  reconcileListPagesPreviewClassification,
  writeListPagesPreviewReconciliation,
} from "../src/listpages-preview-reconciliation.mjs";

function nextValue(argv, index, option) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`missing value for ${option}`);
  }
  return value;
}

export function parseArgs(argv) {
  const args = { classification: null, output: null };
  for (let index = 2; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--classification") {
      args.classification = path.resolve(nextValue(argv, index, option));
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
  if (!args.classification) throw new Error("--classification is required");
  if (!args.output) throw new Error("--output is required");
  return args;
}

function printHelp() {
  console.log(
    "Usage: node install/local/wikidot-verification/scripts/reconcile-listpages-preview-differential.mjs --classification FILE --output FILE",
  );
}

export async function main(argv = process.argv) {
  const args = parseArgs(argv);
  if (args.help) return 0;
  const reconciliation = await reconcileListPagesPreviewClassification({
    classificationPath: args.classification,
  });
  await writeListPagesPreviewReconciliation(reconciliation, args.output);
  console.log(
    JSON.stringify({
      output: args.output,
      summary: reconciliation.summary,
    }),
  );
  return reconciliation.summary.exit_code;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main()
    .then((code) => {
      process.exitCode = code;
    })
    .catch((error) => {
      console.error(error.message);
      process.exitCode = 1;
    });
}
