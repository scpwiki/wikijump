#!/usr/bin/env node

import path from "node:path";

import {
  reconcileListPagesHashMagicAudit,
  writeListPagesHashMagicReconciliation,
} from "../src/listpages-hash-magic-audit.mjs";

function nextValue(argv, index, option) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`missing value for ${option}`);
  }
  return value;
}

export function parseArgs(argv) {
  const args = { audit: null, matrixCases: null, output: null };
  for (let index = 2; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--audit") {
      args.audit = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--matrix-cases") {
      args.matrixCases = path.resolve(nextValue(argv, index, option));
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
  if (!args.audit) throw new Error("--audit is required");
  if (!args.matrixCases) throw new Error("--matrix-cases is required");
  if (!args.output) throw new Error("--output is required");
  return args;
}

function printHelp() {
  console.log(
    "Usage: node install/local/wikidot-verification/scripts/reconcile-listpages-hash-magic-audit.mjs --audit FILE --matrix-cases FILE --output FILE",
  );
}

export async function main(argv = process.argv) {
  const args = parseArgs(argv);
  if (args.help) return 0;
  const reconciliation = await reconcileListPagesHashMagicAudit({
    auditPath: args.audit,
    matrixCasesPath: args.matrixCases,
  });
  await writeListPagesHashMagicReconciliation(reconciliation, args.output);
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
