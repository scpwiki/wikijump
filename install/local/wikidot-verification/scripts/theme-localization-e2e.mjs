#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";

import {
  ALLOWED_SITE_SLUG,
  DEFAULT_WIKIDOT_ORIGIN,
  DEFAULT_WIKIJUMP_ORIGIN,
  buildThemeLocalizationE2EPlan,
} from "../src/theme-localization-e2e.mjs";

const SCRIPT_PATH = fileURLToPath(import.meta.url);

function nextArg(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`);
  return value;
}

export function parseArgs(argv) {
  const args = {
    siteSlug: ALLOWED_SITE_SLUG,
    wikidotOrigin: DEFAULT_WIKIDOT_ORIGIN,
    wikijumpOrigin: DEFAULT_WIKIJUMP_ORIGIN,
    tiers: [],
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--translation-root") {
      args.translationRoot = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--run-id") {
      args.runId = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--site") {
      args.siteSlug = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--wikidot-origin") {
      args.wikidotOrigin = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--wikijump-origin") {
      args.wikijumpOrigin = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--tier") {
      args.tiers.push(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--output") {
      args.output = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--dry-run") {
      args.dryRun = true;
    } else if (arg === "--execute") {
      throw new Error("--execute is intentionally disabled until guarded mutation and finally-cleanup adapters are implemented");
    } else if (arg === "--json") {
      args.jsonOnly = true;
    } else if (arg === "--help" || arg === "-h") {
      printHelpAndExit();
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.translationRoot) throw new Error("--translation-root is required");
  if (!args.runId) throw new Error("--run-id is required");
  if (!args.output) throw new Error("--output is required");
  if (!args.dryRun) throw new Error("--dry-run is required; live page mutation is not implemented by this runner");
  if (args.tiers.length === 0) args.tiers.push("all");
  return args;
}

function printHelpAndExit() {
  console.log(`Usage: node install/local/wikidot-verification/scripts/theme-localization-e2e.mjs --dry-run --translation-root PATH --run-id ID --output FILE [--tier yossistyle|ashes-to-ashes|basalt|all ...] [--site ${ALLOWED_SITE_SLUG}] [--wikidot-origin ${DEFAULT_WIKIDOT_ORIGIN}] [--wikijump-origin ${DEFAULT_WIKIJUMP_ORIGIN}] [--json]

Builds a deterministic, mutation-free theme localization E2E plan. The runner rejects every site outside the dedicated scpaiueouiuiui sandbox, validates run-owned slugs and accepted translation artifacts, and emits capture and finally-cleanup contracts. Live execution is deliberately unavailable until both target adapters implement creation ledgers and verified cleanup.`);
  process.exit(0);
}

export async function run(argv = process.argv) {
  const args = parseArgs(argv);
  const plan = await buildThemeLocalizationE2EPlan(args);
  await fs.mkdir(path.dirname(args.output), {recursive: true});
  await fs.writeFile(args.output, `${JSON.stringify(plan, null, 2)}\n`, "utf8");
  const summary = {output: args.output, mode: plan.mode, preflight: plan.preflight, page_mutations_performed: 0};
  if (args.jsonOnly) console.log(JSON.stringify(summary));
  else console.log(`wrote mutation-free theme localization E2E plan to ${args.output} (${plan.preflight.status})`);
  return plan.preflight.status === "pass" ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  run().then((code) => {
    process.exitCode = code;
  }).catch((error) => {
    console.error(error.message);
    process.exitCode = 2;
  });
}
