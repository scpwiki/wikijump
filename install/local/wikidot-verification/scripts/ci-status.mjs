#!/usr/bin/env node
import {collectCiStatus, defaultStatusPath, redactText} from "../src/ci-status.mjs";
import {parsePositiveIntegerOption as parsePositiveInteger, readRequiredOptionValue as readValue, UsageError} from "../src/cli-options.mjs";
function usage() {
  return [
    "Usage: node scripts/ci-status.mjs (--pr <N> | --branch <name> | --sha <sha>) [--repo <owner/name>] [--ttl <ms>] [--completed-ttl <ms>] [--refresh] [--json] [--quiet] [--status <path>] [--fail-on-failing] [--help]",
    "",
    "Build or serve a cached GitHub CI/PR status artifact. Cache hits do no GitHub network work.",
    "Options:",
    "  --pr <N> | --branch <name> | --sha <sha>  Exactly one subject.",
    "  --repo <owner/name>   Default: Rokurolize/wikijump.",
    "  --ttl <ms>            TTL while required checks are pending/missing. Default: 30000.",
    "  --completed-ttl <ms>  TTL when overall is passing/failing. Default: 300000.",
    "  --refresh             Bypass cache reads and run live read-only gh calls.",
    "  --json                No-op alias; JSON is default output.",
    "  --quiet               No stdout; artifact is still written or served.",
    "  --status <path>       Override the artifact path.",
    "  --fail-on-failing     Exit 4 when overall is failing.",
    "  --help                Print this help.",
    "",
    "cacheStatus meanings:",
    "  hit=fresh disk artifact with no GitHub network work; miss=no usable artifact; stale=head SHA changed or TTL expired; refresh=--refresh forced live calls.",
    "",
    "Exit codes:",
    "  0=artifact produced/served; 1=tool failure; 2=usage error; 4=failing and --fail-on-failing.",
    "",
    "This tool is read-only; never mutates GitHub state.",
    `Example default PR path: ${defaultStatusPath({kind: "pr", prNumber: 330})}`,
  ].join("\n");
}
function setSubject(options, subject) {
  if (options.subject !== null) {
    throw new UsageError("exactly one of --pr, --branch, or --sha is required");
  }
  options.subject = subject;
}
function parseArgs(argv) {
  const options = {repo: "Rokurolize/wikijump", subject: null, ttlMs: 30000, completedTtlMs: 300000, refresh: false, quiet: false, statusPath: null, failOnFailing: false};
  let help = false;
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      help = true;
    } else if (arg === "--pr") {
      setSubject(options, {kind: "pr", prNumber: parsePositiveInteger(readValue(argv, index, "--pr"), "--pr")});
      index += 1;
    } else if (arg === "--branch") {
      setSubject(options, {kind: "branch", branch: readValue(argv, index, "--branch")});
      index += 1;
    } else if (arg === "--sha") {
      setSubject(options, {kind: "sha", sha: readValue(argv, index, "--sha")});
      index += 1;
    } else if (arg === "--repo") {
      options.repo = readValue(argv, index, "--repo");
      index += 1;
    } else if (arg === "--ttl") {
      options.ttlMs = parsePositiveInteger(readValue(argv, index, "--ttl"), "--ttl");
      index += 1;
    } else if (arg === "--completed-ttl") {
      options.completedTtlMs = parsePositiveInteger(readValue(argv, index, "--completed-ttl"), "--completed-ttl");
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
    } else if (arg === "--fail-on-failing") {
      options.failOnFailing = true;
    } else {
      throw new UsageError(`unknown argument: ${arg}`);
    }
  }
  if (!help && options.subject === null) {
    throw new UsageError("exactly one of --pr, --branch, or --sha is required");
  }
  return {help, options};
}
try {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    console.log(usage());
  } else {
    const {failOnFailing, quiet, ...statusOptions} = parsed.options;
    const status = await collectCiStatus({
      ...statusOptions,
      statusPath: statusOptions.statusPath ?? undefined,
      nowMs: Date.now(),
    });
    if (!quiet) {
      console.log(JSON.stringify(status, null, 2));
    }
    process.exitCode = failOnFailing && status.overall === "failing" ? 4 : 0;
  }
} catch (error) {
  if (error instanceof UsageError) {
    console.error(error.message);
    console.error(usage());
    process.exitCode = 2;
  } else {
    console.error(`ci-status failed: ${redactText(error.errorExcerpt ?? error.stderr ?? error.message)}`);
    process.exitCode = 1;
  }
}
