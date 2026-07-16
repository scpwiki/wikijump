#!/usr/bin/env node
import {createRequire} from "node:module";
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";
import {defaultBrowserRoot} from "./capture-browser-rendering.mjs";
import {inventoryRows, readJson, rowsForShard} from "../src/browser-render-evidence.mjs";
import {runLocalBrowserSmoke, validateRuntimeIdentity} from "../src/local-browser-console-smoke.mjs";

const SCRIPT_PATH = fileURLToPath(import.meta.url);

function value(argv, index, flag) {
  const next = argv[index + 1];
  if (!next || next.startsWith("--")) throw new Error(`${flag} requires a value`);
  return next;
}

export function parseArgs(argv) {
  const args = {workers: 4, timeoutMs: 30_000, settleMs: 1_000, ignoreHttpsErrors: false};
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (["--inventory", "--shard-manifest", "--output", "--browser-root", "--browser-executable", "--runtime-identity"].includes(flag)) {
      const key = {"--inventory": "inventory", "--shard-manifest": "shardManifest", "--output": "output", "--browser-root": "browserRoot", "--browser-executable": "browserExecutable", "--runtime-identity": "runtimeIdentity"}[flag];
      args[key] = path.resolve(value(argv, index, flag));
      index += 1;
    } else if (flag === "--shard-id") {
      args.shardId = value(argv, index, flag);
      index += 1;
    } else if (["--workers", "--timeout-ms", "--settle-ms"].includes(flag)) {
      const raw = value(argv, index, flag);
      const parsed = Number(raw);
      if (!/^\d+$/.test(raw) || !Number.isSafeInteger(parsed) || (flag !== "--settle-ms" && parsed === 0)) throw new Error(`${flag} must be ${flag === "--settle-ms" ? "a non-negative" : "a positive"} integer`);
      args[{"--workers": "workers", "--timeout-ms": "timeoutMs", "--settle-ms": "settleMs"}[flag]] = parsed;
      index += 1;
    } else if (flag === "--ignore-https-errors") {
      args.ignoreHttpsErrors = true;
    } else if (flag === "--help") {
      args.help = true;
    } else {
      throw new Error(`unknown argument: ${flag}`);
    }
  }
  for (const required of ["inventory", "shardManifest", "shardId", "output", "runtimeIdentity"]) {
    if (!args[required] && !args.help) throw new Error(`--${required.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)} is required`);
  }
  return args;
}

function usage() {
  return "Usage: local-browser-console-smoke.mjs --inventory FILE --shard-manifest FILE --shard-id ID --output FILE.jsonl --runtime-identity FILE.json [--browser-root DIR] [--browser-executable FILE] [--workers 4] [--timeout-ms 30000] [--settle-ms 1000] [--ignore-https-errors]";
}

function loadChromium(browserRoot) {
  const requireFromRoot = createRequire(path.join(browserRoot, "package.json"));
  try {
    return requireFromRoot("playwright").chromium;
  } catch (error) {
    try {
      return requireFromRoot("@playwright/test").chromium;
    } catch (fallback) {
      throw new Error(`could not load Playwright from ${browserRoot} (${error.message}; ${fallback.message})`);
    }
  }
}

export async function main(argv) {
  const args = parseArgs(argv);
  if (args.help) {
    console.log(usage());
    return 0;
  }
  const inventory = await readJson(args.inventory);
  const manifest = await readJson(args.shardManifest);
  const runtimeIdentity = validateRuntimeIdentity(await readJson(args.runtimeIdentity));
  const shardFixtureIds = manifest.shards?.find((shard) => shard?.shard_id === args.shardId)?.fixture_ids;
  if (Array.isArray(shardFixtureIds) && new Set(shardFixtureIds).size !== shardFixtureIds.length) throw new Error(`shard ${args.shardId} contains duplicate fixture IDs`);
  const rows = rowsForShard({rows: inventoryRows(inventory), shardManifest: manifest, shardId: args.shardId});
  if (rows.length === 0) throw new Error(`shard ${args.shardId} selects no fixtures`);
  const {summary, summaryPath} = await runLocalBrowserSmoke({chromium: loadChromium(args.browserRoot ?? defaultBrowserRoot()), rows, outputPath: args.output, runtimeIdentity, inventoryPath: args.inventory, shardManifestPath: args.shardManifest, shardId: args.shardId, browserExecutable: args.browserExecutable, workers: args.workers, timeoutMs: args.timeoutMs, settleMs: args.settleMs, ignoreHttpsErrors: args.ignoreHttpsErrors});
  console.log(JSON.stringify({output: args.output, summary: summaryPath, status: summary.status, expected: summary.expected.length, observed: summary.observed.length}));
  return summary.status === "pass" ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  main(process.argv.slice(2)).then((code) => { process.exitCode = code; }).catch((error) => { console.error(error.message); process.exitCode = 1; });
}
