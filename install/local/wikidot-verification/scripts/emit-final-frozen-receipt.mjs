#!/usr/bin/env node

// Emit a final-frozen receipt from executed evidence only: the clean source
// identity, a passing consolidated validation of exactly that source, and the
// sealed image producer output for exactly that source.

import {execFile} from "node:child_process";
import path from "node:path";
import {promisify} from "node:util";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {
  readStableRegularFile,
  sealJsonNoReplace,
} from "../src/standing-browser-parity-util.mjs";
import {
  FINAL_FROZEN_RECEIPT_SCHEMA,
  validateConsolidatedValidation,
  validateFinalFrozenReceipt,
  validateImageProducer,
} from "../src/final-frozen-receipt-contract.mjs";

const execFileAsync = promisify(execFile);

export const LOCKFILES = Object.freeze([
  "deepwell/Cargo.lock",
  "framerail/pnpm-lock.yaml",
  "install/local/wikidot-verification/pnpm-lock.yaml",
  "locales/validator/Cargo.lock",
  "wws/Cargo.lock",
]);

function parseJson(bytes, name) {
  try {
    return JSON.parse(new TextDecoder("utf-8", {fatal: true}).decode(bytes));
  } catch (error) {
    throw new Error(`${name} must contain valid UTF-8 JSON`, {cause: error});
  }
}

export async function buildFinalFrozenReceipt({
  source,
  sourceRoot,
  validationReceiptPath,
  imageProducerPath,
}) {
  const lockfiles = await Promise.all(
    LOCKFILES.map(async (relative) => {
      const absolute = path.resolve(sourceRoot, relative);
      const file = await readStableRegularFile(absolute, `lockfile ${relative}`);
      return {path: absolute, sha256: file.sha256};
    }),
  );
  const normalizedSource = {...source, lockfiles};
  const validationPath = path.resolve(validationReceiptPath);
  const validationFile = await readStableRegularFile(validationPath, "consolidated validation");
  const validation = validateConsolidatedValidation(
    parseJson(validationFile.bytes, "consolidated validation"),
    normalizedSource,
  );
  const producerPath = path.resolve(imageProducerPath);
  const producerFile = await readStableRegularFile(producerPath, "image producer output");
  const identities = validateImageProducer(
    parseJson(producerFile.bytes, "image producer output"),
    normalizedSource,
  );
  const receipt = {
    schema: FINAL_FROZEN_RECEIPT_SCHEMA,
    status: "FINAL_FROZEN",
    source: normalizedSource,
    validation: {
      path: validationPath,
      sha256: validationFile.sha256,
      base_commit: validation.base_commit,
      check_count: validation.check_count,
      groups: validation.groups,
    },
    images: {
      producer: {path: producerPath, sha256: producerFile.sha256},
      identities,
    },
  };
  validateFinalFrozenReceipt(receipt, {source: normalizedSource});
  return receipt;
}

async function git(sourceRoot, args) {
  const {stdout} = await execFileAsync("git", ["-C", sourceRoot, ...args], {
    encoding: "utf8",
    maxBuffer: 1024 * 1024,
  });
  return stdout.trim();
}

export async function sourceIdentity(sourceRoot) {
  if (await git(sourceRoot, ["status", "--porcelain=v1", "--untracked-files=all"])) {
    throw new Error("source checkout must be clean");
  }
  const lockFile = await readStableRegularFile(
    path.join(sourceRoot, "deepwell/Cargo.lock"),
    "deepwell/Cargo.lock",
  );
  const lock = new TextDecoder("utf-8", {fatal: true}).decode(lockFile.bytes);
  const matches = [
    ...lock.matchAll(
      /\[\[package\]\]\nname = "ftml"[\s\S]*?^source = "[^"\n]+#([0-9a-f]{40})"$/gmu,
    ),
  ].map((match) => match[1]);
  if (new Set(matches).size !== 1) {
    throw new Error("deepwell/Cargo.lock must contain exactly one FTML revision");
  }
  return {
    wikijump_commit: await git(sourceRoot, ["rev-parse", "HEAD"]),
    wikijump_tree: await git(sourceRoot, ["rev-parse", "HEAD^{tree}"]),
    ftml_sha: matches[0],
  };
}

export function parseArgs(argv) {
  const names = ["source-root", "validation-receipt", "image-producer", "output"];
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index];
    if (flag === "--help" || flag === "-h") return {help: true};
    const name = flag?.startsWith("--") ? flag.slice(2) : "";
    if (!names.includes(name) || Object.hasOwn(args, name)) {
      throw new Error(`unknown or duplicate option: ${flag}`);
    }
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`);
    args[name] = path.resolve(value);
  }
  for (const name of names) {
    if (!args[name]) throw new Error(`--${name} is required`);
  }
  return args;
}

export function usage() {
  return "Usage: emit-final-frozen-receipt.mjs --source-root DIR --validation-receipt FILE --image-producer FILE --output FILE";
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const receipt = await buildFinalFrozenReceipt({
    source: await sourceIdentity(args["source-root"]),
    sourceRoot: args["source-root"],
    validationReceiptPath: args["validation-receipt"],
    imageProducerPath: args["image-producer"],
  });
  const sealed = await sealJsonNoReplace(args.output, receipt);
  console.log(
    JSON.stringify({
      schema: receipt.schema,
      status: receipt.status,
      output: sealed.path,
      sha256: sealed.sha256,
    }),
  );
  return 0;
}

await runCliIfMain(import.meta.url, main);
