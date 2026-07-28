#!/usr/bin/env node

import path from "node:path";

import {
  buildListPagesMatrix,
  writeListPagesMatrix,
} from "../src/listpages-campaign-matrix.mjs";

function nextValue(argv, index, option) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`missing value for ${option}`);
  return value;
}

export function parseArgs(argv) {
  const args = { inventoryDir: null, outputDir: null };
  for (let index = 2; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--inventory-dir") {
      args.inventoryDir = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--output-dir") {
      args.outputDir = path.resolve(nextValue(argv, index, option));
      index += 1;
    } else if (option === "--help" || option === "-h") {
      return { help: true };
    } else {
      throw new Error(`unknown argument: ${option}`);
    }
  }
  if (!args.inventoryDir) throw new Error("--inventory-dir is required");
  if (!args.outputDir) throw new Error("--output-dir is required");
  return args;
}

function printHelp() {
  console.log("Usage: node install/local/wikidot-verification/scripts/build-listpages-campaign-matrix.mjs --inventory-dir DIR --output-dir DIR");
}

export async function main(argv = process.argv) {
  const args = parseArgs(argv);
  if (args.help) {
    printHelp();
    return;
  }
  const matrix = await buildListPagesMatrix({ inventoryDir: args.inventoryDir });
  await writeListPagesMatrix(matrix, args.outputDir);
  console.log(JSON.stringify({
    output_dir: args.outputDir,
    summary: matrix.summary,
  }));
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
