#!/usr/bin/env node

import fs from "node:fs";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {classifySyntaxDispositions} from "../src/syntax-differential-dispositions.mjs";

export function usage() {
  return "Usage: check-syntax-differential-dispositions.mjs --verdict FILE --policy FILE";
}

export function parseArgs(argv) {
  const args = {verdict: null, policy: null};
  for (let index = 0; index < argv.length; index += 1) {
    const option = argv[index];
    if (option === "--") continue;
    if (option === "--help" || option === "-h") return {help: true};
    if (!["--verdict", "--policy"].includes(option)) throw new Error(`unknown option: ${option}`);
    const value = argv[++index];
    if (!value || value.startsWith("--")) throw new Error(`${option} requires a value`);
    args[option.slice(2)] = value;
  }
  if (!args.verdict || !args.policy) throw new Error("--verdict and --policy are required");
  return args;
}

export function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const verdict = JSON.parse(fs.readFileSync(args.verdict, "utf8"));
  const policy = JSON.parse(fs.readFileSync(args.policy, "utf8"));
  const result = classifySyntaxDispositions(verdict, policy);
  console.log(JSON.stringify(result, null, 2));
  return result.status === "accepted" ? 0 : 1;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error.message);
    return 2;
  },
});
