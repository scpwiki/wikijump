#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";

import { buildListPagesLiveRegressionCoverage } from "../src/listpages-live-regression-coverage.mjs";

function parseArgs(argv) {
  const outputIndex = argv.indexOf("--output");
  if (outputIndex === -1 || !argv[outputIndex + 1]) {
    throw new Error("--output is required");
  }
  return { output: path.resolve(argv[outputIndex + 1]) };
}

export async function main(argv = process.argv) {
  const { output } = parseArgs(argv);
  const coverage = buildListPagesLiveRegressionCoverage();
  await fs.mkdir(path.dirname(output), { recursive: true });
  await fs.writeFile(output, `${JSON.stringify(coverage, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
  });
  console.log(
    JSON.stringify({
      output,
      summary: coverage.summary,
    }),
  );
  return 0;
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
