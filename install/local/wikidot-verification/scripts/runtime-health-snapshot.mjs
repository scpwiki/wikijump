#!/usr/bin/env node

import {runCliIfMain} from "../src/cli-entry.mjs";
import {collectRuntimeHealthSnapshot, defaultSnapshotPath} from "../src/runtime-health-snapshot.mjs";
import {parsePositiveIntegerOption as parsePositiveInteger, readRequiredOptionValue as readValue, UsageError} from "../src/cli-options.mjs";

export function usage() {
  return [
    "Usage: node scripts/runtime-health-snapshot.mjs [--ttl <ms>] [--refresh] [--json] [--quiet] [--snapshot <path>] [--project <name>] [--db-container <name>] [--fail-on-degraded] [--help]",
    "",
    "Build or serve a cached runtime health snapshot for agents, covering local containers, runtime URLs, database reachability, and disk health.",
    "",
    "The default snapshot path is:",
    `  ${defaultSnapshotPath()}`,
    "",
    "Options:",
    "  --ttl <ms>            Positive cache TTL in milliseconds. Default: 30000.",
    "  --refresh             Bypass cache reads, run live probes, and write a new snapshot.",
    "  --json                No-op alias; JSON is already the default output.",
    "  --quiet               Print nothing to stdout while still writing or updating the snapshot.",
    "  --snapshot <path>     Override the snapshot file path.",
    "  --project <name>      Override the compose project name.",
    "  --db-container <name> Override the database container name.",
    "  --fail-on-degraded    Exit 3 when the produced or served snapshot status is degraded.",
    "  --help                Print this help.",
    "",
    "cacheStatus meanings:",
    "  hit      A valid fresh snapshot was served from disk with no live probes.",
    "  miss     No usable snapshot was present, so live probes ran.",
    "  stale    A valid snapshot existed but its TTL expired, so live probes ran.",
    "  refresh  --refresh forced live probes.",
    "",
    "Exit codes:",
    "  0  Snapshot produced or served.",
    "  1  Tool failure prevented producing or serving a snapshot.",
    "  2  Usage error.",
    "  3  Snapshot status is degraded and --fail-on-degraded was set.",
    "",
    "No credentials are ever read or printed. Database readiness is checked inside the container without importing secret environment values into this process.",
  ].join("\n");
}

export function parseArgs(argv) {
  const options = {
    ttlMs: 30000,
    refresh: false,
    quiet: false,
    snapshotPath: null,
    project: null,
    dbContainer: null,
    failOnDegraded: false,
  };
  let help = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      help = true;
    } else if (arg === "--ttl") {
      options.ttlMs = parsePositiveInteger(readValue(argv, index, "--ttl"), "--ttl");
      index += 1;
    } else if (arg === "--refresh") {
      options.refresh = true;
    } else if (arg === "--json") {
      continue;
    } else if (arg === "--quiet") {
      options.quiet = true;
    } else if (arg === "--snapshot") {
      options.snapshotPath = readValue(argv, index, "--snapshot");
      index += 1;
    } else if (arg === "--project") {
      options.project = readValue(argv, index, "--project");
      index += 1;
    } else if (arg === "--db-container") {
      options.dbContainer = readValue(argv, index, "--db-container");
      index += 1;
    } else if (arg === "--fail-on-degraded") {
      options.failOnDegraded = true;
    } else {
      throw new UsageError(`unknown argument: ${arg}`);
    }
  }

  return {help, options};
}

export async function main(argv, {
  collectSnapshot = collectRuntimeHealthSnapshot,
  now = Date.now,
  stdout = console.log,
  stderr = console.error,
} = {}) {
  try {
    const parsed = parseArgs(argv);
    if (parsed.help) {
      stdout(usage());
      return 0;
    }

    const {failOnDegraded, quiet, ...snapshotOptions} = parsed.options;
    const snapshot = await collectSnapshot({
      ...snapshotOptions,
      snapshotPath: snapshotOptions.snapshotPath ?? undefined,
      project: snapshotOptions.project ?? undefined,
      dbContainer: snapshotOptions.dbContainer ?? undefined,
      nowMs: now(),
    });
    if (!quiet) {
      stdout(JSON.stringify(snapshot, null, 2));
    }
    return failOnDegraded && snapshot.status === "degraded" ? 3 : 0;
  } catch (error) {
    if (error instanceof UsageError) {
      stderr(error.message);
      stderr(usage());
      return 2;
    }
    stderr(error.stack ?? error.message);
    return 1;
  }
}

await runCliIfMain(import.meta.url, main);
