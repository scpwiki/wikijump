#!/usr/bin/env node
import {collectRuntimeHealthSnapshot, defaultSnapshotPath} from "../src/runtime-health-snapshot.mjs";
import {parsePositiveIntegerOption as parsePositiveInteger, readRequiredOptionValue as readValue, UsageError} from "../src/cli-options.mjs";

function usage() {
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

function parseArgs(argv) {
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

// Set process.exitCode instead of calling process.exit() so pending async
// writes to piped stdout/stderr flush before the process exits naturally.
try {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    console.log(usage());
  } else {
    const {failOnDegraded, quiet, ...snapshotOptions} = parsed.options;
    const snapshot = await collectRuntimeHealthSnapshot({
      ...snapshotOptions,
      snapshotPath: snapshotOptions.snapshotPath ?? undefined,
      project: snapshotOptions.project ?? undefined,
      dbContainer: snapshotOptions.dbContainer ?? undefined,
      nowMs: Date.now(),
    });
    if (!quiet) {
      console.log(JSON.stringify(snapshot, null, 2));
    }
    process.exitCode = failOnDegraded && snapshot.status === "degraded" ? 3 : 0;
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
