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
import {executeGuardedThemeAction, GUARDED_THEME_WIKIJUMP_RPC_URL, recoverGuardedThemeAction, validateThemeCdpEndpoint, writeExecutableThemePlan} from "../src/theme-localization-runner.mjs";

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
      args.mode = args.mode ? "conflict" : "dry-run";
    } else if (arg === "--execute") {
      args.mode = args.mode ? "conflict" : "execute";
    } else if (arg === "--recover") {
      args.mode = args.mode ? "conflict" : "recover";
    } else if (arg === "--plan") {
      args.plan = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--ledger") {
      args.ledgerPath = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--result") {
      args.resultPath = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--artifact-dir") {
      args.artifactDir = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--browser-root") {
      args.browserRoot = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--browser-executable") {
      args.browserExecutable = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--cdp-endpoint") {
      args.cdpEndpoint = nextArg(argv, index, arg);
      index += 1;
    } else if (arg === "--wikidot-storage-state") {
      args.wikidotStorageState = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--wikijump-storage-state") {
      args.wikijumpStorageState = path.resolve(nextArg(argv, index, arg));
      index += 1;
    } else if (arg === "--ignore-https-errors") {
      args.ignoreHttpsErrors = true;
    } else if (arg === "--json") {
      args.jsonOnly = true;
    } else if (arg === "--help" || arg === "-h") {
      return {...args, help: true};
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!args.mode || args.mode === "conflict") throw new Error("exactly one of --dry-run, --execute, or --recover is required");
  if (args.cdpEndpoint && args.browserExecutable) throw new Error("--cdp-endpoint cannot be combined with --browser-executable");
  if (args.cdpEndpoint) args.cdpEndpoint = validateThemeCdpEndpoint(args.cdpEndpoint);
  if (args.mode === "recover") {
    if (!args.plan || !args.ledgerPath || !args.resultPath) throw new Error("--recover requires --plan, --ledger, and --result");
    if (args.translationRoot || args.runId || args.output) throw new Error("--recover reads its immutable plan from --plan");
    return args;
  }
  if (!args.translationRoot) throw new Error("--translation-root is required");
  if (!args.runId) throw new Error("--run-id is required");
  if (!args.output) throw new Error("--output is required");
  if (args.mode === "execute" && (!args.ledgerPath || !args.resultPath || !args.artifactDir)) throw new Error("--execute requires --ledger, --result, and --artifact-dir");
  if (args.tiers.length === 0) args.tiers.push("all");
  return args;
}

function printHelp() {
  console.log(`Usage:
  node install/local/wikidot-verification/scripts/theme-localization-e2e.mjs --dry-run --translation-root PATH --run-id ID --output PLAN [--tier yossistyle|ashes-to-ashes|basalt|all ...] [--json]
  node install/local/wikidot-verification/scripts/theme-localization-e2e.mjs --execute --translation-root PATH --run-id ID --output PLAN --ledger FILE --result FILE --artifact-dir DIR [browser options] [--json]
  node install/local/wikidot-verification/scripts/theme-localization-e2e.mjs --recover --plan PLAN --ledger FILE --result FILE [--json]

New plans reserve codex-l10n:<run-id>-<tier>; recovery also accepts exact legacy theme:codex-l10n-<run-id>-<tier> plans whose ledger fingerprint and resources match.

The exact site allowlist is ${ALLOWED_SITE_SLUG}. Execute and recover require WIKIJUMP_THEME_RPC_URL=${GUARDED_THEME_WIKIJUMP_RPC_URL}. Credentials are accepted only through WIKIDOT_USERNAME, WIKIDOT_PASSWORD, WIKIJUMP_THEME_ADMIN_EMAIL, and WIKIJUMP_THEME_ADMIN_PASSWORD. Optional browser flags are --browser-root, --browser-executable or --cdp-endpoint, --wikidot-storage-state, --wikijump-storage-state, and --ignore-https-errors.`);
}

export async function run(argv = process.argv) {
  const args = parseArgs(argv);
  if (args.help) {
    printHelp();
    return 0;
  }
  const plan = args.mode === "recover" ? JSON.parse(await fs.readFile(args.plan, "utf8")) : await buildThemeLocalizationE2EPlan(args);
  if (args.mode !== "recover") {
    if (args.mode === "execute") await writeExecutableThemePlan(args.output, plan);
    else {
      await fs.mkdir(path.dirname(args.output), {recursive: true});
      await fs.writeFile(args.output, `${JSON.stringify(plan, null, 2)}\n`, "utf8");
    }
  }
  if (args.mode !== "dry-run" && plan.preflight?.status === "pass") {
    const runAction = args.mode === "recover" ? recoverGuardedThemeAction : executeGuardedThemeAction;
    await runAction({
      plan, ledgerPath: args.ledgerPath, resultPath: args.resultPath, artifactDir: args.artifactDir,
      dependencyOptions: {browserRoot: args.browserRoot, browserExecutable: args.browserExecutable, cdpEndpoint: args.cdpEndpoint, wikidotStorageState: args.wikidotStorageState, wikijumpStorageState: args.wikijumpStorageState, ignoreHttpsErrors: args.ignoreHttpsErrors},
    });
  }
  const summary = args.mode === "dry-run" ? {output: args.output, mode: plan.mode, preflight: plan.preflight, page_mutations_performed: 0} : {mode: args.mode, preflight: plan.preflight, result: args.resultPath};
  if (args.jsonOnly) console.log(JSON.stringify(summary));
  else if (args.mode === "dry-run") console.log(`wrote mutation-free theme localization E2E plan to ${args.output} (${plan.preflight.status})`);
  else console.log(`theme localization ${args.mode} completed (${plan.preflight.status})`);
  return plan.preflight.status === "pass" ? 0 : 1;
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  run().then((code) => {
    process.exitCode = code;
  }).catch((error) => {
    console.error(error.message);
    process.exitCode = error.signal === "SIGINT" ? 130 : error.signal === "SIGTERM" ? 143 : 2;
  });
}
