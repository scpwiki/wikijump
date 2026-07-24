#!/usr/bin/env node

import {artifactValidatorExitCode, validateArtifactDirectory} from "../src/artifact-validator.mjs";
import {runCliIfMain} from "../src/cli-entry.mjs";

export function usage() {
  return [
    "Usage: node scripts/validate-artifact.mjs <artifact-root> [options]",
    "",
    "Options:",
    "  --kind <auto|pro|codex>             Artifact schema family. Default: auto.",
    "  --expected-task-id <id>             Require this Codex result task_id.",
    "  --expected-assignment-id <id>       Require this Codex result assignment_id.",
    "  --require <relative-path>           Require an additional artifact file. Repeatable.",
    "  --help                             Show this help.",
  ].join("\n");
}

function readOptionValue(argv, index, optionName) {
  const value = argv.at(index + 1);
  if (typeof value !== "string" || value.length === 0 || value.startsWith("--")) {
    throw new Error(`${optionName} needs a value`);
  }
  return value;
}

export function parseArguments(argv) {
  const options = {
    artifactRoot: null,
    kind: "auto",
    expectedTaskId: null,
    expectedAssignmentId: null,
    requiredFiles: [],
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--help") {
      return {help: true, options};
    }
    if (argument === "--kind") {
      options.kind = readOptionValue(argv, index, "--kind");
      index += 1;
      continue;
    }
    if (argument === "--expected-task-id") {
      options.expectedTaskId = readOptionValue(argv, index, "--expected-task-id");
      index += 1;
      continue;
    }
    if (argument === "--expected-assignment-id") {
      options.expectedAssignmentId = readOptionValue(argv, index, "--expected-assignment-id");
      index += 1;
      continue;
    }
    if (argument === "--require") {
      options.requiredFiles.push(readOptionValue(argv, index, "--require"));
      index += 1;
      continue;
    }
    if (argument.startsWith("--")) {
      throw new Error(`unknown option: ${argument}`);
    }
    if (options.artifactRoot !== null) {
      throw new Error(`unexpected extra argument: ${argument}`);
    }
    options.artifactRoot = argument;
  }

  if (options.artifactRoot === null) {
    throw new Error("missing artifact root");
  }
  if (!["auto", "pro", "codex"].includes(options.kind)) {
    throw new Error(`unsupported artifact kind: ${options.kind}`);
  }
  if (options.requiredFiles.some((value) => typeof value !== "string" || value.length === 0)) {
    throw new Error("--require needs a non-empty relative path");
  }

  return {help: false, options};
}

export async function main(argv, {
  validate = validateArtifactDirectory,
  exitCode = artifactValidatorExitCode,
  stdout = console.log,
  stderr = console.error,
} = {}) {
  try {
    const {help, options} = parseArguments(argv);
    if (help) {
      stdout(usage());
      return 0;
    }
    const report = await validate(options);
    stdout(JSON.stringify(report, null, 2));
    return exitCode(report);
  } catch (error) {
    stderr(error.message);
    stderr(usage());
    return 1;
  }
}

await runCliIfMain(import.meta.url, main);
