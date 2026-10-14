#!/usr/bin/env node

// Run the final preflight barrier on one exact clean commit and seal what ran.
//
// The receipt lists every selected check group and every check with its
// result, the tool versions, and the base the change set was classified
// against. A failing run still seals a receipt (status "fail"), so the
// missing or failing verification is identified by name rather than by
// reading a log.

import {execFile as execFileCallback, spawn} from "node:child_process";
import {createHash} from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import {promisify} from "node:util";

import {runCliIfMain} from "../src/cli-entry.mjs";
import {sealJsonNoReplace} from "../src/standing-browser-parity-util.mjs";
import {CONSOLIDATED_VALIDATION_SCHEMA} from "../src/final-frozen-receipt-contract.mjs";

const execFile = promisify(execFileCallback);
const SOURCE_FILES = Object.freeze([
  "scripts/preflight.sh",
  "scripts/run-test-no-external-network.sh",
  "install/local/wikidot-verification/scripts/run-deepwell-integration-validation.mjs",
]);
const TOOLS = Object.freeze({
  node: ["node", ["--version"]],
  pnpm: ["pnpm", ["--version"]],
  cargo: ["cargo", ["--version"]],
  rustc: ["rustc", ["--version"]],
});

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (!["--repository", "--base", "--output", "--log"].includes(flag) || !value || value.startsWith("--")) {
      fail(`invalid option: ${flag ?? "<missing>"}`);
    }
    const key = flag.slice(2);
    if (Object.hasOwn(args, key)) fail(`duplicate option: ${flag}`);
    args[key] = value;
  }
  for (const key of ["repository", "base", "output"]) if (!args[key]) fail(`--${key} is required`);
  return {
    repository: path.resolve(args.repository),
    base: args.base,
    output: path.resolve(args.output),
    log: args.log ? path.resolve(args.log) : `${path.resolve(args.output)}.log`,
  };
}

async function git(repository, ...args) {
  const {stdout} = await execFile("git", ["-C", repository, ...args], {encoding: "utf8"});
  return stdout.trim();
}

async function sha256File(file) {
  return createHash("sha256").update(await fs.readFile(file)).digest("hex");
}

/** Selected groups and per-check results from preflight's own output. */
export function parsePreflightOutput(output) {
  const groups = {};
  const groupLine = /^preflight: groups: (.*)$/mu.exec(output);
  for (const pair of groupLine?.[1].trim().split(/\s+/u) ?? []) {
    const [name, value] = pair.split("=");
    if (name) groups[name] = value === "true";
  }
  const checks = [...output.matchAll(/^--- (.+): (ok|FAILED)$/gmu)].map((match) => ({
    name: match[1],
    status: match[2] === "ok" ? "ok" : "failed",
  }));
  return {groups, checks};
}

async function toolVersions() {
  const versions = {};
  for (const [name, [command, args]] of Object.entries(TOOLS)) {
    try {
      versions[name] = (await execFile(command, args, {encoding: "utf8"})).stdout.trim();
    } catch {
      versions[name] = "unavailable";
    }
  }
  return versions;
}

async function runPreflight(repository, base, logPath) {
  const log = await fs.open(logPath, "wx", 0o600);
  let output = "";
  try {
    const code = await new Promise((resolve, reject) => {
      const child = spawn("bash", ["scripts/preflight.sh", "--base", base, "--final"], {
        cwd: repository,
        env: process.env,
        stdio: ["ignore", "pipe", "pipe"],
      });
      const record = (chunk) => {
        output += chunk.toString("utf8");
        process.stdout.write(chunk);
        log.write(chunk);
      };
      child.stdout.on("data", record);
      child.stderr.on("data", record);
      child.on("error", reject);
      child.on("exit", (status, signal) => resolve(signal ? `signal ${signal}` : status));
    });
    return {code, output};
  } finally {
    await log.close();
  }
}

export async function runConsolidatedValidation({repository, base, output, log}) {
  const status = await git(repository, "status", "--porcelain=v1", "--untracked-files=all");
  if (status !== "") fail("consolidated validation requires a clean checkout");
  const commit = await git(repository, "rev-parse", "HEAD^{commit}");
  const tree = await git(repository, "rev-parse", "HEAD^{tree}");
  const baseCommit = await git(repository, "rev-parse", `${base}^{commit}`);
  const startedAt = new Date().toISOString();
  const run = await runPreflight(repository, baseCommit, log);
  const {groups, checks} = parsePreflightOutput(run.output);
  const source_sha256 = {};
  for (const relative of SOURCE_FILES) {
    source_sha256[relative] = await sha256File(path.join(repository, relative));
  }
  const passed = run.code === 0 && checks.length > 0 && checks.every((check) => check.status === "ok");
  const receipt = {
    schema: CONSOLIDATED_VALIDATION_SCHEMA,
    status: passed ? "pass" : "fail",
    started_at: startedAt,
    completed_at: new Date().toISOString(),
    wikijump_commit: commit,
    wikijump_tree: tree,
    base_commit: baseCommit,
    mode: "final",
    groups,
    checks,
    tool_versions: await toolVersions(),
    preflight: {
      exit: run.code,
      log: {path: log, sha256: await sha256File(log)},
      source_sha256,
    },
  };
  const sealed = await sealJsonNoReplace(output, receipt);
  return {receipt, sealed};
}

export async function main(argv, {stdout = console.log} = {}) {
  const args = parseArgs(argv);
  const {receipt, sealed} = await runConsolidatedValidation(args);
  stdout(JSON.stringify({
    schema: receipt.schema,
    status: receipt.status,
    failed: receipt.checks.filter((check) => check.status !== "ok").map((check) => check.name),
    output: sealed.path,
    sha256: sealed.sha256,
  }));
  return receipt.status === "pass" ? 0 : 1;
}

await runCliIfMain(import.meta.url, main, {
  onError: (error) => {
    console.error(error?.stack ?? error?.message ?? String(error));
    return 1;
  },
});
