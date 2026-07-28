#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";

import {
  classifyListPagesLiveFixtures,
} from "../src/listpages-live-fixture-classification.mjs";

function parseArgs(argv) {
  const capturesIndex = argv.indexOf("--captures");
  const outputIndex = argv.indexOf("--output");
  const planIndex = argv.indexOf("--plan");
  if (capturesIndex === -1 || !argv[capturesIndex + 1]) {
    throw new Error("--captures is required");
  }
  if (outputIndex === -1 || !argv[outputIndex + 1]) {
    throw new Error("--output is required");
  }
  return {
    captures: path.resolve(argv[capturesIndex + 1]),
    output: path.resolve(argv[outputIndex + 1]),
    plan: planIndex === -1 ? null : path.resolve(argv[planIndex + 1]),
  };
}

async function readJsonl(file) {
  const text = await fs.readFile(file, "utf8");
  return text.split(/\n/u).filter(Boolean).map((line) => JSON.parse(line));
}

export async function main(argv = process.argv) {
  const args = parseArgs(argv);
  const captures = await readJsonl(args.captures);
  const plan = args.plan == null
    ? null
    : JSON.parse(await fs.readFile(args.plan, "utf8"));
  const classification = classifyListPagesLiveFixtures(captures, plan);
  await fs.mkdir(path.dirname(args.output), { recursive: true });
  await fs.writeFile(args.output, `${JSON.stringify(classification, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
  });
  console.log(JSON.stringify({
    output: args.output,
    captures: classification.summary.captures,
    blocks: classification.summary.blocks,
  }));
  return 0;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().then((code) => {
    process.exitCode = code;
  }).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
