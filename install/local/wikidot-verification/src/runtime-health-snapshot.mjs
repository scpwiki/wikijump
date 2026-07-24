import {execFile, execFileSync} from "node:child_process";
import {randomUUID} from "node:crypto";
import {mkdir, readFile, rename, rm, statfs, writeFile} from "node:fs/promises";
import http from "node:http";
import https from "node:https";
import net from "node:net";
import path from "node:path";
import {fileURLToPath} from "node:url";

const SCHEMA_VERSION = 1;
const DEFAULT_TTL_MS = 30000;
const PROBE_TIMEOUT_MS = 3000;
const PACKAGE_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function shellQuote(value) {
  const text = String(value);
  return text.length === 0 ? "''" : `'${text.replaceAll("'", "'\\''")}'`;
}

function createRunId(nowMs) {
  return `${new Date(nowMs).toISOString().replace(/[:.]/g, "-")}-${randomUUID().slice(0, 8)}`;
}

function elapsedSince(startNs) {
  return Number(process.hrtime.bigint() - startNs) / 1_000_000;
}

function deepEqual(left, right) {
  if (left === right) {
    return true;
  }
  if (typeof left !== "object" || left === null || typeof right !== "object" || right === null) {
    return false;
  }
  if (Array.isArray(left) || Array.isArray(right)) {
    return Array.isArray(left) && Array.isArray(right) && left.length === right.length && left.every((value, index) => deepEqual(value, right[index]));
  }
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);
  return leftKeys.length === rightKeys.length && leftKeys.every((key) => Object.hasOwn(right, key) && deepEqual(left[key], right[key]));
}

function readGitHead(cwd) {
  try {
    const head = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    return /^[0-9a-f]{40}$/i.test(head) ? head : null;
  } catch {
    return null;
  }
}

function execFileText(command, args) {
  const startNs = process.hrtime.bigint();
  return new Promise((resolve) => {
    execFile(command, args, {encoding: "utf8", timeout: PROBE_TIMEOUT_MS, maxBuffer: 1024 * 1024}, (error, stdout, stderr) => {
      const elapsedMs = elapsedSince(startNs);
      if (error === null) {
        resolve({ok: true, stdout, stderr, elapsedMs, errorExcerpt: null});
        return;
      }
      const timedOut = error.killed === true && error.signal === "SIGTERM";
      resolve({
        ok: false,
        stdout: stdout ?? "",
        stderr: stderr ?? "",
        elapsedMs,
        errorExcerpt: timedOut ? `timed out after ${PROBE_TIMEOUT_MS}ms` : redact(stderr || error.message),
      });
    });
  });
}

export function parseDockerContainers(stdout) {
  return stdout
    .trim()
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => {
      const [id = "", name = "", status = "", project = "", service = ""] = line.split("\t");
      // Docker renders the starting phase as "(health: starting)", not "(starting)".
      const healthMatch = status.match(/\((healthy|unhealthy|(?:health: )?starting)\)/);
      const health = healthMatch === null ? null : healthMatch[1].replace("health: ", "");
      return {id: id.slice(0, 12), name, state: status.startsWith("Up") ? "running" : "not-running", health, project: project || null, service: service || null};
    });
}

function detectProject(containers, projectSetting) {
  if (projectSetting !== null) {
    return projectSetting;
  }
  const projects = new Set(
    containers
      .filter((container) => container.state === "running" && container.service === "database" && container.project !== null)
      .map((container) => container.project),
  );
  return projects.size === 1 ? [...projects][0] : null;
}

async function probeDocker({projectSetting}) {
  const result = await execFileText("docker", [
    "ps",
    "--all",
    "--format",
    '{{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Label "com.docker.compose.project"}}\t{{.Label "com.docker.compose.service"}}',
  ]);
  if (!result.ok) {
    return {available: false, elapsedMs: result.elapsedMs, errorExcerpt: result.errorExcerpt, detectedProject: projectSetting, containers: []};
  }
  const containers = parseDockerContainers(result.stdout);
  return {available: true, elapsedMs: result.elapsedMs, errorExcerpt: null, detectedProject: detectProject(containers, projectSetting), containers};
}

async function probeDb({dbContainerSetting}, dockerResult) {
  const detectedProject = dockerResult.detectedProject ?? null;
  if (dbContainerSetting === null && detectedProject === null) {
    return {available: false, mode: "skipped", container: null, elapsedMs: 0, errorExcerpt: "no compose project detected"};
  }
  const container = dbContainerSetting ?? `${detectedProject}-database-1`;
  const result = await execFileText("docker", [
    "exec",
    container,
    "sh",
    "-lc",
    'pg_isready -h localhost -U "$POSTGRES_USER" -d "$POSTGRES_DB"',
  ]);
  return {available: result.ok, mode: "docker-pg-isready", container, elapsedMs: result.elapsedMs, errorExcerpt: result.ok ? null : result.errorExcerpt};
}

function probeHttpEndpoint({name, method, url, headers = {}, okStatus}) {
  const startNs = process.hrtime.bigint();
  const parsed = new URL(url);
  const client = parsed.protocol === "https:" ? https : http;
  return new Promise((resolve) => {
    let settled = false;
    const finish = (entry) => {
      if (settled) {
        return;
      }
      settled = true;
      resolve({name, kind: "http", target: url, available: false, statusCode: null, elapsedMs: elapsedSince(startNs), errorExcerpt: null, ...entry});
    };
    const request = client.request(parsed, {method, headers, timeout: PROBE_TIMEOUT_MS}, (response) => {
      response.resume();
      const available = okStatus(response.statusCode ?? 0);
      finish({available, statusCode: response.statusCode ?? null, errorExcerpt: available ? null : `HTTP ${response.statusCode ?? "unknown"}`});
    });
    request.on("timeout", () => {
      request.destroy();
      finish({errorExcerpt: `timed out after ${PROBE_TIMEOUT_MS}ms`});
    });
    request.on("error", (error) => {
      finish({errorExcerpt: redact(error.message)});
    });
    request.end(name === "deepwell" ? JSON.stringify({jsonrpc: "2.0", method: "ping", id: 0}) : undefined);
  });
}

function probeTcpEndpoint({name, host, port}) {
  const startNs = process.hrtime.bigint();
  return new Promise((resolve) => {
    let settled = false;
    const socket = net.createConnection({host, port});
    const finish = (entry) => {
      if (settled) {
        return;
      }
      settled = true;
      socket.destroy();
      resolve({name, kind: "tcp", target: `${host}:${port}`, available: false, elapsedMs: elapsedSince(startNs), errorExcerpt: null, ...entry});
    };
    socket.setTimeout(PROBE_TIMEOUT_MS);
    socket.once("connect", () => finish({available: true}));
    socket.once("timeout", () => finish({errorExcerpt: `timed out after ${PROBE_TIMEOUT_MS}ms`}));
    socket.once("error", (error) => finish({errorExcerpt: redact(error.message)}));
  });
}

async function probeRuntimeUrls() {
  return Promise.all([
    probeHttpEndpoint({name: "deepwell", method: "POST", url: "http://127.0.0.1:2747/jsonrpc", headers: {"content-type": "application/json"}, okStatus: (statusCode) => statusCode === 200}),
    probeHttpEndpoint({name: "framerail", method: "HEAD", url: "http://127.0.0.1:3393/", headers: {"x-wikijump-site-slug": "www", "x-wikijump-site-id": "6000000"}, okStatus: (statusCode) => statusCode < 500}),
    probeHttpEndpoint({name: "wws", method: "HEAD", url: "http://127.0.0.1:3466/-/health-check", okStatus: (statusCode) => statusCode >= 200 && statusCode < 300}),
    probeHttpEndpoint({name: "caddy", method: "HEAD", url: "https://wikijump.localhost/-/health-check/caddy", okStatus: (statusCode) => statusCode >= 200 && statusCode < 300}),
    probeTcpEndpoint({name: "minio", host: "127.0.0.1", port: 9000}),
  ]);
}

async function probeDisk({packageRoot}) {
  const startNs = process.hrtime.bigint();
  try {
    const stats = await statfs(packageRoot);
    return {available: true, path: packageRoot, freeBytes: stats.bavail * stats.bsize, totalBytes: stats.blocks * stats.bsize, elapsedMs: elapsedSince(startNs), errorExcerpt: null};
  } catch (error) {
    return {available: false, path: packageRoot, freeBytes: null, totalBytes: null, elapsedMs: elapsedSince(startNs), errorExcerpt: redact(error.message)};
  }
}

function isHealthyContainer(container) {
  return container.state === "running" && (container.health === null || container.health === "healthy");
}

// Known limit: services defined in compose but never created have no container to observe; URL and DB probes cover them functionally.
function failedService(detectedProject, containers) {
  const services = new Map();
  for (const container of containers) {
    if (container.project === detectedProject && container.service !== null) {
      services.set(container.service, [...(services.get(container.service) ?? []), container]);
    }
  }
  for (const [service, serviceContainers] of services) {
    if (!serviceContainers.some(isHealthyContainer)) {
      return {service, container: serviceContainers[0]};
    }
  }
  return null;
}

function firstFailure({docker, db, runtimeUrls, disk, detectedProject, containers}) {
  if (!docker.available) {
    return docker.errorExcerpt ?? "probe docker unavailable";
  }
  if (detectedProject === null) {
    return "no compose project detected";
  }
  if (!db.available) {
    return db.errorExcerpt ?? "probe db unavailable";
  }
  const failedRuntime = runtimeUrls.find((entry) => !entry.available);
  if (failedRuntime !== undefined) {
    return failedRuntime.errorExcerpt ?? `probe runtimeUrls.${failedRuntime.name} unavailable`;
  }
  if (!disk.available) {
    return disk.errorExcerpt ?? "probe disk unavailable";
  }
  const failed = failedService(detectedProject, containers);
  return failed === null ? null : `service ${failed.service} has no running healthy container (${failed.container.name}: ${failed.container.state}/${failed.container.health ?? "none"})`;
}

function overallStatus({docker, db, runtimeUrls, disk, detectedProject, containers}) {
  const projectServicesHealthy = detectedProject !== null && failedService(detectedProject, containers) === null;
  return docker.available && db.available && runtimeUrls.every((entry) => entry.available) && disk.available && projectServicesHealthy ? "healthy" : "degraded";
}

async function writeJsonAtomically(snapshotPath, snapshot, runId) {
  const directory = path.dirname(snapshotPath);
  const temporaryPath = path.join(directory, `${path.basename(snapshotPath)}.tmp-${process.pid}-${runId}`);
  await mkdir(directory, {recursive: true});
  try {
    await writeFile(temporaryPath, `${JSON.stringify(snapshot, null, 2)}\n`, "utf8");
    await rename(temporaryPath, snapshotPath);
  } finally {
    await rm(temporaryPath, {force: true}).catch(() => {});
  }
}

export function defaultSnapshotPath() {
  return path.join(PACKAGE_ROOT, "artifacts", "runtime-health", "snapshot.json");
}

export function buildFingerprint(config) {
  return {
    schemaVersion: SCHEMA_VERSION,
    packageRoot: config.packageRoot,
    projectSetting: config.projectSetting,
    dbContainerSetting: config.dbContainerSetting,
    probeSetVersion: 2,
  };
}

export function evaluateCache({raw, nowMs, fingerprint, ttlMs}) {
  if (raw === null) {
    return {cacheStatus: "miss", cacheReason: "no-file", snapshot: null};
  }
  let snapshot;
  try {
    snapshot = JSON.parse(raw);
  } catch {
    return {cacheStatus: "miss", cacheReason: "invalid-json", snapshot: null};
  }
  if (snapshot.schemaVersion !== SCHEMA_VERSION) {
    return {cacheStatus: "miss", cacheReason: "schema-mismatch", snapshot: null};
  }
  if (!deepEqual(snapshot.fingerprint, fingerprint)) {
    return {cacheStatus: "miss", cacheReason: "fingerprint-mismatch", snapshot: null};
  }
  const ageMs = nowMs - Date.parse(snapshot.createdAt);
  if (!Number.isFinite(ageMs) || ageMs > ttlMs) {
    return {cacheStatus: "stale", cacheReason: "ttl-expired", snapshot};
  }
  return {
    cacheStatus: "hit",
    cacheReason: "fresh",
    snapshot: {
      ...snapshot,
      cacheStatus: "hit",
      cacheReason: "fresh",
      ageMs,
      expired: false,
    },
  };
}

export function redact(text) {
  if (text === null || text === undefined) {
    return null;
  }
  return String(text)
    .replace(/\b([a-z][a-z0-9+.-]*:\/\/)([^/\s@]+)@([^/\s?#]+)/gi, "$1[redacted]@[redacted]")
    .replace(/\b([a-z][a-z0-9+.-]*:\/\/[^\s?#]+)(?:[?#][^\s]*)?/gi, "$1")
    .replace(
      /(^|[\s,;])([A-Za-z_][A-Za-z0-9_]*(?:PASSWORD|SECRET|TOKEN|KEY|DATABASE_URL|DB_URL|PG[A-Z]+)[A-Za-z0-9_]*)=(?:"[^"]*"|'[^']*'|[^\s,;]+)/gi,
      "$1$2=[redacted]",
    )
    .replace(/\b(user|database|host)\s+"[^"]*"/gi, '$1 "[redacted]"')
    .replace(/\b(user|password|host|dbname)=[^\s,;]+/gi, "$1=[redacted]")
    .slice(0, 500);
}

export function createNodeProbes() {
  return {
    docker: probeDocker,
    db: probeDb,
    runtimeUrls: probeRuntimeUrls,
    disk: probeDisk,
    gitHead: readGitHead,
  };
}

export async function collectRuntimeHealthSnapshot(options = {}) {
  const nowMs = options.nowMs ?? Date.now();
  const ttlMs = options.ttlMs ?? DEFAULT_TTL_MS;
  const snapshotPath = path.resolve(options.snapshotPath ?? defaultSnapshotPath());
  const projectSetting = options.project ?? process.env.COMPOSE_PROJECT_NAME ?? null;
  const dbContainerSetting = options.dbContainer ?? null;
  const config = {packageRoot: PACKAGE_ROOT, projectSetting, dbContainerSetting, snapshotPath, ttlMs};
  const fingerprint = buildFingerprint(config);

  if (options.refresh !== true) {
    let raw = null;
    try {
      raw = await readFile(snapshotPath, "utf8");
    } catch (error) {
      if (error.code !== "ENOENT") {
        raw = null;
      }
    }
    const cache = evaluateCache({raw, nowMs, fingerprint, ttlMs});
    if (cache.cacheStatus === "hit") {
      return cache.snapshot;
    }
    options = {...options, cacheStatus: cache.cacheStatus, cacheReason: cache.cacheReason};
  } else {
    options = {...options, cacheStatus: "refresh", cacheReason: "refresh"};
  }

  const probes = options.probes ?? createNodeProbes();
  const runId = createRunId(nowMs);
  const totalStartNs = process.hrtime.bigint();
  const dockerPromise = probes.docker(config);
  const runtimePromise = probes.runtimeUrls(config);
  const diskPromise = probes.disk(config);
  const docker = await dockerPromise;
  const dbPromise = probes.db(config, docker);
  const [runtimeUrls, disk, db] = await Promise.all([runtimePromise, diskPromise, dbPromise]);
  const containers = docker.containers ?? [];
  const detectedProject = docker.detectedProject ?? null;
  const gitHead = (probes.gitHead ?? readGitHead)(PACKAGE_ROOT);
  const status = overallStatus({docker, db, runtimeUrls, disk, detectedProject, containers});
  const firstFailureExcerpt = redact(firstFailure({docker, db, runtimeUrls, disk, detectedProject, containers}));
  const createdAt = new Date(nowMs).toISOString();
  const snapshot = {
    schemaVersion: SCHEMA_VERSION,
    runId,
    createdAt,
    ttlMs,
    expiresAt: new Date(nowMs + ttlMs).toISOString(),
    ageMs: 0,
    expired: false,
    cacheStatus: options.cacheStatus,
    cacheReason: options.cacheReason,
    status,
    cwd: process.cwd(),
    packageRoot: PACKAGE_ROOT,
    gitHead,
    detectedProject,
    fingerprint,
    probes: {
      docker: {
        available: docker.available,
        elapsedMs: docker.elapsedMs,
        errorExcerpt: redact(docker.errorExcerpt),
      },
      containers,
      runtimeUrls: runtimeUrls.map((entry) => ({...entry, errorExcerpt: redact(entry.errorExcerpt)})),
      db: {...db, errorExcerpt: redact(db.errorExcerpt)},
      disk: {...disk, errorExcerpt: redact(disk.errorExcerpt)},
    },
    firstFailureExcerpt,
    artifactPaths: {
      snapshot: snapshotPath,
    },
    rerunCommand: `cd ${shellQuote(PACKAGE_ROOT)} && node scripts/runtime-health-snapshot.mjs --refresh`,
    elapsedMs: {
      total: elapsedSince(totalStartNs),
      byProbe: {
        docker: docker.elapsedMs,
        db: db.elapsedMs,
        runtimeUrls: Math.max(0, ...runtimeUrls.map((entry) => entry.elapsedMs)),
        disk: disk.elapsedMs,
      },
    },
  };
  await writeJsonAtomically(snapshotPath, snapshot, runId);
  return snapshot;
}
