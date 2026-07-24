#!/usr/bin/env node

import process from "node:process";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {
  parseStandingBrowserParityArgs,
  runStandingBrowserParity,
} from "../src/standing-browser-parity-runner.mjs";

export function usage() {
  return `Usage:
  run-standing-browser-parity.mjs --mode live-reference --output-dir DIR --live-completion-policy POLICY.json [--browser-root DIR] [--browser-executable PATH] [--viewport WIDTHxHEIGHT] [--timeout-ms N] [--settle-ms N]
  run-standing-browser-parity.mjs --mode candidate --output-dir DIR --live-completion-policy POLICY.json --candidate-identity IDENTITY.json --live-reference-ledger REFERENCE.json --live-reference-sha256 SHA256 [--browser-root DIR] [--browser-executable PATH] [--viewport WIDTHxHEIGHT] [--timeout-ms N] [--settle-ms N]

The live-reference mode is read-only and begins at 0.25 requests per second. Candidate mode requires an exact non-443 identity, binds local screenshots to it, and writes a receipt accepted by the standing promotion controller only when every canary passes. DOMContentLoaded capture is an immediate DOM/CSS observation, not a compositor-filmstrip assertion.`;
}

export async function main(argv, {
  parseArgs = parseStandingBrowserParityArgs,
  runParity = runStandingBrowserParity,
  stdout = console.log,
  stderr = console.error,
} = {}) {
  if (argv.includes("--help") || argv.includes("-h")) {
    stdout(usage());
    return 0;
  }
  try {
    const result = await runParity(parseArgs(argv));
    stdout(JSON.stringify({
      schema: "wikijump.standing_browser_parity_cli_result.v1",
      mode: result.mode,
      status: result.status,
      output_dir: result.output_dir,
    }));
    return result.status === "pass" || result.status === "sealed" ? 0 : 1;
  } catch (error) {
    stderr(JSON.stringify({
      schema: "wikijump.standing_browser_parity_cli_result.v1",
      status: "error",
      error: error?.message ?? String(error),
    }));
    return 1;
  }
}

await runCliIfMain(import.meta.url, main, {argv: process.argv});
