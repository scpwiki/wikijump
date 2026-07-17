#!/usr/bin/env node
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";

import {runRedirectRuntimeRepro} from "../src/redirect-runtime-repro.mjs";

function nextArg(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`);
  return value;
}

export function parseArgs(argv) {
  const args = {timeoutMs: 30_000, workers: 4, ignoreHttpsErrors: false};
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (["--inventory", "--authority", "--corpus-redirects", "--runtime-identity", "--output"].includes(flag)) {
      args[flag.slice(2).replaceAll("-", "_")] = path.resolve(nextArg(argv, index, flag));
      index += 1;
    } else if (flag === "--local-base") {
      args.localBase = nextArg(argv, index, flag);
      index += 1;
    } else if (flag === "--resolved-address") {
      args.resolvedAddress = nextArg(argv, index, flag);
      index += 1;
    } else if (flag === "--site-id") {
      args.siteId = nextArg(argv, index, flag);
      index += 1;
    } else if (flag === "--timeout-ms" || flag === "--workers") {
      const raw = nextArg(argv, index, flag);
      if (!/^\d+$/u.test(raw)) throw new Error(`${flag} requires an integer`);
      args[flag === "--timeout-ms" ? "timeoutMs" : "workers"] = Number.parseInt(raw, 10);
      index += 1;
    } else if (flag === "--ignore-https-errors") {
      args.ignoreHttpsErrors = true;
    } else if (flag === "--help" || flag === "-h") {
      args.help = true;
    } else {
      throw new Error(`unknown argument: ${flag}`);
    }
  }
  if (!args.help) {
    for (const field of ["inventory", "authority", "corpus_redirects", "runtime_identity", "output", "localBase", "resolvedAddress"]) {
      if (!args[field]) throw new Error(`--${field.replaceAll("_", "-")} is required`);
    }
  }
  return args;
}

function usage() {
  return "Usage: validate-redirect-runtime.mjs --inventory FILE --authority FILE --corpus-redirects FILE --runtime-identity FILE --local-base URL --resolved-address LOOPBACK --output FILE [--site-id ID] [--workers 4] [--timeout-ms 30000] [--ignore-https-errors]";
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const verdict = await runRedirectRuntimeRepro({
    inventoryPath: args.inventory,
    authorityPath: args.authority,
    corpusRedirectsPath: args.corpus_redirects,
    runtimeIdentityPath: args.runtime_identity,
    localBase: args.localBase,
    resolvedAddress: args.resolvedAddress,
    outputPath: args.output,
    timeoutMs: args.timeoutMs,
    workers: args.workers,
    ignoreHttpsErrors: args.ignoreHttpsErrors,
    siteId: args.siteId ?? null,
  });
  console.log(JSON.stringify({status: verdict.status, expected_count: verdict.expected_count, failed_count: verdict.failed_count, output: args.output}));
  return verdict.status === "pass" ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then((code) => {
    process.exitCode = code;
  }).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
