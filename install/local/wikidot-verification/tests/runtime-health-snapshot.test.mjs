import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import {readFileSync, writeFileSync} from "node:fs";
import {mkdtemp, rm} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {fileURLToPath} from "node:url";
import {main as runRuntimeHealthCli, parseArgs as parseRuntimeHealthArgs, usage as runtimeHealthUsage} from "../scripts/runtime-health-snapshot.mjs";
import {buildFingerprint, collectRuntimeHealthSnapshot, parseDockerContainers, redact} from "../src/runtime-health-snapshot.mjs";

const PACKAGE_ROOT = fileURLToPath(new URL("..", import.meta.url));
const CLI_SCRIPT = fileURLToPath(new URL("../scripts/runtime-health-snapshot.mjs", import.meta.url));

async function tempSnapshot(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), "wikijump-runtime-health-"));
  t.after(() => rm(directory, {recursive: true, force: true}));
  return path.join(directory, "snapshot.json");
}

function goodRuntimeUrls() {
  return [
    {name: "deepwell", kind: "http", target: "http://127.0.0.1:2747/jsonrpc", available: true, statusCode: 200, elapsedMs: 1, errorExcerpt: null},
    {name: "framerail", kind: "http", target: "http://127.0.0.1:3393/", available: true, statusCode: 200, elapsedMs: 1, errorExcerpt: null},
    {name: "wws", kind: "http", target: "http://127.0.0.1:3466/-/health-check", available: true, statusCode: 200, elapsedMs: 1, errorExcerpt: null},
    {name: "caddy", kind: "http", target: "https://wikijump.localhost/-/health-check/caddy", available: true, statusCode: 200, elapsedMs: 1, errorExcerpt: null},
    {name: "minio", kind: "tcp", target: "127.0.0.1:9000", available: true, elapsedMs: 1, errorExcerpt: null},
  ];
}

function makeHealthyProbes(overrides = {}) {
  const calls = {docker: 0, db: 0, runtimeUrls: 0, disk: 0, gitHead: 0};
  const probes = {
    async docker() {
      calls.docker += 1;
      return overrides.docker ?? {
        available: true,
        elapsedMs: 1,
        errorExcerpt: null,
        detectedProject: "wikijump",
        containers: [
          {id: "123456789abc", name: "wikijump-database-1", state: "running", health: "healthy", project: "wikijump", service: "database"},
          {id: "abcdef123456", name: "wikijump-deepwell-1", state: "running", health: null, project: "wikijump", service: "deepwell"},
        ],
      };
    },
    async db() {
      calls.db += 1;
      return overrides.db ?? {
        available: true,
        mode: "docker-pg-isready",
        container: "wikijump-database-1",
        elapsedMs: 1,
        errorExcerpt: null,
      };
    },
    async runtimeUrls() {
      calls.runtimeUrls += 1;
      return overrides.runtimeUrls ?? goodRuntimeUrls();
    },
    async disk() {
      calls.disk += 1;
      return overrides.disk ?? {
        available: true,
        path: PACKAGE_ROOT,
        freeBytes: 1000,
        totalBytes: 2000,
        elapsedMs: 1,
        errorExcerpt: null,
      };
    },
    gitHead() {
      calls.gitHead += 1;
      return "a".repeat(40);
    },
  };
  return {calls, probes};
}

function totalProbeCalls(calls) {
  return calls.docker + calls.db + calls.runtimeUrls + calls.disk + calls.gitHead;
}

test("docker status parser recognizes health: starting and degrades the snapshot", async (t) => {
  const parsed = parseDockerContainers(
    [
      "123456789abcdef\twikijump-deepwell-1\tUp 3 seconds (health: starting)\twikijump\tdeepwell",
      "abcdef123456789\twikijump-database-1\tUp 2 days (healthy)\twikijump\tdatabase",
      "fedcba987654321\twikijump-cache-1\tUp 2 days (unhealthy)\twikijump\tcache",
    ].join("\n"),
  );
  assert.equal(parsed[0].health, "starting");
  assert.equal(parsed[1].health, "healthy");
  assert.equal(parsed[2].health, "unhealthy");

  const snapshotPath = await tempSnapshot(t);
  const {probes} = makeHealthyProbes({
    docker: {
      available: true,
      elapsedMs: 1,
      errorExcerpt: null,
      detectedProject: "wikijump",
      containers: [
        {id: "123456789abc", name: "wikijump-database-1", state: "running", health: "healthy", project: "wikijump", service: "database"},
        {id: "abcdef123456", name: "wikijump-deepwell-1", state: "running", health: "starting", project: "wikijump", service: "deepwell"},
      ],
    },
  });
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes});
  assert.equal(snapshot.status, "degraded");
  assert.match(snapshot.firstFailureExcerpt, /wikijump-deepwell-1/);
});

test("single stopped container for a detected service degrades the snapshot", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  const {probes} = makeHealthyProbes({
    docker: {
      available: true,
      elapsedMs: 1,
      errorExcerpt: null,
      detectedProject: "wikijump",
      containers: [
        {id: "123456789abc", name: "wikijump-database-1", state: "running", health: "healthy", project: "wikijump", service: "database"},
        {id: "abcdef123456", name: "wikijump-deepwell-1", state: "running", health: null, project: "wikijump", service: "deepwell"},
        {id: "fedcba987654", name: "wikijump-cache-1", state: "not-running", health: null, project: "wikijump", service: "cache"},
      ],
    },
  });
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes});

  assert.equal(snapshot.status, "degraded");
  assert.match(snapshot.firstFailureExcerpt, /cache/);
});

test("stopped leftover beside a healthy sibling does not degrade the service", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  const {probes} = makeHealthyProbes({
    docker: {
      available: true,
      elapsedMs: 1,
      errorExcerpt: null,
      detectedProject: "wikijump",
      containers: [
        {id: "123456789abc", name: "wikijump-database-1", state: "running", health: "healthy", project: "wikijump", service: "database"},
        {id: "abcdef123456", name: "wikijump-deepwell-1", state: "running", health: null, project: "wikijump", service: "deepwell"},
        {id: "fedcba987654", name: "wikijump-cache-old", state: "not-running", health: null, project: "wikijump", service: "cache"},
        {id: "111111111111", name: "wikijump-cache-1", state: "running", health: "healthy", project: "wikijump", service: "cache"},
      ],
    },
  });
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes});

  assert.equal(snapshot.status, "healthy");
});

test("miss probes and writes a healthy snapshot when no file exists", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  const {calls, probes} = makeHealthyProbes();
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes});

  assert.equal(snapshot.cacheStatus, "miss");
  assert.equal(snapshot.cacheReason, "no-file");
  assert.equal(snapshot.status, "healthy");
  assert.equal(calls.docker, 1);
  assert.equal(calls.db, 1);
  assert.equal(calls.runtimeUrls, 1);
  assert.equal(calls.disk, 1);
  assert.equal(calls.gitHead, 1);
  assert.equal(JSON.parse(readFileSync(snapshotPath, "utf8")).status, "healthy");
});

test("hit serves fresh cached data without running probes", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  const first = makeHealthyProbes();
  await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes: first.probes});
  const second = makeHealthyProbes();
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1500, ttlMs: 30000, probes: second.probes});

  assert.equal(snapshot.cacheStatus, "hit");
  assert.equal(snapshot.cacheReason, "fresh");
  assert.ok(snapshot.ageMs > 0);
  assert.equal(totalProbeCalls(second.calls), 0);
});

test("ttl expiry marks the cache stale and probes again", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 100, probes: makeHealthyProbes().probes});
  const second = makeHealthyProbes();
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1200, ttlMs: 100, probes: second.probes});

  assert.equal(snapshot.cacheStatus, "stale");
  assert.equal(snapshot.cacheReason, "ttl-expired");
  assert.equal(second.calls.docker, 1);
});

test("refresh bypasses a fresh cache and probes", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes: makeHealthyProbes().probes});
  const second = makeHealthyProbes();
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1100, ttlMs: 30000, refresh: true, probes: second.probes});

  assert.equal(snapshot.cacheStatus, "refresh");
  assert.equal(second.calls.docker, 1);
});

test("corrupt cache file is overwritten with valid JSON", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  writeFileSync(snapshotPath, "not json", "utf8");
  const {calls, probes} = makeHealthyProbes();
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes});

  assert.equal(snapshot.cacheStatus, "miss");
  assert.equal(snapshot.cacheReason, "invalid-json");
  assert.equal(calls.docker, 1);
  assert.equal(JSON.parse(readFileSync(snapshotPath, "utf8")).schemaVersion, 1);
});

test("fingerprint mismatch probes and rewrites the snapshot", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  const fingerprint = buildFingerprint({packageRoot: PACKAGE_ROOT, projectSetting: "other", dbContainerSetting: null});
  writeFileSync(snapshotPath, JSON.stringify({schemaVersion: 1, createdAt: new Date(1000).toISOString(), fingerprint}), "utf8");
  const {calls, probes} = makeHealthyProbes();
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1100, ttlMs: 30000, project: "wikijump", probes});

  assert.equal(snapshot.cacheStatus, "miss");
  assert.equal(snapshot.cacheReason, "fingerprint-mismatch");
  assert.equal(calls.docker, 1);
});

test("redact removes secret values, URL query text, and caps output", () => {
  const input = [
    "postgres://alice:hunter2@db.internal/wiki?sslmode=require",
    "PGPASSWORD=hunter2",
    'DATABASE_URL="postgres://x"',
    'user "alice"',
    'host "db.internal"',
    "password=hunter2",
  ].join(" ");
  const output = redact(input);

  assert.doesNotMatch(output, /alice/);
  assert.doesNotMatch(output, /hunter2/);
  assert.doesNotMatch(output, /db\.internal/);
  assert.doesNotMatch(output, /sslmode/);
  assert.equal(redact("x".repeat(600)).length, 500);
});

test("degraded snapshot is still valid JSON with first failure", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  const {probes} = makeHealthyProbes({
    docker: {
      available: false,
      elapsedMs: 1,
      errorExcerpt: "docker unavailable",
      detectedProject: null,
      containers: [],
    },
  });
  const snapshot = await collectRuntimeHealthSnapshot({snapshotPath, nowMs: 1000, ttlMs: 30000, probes});

  assert.equal(snapshot.status, "degraded");
  assert.ok(snapshot.firstFailureExcerpt);
  assert.equal(JSON.parse(readFileSync(snapshotPath, "utf8")).status, "degraded");
});

test("runtime health CLI exposes parsing and orchestration without process side effects", async () => {
  assert.deepEqual(parseRuntimeHealthArgs([
    "--ttl", "42",
    "--refresh",
    "--json",
    "--quiet",
    "--snapshot", "/tmp/runtime.json",
    "--project", "wikijump",
    "--db-container", "database",
    "--fail-on-degraded",
  ]), {
    help: false,
    options: {
      ttlMs: 42,
      refresh: true,
      quiet: true,
      snapshotPath: "/tmp/runtime.json",
      project: "wikijump",
      dbContainer: "database",
      failOnDegraded: true,
    },
  });
  assert.match(runtimeHealthUsage(), /--fail-on-degraded/u);

  const calls = [];
  const output = [];
  const code = await runRuntimeHealthCli(["--quiet", "--fail-on-degraded"], {
    collectSnapshot: async (options) => {
      calls.push(options);
      return {status: "degraded"};
    },
    now: () => 1234,
    stdout: (line) => output.push(line),
  });
  assert.equal(code, 3);
  assert.equal(output.length, 0);
  assert.deepEqual(calls, [{
    ttlMs: 30000,
    refresh: false,
    snapshotPath: undefined,
    project: undefined,
    dbContainer: undefined,
    nowMs: 1234,
  }]);
});

test("cli help, usage errors, and quiet snapshot smoke", async (t) => {
  const snapshotPath = await tempSnapshot(t);
  const help = spawnSync(process.execPath, [CLI_SCRIPT, "--help"], {cwd: PACKAGE_ROOT, encoding: "utf8"});
  assert.equal(help.status, 0);
  assert.match(help.stdout, /Usage/);

  const unknown = spawnSync(process.execPath, [CLI_SCRIPT, "--unknown"], {cwd: PACKAGE_ROOT, encoding: "utf8"});
  assert.equal(unknown.status, 2);
  assert.match(unknown.stderr, /Usage/);

  const quiet = spawnSync(process.execPath, [CLI_SCRIPT, "--quiet", "--snapshot", snapshotPath], {
    cwd: PACKAGE_ROOT,
    encoding: "utf8",
    timeout: 10000,
  });
  assert.equal(quiet.status, 0);
  assert.equal(quiet.stdout, "");
  assert.equal(JSON.parse(readFileSync(snapshotPath, "utf8")).schemaVersion, 1);
});
