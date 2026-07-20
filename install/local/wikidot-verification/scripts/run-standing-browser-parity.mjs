#!/usr/bin/env node

import process from "node:process";

import {
  parseStandingBrowserParityArgs,
  runStandingBrowserParity,
} from "../src/standing-browser-parity-runner.mjs";

function printHelp() {
  console.log(`Usage:
  run-standing-browser-parity.mjs --mode live-reference --output-dir DIR --live-completion-policy POLICY.json [--browser-root DIR] [--browser-executable PATH] [--viewport WIDTHxHEIGHT] [--timeout-ms N] [--settle-ms N]
  run-standing-browser-parity.mjs --mode candidate --output-dir DIR --live-completion-policy POLICY.json --candidate-identity IDENTITY.json --live-reference-ledger REFERENCE.json --live-reference-sha256 SHA256 [--browser-root DIR] [--browser-executable PATH] [--viewport WIDTHxHEIGHT] [--timeout-ms N] [--settle-ms N]

The live-reference mode is read-only and begins at 0.25 requests per second. Candidate mode requires an exact non-443 identity, binds local screenshots to it, and writes a receipt accepted by the standing promotion controller only when every canary passes. DOMContentLoaded capture is an immediate DOM/CSS observation, not a compositor-filmstrip assertion.`);
}

if (process.argv.includes("--help")) {
  printHelp();
  process.exit(0);
}

try {
  const result = await runStandingBrowserParity(
    parseStandingBrowserParityArgs(process.argv),
  );
  console.log(
    JSON.stringify({
      schema: "wikijump.standing_browser_parity_cli_result.v1",
      mode: result.mode,
      status: result.status,
      output_dir: result.output_dir,
    }),
  );
  process.exit(result.status === "pass" || result.status === "sealed" ? 0 : 1);
} catch (error) {
  console.error(
    JSON.stringify({
      schema: "wikijump.standing_browser_parity_cli_result.v1",
      status: "error",
      error: error?.message ?? String(error),
    }),
  );
  process.exit(1);
}
