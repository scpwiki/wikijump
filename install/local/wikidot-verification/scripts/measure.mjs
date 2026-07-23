#!/usr/bin/env node
import {runMeasuredCommand} from "../src/command-ledger.mjs";
import {parsePositiveIntegerOption as parsePositiveInteger, readRequiredOptionValue as readValue, UsageError} from "../src/cli-options.mjs";

const SAFE_FAMILY_RE = /^[A-Za-z0-9_.:-]+$/;

function usage() {
  return [
    "Usage: node scripts/measure.mjs --family <name> [--label <label>] [--ledger <path>] [--timeout <ms>] [--quiet] -- <command> [args...]",
    "",
    "Wrap validation/import/capture commands with this to build the timing ledger; the wrapper never changes the wrapped command's outcome.",
    "",
    "Known families: cargo, framerail, node-mjs, import, capture, db, ci-poll, health. Free-form family values are allowed if they match /^[A-Za-z0-9_.:-]+$/.",
  ].join("\n");
}

function parseArgs(argv) {
  const separator = argv.indexOf("--");
  const optionArgv = separator === -1 ? argv : argv.slice(0, separator);

  const options = {
    family: null,
    label: null,
    ledgerPath: null,
    timeoutMs: null,
    quiet: false,
    command: null,
    args: [],
  };
  let help = false;

  for (let index = 0; index < optionArgv.length; index += 1) {
    const arg = optionArgv[index];
    if (arg === "--help" || arg === "-h") {
      help = true;
    } else if (arg === "--family") {
      options.family = readValue(optionArgv, index, "--family");
      index += 1;
    } else if (arg === "--label") {
      options.label = readValue(optionArgv, index, "--label");
      index += 1;
    } else if (arg === "--ledger") {
      options.ledgerPath = readValue(optionArgv, index, "--ledger");
      index += 1;
    } else if (arg === "--timeout") {
      options.timeoutMs = parsePositiveInteger(readValue(optionArgv, index, "--timeout"), "--timeout");
      index += 1;
    } else if (arg === "--quiet") {
      options.quiet = true;
    } else {
      throw new UsageError(`unknown argument: ${arg}`);
    }
  }

  if (help) {
    return {help: true};
  }
  if (separator === -1) {
    throw new UsageError("missing -- separator");
  }
  if (options.family === null) {
    throw new UsageError("missing --family");
  }
  if (!SAFE_FAMILY_RE.test(options.family)) {
    throw new UsageError("--family must match /^[A-Za-z0-9_.:-]+$/");
  }

  const commandArgv = argv.slice(separator + 1);
  if (commandArgv.length === 0) {
    throw new UsageError("missing command after --");
  }
  options.command = commandArgv[0];
  options.args = commandArgv.slice(1);
  return {help: false, options};
}

// Set process.exitCode instead of calling process.exit() so pending async
// writes to piped stdout/stderr flush before the process exits naturally.
try {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    console.log(usage());
  } else {
    const result = await runMeasuredCommand({
      ...parsed.options,
      cwd: process.cwd(),
    });
    process.exitCode = result.exitCode;
  }
} catch (error) {
  if (error instanceof UsageError) {
    console.error(error.message);
    console.error(usage());
    process.exitCode = 2;
  } else {
    console.error(error.stack ?? error.message);
    process.exitCode = 1;
  }
}
