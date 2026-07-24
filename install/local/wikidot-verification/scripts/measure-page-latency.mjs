#!/usr/bin/env node
// Measure local Wikijump article response latency and response-body stability.

import process from "node:process";

import {runCliIfMain} from "../src/cli-entry.mjs";
import { parseArgs, runPageLatency, writeReport } from "../src/page-latency.mjs";

export function usage() {
  return [
    "Usage: measure-page-latency.mjs --url <url> [options]",
    "",
    "Options:",
    "  --requests <n>            measured requests, default 20",
    "  --warmups <n>             warmup requests excluded from summary, default 3",
    "  --compare-url <url>       fetch once and compare bytes/body hash against measured samples",
    "  --header <name:value>     repeatable request header, useful for local Host routing",
    "  --require-stable-body     exit nonzero if measured responses differ by SHA-256",
    "  --output <path>           write the full JSON report",
  ].join("\n");
}

export async function main(argv, {
  parse = parseArgs,
  run = runPageLatency,
  write = writeReport,
  stdout = console.log,
  stderr = console.error,
} = {}) {
  try {
    const args = parse(argv);
    if (args.help) {
      stdout(usage());
      return 0;
    }
    const report = await run(args);
    stdout(write(report, args.output));
    if (report.summary.ok !== report.summary.requests) return 2;
    if (args.requireStableBody && !report.summary.body_stable) return 3;
    if (report.summary.comparison && (!report.summary.comparison.same_body || !report.summary.comparison.same_bytes)) return 4;
    return 0;
  } catch (error) {
    stderr(error.stack ?? error.message);
    return 1;
  }
}

await runCliIfMain(import.meta.url, main, {argv: process.argv});
