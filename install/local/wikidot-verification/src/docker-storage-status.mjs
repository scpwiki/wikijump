import {execFile} from "node:child_process";
import {randomUUID} from "node:crypto";
import {existsSync} from "node:fs";
import {mkdir, readFile, rename, rm, statfs, writeFile} from "node:fs/promises";
import path from "node:path";
import {fileURLToPath} from "node:url";

const SCHEMA_VERSION = 1;
const DEFAULT_TTL_MS = 300000;
const PROBE_TIMEOUT_MS = 30000;
const DOCKER_ROOT = "/var/lib/docker";
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

function withCacheFields(artifact, {cacheStatus, cacheReason, ageMs, expired, ttlMs}) {
  const createdMs = Date.parse(artifact.createdAt);
  return {
    ...artifact,
    ttlMs,
    expiresAt: Number.isFinite(createdMs) ? new Date(createdMs + ttlMs).toISOString() : artifact.expiresAt,
    cacheStatus,
    cacheReason,
    ageMs,
    expired,
  };
}

function parseNullableInteger(value) {
  if (typeof value === "number" && Number.isFinite(value)) {
    return Math.trunc(value);
  }
  if (typeof value === "string" && /^[0-9]+$/.test(value.trim())) {
    return Number.parseInt(value, 10);
  }
  return null;
}

function reclaimableSizeText(text) {
  return String(text ?? "").split("(")[0].trim();
}

function normalizeDockerDfEntry(entry) {
  const size = String(entry?.Size ?? "");
  const reclaimable = String(entry?.Reclaimable ?? "");
  return {
    type: String(entry?.Type ?? ""),
    totalCount: parseNullableInteger(entry?.TotalCount),
    active: parseNullableInteger(entry?.Active),
    size,
    sizeBytes: parseSizeToBytes(size),
    reclaimable,
    reclaimableBytes: parseSizeToBytes(reclaimableSizeText(reclaimable)),
  };
}

function parseDockerDfOutput(stdout) {
  const entries = [];
  for (const line of stdout.trim().split("\n").filter((candidate) => candidate.length > 0)) {
    let parsed;
    try {
      parsed = JSON.parse(line);
    } catch {
      return {available: false, entries: [], errorExcerpt: "unparseable docker system df output"};
    }
    entries.push(normalizeDockerDfEntry(parsed));
  }
  return {available: true, entries, errorExcerpt: null};
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
        errorExcerpt: timedOut ? `timed out after ${PROBE_TIMEOUT_MS}ms` : redactText(stderr || error.message),
      });
    });
  });
}

async function probeDockerDf() {
  const result = await execFileText("docker", ["system", "df", "--format", "{{json .}}"]);
  if (!result.ok) {
    return {available: false, elapsedMs: result.elapsedMs, entries: [], errorExcerpt: result.errorExcerpt};
  }
  const parsed = parseDockerDfOutput(result.stdout);
  return {available: parsed.available, elapsedMs: result.elapsedMs, entries: parsed.entries, errorExcerpt: parsed.errorExcerpt};
}

function usedPercent({freeBytes, totalBytes}) {
  if (!Number.isFinite(freeBytes) || !Number.isFinite(totalBytes) || totalBytes <= 0) {
    return null;
  }
  return Math.max(0, Math.min(100, Math.round(((totalBytes - freeBytes) / totalBytes) * 100)));
}

async function statfsEntry(targetPath) {
  const startNs = process.hrtime.bigint();
  try {
    const stats = await statfs(targetPath);
    const freeBytes = stats.bavail * stats.bsize;
    const totalBytes = stats.blocks * stats.bsize;
    return {path: targetPath, available: true, freeBytes, totalBytes, usedPercent: usedPercent({freeBytes, totalBytes}), elapsedMs: elapsedSince(startNs), errorExcerpt: null};
  } catch (error) {
    return {path: targetPath, available: false, freeBytes: null, totalBytes: null, usedPercent: null, elapsedMs: elapsedSince(startNs), errorExcerpt: redactText(error.message)};
  }
}

export function parseDockerRootDir(text) {
  const firstLine = String(text ?? "").trim().split("\n")[0] ?? "";
  return firstLine.startsWith("/") ? firstLine : null;
}

async function resolveDockerRootDir() {
  const result = await execFileText("docker", ["info", "--format", "{{.DockerRootDir}}"]);
  return result.ok ? parseDockerRootDir(result.stdout) : null;
}

async function probeDisk({packageRoot}) {
  const startNs = process.hrtime.bigint();
  // Rootless Docker, Docker Desktop, and custom data-root hosts store layers
  // outside /var/lib/docker, so discover the live root; the literal path is
  // only a fallback when discovery fails.
  const [packageRootEntry, dockerRootDir] = await Promise.all([statfsEntry(packageRoot), resolveDockerRootDir()]);
  const entries = [packageRootEntry];
  const dockerPath = dockerRootDir ?? (existsSync(DOCKER_ROOT) ? DOCKER_ROOT : null);
  if (dockerPath !== null && !samePath(dockerPath, packageRoot)) {
    entries.push(await statfsEntry(dockerPath));
  }
  return {available: packageRootEntry.available === true, elapsedMs: elapsedSince(startNs), entries, errorExcerpt: null};
}

function samePath(left, right) {
  try {
    return path.resolve(String(left)) === path.resolve(String(right));
  } catch {
    return left === right;
  }
}

function packageRootDiskEntry(disk, packageRoot) {
  return (disk.entries ?? []).find((entry) => samePath(entry.path, packageRoot)) ?? null;
}

function firstFailure({dockerDf, disk, packageRoot}) {
  if (!dockerDf.available) {
    return dockerDf.errorExcerpt ?? "probe dockerDf unavailable";
  }
  const packageDisk = packageRootDiskEntry(disk, packageRoot);
  if (packageDisk?.available !== true) {
    return packageDisk?.errorExcerpt ?? "probe disk packageRoot unavailable";
  }
  const failedEntry = (disk.entries ?? []).find((entry) => entry.available !== true);
  if (failedEntry !== undefined) {
    return failedEntry.errorExcerpt ?? `probe disk ${failedEntry.path} unavailable`;
  }
  return null;
}

// A probed-but-failed disk entry (including the Docker root) degrades the
// status; a Docker root that could not be discovered produces no entry and
// does not degrade, since dockerDf covers daemon-level failures.
function overallStatus({dockerDf, disk, packageRoot}) {
  const entries = disk.entries ?? [];
  return dockerDf.available && packageRootDiskEntry(disk, packageRoot)?.available === true && entries.every((entry) => entry.available === true)
    ? "ok"
    : "degraded";
}

async function writeJsonAtomically(statusPath, artifact, runId) {
  const directory = path.dirname(statusPath);
  const temporaryPath = path.join(directory, `${path.basename(statusPath)}.tmp-${process.pid}-${runId}`);
  await mkdir(directory, {recursive: true});
  try {
    await writeFile(temporaryPath, `${JSON.stringify(artifact, null, 2)}\n`, "utf8");
    await rename(temporaryPath, statusPath);
  } finally {
    await rm(temporaryPath, {force: true}).catch(() => {});
  }
}

export function defaultStatusPath() {
  return path.join(PACKAGE_ROOT, "artifacts", "docker-storage", "status.json");
}

export function buildFingerprint(config) {
  return {
    schemaVersion: SCHEMA_VERSION,
    packageRoot: config.packageRoot,
    probeSetVersion: 1,
  };
}

export function evaluateCache({raw, nowMs, fingerprint, ttlMs}) {
  if (raw === null) {
    return {cacheStatus: "miss", cacheReason: "no-file", artifact: null, snapshot: null};
  }
  let artifact;
  try {
    artifact = JSON.parse(raw);
  } catch {
    return {cacheStatus: "miss", cacheReason: "invalid-json", artifact: null, snapshot: null};
  }
  if (artifact.schemaVersion !== SCHEMA_VERSION) {
    return {cacheStatus: "miss", cacheReason: "schema-mismatch", artifact: null, snapshot: null};
  }
  if (!deepEqual(artifact.fingerprint, fingerprint)) {
    return {cacheStatus: "miss", cacheReason: "fingerprint-mismatch", artifact: null, snapshot: null};
  }
  const ageMs = nowMs - Date.parse(artifact.createdAt);
  if (!Number.isFinite(ageMs) || ageMs > ttlMs) {
    const stale = withCacheFields(artifact, {cacheStatus: "stale", cacheReason: "ttl-expired", ageMs, expired: true, ttlMs});
    return {cacheStatus: "stale", cacheReason: "ttl-expired", artifact: stale, snapshot: stale};
  }
  const fresh = withCacheFields(artifact, {cacheStatus: "hit", cacheReason: "fresh", ageMs, expired: false, ttlMs});
  return {cacheStatus: "hit", cacheReason: "fresh", artifact: fresh, snapshot: fresh};
}

export function parseSizeToBytes(text) {
  try {
    const value = String(text ?? "").trim();
    if (value.length === 0) {
      return null;
    }
    if (/^[0-9]+$/.test(value)) {
      const parsed = Number.parseInt(value, 10);
      return Number.isSafeInteger(parsed) ? parsed : null;
    }
    const match = value.match(/^([0-9]+(?:\.[0-9]+)?)(B|kB|KB|MB|GB|TB|KiB|MiB|GiB|TiB)$/);
    if (match === null) {
      return null;
    }
    const amount = Number.parseFloat(match[1]);
    const multipliers = {
      B: 1,
      kB: 1e3,
      KB: 1e3,
      MB: 1e6,
      GB: 1e9,
      TB: 1e12,
      KiB: 1024,
      MiB: 1024 ** 2,
      GiB: 1024 ** 3,
      TiB: 1024 ** 4,
    };
    const bytes = amount * multipliers[match[2]];
    return Number.isFinite(bytes) ? Math.round(bytes) : null;
  } catch {
    return null;
  }
}

export function redactText(text) {
  if (text === null || text === undefined) {
    return null;
  }
  return String(text)
    .replace(/\b([a-z][a-z0-9+.-]*:\/\/)([^/\s@]+)@([^/\s?#]+)/gi, "$1[redacted]@[redacted]")
    .replace(/\b([a-z][a-z0-9+.-]*:\/\/[^\s?#]+)(?:[?#][^\s]*)?/gi, "$1")
    .replace(/(^|[\s,;])([A-Za-z_][A-Za-z0-9_]*(?:PASSWORD|SECRET|TOKEN|KEY)[A-Za-z0-9_]*|PASSWORD|SECRET|TOKEN|KEY|DATABASE_URL|DB_URL|PG[A-Z]+)=(?:"[^"]*"|'[^']*'|[^\s,;]+)/gi, "$1$2=[redacted]")
    .slice(0, 500);
}

export function createNodeProbes() {
  return {
    dockerDf: probeDockerDf,
    disk: probeDisk,
  };
}

export async function collectDockerStorageStatus(options = {}) {
  const nowMs = options.nowMs ?? Date.now();
  const ttlMs = options.ttlMs ?? DEFAULT_TTL_MS;
  const statusPath = path.resolve(options.statusPath ?? defaultStatusPath());
  const config = {packageRoot: PACKAGE_ROOT, statusPath, ttlMs};
  const fingerprint = buildFingerprint(config);

  if (options.refresh !== true) {
    let raw = null;
    try {
      raw = await readFile(statusPath, "utf8");
    } catch (error) {
      if (error.code !== "ENOENT") {
        raw = null;
      }
    }
    const cache = evaluateCache({raw, nowMs, fingerprint, ttlMs});
    if (cache.cacheStatus === "hit") {
      return {...cache.artifact, artifactPaths: {...cache.artifact.artifactPaths, status: statusPath}};
    }
    options = {...options, cacheStatus: cache.cacheStatus, cacheReason: cache.cacheReason};
  } else {
    options = {...options, cacheStatus: "refresh", cacheReason: "refresh"};
  }

  const probes = options.probes ?? createNodeProbes();
  const runId = createRunId(nowMs);
  const totalStartNs = process.hrtime.bigint();
  const [dockerDf, disk] = await Promise.all([probes.dockerDf(config), probes.disk(config)]);
  const status = overallStatus({dockerDf, disk, packageRoot: PACKAGE_ROOT});
  const firstFailureExcerpt = redactText(firstFailure({dockerDf, disk, packageRoot: PACKAGE_ROOT}));
  const createdAt = new Date(nowMs).toISOString();
  const artifact = {
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
    packageRoot: PACKAGE_ROOT,
    fingerprint,
    probes: {
      dockerDf: {available: dockerDf.available, elapsedMs: dockerDf.elapsedMs, entries: dockerDf.entries ?? [], errorExcerpt: redactText(dockerDf.errorExcerpt)},
      disk: {available: disk.available, elapsedMs: disk.elapsedMs, entries: disk.entries ?? [], errorExcerpt: null},
    },
    firstFailureExcerpt,
    elapsedMs: {
      total: elapsedSince(totalStartNs),
      byProbe: {
        dockerDf: dockerDf.elapsedMs,
        disk: disk.elapsedMs,
      },
    },
    artifactPaths: {
      status: statusPath,
    },
    rerunCommand: `cd ${shellQuote(PACKAGE_ROOT)} && node scripts/docker-storage-status.mjs --refresh --json${options.statusPath === undefined ? "" : ` --status ${shellQuote(statusPath)}`}`,
  };
  await writeJsonAtomically(statusPath, artifact, runId);
  return artifact;
}
