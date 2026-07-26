#!/usr/bin/env node

import fs from "node:fs";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {DeepwellJsonRpcClient} from "../src/theme-localization-deepwell-adapter.mjs";
import {selectSavedPageReferences} from "../src/saved-page-runtime-differential.mjs";
import {rerenderSavedPageRuntime} from "../src/saved-page-runtime-rerender.mjs";

function valueAfter(argv, index, option) {
  const value = argv[index + 1];
  if (value == null || value.startsWith("--")) throw new Error(`${option} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = {
    references: null,
    runtimeIdentity: null,
    rpcUrl: "http://127.0.0.1:12747/jsonrpc",
    caseIds: [],
    output: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--references") args.references = valueAfter(argv, index++, option);
    else if (option === "--runtime-identity") args.runtimeIdentity = valueAfter(argv, index++, option);
    else if (option === "--rpc-url") args.rpcUrl = valueAfter(argv, index++, option);
    else if (option === "--case-id") args.caseIds.push(valueAfter(argv, index++, option));
    else if (option === "--output") args.output = valueAfter(argv, index++, option);
    else throw new Error(`unknown option: ${option}`);
  }
  for (const name of ["references", "runtimeIdentity", "output"]) {
    if (!args[name]) throw new Error(`--${name.replace(/[A-Z]/gu, (letter) => `-${letter.toLowerCase()}`)} is required`);
  }
  return args;
}

export async function main(argv) {
  const args = parseArgs(argv);
  const administratorEmail = process.env.WIKIDOT_VERIFY_ADMIN_EMAIL;
  const administratorPassword = process.env.WIKIDOT_VERIFY_ADMIN_PASS;
  if (!administratorEmail || !administratorPassword) {
    throw new Error("WIKIDOT_VERIFY_ADMIN_EMAIL and WIKIDOT_VERIFY_ADMIN_PASS are required");
  }
  const references = selectSavedPageReferences(
    fs
      .readFileSync(args.references, "utf8")
      .split("\n")
      .filter((line) => line.trim())
      .map((line) => JSON.parse(line)),
    args.caseIds,
  );
  const runtimeIdentity = JSON.parse(fs.readFileSync(args.runtimeIdentity, "utf8"));
  const receipt = await rerenderSavedPageRuntime({
    references,
    runtimeIdentity,
    administratorEmail,
    administratorPassword,
    rpcClient: new DeepwellJsonRpcClient({rpcUrl: args.rpcUrl}),
  });
  fs.writeFileSync(args.output, `${JSON.stringify(receipt, null, 2)}\n`, {flag: "wx"});
  console.log(JSON.stringify({status: receipt.status, pages: receipt.pages.length}));
  return 0;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error);
    return 2;
  },
});
