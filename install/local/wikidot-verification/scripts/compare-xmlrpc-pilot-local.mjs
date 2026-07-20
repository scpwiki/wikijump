#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { runXmlrpcPilotLocalComparison } from "../src/xmlrpc-pilot-local-comparison.mjs";

const REQUIRED_OPTIONS = Object.freeze([
  "output-dir",
  "pilot-root",
  "rpc-url",
  "runtime-identity",
]);

function usage() {
  return "Usage: compare-xmlrpc-pilot-local.mjs --pilot-root ABSOLUTE_DIR --runtime-identity ABSOLUTE_FILE --rpc-url http://127.0.0.1:PORT/jsonrpc --output-dir ABSOLUTE_DIR [--timeout-ms 30000]";
}

function positiveTimeout(value) {
  if (!/^[1-9][0-9]*$/u.test(value ?? "")) {
    throw new Error("--timeout-ms must be a positive integer");
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed > 120_000) {
    throw new Error("--timeout-ms must be an integer through 120000");
  }
  return parsed;
}

export function parseArgs(argv) {
  if (argv.length === 1 && ["--help", "-h"].includes(argv[0])) {
    return Object.freeze({ help: true });
  }
  const values = {};
  for (let index = 0; index < argv.length; index += 2) {
    const token = argv[index];
    const value = argv[index + 1];
    if (
      !token?.startsWith("--") ||
      value === undefined ||
      value.startsWith("--")
    ) {
      throw new Error(`expected --option value at argument ${index + 1}`);
    }
    const option = token.slice(2);
    if (![...REQUIRED_OPTIONS, "timeout-ms"].includes(option)) {
      throw new Error(`unknown option --${option}`);
    }
    if (Object.hasOwn(values, option)) {
      throw new Error(`duplicate option --${option}`);
    }
    values[option] = value;
  }
  for (const option of REQUIRED_OPTIONS) {
    if (!Object.hasOwn(values, option)) {
      throw new Error(`missing required option --${option}`);
    }
  }
  for (const option of REQUIRED_OPTIONS) {
    if (!path.isAbsolute(values[option]) && option !== "rpc-url") {
      throw new Error(`--${option} must be an absolute path`);
    }
  }
  return Object.freeze({
    help: false,
    outputDir: values["output-dir"],
    pilotRoot: values["pilot-root"],
    rpcUrl: values["rpc-url"],
    runtimeIdentityPath: values["runtime-identity"],
    timeoutMs:
      values["timeout-ms"] === undefined
        ? 30_000
        : positiveTimeout(values["timeout-ms"]),
  });
}

export async function main(argv, { stdout = process.stdout } = {}) {
  const args = parseArgs(argv);
  if (args.help) {
    stdout.write(`${usage()}\n`);
    return 0;
  }
  const result = await runXmlrpcPilotLocalComparison(args);
  stdout.write(
    `${JSON.stringify({
      artifact_key: result.verdict.artifact_key,
      gate_status: result.verdict.gate.status,
      output: result.output,
    })}\n`,
  );
  return result.exit_code;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(
    (exitCode) => {
      process.exitCode = exitCode;
    },
    (error) => {
      process.stderr.write(`${error.stack ?? error.message}\n${usage()}\n`);
      process.exitCode = 2;
    },
  );
}
