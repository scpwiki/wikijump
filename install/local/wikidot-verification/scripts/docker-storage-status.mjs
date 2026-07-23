#!/usr/bin/env node
import {collectDockerStorageStatus, defaultStatusPath, redactText} from "../src/docker-storage-status.mjs";
import {parsePositiveIntegerOption as parsePositiveInteger, readRequiredOptionValue as readValue, UsageError} from "../src/cli-options.mjs";

function usage() {
  return [
    "Usage: node scripts/docker-storage-status.mjs [--ttl <ms>] [--refresh] [--json] [--quiet] [--status <path>] [--help]",
    "",
    "Build or serve a cached Docker storage and host disk pressure artifact. Cache hits do no Docker probe work.",
    "",
    "The default status path is:",
    `  ${defaultStatusPath()}`,
    "",
    "Options:",
    "  --ttl <ms>        Positive cache TTL in milliseconds. Default: 300000.",
    "  --refresh         Bypass cache reads, run live read-only probes, and write a new artifact.",
    "  --json            No-op alias; JSON is already the default output.",
    "  --quiet           Print nothing to stdout while still writing or updating the artifact.",
    "  --status <path>   Override the artifact path.",
    "  --help            Print this help.",
    "",
    "cacheStatus meanings:",
    "  hit      A valid fresh artifact was served from disk with no live probes.",
    "  miss     No usable artifact was present, so live probes ran.",
    "  stale    A valid artifact existed but its TTL expired, so live probes ran.",
    "  refresh  --refresh forced live probes.",
    "",
    "Exit codes:",
    "  0  Artifact produced or served.",
    "  1  Tool failure prevented producing or serving an artifact.",
    "  2  Usage error.",
    "",
    "This tool is read-only and never mutates Docker state.",
  ].join("\n");
}

function parseArgs(argv) {
  const options = {ttlMs: 300000, refresh: false, quiet: false, statusPath: null};
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
    } else if (arg === "--status") {
      options.statusPath = readValue(argv, index, "--status");
      index += 1;
    } else {
      throw new UsageError(`unknown argument: ${arg}`);
    }
  }

  return {help, options};
}

try {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    console.log(usage());
  } else {
    const {quiet, ...statusOptions} = parsed.options;
    const status = await collectDockerStorageStatus({
      ...statusOptions,
      statusPath: statusOptions.statusPath ?? undefined,
      nowMs: Date.now(),
    });
    if (!quiet) {
      console.log(JSON.stringify(status, null, 2));
    }
    process.exitCode = 0;
  }
} catch (error) {
  if (error instanceof UsageError) {
    console.error(error.message);
    console.error(usage());
    process.exitCode = 2;
  } else {
    console.error(`docker-storage-status failed: ${redactText(error.errorExcerpt ?? error.stderr ?? error.message)}`);
    process.exitCode = 1;
  }
}
