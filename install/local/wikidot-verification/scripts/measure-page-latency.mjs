#!/usr/bin/env node
// Measure local Wikijump article response latency and response-body stability.
//
// Example:
//   measure-page-latency.mjs --url http://127.0.0.1/scp-173 --compare-url http://127.0.0.1/scp-173?baseline=1 --header Host:scp-wiki.wikijump.local --requests 50 --warmups 5 --require-stable-body --output latency.json

import { parseArgs, runPageLatency, writeReport } from "../src/page-latency.mjs";

function usage() {
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

async function main() {
  const args = parseArgs(process.argv);
  if (args.help) {
    console.log(usage());
    return;
  }
  const report = await runPageLatency(args);
  const json = writeReport(report, args.output);
  console.log(json);
  if (report.summary.ok !== report.summary.requests) {
    process.exit(2);
  }
  if (args.requireStableBody && !report.summary.body_stable) {
    process.exit(3);
  }
  if (report.summary.comparison && (!report.summary.comparison.same_body || !report.summary.comparison.same_bytes)) {
    process.exit(4);
  }
}

main().catch((error) => {
  console.error(error.stack ?? error.message);
  process.exit(1);
});
