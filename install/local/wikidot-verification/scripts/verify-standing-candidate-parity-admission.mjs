#!/usr/bin/env node

import path from "node:path";

import {runCliIfMain} from "../src/cli-entry.mjs";
import { verifyStandingCandidateParityAdmission } from "../src/standing-browser-promotion-admission.mjs";
import { sealJsonNoReplace } from "../src/standing-browser-parity-util.mjs";

const REQUIRED = Object.freeze([
  "receipt",
  "candidate-identity",
  "live-reference",
  "live-completion-policy",
  "output",
]);

export function parseArgs(argv) {
  const values = {};
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (!flag.startsWith("--")) throw new Error(`unknown argument: ${flag}`);
    const key = flag.slice(2);
    if (!REQUIRED.includes(key)) throw new Error(`unknown argument: ${flag}`);
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`);
    if (values[key]) throw new Error(`${flag} may be supplied only once`);
    values[key] = path.resolve(value);
    index += 1;
  }
  for (const key of REQUIRED) {
    if (!values[key]) throw new Error(`--${key} is required`);
  }
  return values;
}

export function usage() {
  return `Usage: verify-standing-candidate-parity-admission.mjs --receipt FILE --candidate-identity FILE --live-reference FILE --live-completion-policy FILE --output FILE

Verifies a passing candidate-parity receipt against the exact source-owned runner and observation modules, the sealed candidate identity, and the sealed live reference. It writes a no-replace source verification receipt. It does not publish port 443, start a container, or run CWG01.`;
}

export async function main(argv, {
  verifyAdmission = verifyStandingCandidateParityAdmission,
  seal = sealJsonNoReplace,
  stdout = console.log,
  stderr = console.error,
} = {}) {
  if (argv.includes("--help") || argv.includes("-h")) {
    stdout(usage());
    return 0;
  }
  try {
    const args = parseArgs(argv);
    const admission = await verifyAdmission({
      receiptPath: args.receipt,
      candidateIdentityPath: args["candidate-identity"],
      liveReferencePath: args["live-reference"],
      liveCompletionPolicyPath: args["live-completion-policy"],
    });
    const sealed = await seal(args.output, admission);
    stdout(JSON.stringify({
      schema: "wikijump.standing_candidate_parity_admission_cli_result.v1",
      status: "pass",
      output: sealed.path,
      sha256: sealed.sha256,
    }));
    return 0;
  } catch (error) {
    stderr(JSON.stringify({
      schema: "wikijump.standing_candidate_parity_admission_cli_result.v1",
      status: "error",
      error: error?.message ?? String(error),
    }));
    return 1;
  }
}

await runCliIfMain(import.meta.url, main);
