#!/usr/bin/env node

import path from "node:path";
import process from "node:process";

import { verifyStandingCandidateParityAdmission } from "../src/standing-browser-promotion-admission.mjs";
import { sealJsonNoReplace } from "../src/standing-browser-parity-util.mjs";

const REQUIRED = Object.freeze([
  "receipt",
  "candidate-identity",
  "live-reference",
  "live-completion-policy",
  "output",
]);

function parseArgs(argv) {
  const values = {};
  for (let index = 2; index < argv.length; index += 1) {
    const flag = argv[index];
    if (!flag.startsWith("--")) throw new Error(`unknown argument: ${flag}`);
    const key = flag.slice(2);
    if (!REQUIRED.includes(key)) throw new Error(`unknown argument: ${flag}`);
    const value = argv[index + 1];
    if (!value || value.startsWith("--"))
      throw new Error(`${flag} requires a value`);
    if (values[key]) throw new Error(`${flag} may be supplied only once`);
    values[key] = path.resolve(value);
    index += 1;
  }
  for (const key of REQUIRED) {
    if (!values[key]) throw new Error(`--${key} is required`);
  }
  return values;
}

function printHelp() {
  console.log(`Usage: verify-standing-candidate-parity-admission.mjs --receipt FILE --candidate-identity FILE --live-reference FILE --live-completion-policy FILE --output FILE

Verifies a passing candidate-parity receipt against the exact source-owned runner and observation modules, the sealed candidate identity, and the sealed live reference. It writes a no-replace source verification receipt. It does not publish port 443, start a container, or run CWG01.`);
}

if (process.argv.includes("--help") || process.argv.includes("-h")) {
  printHelp();
  process.exit(0);
}

try {
  const args = parseArgs(process.argv);
  const admission = await verifyStandingCandidateParityAdmission({
    receiptPath: args.receipt,
    candidateIdentityPath: args["candidate-identity"],
    liveReferencePath: args["live-reference"],
    liveCompletionPolicyPath: args["live-completion-policy"],
  });
  const sealed = await sealJsonNoReplace(args.output, admission);
  console.log(
    JSON.stringify({
      schema: "wikijump.standing_candidate_parity_admission_cli_result.v1",
      status: "pass",
      output: sealed.path,
      sha256: sealed.sha256,
    }),
  );
} catch (error) {
  console.error(
    JSON.stringify({
      schema: "wikijump.standing_candidate_parity_admission_cli_result.v1",
      status: "error",
      error: error?.message ?? String(error),
    }),
  );
  process.exitCode = 1;
}
