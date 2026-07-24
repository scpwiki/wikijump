import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import {readFileSync, writeFileSync} from "node:fs";
import {mkdtemp, rm} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";
import {main as runCiStatusCli, parseArgs as parseCiStatusArgs, usage as ciStatusUsage} from "../scripts/ci-status.mjs";
import {collectCiStatus, redactText} from "../src/ci-status.mjs";

const PACKAGE_ROOT = fileURLToPath(new URL("..", import.meta.url));
const CLI_SCRIPT = fileURLToPath(new URL("../scripts/ci-status.mjs", import.meta.url));
const SHA_X = "1".repeat(40);
const SHA_Y = "2".repeat(40);

async function tempStatus(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "wikijump-ci-status-"));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return path.join(directory, "status.json");
}

function rollup(name, status, conclusion, extra = {}) {
  return {__typename: "CheckRun", name, status, conclusion, startedAt: null, completedAt: null, detailsUrl: `https://example.test/${name}`, ...extra};
}

function makeFetchers(overrides = {}) {
  const calls = {view: 0, branch: 0, checkRuns: 0, protection: 0};
  const fetchers = {
    async fetchPrView() {
      calls.view += 1;
      return {
        number: 331,
        state: "OPEN",
        headRefName: overrides.headRefName ?? "feature/ci",
        headRefOid: overrides.headSha ?? SHA_X,
        mergeable: "MERGEABLE",
        mergeStateStatus: "CLEAN",
        statusCheckRollup: overrides.rollup ?? [rollup("build", "COMPLETED", "SUCCESS"), rollup("test", "COMPLETED", "SUCCESS")],
      };
    },
    async resolveBranchSha() {
      calls.branch += 1;
      return overrides.headSha ?? SHA_X;
    },
    async fetchCheckRuns() {
      calls.checkRuns += 1;
      return {check_runs: overrides.checkRuns ?? []};
    },
    async fetchProtection() {
      calls.protection += 1;
      if (overrides.protectionError !== undefined) {
        throw overrides.protectionError;
      }
      return overrides.protection ?? {strict: true, contexts: ["build", "test"]};
    },
  };
  return {calls, fetchers};
}

function noLocalGit(sha = null) {
  return {resolveHeadSha: () => sha};
}

test("miss fetches once, writes an artifact, and computes passing required checks", async (t) => {
  const statusPath = await tempStatus(t);
  const {calls, fetchers} = makeFetchers();
  const artifact = await collectCiStatus({statusPath, nowMs: 1000, fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.cacheStatus, "miss");
  assert.equal(artifact.cacheReason, "no-file");
  assert.equal(artifact.overall, "passing");
  assert.equal(artifact.requiredChecks.satisfied, true);
  assert.equal(calls.view, 1);
  assert.equal(calls.protection, 1);
  assert.equal(JSON.parse(readFileSync(statusPath, "utf8")).overall, "passing");
});

test("hit serves fresh cache without calling fetchers", async (t) => {
  const statusPath = await tempStatus(t);
  await collectCiStatus({statusPath, nowMs: 1000, fetchers: makeFetchers().fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});
  const second = makeFetchers();
  const artifact = await collectCiStatus({statusPath, nowMs: 1500, fetchers: second.fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.cacheStatus, "hit");
  assert.equal(artifact.cacheReason, "fresh");
  assert.equal(artifact.ageMs, 500);
  assert.deepEqual(second.calls, {view: 0, branch: 0, checkRuns: 0, protection: 0});
});

test("pending artifacts use the short ttl and become stale before completed ttl", async (t) => {
  const statusPath = await tempStatus(t);
  const pending = makeFetchers({rollup: [rollup("build", "IN_PROGRESS", null)], protection: {strict: true, contexts: ["build"]}});
  await collectCiStatus({statusPath, nowMs: 1000, ttlMs: 100, completedTtlMs: 1000, fetchers: pending.fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});
  const second = makeFetchers();
  const artifact = await collectCiStatus({statusPath, nowMs: 1200, ttlMs: 100, completedTtlMs: 1000, fetchers: second.fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.cacheStatus, "stale");
  assert.equal(artifact.cacheReason, "ttl-expired");
  assert.equal(second.calls.view, 1);
});

test("completed artifacts use the longer completed ttl", async (t) => {
  const statusPath = await tempStatus(t);
  await collectCiStatus({statusPath, nowMs: 1000, ttlMs: 100, completedTtlMs: 1000, fetchers: makeFetchers().fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});
  const second = makeFetchers();
  const artifact = await collectCiStatus({statusPath, nowMs: 1200, ttlMs: 100, completedTtlMs: 1000, fetchers: second.fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.cacheStatus, "hit");
  assert.deepEqual(second.calls, {view: 0, branch: 0, checkRuns: 0, protection: 0});
});

test("local head sha changes stale a pr cache", async (t) => {
  const statusPath = await tempStatus(t);
  await collectCiStatus({statusPath, nowMs: 1000, fetchers: makeFetchers({headSha: SHA_X}).fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});
  const second = makeFetchers({headSha: SHA_Y});
  const artifact = await collectCiStatus({statusPath, nowMs: 1100, fetchers: second.fetchers, localGit: noLocalGit(SHA_Y), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.cacheStatus, "stale");
  assert.equal(artifact.cacheReason, "head-sha-changed");
  assert.equal(second.calls.view, 1);
  assert.equal(artifact.subject.headSha, SHA_Y);
});

test("refresh bypasses a fresh cache", async (t) => {
  const statusPath = await tempStatus(t);
  await collectCiStatus({statusPath, nowMs: 1000, fetchers: makeFetchers().fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});
  const second = makeFetchers();
  const artifact = await collectCiStatus({statusPath, nowMs: 1100, refresh: true, fetchers: second.fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.cacheStatus, "refresh");
  assert.equal(artifact.cacheReason, "refresh");
  assert.equal(second.calls.view, 1);
});

test("failing required check sets overall and firstFailure while neutral and skipped do not fail", async (t) => {
  const statusPath = await tempStatus(t);
  const {fetchers} = makeFetchers({
    protection: {strict: true, contexts: ["build", "neutral", "skip"]},
    rollup: [rollup("build", "COMPLETED", "FAILURE"), rollup("neutral", "COMPLETED", "NEUTRAL"), rollup("skip", "COMPLETED", "SKIPPED")],
  });
  const artifact = await collectCiStatus({statusPath, nowMs: 1000, fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.overall, "failing");
  assert.deepEqual(artifact.firstFailure, {name: "build", conclusion: "FAILURE"});
  assert.deepEqual(artifact.requiredChecks.failing, ["build"]);
});

test("missing required context keeps overall pending", async (t) => {
  const statusPath = await tempStatus(t);
  const {fetchers} = makeFetchers({protection: {strict: true, contexts: ["build", "test", "lint"]}});
  const artifact = await collectCiStatus({statusPath, nowMs: 1000, fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.overall, "pending");
  assert.deepEqual(artifact.requiredChecks.missing, ["lint"]);
});

test("unreadable protection never reports passing", async (t) => {
  const statusPath = await tempStatus(t);
  const protectionError = Object.assign(new Error("HTTP 403"), {stderr: "HTTP 403"});
  const {fetchers} = makeFetchers({protectionError, rollup: [rollup("build", "COMPLETED", "SUCCESS")]});
  const artifact = await collectCiStatus({statusPath, nowMs: 1000, fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.requiredChecks.contexts, null);
  assert.equal(artifact.requiredChecks.satisfied, null);
  assert.equal(artifact.overall, "pending");

  const failing = makeFetchers({protectionError, rollup: [rollup("build", "COMPLETED", "FAILURE")]});
  const failingArtifact = await collectCiStatus({statusPath: await tempStatus(t), nowMs: 1000, fetchers: failing.fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});
  assert.equal(failingArtifact.overall, "failing");
});

test("corrupt cache file is a miss and gets replaced", async (t) => {
  const statusPath = await tempStatus(t);
  writeFileSync(statusPath, "not json", "utf8");
  const {calls, fetchers} = makeFetchers();
  const artifact = await collectCiStatus({statusPath, nowMs: 1000, fetchers, localGit: noLocalGit(), subject: {kind: "pr", prNumber: 331}});

  assert.equal(artifact.cacheStatus, "miss");
  assert.equal(artifact.cacheReason, "invalid-json");
  assert.equal(calls.view, 1);
  assert.equal(JSON.parse(readFileSync(statusPath, "utf8")).schemaVersion, 1);
});

test("redactText removes secret assignments and URL userinfo and caps output", () => {
  const output = redactText(`TOKEN=abc SECRET_KEY="def" https://alice:hunter2@example.test/path?x=y ${"x".repeat(600)}`);

  assert.doesNotMatch(output, /abc|def|alice|hunter2|x=y/);
  assert.match(output, /TOKEN=\[redacted\]/);
  assert.match(output, /SECRET_KEY=\[redacted\]/);
  assert.equal(redactText("x".repeat(600)).length, 500);
});

test("CI status CLI exposes subject parsing and failing exit policy", async () => {
  assert.deepEqual(parseCiStatusArgs([
    "--pr", "331",
    "--repo", "Rokurolize/wikijump",
    "--ttl", "40",
    "--completed-ttl", "400",
    "--refresh",
    "--json",
    "--quiet",
    "--status", "/tmp/ci.json",
    "--fail-on-failing",
  ]), {
    help: false,
    options: {
      repo: "Rokurolize/wikijump",
      subject: {kind: "pr", prNumber: 331},
      ttlMs: 40,
      completedTtlMs: 400,
      refresh: true,
      quiet: true,
      statusPath: "/tmp/ci.json",
      failOnFailing: true,
    },
  });
  assert.match(ciStatusUsage(), /read-only/u);

  const calls = [];
  const output = [];
  const code = await runCiStatusCli(["--sha", SHA_X, "--quiet", "--fail-on-failing"], {
    collectStatus: async (options) => {
      calls.push(options);
      return {overall: "failing"};
    },
    now: () => 9876,
    stdout: (line) => output.push(line),
  });
  assert.equal(code, 4);
  assert.equal(output.length, 0);
  assert.deepEqual(calls, [{
    repo: "Rokurolize/wikijump",
    subject: {kind: "sha", sha: SHA_X},
    ttlMs: 30000,
    completedTtlMs: 300000,
    refresh: false,
    statusPath: undefined,
    nowMs: 9876,
  }]);
});

test("cli help and subject usage errors do not run gh", () => {
  const help = spawnSync(process.execPath, [CLI_SCRIPT, "--help"], {cwd: PACKAGE_ROOT, encoding: "utf8"});
  assert.equal(help.status, 0);
  assert.match(help.stdout, /Usage/);

  const missing = spawnSync(process.execPath, [CLI_SCRIPT], {cwd: PACKAGE_ROOT, encoding: "utf8"});
  assert.equal(missing.status, 2);
  assert.match(missing.stderr, /Usage/);

  const both = spawnSync(process.execPath, [CLI_SCRIPT, "--pr", "331", "--branch", "main"], {cwd: PACKAGE_ROOT, encoding: "utf8"});
  assert.equal(both.status, 2);
  assert.match(both.stderr, /exactly one/);
});
