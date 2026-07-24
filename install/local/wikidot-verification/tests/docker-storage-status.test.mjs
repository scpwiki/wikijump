import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import {readFileSync, writeFileSync} from "node:fs";
import {mkdtemp, rm} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";
import {main as runDockerStorageCli, parseArgs as parseDockerStorageArgs, usage as dockerStorageUsage} from "../scripts/docker-storage-status.mjs";
import {collectDockerStorageStatus, parseDockerRootDir, parseSizeToBytes} from "../src/docker-storage-status.mjs";

const PACKAGE_ROOT = fileURLToPath(new URL("..", import.meta.url));
const CLI_SCRIPT = fileURLToPath(new URL("../scripts/docker-storage-status.mjs", import.meta.url));

async function tempStatus(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "wikijump-docker-storage-"));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return path.join(directory, "status.json");
}

function goodDockerDf() {
  return {
    available: true,
    elapsedMs: 1,
    entries: [{type: "Images", totalCount: 2, active: 1, size: "748.6MB", sizeBytes: 748600000, reclaimable: "100MB (13%)", reclaimableBytes: 100000000}],
    errorExcerpt: null,
  };
}

function goodDisk() {
  return {
    available: true,
    elapsedMs: 1,
    entries: [{path: PACKAGE_ROOT.replace(/\/$/, ""), available: true, freeBytes: 1000, totalBytes: 2000, usedPercent: 50, elapsedMs: 1, errorExcerpt: null}],
    errorExcerpt: null,
  };
}

function makeProbes(overrides = {}) {
  const calls = {dockerDf: 0, disk: 0};
  const probes = {
    async dockerDf() {
      calls.dockerDf += 1;
      return overrides.dockerDf ?? goodDockerDf();
    },
    async disk() {
      calls.disk += 1;
      return overrides.disk ?? goodDisk();
    },
  };
  return {calls, probes};
}

function totalProbeCalls(calls) {
  return calls.dockerDf + calls.disk;
}

test("miss writes valid JSON artifact with ok status", async (t) => {
  const statusPath = await tempStatus(t);
  const {calls, probes} = makeProbes();
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 300000, probes});

  assert.equal(artifact.cacheStatus, "miss");
  assert.equal(artifact.cacheReason, "no-file");
  assert.equal(artifact.status, "ok");
  assert.equal(calls.dockerDf, 1);
  assert.equal(calls.disk, 1);
  assert.equal(JSON.parse(readFileSync(statusPath, "utf8")).status, "ok");
});

test("hit within TTL serves cache without running probes", async (t) => {
  const statusPath = await tempStatus(t);
  await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 300000, probes: makeProbes().probes});
  const second = makeProbes();
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1500, ttlMs: 300000, probes: second.probes});

  assert.equal(artifact.cacheStatus, "hit");
  assert.equal(artifact.cacheReason, "fresh");
  assert.equal(artifact.ageMs, 500);
  assert.equal(totalProbeCalls(second.calls), 0);
});

test("hit under a different --ttl reports the effective ttl and expiresAt", async (t) => {
  const statusPath = await tempStatus(t);
  await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 300000, probes: makeProbes().probes});
  const second = makeProbes();
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 400000, ttlMs: 600000, probes: second.probes});

  assert.equal(artifact.cacheStatus, "hit");
  assert.equal(totalProbeCalls(second.calls), 0);
  assert.equal(artifact.ttlMs, 600000);
  assert.equal(artifact.expiresAt, new Date(1000 + 600000).toISOString());
  assert.equal(artifact.expired, false);
});

test("ttl expiry marks cache stale and probes again", async (t) => {
  const statusPath = await tempStatus(t);
  await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 100, probes: makeProbes().probes});
  const second = makeProbes();
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1200, ttlMs: 100, probes: second.probes});

  assert.equal(artifact.cacheStatus, "stale");
  assert.equal(artifact.cacheReason, "ttl-expired");
  assert.equal(second.calls.dockerDf, 1);
});

test("refresh bypasses a fresh cache", async (t) => {
  const statusPath = await tempStatus(t);
  await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 300000, probes: makeProbes().probes});
  const second = makeProbes();
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1100, ttlMs: 300000, refresh: true, probes: second.probes});

  assert.equal(artifact.cacheStatus, "refresh");
  assert.equal(artifact.cacheReason, "refresh");
  assert.equal(second.calls.dockerDf, 1);
});

test("corrupt cache file is a miss and gets replaced", async (t) => {
  const statusPath = await tempStatus(t);
  writeFileSync(statusPath, "not json", "utf8");
  const {calls, probes} = makeProbes();
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 300000, probes});

  assert.equal(artifact.cacheStatus, "miss");
  assert.equal(artifact.cacheReason, "invalid-json");
  assert.equal(calls.dockerDf, 1);
  assert.equal(JSON.parse(readFileSync(statusPath, "utf8")).schemaVersion, 1);
});

test("fingerprint mismatch probes and rewrites the artifact", async (t) => {
  const statusPath = await tempStatus(t);
  writeFileSync(statusPath, JSON.stringify({schemaVersion: 1, createdAt: new Date(1000).toISOString(), fingerprint: {schemaVersion: 1, packageRoot: "elsewhere", probeSetVersion: 1}}), "utf8");
  const {calls, probes} = makeProbes();
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1100, ttlMs: 300000, probes});

  assert.equal(artifact.cacheStatus, "miss");
  assert.equal(artifact.cacheReason, "fingerprint-mismatch");
  assert.equal(calls.dockerDf, 1);
});

test("failed docker-root disk entry degrades the status", async (t) => {
  const statusPath = await tempStatus(t);
  const disk = goodDisk();
  disk.entries.push({path: "/var/lib/docker", available: false, freeBytes: null, totalBytes: null, usedPercent: null, elapsedMs: 1, errorExcerpt: "statfs failed"});
  const {probes} = makeProbes({disk});
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 300000, probes});

  assert.equal(artifact.status, "degraded");
  assert.match(artifact.firstFailureExcerpt, /statfs failed/);
});

test("parseDockerRootDir accepts absolute paths only", () => {
  assert.equal(parseDockerRootDir("/var/lib/docker\n"), "/var/lib/docker");
  assert.equal(parseDockerRootDir("/home/user/.local/share/docker"), "/home/user/.local/share/docker");
  assert.equal(parseDockerRootDir(""), null);
  assert.equal(parseDockerRootDir("Cannot connect to the Docker daemon"), null);
  assert.equal(parseDockerRootDir(null), null);
});

test("parseSizeToBytes handles docker decimal units and binary units", () => {
  assert.equal(parseSizeToBytes("0B"), 0);
  assert.equal(parseSizeToBytes("1.5kB"), 1500);
  assert.equal(parseSizeToBytes("748.6MB"), 748600000);
  assert.equal(parseSizeToBytes("23.1GB"), 23100000000);
  assert.equal(parseSizeToBytes("2GiB"), 2147483648);
  assert.equal(parseSizeToBytes("garbage"), null);
  assert.equal(parseSizeToBytes(""), null);
});

test("dockerDf unavailable degrades but still writes valid JSON", async (t) => {
  const statusPath = await tempStatus(t);
  const {probes} = makeProbes({
    dockerDf: {available: false, elapsedMs: 1, entries: [], errorExcerpt: "docker unavailable TOKEN=abc"},
  });
  const artifact = await collectDockerStorageStatus({statusPath, nowMs: 1000, ttlMs: 300000, probes});

  assert.equal(artifact.status, "degraded");
  assert.ok(artifact.firstFailureExcerpt);
  assert.doesNotMatch(artifact.firstFailureExcerpt, /abc/);
  assert.equal(JSON.parse(readFileSync(statusPath, "utf8")).status, "degraded");
});

test("docker storage CLI exposes parsing and orchestration without process side effects", async () => {
  assert.deepEqual(parseDockerStorageArgs([
    "--ttl", "55",
    "--refresh",
    "--json",
    "--quiet",
    "--status", "/tmp/storage.json",
  ]), {
    help: false,
    options: {
      ttlMs: 55,
      refresh: true,
      quiet: true,
      statusPath: "/tmp/storage.json",
    },
  });
  assert.match(dockerStorageUsage(), /read-only/u);

  const calls = [];
  const output = [];
  const code = await runDockerStorageCli(["--quiet"], {
    collectStatus: async (options) => {
      calls.push(options);
      return {status: "ok"};
    },
    now: () => 5678,
    stdout: (line) => output.push(line),
  });
  assert.equal(code, 0);
  assert.equal(output.length, 0);
  assert.deepEqual(calls, [{
    ttlMs: 300000,
    refresh: false,
    statusPath: undefined,
    nowMs: 5678,
  }]);
});

test("cli help, usage errors, and quiet status smoke", async (t) => {
  const statusPath = await tempStatus(t);
  const help = spawnSync(process.execPath, [CLI_SCRIPT, "--help"], {cwd: PACKAGE_ROOT, encoding: "utf8"});
  assert.equal(help.status, 0);
  assert.match(help.stdout, /Usage/);

  const unknown = spawnSync(process.execPath, [CLI_SCRIPT, "--unknown"], {cwd: PACKAGE_ROOT, encoding: "utf8"});
  assert.equal(unknown.status, 2);
  assert.match(unknown.stderr, /Usage/);

  const quiet = spawnSync(process.execPath, [CLI_SCRIPT, "--quiet", "--status", statusPath], {
    cwd: PACKAGE_ROOT,
    encoding: "utf8",
    timeout: 40000,
  });
  assert.equal(quiet.status, 0);
  assert.equal(quiet.stdout, "");
  assert.equal(JSON.parse(readFileSync(statusPath, "utf8")).schemaVersion, 1);
});
