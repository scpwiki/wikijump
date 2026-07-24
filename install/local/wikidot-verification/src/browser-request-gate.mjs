import {randomUUID} from "node:crypto";
import fs from "node:fs/promises";
import {constants as fsConstants} from "node:fs";
import os from "node:os";
import path from "node:path";

export const DEFAULT_REQUEST_INTERVAL_MS = 4_000;
export const DEFAULT_BROWSER_CAPTURE_LOCK = "/var/tmp/wikijump-wikidot-browser-capture.lock";
const DEFAULT_RESPONSE_CACHE_MAX_ENTRIES = 512;
const DEFAULT_RESPONSE_CACHE_MAX_BYTES = 64 * 1024 * 1024;
const DEFAULT_RESPONSE_CACHE_MAX_ENTRY_BYTES = 8 * 1024 * 1024;
const LOCK_SCHEMA = "wikijump_full_parity.browser_capture_lock.v1";
const STATE_SCHEMA = "wikijump_full_parity.browser_request_gate_state.v1";
const STATE_CONFIRMATIONS = new Set(["pending", "sealed"]);
const LOCAL_WIKIJUMP_HOST_RE = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.wikijump\.localhost$/u;

function defaultNow() {
  return Date.now();
}

function defaultSleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function finiteNonNegative(value, name) {
  if (!Number.isFinite(value) || value < 0) throw new Error(`${name} must be a non-negative finite number`);
  return value;
}

function finiteTimestampOrNegativeInfinity(value, name) {
  if (value === Number.NEGATIVE_INFINITY) return value;
  return finiteNonNegative(value, name);
}

function normalizedOrigins(values) {
  const origins = new Set();
  for (const value of values ?? []) {
    let url;
    try {
      url = new URL(value);
    } catch {
      throw new Error(`browser request-gate exemption is not a URL: ${value}`);
    }
    if (!new Set(["http:", "https:"]).has(url.protocol) || url.username || url.password || url.origin !== String(value).replace(/\/$/u, "")) {
      throw new Error(`browser request-gate exemption must be an exact HTTP(S) origin: ${value}`);
    }
    origins.add(url.origin);
  }
  return origins;
}

function positiveSafeInteger(value, name) {
  if (!Number.isSafeInteger(value) || value <= 0) throw new Error(`${name} must be a positive safe integer`);
  return value;
}

export function createBrowserResponseCache({maxEntries = DEFAULT_RESPONSE_CACHE_MAX_ENTRIES, maxBytes = DEFAULT_RESPONSE_CACHE_MAX_BYTES, maxEntryBytes = DEFAULT_RESPONSE_CACHE_MAX_ENTRY_BYTES} = {}) {
  positiveSafeInteger(maxEntries, "maxEntries");
  positiveSafeInteger(maxBytes, "maxBytes");
  positiveSafeInteger(maxEntryBytes, "maxEntryBytes");
  if (maxEntryBytes > maxBytes) throw new Error("maxEntryBytes cannot exceed maxBytes");

  const entries = new Map();
  let bytes = 0;
  let hits = 0;
  let misses = 0;
  let stores = 0;
  let bypasses = 0;
  let evictions = 0;

  return {
    maxEntryBytes,
    get(key) {
      const entry = entries.get(key);
      if (!entry) {
        misses += 1;
        return null;
      }
      entries.delete(key);
      entries.set(key, entry);
      hits += 1;
      return entry;
    },
    store(key, entry) {
      if (!Buffer.isBuffer(entry?.body) || entry.body.length > maxEntryBytes) {
        bypasses += 1;
        return false;
      }
      const existing = entries.get(key);
      if (existing) {
        entries.delete(key);
        bytes -= existing.body.length;
      }
      while (entries.size >= maxEntries || bytes + entry.body.length > maxBytes) {
        const oldestKey = entries.keys().next().value;
        if (oldestKey === undefined) break;
        const oldest = entries.get(oldestKey);
        entries.delete(oldestKey);
        bytes -= oldest.body.length;
        evictions += 1;
      }
      entries.set(key, entry);
      bytes += entry.body.length;
      stores += 1;
      return true;
    },
    recordBypass() {
      bypasses += 1;
    },
    snapshot() {
      return {
        schema: "wikijump_full_parity.browser_response_cache.v1",
        entries: entries.size,
        bytes,
        hits,
        misses,
        stores,
        bypasses,
        evictions,
        max_entries: maxEntries,
        max_bytes: maxBytes,
        max_entry_bytes: maxEntryBytes,
        lookup_key: "exact_url",
        lifetime: "browser_context",
        documents_cached: false,
      };
    },
  };
}

export function localBrowserCaptureOrigins(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error(`local capture URL is not a URL: ${value}`);
  }
  if (url.protocol !== "https:" || url.username || url.password || url.port || !LOCAL_WIKIJUMP_HOST_RE.test(url.hostname)) {
    throw new Error(`local capture URL must be an HTTPS *.wikijump.localhost origin without credentials or a non-default port: ${value}`);
  }
  const site = url.hostname.slice(0, -".wikijump.localhost".length);
  return [url.origin, `https://${site}.wjfiles.localhost`];
}

export function parseRetryAfterMilliseconds(value, {epochNow = Date.now} = {}) {
  if (typeof value !== "string" || value.trim() === "") return null;
  const text = value.trim();
  if (/^\d+$/u.test(text)) {
    const milliseconds = Number(text) * 1_000;
    return Number.isFinite(milliseconds) ? milliseconds : null;
  }
  const retryAt = Date.parse(text);
  const current = epochNow();
  if (!Number.isFinite(retryAt) || !Number.isFinite(current)) return null;
  return Math.max(0, retryAt - current);
}

function stateSnapshot({nextAvailableAt, blockedUntil}) {
  return {
    next_admissible_at_epoch_ms: Math.max(0, nextAvailableAt),
    retry_after_until_epoch_ms: Math.max(0, blockedUntil),
  };
}

export function createBrowserRequestGate({
  intervalMs = DEFAULT_REQUEST_INTERVAL_MS,
  now = defaultNow,
  sleep = defaultSleep,
  epochNow = Date.now,
  initialNextAvailableAt = Number.NEGATIVE_INFINITY,
  initialBlockedUntil = Number.NEGATIVE_INFINITY,
  persistState = null,
} = {}) {
  finiteNonNegative(intervalMs, "intervalMs");
  finiteTimestampOrNegativeInfinity(initialNextAvailableAt, "initialNextAvailableAt");
  finiteTimestampOrNegativeInfinity(initialBlockedUntil, "initialBlockedUntil");
  if (typeof now !== "function" || typeof sleep !== "function" || typeof epochNow !== "function" || (persistState !== null && typeof persistState !== "function")) throw new Error("browser request gate requires valid clock, sleep, and persistence inputs");

  let nextAvailableAt = initialNextAvailableAt;
  let blockedUntil = initialBlockedUntil;
  let queue = Promise.resolve();
  let persistence = Promise.resolve();
  let persistenceFailure = null;
  let enforcementFailure = null;
  let sequence = 0;
  const grants = [];
  const counters = {
    public_requests: 0,
    local_exempt_requests: 0,
    unsupported_requests_blocked: 0,
    websocket_connections_blocked: 0,
    retry_after_honored: 0,
    retry_after_invalid: 0,
  };

  function schedulePersistence() {
    if (!persistState) return Promise.resolve();
    const nextState = stateSnapshot({nextAvailableAt, blockedUntil});
    const turn = persistence.then(async () => {
      if (persistenceFailure) throw persistenceFailure;
      await persistState(nextState);
    });
    persistence = turn.catch((error) => {
      persistenceFailure ??= error;
    });
    return turn;
  }

  async function ensurePersistence() {
    await persistence;
    if (enforcementFailure) throw enforcementFailure;
    if (persistenceFailure) throw persistenceFailure;
  }

  async function acquire() {
    const turn = queue.then(async () => {
      await ensurePersistence();
      counters.public_requests += 1;
      for (;;) {
        if (enforcementFailure) throw enforcementFailure;
        if (persistenceFailure) throw persistenceFailure;
        const current = finiteNonNegative(now(), "request gate clock result");
        const due = Math.max(nextAvailableAt, blockedUntil);
        if (due <= current) {
          nextAvailableAt = current + intervalMs;
          await schedulePersistence();
          const grant = {sequence: ++sequence, released_at_epoch_ms: current};
          grants.push(grant);
          return grant;
        }
        await sleep(due - current);
      }
    });
    queue = turn.catch(() => {});
    return await turn;
  }

  function deferForRetryAfter(value) {
    const milliseconds = parseRetryAfterMilliseconds(value, {epochNow});
    if (milliseconds === null) {
      counters.retry_after_invalid += 1;
      return Promise.resolve(false);
    }
    const current = finiteNonNegative(now(), "request gate clock result");
    blockedUntil = Math.max(blockedUntil, current + milliseconds);
    counters.retry_after_honored += 1;
    return schedulePersistence().then(() => true);
  }

  return {
    intervalMs,
    acquire,
    deferForRetryAfter,
    failClosed(error) {
      enforcementFailure ??= error instanceof Error ? error : new Error("browser request-gate enforcement failed");
    },
    async flush() {
      await ensurePersistence();
    },
    recordLocalExempt() {
      counters.local_exempt_requests += 1;
    },
    recordUnsupportedRequestBlocked() {
      counters.unsupported_requests_blocked += 1;
    },
    recordWebSocketBlocked() {
      counters.websocket_connections_blocked += 1;
    },
    snapshot() {
      return {
        schema: "wikijump_full_parity.browser_request_gate.v1",
        interval_ms: intervalMs,
        ...stateSnapshot({nextAvailableAt, blockedUntil}),
        enforcement_failed: Boolean(enforcementFailure || persistenceFailure),
        grants: [...grants],
        ...counters,
      };
    },
  };
}

function validState(state) {
  return state?.schema === STATE_SCHEMA && Number.isSafeInteger(state.next_admissible_at_epoch_ms) && state.next_admissible_at_epoch_ms >= 0 && Number.isSafeInteger(state.retry_after_until_epoch_ms) && state.retry_after_until_epoch_ms >= 0;
}

async function secureJsonFile(filePath) {
  let stat;
  try {
    stat = await fs.lstat(filePath);
  } catch (error) {
    if (error?.code === "ENOENT") return null;
    throw error;
  }
  if (!stat.isFile() || stat.isSymbolicLink() || (stat.mode & 0o077) !== 0) throw new Error(`browser request gate state is not a private regular file: ${filePath}`);
  try {
    return JSON.parse(await fs.readFile(filePath, "utf8"));
  } catch {
    throw new Error(`browser request gate state is malformed: ${filePath}`);
  }
}

async function writeDurablePrivateJson(filePath, value) {
  const directory = path.dirname(filePath);
  await fs.mkdir(directory, {recursive: true, mode: 0o700});
  const existing = await secureJsonFile(filePath);
  if (existing !== null && !validState(existing)) throw new Error(`browser request gate state is malformed: ${filePath}`);
  const temporary = path.join(directory, `.${path.basename(filePath)}.${process.pid}.${randomUUID()}.tmp`);
  let handle = null;
  try {
    handle = await fs.open(temporary, fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_EXCL | (fsConstants.O_NOFOLLOW ?? 0), 0o600);
    await handle.writeFile(`${JSON.stringify(value)}\n`, "utf8");
    await handle.sync();
    await handle.close();
    handle = null;
    await fs.rename(temporary, filePath);
    const directoryHandle = await fs.open(directory, "r");
    try {
      await directoryHandle.sync();
    } finally {
      await directoryHandle.close();
    }
  } catch (error) {
    await handle?.close().catch(() => {});
    await fs.unlink(temporary).catch(() => {});
    throw error;
  }
}

export async function createPersistentBrowserRequestGate({statePath, intervalMs = DEFAULT_REQUEST_INTERVAL_MS, now = defaultNow, sleep = defaultSleep, epochNow = Date.now} = {}) {
  if (typeof statePath !== "string" || statePath === "") throw new Error("browser request gate requires a state path");
  const existing = await secureJsonFile(statePath);
  if (existing !== null && !validState(existing)) throw new Error(`browser request gate state is malformed: ${statePath}`);
  return createBrowserRequestGate({
    intervalMs,
    now,
    sleep,
    epochNow,
    initialNextAvailableAt: existing?.next_admissible_at_epoch_ms ?? Number.NEGATIVE_INFINITY,
    initialBlockedUntil: existing?.retry_after_until_epoch_ms ?? Number.NEGATIVE_INFINITY,
    persistState: async ({next_admissible_at_epoch_ms, retry_after_until_epoch_ms}) => {
      await writeDurablePrivateJson(statePath, {
        schema: STATE_SCHEMA,
        next_admissible_at_epoch_ms,
        retry_after_until_epoch_ms,
      });
    },
  });
}

async function abortRoute(route) {
  try {
    await route.abort("blockedbyclient");
    return true;
  } catch {
    // A route can already be disposed after navigation teardown. Never continue it after a failed gate path.
    return false;
  }
}

function requestCanUseResponseCache(request) {
  if (request.method() !== "GET" || request.resourceType() === "document") return false;
  const headers = request.headers();
  return headers.range === undefined && headers.authorization === undefined;
}

function responseCanBeCached(response, cache) {
  if (response.status() !== 200) return false;
  const headers = response.headers();
  const cacheControl = headers["cache-control"]?.toLowerCase() ?? "";
  if (/(?:^|,)\s*no-store(?:\s|,|$)/u.test(cacheControl)) return false;
  if (headers["set-cookie"] !== undefined || headers.vary?.trim() === "*") return false;
  const contentLength = headers["content-length"];
  return contentLength === undefined || (/^\d+$/u.test(contentLength) && Number(contentLength) <= cache.maxEntryBytes);
}

function reusableResponseHeaders(response) {
  const headers = {...response.headers()};
  delete headers["content-encoding"];
  delete headers["content-length"];
  delete headers["transfer-encoding"];
  return headers;
}

async function servePublicRoute(route, {gate, responseCache}) {
  const request = route.request();
  if (!responseCache || !requestCanUseResponseCache(request)) {
    responseCache?.recordBypass();
    await gate.acquire();
    await route.continue();
    return;
  }

  const cacheKey = request.url();
  const cached = responseCache.get(cacheKey);
  if (cached) {
    await route.fulfill(cached);
    return;
  }

  await gate.acquire();
  const response = await route.fetch({maxRedirects: 0});
  if (!responseCanBeCached(response, responseCache)) {
    responseCache.recordBypass();
    await route.fulfill({response});
    return;
  }
  const entry = {
    status: response.status(),
    headers: reusableResponseHeaders(response),
    body: await response.body(),
  };
  responseCache.store(cacheKey, entry);
  await route.fulfill(entry);
}

export async function installBrowserRequestGate(context, {gate, exemptOrigins = [], responseCache = null} = {}) {
  if (!gate || typeof gate.acquire !== "function" || typeof gate.deferForRetryAfter !== "function" || typeof gate.failClosed !== "function" || typeof gate.recordLocalExempt !== "function" || typeof gate.recordUnsupportedRequestBlocked !== "function" || typeof gate.recordWebSocketBlocked !== "function") throw new Error("browser request gate is malformed");
  if (!context || typeof context.route !== "function" || typeof context.routeWebSocket !== "function" || typeof context.on !== "function") throw new Error("browser context cannot enforce request-level capture controls");
  if (responseCache !== null && (typeof responseCache.get !== "function" || typeof responseCache.store !== "function" || typeof responseCache.recordBypass !== "function" || typeof responseCache.snapshot !== "function")) throw new Error("browser response cache is malformed");
  const exempt = normalizedOrigins(exemptOrigins);
  await context.route("**/*", async (route) => {
    try {
      const url = new URL(route.request().url());
      if (!new Set(["http:", "https:"]).has(url.protocol)) {
        gate.recordUnsupportedRequestBlocked();
        await abortRoute(route);
        return;
      }
      if (exempt.has(url.origin)) {
        gate.recordLocalExempt();
        await route.continue();
        return;
      }
      await servePublicRoute(route, {gate, responseCache});
    } catch {
      gate.recordUnsupportedRequestBlocked();
      await abortRoute(route);
    }
  });
  context.on("response", (response) => {
    let url;
    try {
      url = new URL(response.url());
    } catch {
      gate.failClosed(new Error("browser response URL cannot be inspected for request-gate enforcement"));
      return;
    }
    if (!new Set(["http:", "https:"]).has(url.protocol) || exempt.has(url.origin)) return;
    let retryAfter;
    try {
      retryAfter = response.headers()?.["retry-after"];
    } catch {
      gate.failClosed(new Error("browser response headers cannot be inspected for request-gate enforcement"));
      return;
    }
    if (retryAfter !== undefined) void gate.deferForRetryAfter(retryAfter).catch((error) => gate.failClosed(error));
  });
  await context.routeWebSocket("**/*", () => {
    gate.recordWebSocketBlocked();
    // Do not call connectToServer: Playwright keeps this as an in-page mock and no unmetered socket reaches the network.
  });
  return {exempt_origins: [...exempt].sort(), response_cache: responseCache};
}

function processStartTicksFromStat(text) {
  const closing = text.lastIndexOf(")");
  if (closing < 0) return null;
  const fields = text.slice(closing + 2).trim().split(/\s+/u);
  return /^\d+$/u.test(fields[19] ?? "") ? fields[19] : null;
}

async function currentProcessStartTicks(pid) {
  if (!Number.isSafeInteger(pid) || pid <= 0) return null;
  try {
    return processStartTicksFromStat(await fs.readFile(`/proc/${pid}/stat`, "utf8"));
  } catch (error) {
    if (error?.code === "ENOENT") return null;
    throw error;
  }
}

async function writeLockOwner(lockPath, owner) {
  const handle = await fs.open(lockPath, fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_EXCL | (fsConstants.O_NOFOLLOW ?? 0), 0o600);
  try {
    await handle.writeFile(`${JSON.stringify(owner)}\n`, "utf8");
    await handle.sync();
  } finally {
    await handle.close();
  }
  return await fs.lstat(lockPath);
}

function validOwner(owner) {
  return owner?.schema === LOCK_SCHEMA && typeof owner.hostname === "string" && Number.isSafeInteger(owner.pid) && owner.pid > 0 && /^\d+$/u.test(owner.process_start_ticks ?? "") && typeof owner.run_id === "string" && owner.run_id !== "" && STATE_CONFIRMATIONS.has(owner.state_confirmation);
}

export async function acquireBrowserCaptureLock({lockPath = DEFAULT_BROWSER_CAPTURE_LOCK, runId, processStartTicks = currentProcessStartTicks, hostname = os.hostname(), now = () => new Date().toISOString()} = {}) {
  if (typeof runId !== "string" || runId === "") throw new Error("browser capture lock requires a non-empty run ID");
  if (typeof processStartTicks !== "function" || typeof hostname !== "string" || hostname === "" || typeof now !== "function") throw new Error("browser capture lock inputs are malformed");
  const absolute = path.resolve(lockPath);
  await fs.mkdir(path.dirname(absolute), {recursive: true, mode: 0o700});
  const startTicks = await processStartTicks(process.pid);
  if (!startTicks) throw new Error("cannot bind browser capture lock to this process start time");
  let owner = {schema: LOCK_SCHEMA, hostname, pid: process.pid, process_start_ticks: startTicks, run_id: runId, acquired_at: now(), state_confirmation: "pending"};
  let lockStat;
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      lockStat = await writeLockOwner(absolute, owner);
      break;
    } catch (error) {
      if (error?.code !== "EEXIST" || attempt > 0) throw error;
      const stat = await fs.lstat(absolute);
      if (!stat.isFile() || stat.isSymbolicLink() || (stat.mode & 0o077) !== 0) throw new Error("browser capture lock is malformed");
      let existing;
      try {
        existing = JSON.parse(await fs.readFile(absolute, "utf8"));
      } catch {
        throw new Error("browser capture lock owner is malformed");
      }
      if (!validOwner(existing) || existing.hostname !== hostname) throw new Error("browser capture lock is held by an unverifiable owner");
      const existingTicks = await processStartTicks(existing.pid);
      if (existingTicks === existing.process_start_ticks) throw new Error(`browser capture source lock is held by run ${existing.run_id}`);
      if (existing.state_confirmation !== "sealed") {
        let persistedState = null;
        let persistedStateError = null;
        try {
          persistedState = await secureJsonFile(`${absolute}.state.json`);
        } catch (error) {
          persistedStateError = error;
        }
        if (!validState(persistedState)) {
          throw new Error(
            `browser capture source lock has unconfirmed request-gate state from run ${existing.run_id}; operator review is required`,
            persistedStateError === null ? undefined : {cause: persistedStateError},
          );
        }
      }
      const current = await fs.lstat(absolute);
      if (current.dev !== stat.dev || current.ino !== stat.ino) throw new Error("browser capture lock changed while recovering stale owner");
      await fs.unlink(absolute);
    }
  }
  if (!lockStat) throw new Error("browser capture lock was not acquired");
  let released = false;
  return {
    path: absolute,
    statePath: `${absolute}.state.json`,
    owner,
    async confirmState() {
      if (owner.state_confirmation === "sealed") return;
      const handle = await fs.open(absolute, fsConstants.O_WRONLY | (fsConstants.O_NOFOLLOW ?? 0));
      try {
        const current = await handle.stat();
        if (current.dev !== lockStat.dev || current.ino !== lockStat.ino) throw new Error("browser capture lock changed before request-gate state confirmation");
        owner = {...owner, state_confirmation: "sealed"};
        await handle.truncate(0);
        await handle.writeFile(`${JSON.stringify(owner)}\n`, "utf8");
        await handle.sync();
      } finally {
        await handle.close();
      }
    },
    async release() {
      if (released) return;
      if (owner.state_confirmation !== "sealed") throw new Error("browser capture lock cannot be released before request-gate state confirmation");
      released = true;
      let current = null;
      try {
        current = await fs.lstat(absolute);
      } catch (error) {
        if (error?.code !== "ENOENT") throw error;
      }
      if (current && current.dev === lockStat.dev && current.ino === lockStat.ino) await fs.unlink(absolute);
    },
  };
}
