import {createHash, randomUUID} from "node:crypto";
import fs from "node:fs/promises";
import {constants as fsConstants} from "node:fs";
import os from "node:os";
import path from "node:path";
import {isWikidotResourceHost} from "./resource-manifest.mjs";

export const DEFAULT_REQUEST_INTERVAL_MS = 0;
export const DEFAULT_BROWSER_CAPTURE_LOCK = "/var/tmp/wikijump-wikidot-browser-capture.lock";
const DEFAULT_RESPONSE_CACHE_MAX_ENTRIES = 512;
const DEFAULT_RESPONSE_CACHE_MAX_BYTES = 64 * 1024 * 1024;
const DEFAULT_RESPONSE_CACHE_MAX_ENTRY_BYTES = 8 * 1024 * 1024;
const DEFAULT_PUBLIC_EVIDENCE_CACHE_IDENTITY = "wikijump-candidate-public-evidence-cache-v1";
const DEFAULT_PUBLIC_EVIDENCE_CACHE_MAX_ENTRIES = 8192;
const DEFAULT_PUBLIC_EVIDENCE_CACHE_MAX_BYTES = 512 * 1024 * 1024;
const DEFAULT_PUBLIC_EVIDENCE_CACHE_MAX_ENTRY_BYTES = 32 * 1024 * 1024;
const RESPONSE_CACHE_STORE_SCHEMA = "wikijump_full_parity.browser_response_cache_store.v1";
const RESPONSE_CACHE_MANIFEST_LOCK_SCHEMA = "wikijump_full_parity.browser_response_cache_manifest_lock.v1";
const RESPONSE_CACHE_REDIRECT_STATUSES = new Set([301, 302, 303, 307, 308]);
const MAX_RESPONSE_CACHE_REDIRECTS = 10;
const HTTP_HEADER_NAME_RE = /^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/u;
const RESPONSE_CACHE_REQUEST_IDENTITY_HEADERS = [
  "if-match",
  "if-none-match",
  "if-modified-since",
  "if-unmodified-since",
  "if-range",
];

function responseCacheKey(value) {
  if (/^https:\/\/www\.wikidot\.com\/avatar\.php\?/u.test(value)) return value.replace(/(?:&amp;|&)timestamp=[^&]*/u, "");
  return value;
}

function reusableStoredHeaders(headers) {
  const reusable = normalizedHeaderRecord(headers);
  for (const name of ["content-encoding", "content-length", "transfer-encoding", "set-cookie", "set-cookie2"]) {
    delete reusable[name];
  }
  return reusable;
}

function normalizedHeaderRecord(headers) {
  const normalized = {};
  for (const [rawName, rawValue] of Object.entries(headers ?? {})) {
    const name = rawName.toLowerCase();
    if (!HTTP_HEADER_NAME_RE.test(name) || typeof rawValue !== "string") {
      throw new Error("browser response cache request headers are malformed");
    }
    normalized[name] = rawValue;
  }
  return normalized;
}

function varyHeaderNames(headers) {
  const raw = headers?.vary;
  if (raw === undefined) return [];
  if (typeof raw !== "string") throw new Error("browser response cache Vary header is malformed");
  const names = raw
    .split(",")
    .map((name) => name.trim().toLowerCase())
    .filter(Boolean);
  if (names.length === 0 || names.some((name) => name !== "*" && !HTTP_HEADER_NAME_RE.test(name))) {
    throw new Error("browser response cache Vary header is malformed");
  }
  if (names.includes("*") && names.length !== 1) throw new Error("browser response cache Vary header mixes * with named fields");
  return [...new Set(names)].sort();
}

function varyBindingForRequest(varyNames, requestHeaders) {
  if (varyNames.length === 0) return null;
  const normalized = normalizedHeaderRecord(requestHeaders);
  if (varyNames.length === 1 && varyNames[0] === "*") {
    return {
      names: ["*"],
      values: Object.fromEntries(Object.entries(normalized).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)),
    };
  }
  return {
    names: varyNames,
    values: Object.fromEntries(varyNames.map((name) => [name, Object.hasOwn(normalized, name) ? normalized[name] : null])),
  };
}

function canonicalVaryBinding(value) {
  if (value === null) return null;
  if (!value || !Array.isArray(value.names) || value.names.length === 0 || typeof value.values !== "object" || value.values === null || Array.isArray(value.values)) {
    throw new Error("browser response cache Vary binding is malformed");
  }
  const names = value.names.map((name) => {
    if (typeof name !== "string") throw new Error("browser response cache Vary binding is malformed");
    const normalized = name.toLowerCase();
    if (normalized !== "*" && !HTTP_HEADER_NAME_RE.test(normalized)) throw new Error("browser response cache Vary binding is malformed");
    return normalized;
  });
  const uniqueNames = [...new Set(names)].sort();
  if (uniqueNames.length !== names.length || (uniqueNames.includes("*") && uniqueNames.length !== 1)) {
    throw new Error("browser response cache Vary binding is malformed");
  }
  const values = {};
  for (const [rawName, rawValue] of Object.entries(value.values)) {
    const name = rawName.toLowerCase();
    if (!HTTP_HEADER_NAME_RE.test(name) || (rawValue !== null && typeof rawValue !== "string")) {
      throw new Error("browser response cache Vary binding is malformed");
    }
    values[name] = rawValue;
  }
  const sortedValues = Object.fromEntries(Object.entries(values).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0));
  if (uniqueNames[0] !== "*") {
    const valueNames = Object.keys(sortedValues).sort();
    if (JSON.stringify(valueNames) !== JSON.stringify(uniqueNames)) throw new Error("browser response cache Vary binding fields do not match Vary");
  } else if (Object.values(sortedValues).some((headerValue) => headerValue === null)) {
    throw new Error("browser response cache Vary * binding cannot contain absent headers");
  }
  return {names: uniqueNames, values: sortedValues};
}

function varyBindingsEqual(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function retainedEntryIdentity(baseKey, varyBinding) {
  return varyBinding === null
    ? `legacy\0${baseKey}`
    : `variant\0${baseKey}\0${JSON.stringify(varyBinding)}`;
}

function responseCacheRequestKey(request, overrideUrl = null, requestHeaders = null) {
  const url = responseCacheKey(overrideUrl ?? request.url());
  const method = request.method();
  const headers = requestHeaders ?? (typeof request.headers === "function" ? request.headers() : {});
  const range = headers.range;
  const identity = [];
  if (method !== "GET") identity.push(`method=${encodeURIComponent(method)}`);
  if (range !== undefined) identity.push(`range=${encodeURIComponent(range)}`);
  for (const name of RESPONSE_CACHE_REQUEST_IDENTITY_HEADERS) {
    if (headers[name] !== undefined) identity.push(`${name}=${encodeURIComponent(headers[name])}`);
  }
  return identity.length === 0
    ? url
    : `${url}#__wikijump_evidence_request=${identity.join("&")}`;
}

function unconditionalResponseCacheKey(baseKey) {
  const marker = "#__wikijump_evidence_request=";
  const markerIndex = baseKey.indexOf(marker);
  if (markerIndex === -1) return null;
  const identity = baseKey.slice(markerIndex + marker.length).split("&");
  const retained = identity.filter((part) => {
    const separator = part.indexOf("=");
    return !RESPONSE_CACHE_REQUEST_IDENTITY_HEADERS.includes(decodeURIComponent(separator === -1 ? part : part.slice(0, separator)));
  });
  return retained.length === 0
    ? baseKey.slice(0, markerIndex)
    : `${baseKey.slice(0, markerIndex)}${marker}${retained.join("&")}`;
}
const LOCK_SCHEMA = "wikijump_full_parity.browser_capture_lock.v1";
const STATE_SCHEMA = "wikijump_full_parity.browser_request_gate_state.v1";
const STATE_CONFIRMATIONS = new Set(["pending", "sealed"]);
const LOCAL_WIKIJUMP_HOST_RE = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.wikijump\.localhost$/u;
const WIKIDOT_STATIC_CDN_RE = /^[a-z0-9-]+\.cloudfront\.net$/u;
const WIKIDOT_INTERWIKI_GET_PATH_TYPES = new Map([
  ["/interwiki.js", "script"],
  ["/interwikiFrame.html", "document"],
  ["/resizeIframe.js", "script"],
  ["/styleFrame.html", "document"],
]);
const CAPTURE_DEPENDENCY_RESOURCE_TYPES = new Set([
  "stylesheet",
  "font",
  "image",
]);
const INTERWIKI_FRAME_PATHS = new Set(["/interwikiFrame.html", "/styleFrame.html"]);

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

export function defaultPublicEvidenceResponseCacheOptions({
  environment = process.env,
  homeDirectory = os.homedir(),
} = {}) {
  const xdgCacheHome = environment.XDG_CACHE_HOME;
  if (xdgCacheHome !== undefined && (typeof xdgCacheHome !== "string" || !path.isAbsolute(xdgCacheHome))) {
    throw new Error("XDG_CACHE_HOME must be an absolute path when set");
  }
  if (typeof homeDirectory !== "string" || !path.isAbsolute(homeDirectory)) {
    throw new Error("public evidence response cache home directory must be absolute");
  }
  const cacheHome = xdgCacheHome ?? path.join(homeDirectory, ".cache");
  return {
    persistentDir: path.join(cacheHome, "wikijump-verification", "candidate-public-evidence-v1"),
    persistentIdentity: DEFAULT_PUBLIC_EVIDENCE_CACHE_IDENTITY,
    cacheDocuments: true,
    evidenceReplay: true,
    maxEntries: DEFAULT_PUBLIC_EVIDENCE_CACHE_MAX_ENTRIES,
    maxBytes: DEFAULT_PUBLIC_EVIDENCE_CACHE_MAX_BYTES,
    maxEntryBytes: DEFAULT_PUBLIC_EVIDENCE_CACHE_MAX_ENTRY_BYTES,
  };
}

export function candidatePublicEvidenceResponseCacheOptions({
  environment = process.env,
  homeDirectory = os.homedir(),
} = {}) {
  const options = defaultPublicEvidenceResponseCacheOptions({environment, homeDirectory});
  const directory = environment.WIKIJUMP_CANDIDATE_RESPONSE_CACHE_DIR;
  const identity = environment.WIKIJUMP_CANDIDATE_RESPONSE_CACHE_IDENTITY;
  if ((directory === undefined) !== (identity === undefined)) {
    throw new Error("candidate response cache directory and identity must be configured together");
  }
  if (directory !== undefined) {
    if (typeof directory !== "string" || directory === "" || typeof identity !== "string" || identity === "") {
      throw new Error("candidate response cache directory and identity must be non-empty strings");
    }
    options.persistentDir = path.resolve(directory);
    options.persistentIdentity = identity;
  }
  return options;
}

async function ensurePrivateCacheDirectory(directory) {
  await fs.mkdir(directory, {recursive: true, mode: 0o700});
  const stat = await fs.lstat(directory);
  if (!stat.isDirectory() || stat.isSymbolicLink() || (stat.mode & 0o077) !== 0) throw new Error(`browser response cache directory is not private: ${directory}`);
}

async function readPrivateCacheManifest(filePath) {
  let stat;
  try {
    stat = await fs.lstat(filePath);
  } catch (error) {
    if (error?.code === "ENOENT") return null;
    throw error;
  }
  if (!stat.isFile() || stat.isSymbolicLink() || (stat.mode & 0o077) !== 0) throw new Error(`browser response cache manifest is not a private regular file: ${filePath}`);
  try {
    return JSON.parse(await fs.readFile(filePath, "utf8"));
  } catch {
    throw new Error(`browser response cache manifest is malformed: ${filePath}`);
  }
}

function validCacheManifestLockOwner(value) {
  return value?.schema === RESPONSE_CACHE_MANIFEST_LOCK_SCHEMA &&
    typeof value.hostname === "string" && value.hostname !== "" &&
    Number.isSafeInteger(value.pid) && value.pid > 0 &&
    /^\d+$/u.test(value.process_start_ticks ?? "");
}

async function syncDirectory(directory) {
  const handle = await fs.open(directory, fsConstants.O_RDONLY | (fsConstants.O_DIRECTORY ?? 0));
  try {
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function withPrivateCacheManifestLock(filePath, operation, {lockPath = `${filePath}.lock`} = {}) {
  const directory = path.dirname(filePath);
  await ensurePrivateCacheDirectory(directory);
  const startTicks = await currentProcessStartTicks(process.pid);
  if (!startTicks) throw new Error("cannot bind browser response cache manifest lock to this process");
  const owner = {
    schema: RESPONSE_CACHE_MANIFEST_LOCK_SCHEMA,
    hostname: os.hostname(),
    pid: process.pid,
    process_start_ticks: startTicks,
  };
  let lockStat = null;
  for (let attempt = 0; attempt < 6_000; attempt += 1) {
    let handle = null;
    try {
      handle = await fs.open(lockPath, fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_EXCL | (fsConstants.O_NOFOLLOW ?? 0), 0o600);
      await handle.writeFile(`${JSON.stringify(owner)}\n`, "utf8");
      await handle.sync();
      lockStat = await handle.stat();
      await handle.close();
      handle = null;
      await syncDirectory(directory);
      break;
    } catch (error) {
      await handle?.close().catch(() => {});
      if (error?.code !== "EEXIST") throw error;
      let stat;
      try {
        stat = await fs.lstat(lockPath);
      } catch (statError) {
        if (statError?.code === "ENOENT") continue;
        throw statError;
      }
      if (!stat.isFile() || stat.isSymbolicLink() || (stat.mode & 0o077) !== 0) throw new Error(`browser response cache manifest lock is malformed: ${lockPath}`);
      let existing;
      try {
        existing = JSON.parse(await fs.readFile(lockPath, "utf8"));
      } catch (readError) {
        if (readError?.code === "ENOENT") continue;
        if (Date.now() - stat.mtimeMs < 1_000) {
          await defaultSleep(5);
          continue;
        }
        throw new Error(`browser response cache manifest lock owner is malformed: ${lockPath}`);
      }
      if (!validCacheManifestLockOwner(existing) || existing.hostname !== owner.hostname) {
        throw new Error(`browser response cache manifest lock is held by an unverifiable owner: ${lockPath}`);
      }
      const existingTicks = await currentProcessStartTicks(existing.pid);
      if (existingTicks === existing.process_start_ticks) {
        await defaultSleep(5);
        continue;
      }
      let current;
      try {
        current = await fs.lstat(lockPath);
      } catch (currentError) {
        if (currentError?.code === "ENOENT") continue;
        throw currentError;
      }
      if (current.dev !== stat.dev || current.ino !== stat.ino) continue;
      await fs.unlink(lockPath);
      await syncDirectory(directory);
    }
  }
  if (!lockStat) throw new Error(`browser response cache manifest lock timed out: ${lockPath}`);
  let result;
  let operationError = null;
  try {
    result = await operation();
  } catch (error) {
    operationError = error;
  }
  let cleanupError = null;
  try {
    let current = null;
    try {
      current = await fs.lstat(lockPath);
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
    if (current && current.dev === lockStat.dev && current.ino === lockStat.ino) {
      await fs.unlink(lockPath);
      await syncDirectory(directory);
    }
  } catch (error) {
    cleanupError = error;
  }
  if (operationError !== null && cleanupError !== null) throw new AggregateError([operationError, cleanupError], "browser response cache operation and manifest-lock cleanup both failed");
  if (operationError !== null) throw operationError;
  if (cleanupError !== null) throw cleanupError;
  return result;
}

async function writePrivateCacheManifest(filePath, value) {
  const directory = path.dirname(filePath);
  await ensurePrivateCacheDirectory(directory);
  const existing = await readPrivateCacheManifest(filePath);
  if (existing !== null && existing.schema !== RESPONSE_CACHE_STORE_SCHEMA) throw new Error(`browser response cache manifest is malformed: ${filePath}`);
  const temporary = path.join(directory, `.${path.basename(filePath)}.${process.pid}.${randomUUID()}.tmp`);
  let handle = null;
  try {
    handle = await fs.open(temporary, fsConstants.O_WRONLY | fsConstants.O_CREAT | fsConstants.O_EXCL | (fsConstants.O_NOFOLLOW ?? 0), 0o600);
    await handle.writeFile(`${JSON.stringify(value)}\n`, "utf8");
    await handle.sync();
    await handle.close();
    handle = null;
    await fs.rename(temporary, filePath);
    await syncDirectory(directory);
  } catch (error) {
    await handle?.close().catch(() => {});
    await fs.unlink(temporary).catch(() => {});
    throw error;
  }
}

export function createBrowserResponseCache({maxEntries = DEFAULT_RESPONSE_CACHE_MAX_ENTRIES, maxBytes = DEFAULT_RESPONSE_CACHE_MAX_BYTES, maxEntryBytes = DEFAULT_RESPONSE_CACHE_MAX_ENTRY_BYTES, persistentDir = null, persistentIdentity = null, cacheDocuments = false, evidenceReplay = false} = {}) {
  positiveSafeInteger(maxEntries, "maxEntries");
  positiveSafeInteger(maxBytes, "maxBytes");
  positiveSafeInteger(maxEntryBytes, "maxEntryBytes");
  if (maxEntryBytes > maxBytes) throw new Error("maxEntryBytes cannot exceed maxBytes");
  if (persistentDir !== null && (typeof persistentDir !== "string" || persistentDir === "")) throw new Error("persistentDir must be a non-empty path or null");
  if (persistentDir !== null && (typeof persistentIdentity !== "string" || persistentIdentity === "")) throw new Error("persistentIdentity is required for a persistent browser response cache");
  if (persistentDir !== null && !evidenceReplay) throw new Error("persistent browser response caches are append-only evidence replay stores");

  const entries = new Map();
  const persistentPath = persistentDir === null ? null : path.resolve(persistentDir, "manifest.json");
  const retainedEntryMaxBytes = persistentPath === null ? maxEntryBytes : Number.MAX_SAFE_INTEGER;
  let bytes = 0;
  let hits = 0;
  let misses = 0;
  let stores = 0;
  let bypasses = 0;
  let evictions = 0;
  let loaded = persistentPath === null;
  let loadedEntries = 0;
  let mutationGeneration = 0;
  let persistedGeneration = 0;
  let flushQueue = Promise.resolve();
  const fills = new Map();
  let exactVariantHits = 0;
  let legacyVariantMisses = 0;
  let variantStores = 0;
  let acquisitionBarriers = new Set();

  function decodeManifestEntry(entry, sourcePath) {
    if (typeof entry?.key !== "string" || !/^https?:\/\//u.test(entry.key) || !isCacheableResponseStatus(entry.status, {evidenceReplay}) || typeof entry.headers !== "object" || entry.headers === null || typeof entry.body_base64 !== "string") {
      throw new Error(`browser response cache entry is malformed: ${sourcePath}`);
    }
    const body = Buffer.from(entry.body_base64, "base64");
    if (body.toString("base64") !== entry.body_base64 || body.length > retainedEntryMaxBytes) {
      throw new Error(`browser response cache entry exceeds limits or is malformed: ${sourcePath}`);
    }
    const headers = reusableStoredHeaders(entry.headers);
    const varyNames = varyHeaderNames(headers);
    const varyBinding = entry.vary_binding === undefined
      ? null
      : canonicalVaryBinding(entry.vary_binding);
    if (varyBinding !== null && JSON.stringify(varyBinding.names) !== JSON.stringify(varyNames)) {
      throw new Error(`browser response cache Vary binding does not match response Vary header: ${sourcePath}`);
    }
    const baseKey = responseCacheKey(entry.key);
    return {
      key: retainedEntryIdentity(baseKey, varyBinding),
      value: {baseKey, status: entry.status, headers, body, varyBinding},
      canonicalized: baseKey !== entry.key || JSON.stringify(headers) !== JSON.stringify(entry.headers),
    };
  }

  function entriesEqual(left, right) {
    if (left.status !== right.status || !left.body.equals(right.body)) return false;
    const normalize = (headers) => Object.entries(headers).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0);
    return JSON.stringify(normalize(left.headers)) === JSON.stringify(normalize(right.headers));
  }

  function mergeRetainedEntry(identity, value, sourcePath) {
    const existing = entries.get(identity);
    if (existing) {
      if (!entriesEqual(existing, value)) {
        throw new Error(`browser response cache retained response conflicts with persisted evidence: ${sourcePath}`);
      }
      return false;
    }
    if (persistentPath === null && (entries.size + 1 > maxEntries || bytes + value.body.length > maxBytes)) {
      throw new Error(`browser response cache merged manifest exceeds limits: ${sourcePath}`);
    }
    entries.set(identity, value);
    bytes += value.body.length;
    return true;
  }

  function matchingRetainedEntry(baseKey, requestHeaders, {count = true} = {}) {
    if (acquisitionBarriers.has(baseKey)) {
      if (count) throw new Error(`browser response cache refuses to reacquire an identity with uncertain prior acquisition: ${baseKey}`);
      return null;
    }
    const normalizedRequestHeaders = requestHeaders === null ? null : normalizedHeaderRecord(requestHeaders);
    let retained304 = false;
    for (const candidateKey of [baseKey, unconditionalResponseCacheKey(baseKey)].filter((key, index, keys) => key !== null && keys.indexOf(key) === index)) {
      let legacy = null;
      for (const entry of entries.values()) {
        if (entry.baseKey !== candidateKey) continue;
        if (entry.status === 304) {
          retained304 = true;
          continue;
        }
        if (entry.varyBinding !== null) {
          if (entry.varyBinding.names.length === 1 && entry.varyBinding.names[0] === "*") {
            throw new Error(`browser response cache retained Vary * response is not exact-replayable: ${candidateKey}`);
          }
          if (normalizedRequestHeaders === null) continue;
          const expected = varyBindingForRequest(entry.varyBinding.names, normalizedRequestHeaders);
          if (varyBindingsEqual(entry.varyBinding, expected)) {
            if (count) {
              hits += 1;
              exactVariantHits += 1;
            }
            return {status: entry.status, headers: entry.headers, body: entry.body};
          }
          continue;
        }
        legacy = entry;
      }
      if (legacy !== null) {
        const legacyVary = varyHeaderNames(legacy.headers);
        if (legacyVary.length === 0) {
          if (count) hits += 1;
          return {status: legacy.status, headers: legacy.headers, body: legacy.body};
        }
        if (count) legacyVariantMisses += 1;
      }
    }
    if (retained304) throw new Error(`browser response cache retained 304 response is not standalone-replayable: ${baseKey}`);
    if (count) misses += 1;
    return null;
  }

  async function mergeCurrentManifest() {
    const current = await readPrivateCacheManifest(persistentPath);
    if (current === null) return;
    if (current.schema !== RESPONSE_CACHE_STORE_SCHEMA || current.identity !== persistentIdentity || !Array.isArray(current.entries)) {
      throw new Error(`browser response cache identity or schema mismatch: ${persistentPath}`);
    }
    if (current.acquisition_barriers !== undefined && (!Array.isArray(current.acquisition_barriers) || current.acquisition_barriers.some((key) => typeof key !== "string" || !/^https?:\/\//u.test(key)))) {
      throw new Error(`browser response cache acquisition barriers are malformed: ${persistentPath}`);
    }
    for (const raw of current.entries) {
      const decoded = decodeManifestEntry(raw, persistentPath);
      if (mergeRetainedEntry(decoded.key, decoded.value, persistentPath)) mutationGeneration += 1;
      if (decoded.canonicalized) mutationGeneration += 1;
    }
    acquisitionBarriers = new Set((current.acquisition_barriers ?? []).map(responseCacheKey));
  }

  async function writeCurrentManifest() {
    await writePrivateCacheManifest(persistentPath, {
      schema: RESPONSE_CACHE_STORE_SCHEMA,
      identity: persistentIdentity,
      entries: [...entries.values()].map((entry) => ({
        key: entry.baseKey,
        ...(entry.varyBinding === null ? {} : {vary_binding: entry.varyBinding, body_representation: "decoded"}),
        status: entry.status,
        headers: entry.headers,
        body_base64: entry.body.toString("base64"),
      })),
      ...(acquisitionBarriers.size === 0 ? {} : {acquisition_barriers: [...acquisitionBarriers].sort()}),
    });
  }

  return {
    maxEntryBytes: retainedEntryMaxBytes,
    cacheDocuments,
    evidenceReplay,
    async load() {
      if (loaded) return;
      await ensurePrivateCacheDirectory(path.dirname(persistentPath));
      const manifest = await readPrivateCacheManifest(persistentPath);
      if (manifest === null) {
        loaded = true;
        return;
      }
      if (manifest.schema !== RESPONSE_CACHE_STORE_SCHEMA || manifest.identity !== persistentIdentity || !Array.isArray(manifest.entries)) throw new Error(`browser response cache identity or schema mismatch: ${persistentPath}`);
      if (manifest.acquisition_barriers !== undefined && (!Array.isArray(manifest.acquisition_barriers) || manifest.acquisition_barriers.some((key) => typeof key !== "string" || !/^https?:\/\//u.test(key)))) {
        throw new Error(`browser response cache acquisition barriers are malformed: ${persistentPath}`);
      }
      for (const entry of manifest.entries) {
        const decoded = decodeManifestEntry(entry, persistentPath);
        if (entries.has(decoded.key)) throw new Error(`browser response cache entry exceeds limits or is duplicated: ${persistentPath}`);
        mergeRetainedEntry(decoded.key, decoded.value, persistentPath);
        if (decoded.canonicalized) mutationGeneration += 1;
      }
      acquisitionBarriers = new Set((manifest.acquisition_barriers ?? []).map(responseCacheKey));
      loadedEntries = entries.size;
      loaded = true;
    },
    async flush() {
      if (persistentPath === null || mutationGeneration <= persistedGeneration) return;
      const turn = flushQueue.then(async () => {
        if (mutationGeneration <= persistedGeneration) return;
        const generation = mutationGeneration;
        await withPrivateCacheManifestLock(persistentPath, async () => {
          await mergeCurrentManifest();
          await writeCurrentManifest();
        });
        persistedGeneration = Math.max(generation, mutationGeneration);
      });
      flushQueue = turn.catch(() => {});
      await turn;
    },
    async withFill(key, options, producer) {
      if (!loaded) throw new Error("persistent browser response cache must be loaded before fill");
      if (typeof options === "function") {
        producer = options;
        options = {};
      }
      if (typeof producer !== "function") throw new Error("browser response cache fill producer must be a function");
      const cacheKey = responseCacheKey(key);
      const requestHeaders = options?.requestHeaders ?? null;
      const retained = matchingRetainedEntry(cacheKey, requestHeaders, {count: false});
      if (retained) return retained;
      const fillIdentity = `${cacheKey}\0${JSON.stringify(normalizedHeaderRecord(requestHeaders ?? {}))}`;
      const active = fills.get(fillIdentity);
      if (active) return await active;
      const fill = persistentPath === null
        ? Promise.resolve().then(() => producer({deferFlush: false, markAcquisitionStarted: async () => {}}))
        : withPrivateCacheManifestLock(persistentPath, async () => {
            const diskRetained = await withPrivateCacheManifestLock(persistentPath, async () => {
              await mergeCurrentManifest();
              return matchingRetainedEntry(cacheKey, requestHeaders, {count: false});
            });
            if (diskRetained) {
              hits += 1;
              if (varyHeaderNames(diskRetained.headers).length > 0) exactVariantHits += 1;
              return diskRetained;
            }
            if (acquisitionBarriers.has(cacheKey)) {
              throw new Error(`browser response cache refuses to reacquire an identity with uncertain prior acquisition: ${cacheKey}`);
            }
            let acquisitionStarted = false;
            const markAcquisitionStarted = async () => {
              if (acquisitionStarted) return;
              await withPrivateCacheManifestLock(persistentPath, async () => {
                await mergeCurrentManifest();
                acquisitionBarriers.add(cacheKey);
                mutationGeneration += 1;
                await writeCurrentManifest();
                persistedGeneration = mutationGeneration;
              });
              acquisitionStarted = true;
            };
            const value = await producer({deferFlush: true, markAcquisitionStarted});
            await withPrivateCacheManifestLock(persistentPath, async () => {
              await mergeCurrentManifest();
              if (acquisitionStarted) {
                acquisitionBarriers.delete(cacheKey);
                mutationGeneration += 1;
              }
              await writeCurrentManifest();
              persistedGeneration = mutationGeneration;
            });
            return value;
          }, {
            lockPath: `${persistentPath}.fill-${createHash("sha256").update(cacheKey).digest("hex")}.lock`,
          });
      fills.set(fillIdentity, fill);
      try {
        return await fill;
      } finally {
        if (fills.get(fillIdentity) === fill) fills.delete(fillIdentity);
      }
    },
    lookup(key, {requestHeaders = null} = {}) {
      if (!loaded) throw new Error("persistent browser response cache must be loaded before lookup");
      return matchingRetainedEntry(responseCacheKey(key), requestHeaders);
    },
    get(key) {
      if (!loaded) throw new Error("persistent browser response cache must be loaded before lookup");
      return matchingRetainedEntry(responseCacheKey(key), null);
    },
    store(key, entry, {requestHeaders = null} = {}) {
      if (!loaded) throw new Error("persistent browser response cache must be loaded before storage");
      if (!Buffer.isBuffer(entry?.body) || entry.body.length > retainedEntryMaxBytes) {
        bypasses += 1;
        return false;
      }
      const baseKey = responseCacheKey(key);
      const headers = reusableStoredHeaders(entry.headers ?? {});
      const varyNames = varyHeaderNames(headers);
      let varyBinding = null;
      if (varyNames.length > 0) {
        if (requestHeaders === null) {
          bypasses += 1;
          return false;
        }
        varyBinding = varyBindingForRequest(varyNames, requestHeaders);
      }
      const cacheKey = retainedEntryIdentity(baseKey, varyBinding);
      const normalizedEntry = {baseKey, status: entry.status, headers, body: entry.body, varyBinding};
      const existing = entries.get(cacheKey);
      if (existing) {
        if (persistentPath !== null && !entriesEqual(existing, normalizedEntry)) {
          bypasses += 1;
          return false;
        }
        entries.delete(cacheKey);
        bytes -= existing.body.length;
      }
      while (persistentPath === null && (entries.size >= maxEntries || bytes + normalizedEntry.body.length > maxBytes)) {
        const oldestKey = entries.keys().next().value;
        if (oldestKey === undefined) break;
        const oldest = entries.get(oldestKey);
        entries.delete(oldestKey);
        bytes -= oldest.body.length;
        evictions += 1;
      }
      entries.set(cacheKey, normalizedEntry);
      bytes += normalizedEntry.body.length;
      stores += 1;
      if (varyBinding !== null) variantStores += 1;
      mutationGeneration += 1;
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
        max_entry_bytes: retainedEntryMaxBytes,
        lookup_key: evidenceReplay ? "exact_url_method_range_conditional_vary" : "exact_url",
        lifetime: persistentPath === null ? "browser_context" : "persistent",
        documents_cached: cacheDocuments,
        evidence_replay: evidenceReplay,
        retention_policy: persistentPath === null ? "bounded_lru" : "append_only_no_eviction",
        exact_variant_hits: exactVariantHits,
        legacy_variant_misses: legacyVariantMisses,
        variant_stores: variantStores,
        ...(persistentPath === null ? {} : {
          acquisition_barriers: acquisitionBarriers.size,
          persistent_identity: persistentIdentity,
          persistent_entries_loaded: loadedEntries,
        }),
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
  if (url.protocol !== "https:" || url.username || url.password || !LOCAL_WIKIJUMP_HOST_RE.test(url.hostname)) {
    throw new Error(`local capture URL must be an HTTPS *.wikijump.localhost origin without credentials: ${value}`);
  }
  const site = url.hostname.slice(0, -".wikijump.localhost".length);
  const port = url.port === "" ? "" : `:${url.port}`;
  return [url.origin, `https://${site}.wjfiles.localhost${port}`];
}

export function isWikidotCapturePublicOrigin(value, resourceType, method, initiatorUrl = null) {
  const url = value instanceof URL ? value : new URL(value);
  const hostname = url.hostname.toLowerCase();
  const interwikiScript = url.protocol === "https:" && hostname === "interwiki.scpwiki.com" && method === "GET" && resourceType === "script" && ["/interwiki.js", "/resizeIframe.js"].includes(url.pathname);
  let initiatedByInterwikiFrame = false;
  if (interwikiScript && typeof initiatorUrl === "string") {
    try {
      const initiator = new URL(initiatorUrl);
      initiatedByInterwikiFrame = initiator.protocol === "https:" && initiator.hostname === hostname && INTERWIKI_FRAME_PATHS.has(initiator.pathname);
    } catch {
      initiatedByInterwikiFrame = false;
    }
  }
  return new Set(["http:", "https:"]).has(url.protocol) &&
    !url.username &&
    !url.password &&
    !url.port &&
    (hostname === "wikidot.com" ||
      isWikidotResourceHost(hostname) ||
      (WIKIDOT_STATIC_CDN_RE.test(hostname) && url.pathname.startsWith("/v--")) ||
      (url.protocol === "https:" &&
        hostname === "interwiki.scpwiki.com" &&
        method === "GET" &&
        WIKIDOT_INTERWIKI_GET_PATH_TYPES.get(url.pathname) === resourceType &&
        (resourceType !== "script" || initiatedByInterwikiFrame)));
}

export function isCaptureDependencyResourceType(resourceType) {
  return CAPTURE_DEPENDENCY_RESOURCE_TYPES.has(resourceType);
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
    external_network_requests: 0,
    synthetic_public_admissions: 0,
    local_exempt_requests: 0,
    unsupported_requests_blocked: 0,
    websocket_connections_blocked: 0,
    retry_after_honored: 0,
    retry_after_invalid: 0,
  };
  const blockedHosts = new Map();
  const blockedHostsByFixture = new Map();
  let activeFixtureId = null;

  function recordBlockedHost(hostname) {
    const host = typeof hostname === "string" && hostname !== "" ? hostname.toLowerCase() : "<unsupported>";
    blockedHosts.set(host, (blockedHosts.get(host) ?? 0) + 1);
    const fixtureKey = activeFixtureId ?? "<unattributed>";
    const fixtureHosts = blockedHostsByFixture.get(fixtureKey) ?? new Map();
    fixtureHosts.set(host, (fixtureHosts.get(host) ?? 0) + 1);
    blockedHostsByFixture.set(fixtureKey, fixtureHosts);
  }

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
      counters.external_network_requests += 1;
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
    recordSyntheticPublicAdmission(count = 1) {
      if (!Number.isSafeInteger(count) || count <= 0) {
        throw new Error("synthetic public admission count must be a positive safe integer");
      }
      const current = finiteNonNegative(now(), "request gate clock result");
      counters.public_requests += count;
      counters.synthetic_public_admissions += count;
      for (let index = 0; index < count; index += 1) {
        grants.push({ sequence: ++sequence, released_at_epoch_ms: current });
      }
    },
    recordLocalExempt() {
      counters.local_exempt_requests += 1;
    },
    recordUnsupportedRequestBlocked(hostname = null) {
      counters.unsupported_requests_blocked += 1;
      recordBlockedHost(hostname);
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
        blocked_hosts: Object.fromEntries([...blockedHosts].sort(([left], [right]) => left.localeCompare(right))),
        blocked_hosts_by_fixture: Object.fromEntries([...blockedHostsByFixture].sort(([left], [right]) => left.localeCompare(right)).map(([fixture, hosts]) => [fixture, Object.fromEntries([...hosts].sort(([left], [right]) => left.localeCompare(right)))])),
        ...counters,
      };
    },
    setActiveFixture(fixtureId) {
      if (fixtureId !== null && (typeof fixtureId !== "string" || fixtureId === "")) throw new Error("browser request-gate fixture id must be a non-empty string or null");
      activeFixtureId = fixtureId;
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

async function requestHeadersForResponseCache(request) {
  return normalizedHeaderRecord(
    typeof request.allHeaders === "function"
      ? await request.allHeaders()
      : typeof request.headers === "function"
        ? request.headers()
        : request.headers ?? {},
  );
}

async function requestCanUseResponseCache(request, responseCache, requestHeaders = null) {
  const method = request.method();
  if (
    (method !== "GET" && !(responseCache.evidenceReplay && method === "HEAD")) ||
    (request.resourceType() === "document" && !responseCache.cacheDocuments)
  ) return false;
  const requestUrl = new URL(request.url());
  if (requestUrl.username || requestUrl.password) return false;
  const headers = requestHeaders ?? await requestHeadersForResponseCache(request);
  if (headers.authorization !== undefined || headers.cookie !== undefined) return false;
  return headers.range === undefined || responseCache.evidenceReplay;
}

export async function lookupBrowserEvidenceResponse({responseCache, request, targetUrl = request.url()}) {
  const target = new URL(targetUrl);
  if (target.username || target.password) return null;
  const requestHeaders = await requestHeadersForResponseCache(request);
  if (!await requestCanUseResponseCache(request, responseCache, requestHeaders)) return null;
  const cacheKey = responseCacheRequestKey(request, target.href, requestHeaders);
  return responseCache.lookup(cacheKey, {requestHeaders});
}

function isCacheableResponseStatus(status, {evidenceReplay = false} = {}) {
  if (status === 200 || (status >= 400 && status < 500)) return true;
  return evidenceReplay && status >= 200 && status < 600;
}

async function responseCanBeCached(response, cache) {
  if (!isCacheableResponseStatus(response.status(), {evidenceReplay: cache.evidenceReplay})) return false;
  const headers = typeof response.allHeaders === "function"
    ? await response.allHeaders()
    : response.headers();
  const contentLength = headers["content-length"];
  if (contentLength !== undefined && (!/^\d+$/u.test(contentLength) || Number(contentLength) > cache.maxEntryBytes)) return false;
  if (cache.evidenceReplay) return true;
  const cacheControl = headers["cache-control"]?.toLowerCase() ?? "";
  const cacheDirectives = new Map(
    cacheControl
      .split(",")
      .map((directive) => directive.trim())
      .filter(Boolean)
      .map((directive) => {
        const separator = directive.indexOf("=");
        return separator === -1
          ? [directive, null]
          : [directive.slice(0, separator).trim(), directive.slice(separator + 1).trim()];
      }),
  );
  if (
    ["no-store", "no-cache", "private", "must-revalidate"].some((directive) =>
      cacheDirectives.has(directive),
    )
  ) {
    return false;
  }
  const maxAge = cacheDirectives.get("max-age");
  if (maxAge !== undefined) {
    const normalizedMaxAge = maxAge?.replace(/^"|"$/gu, "") ?? "";
    if (!/^\d+$/u.test(normalizedMaxAge) || Number(normalizedMaxAge) === 0) return false;
  }
  const variesByRequestHeader =
    headers.vary
      ?.split(",")
      .map((header) => header.trim())
      .some(Boolean) ?? false;
  if (headers["set-cookie"] !== undefined || variesByRequestHeader) return false;
  return true;
}

async function reusableResponseHeaders(response) {
  return reusableStoredHeaders(typeof response.allHeaders === "function"
    ? await response.allHeaders()
    : response.headers());
}

function redirectTarget(entry, baseUrl) {
  if (!RESPONSE_CACHE_REDIRECT_STATUSES.has(entry?.status)) return null;
  const location = entry?.headers?.location;
  if (typeof location !== "string" || location === "") return null;
  const target = new URL(location, baseUrl);
  if (
    !new Set(["http:", "https:"]).has(target.protocol) ||
    target.username ||
    target.password
  ) {
    throw new Error(`browser response cache redirect target is unsupported: ${target.href}`);
  }
  return target;
}

async function retainedEntryFromResponse(responseCache, key, response, {deferFlush = false, requestHeaders = null} = {}) {
  if (!await responseCanBeCached(response, responseCache)) return null;
  const body = await response.body();
  if (body.length > responseCache.maxEntryBytes) return null;
  const entry = {
    status: response.status(),
    headers: await reusableResponseHeaders(response),
    body,
  };
  if (!responseCache.store(key, entry, {requestHeaders})) return null;
  if (!deferFlush) await responseCache.flush();
  return entry;
}

export async function fetchBrowserEvidenceResponse({
  route,
  gate,
  responseCache,
  request = route.request(),
  targetUrl = request.url(),
}) {
  const target = new URL(targetUrl);
  if (target.username || target.password) {
    throw new Error(`browser response cache cannot retain credential-bearing URL ${target.href}`);
  }
  const requestHeaders = await requestHeadersForResponseCache(request);
  if (responseCache === null || !await requestCanUseResponseCache(request, responseCache, requestHeaders)) {
    throw new Error(`browser response cache cannot retain ${target.href}`);
  }
  const cacheKey = responseCacheRequestKey(request, target.href, requestHeaders);
  return await responseCache.withFill(cacheKey, {requestHeaders}, async ({deferFlush, markAcquisitionStarted}) => {
    await gate.acquire();
    await markAcquisitionStarted();
    const response = await route.fetch(
      target.href === request.url()
        ? {maxRedirects: 0}
        : {url: target.href, maxRedirects: 0},
    );
    const entry = await retainedEntryFromResponse(responseCache, cacheKey, response, {deferFlush, requestHeaders});
    if (entry === null) {
      responseCache.recordBypass();
      throw new Error(`browser response cache could not retain ${target.href}`);
    }
    return entry;
  });
}

export async function resolveEvidenceReplaySubresourceRedirect({
  route,
  gate,
  responseCache,
  resourceType,
  sourceUrl,
  entry,
  request = route.request(),
  cacheOnly = false,
  fetchRetainedEntry = null,
}) {
  if (
    !responseCache?.evidenceReplay ||
    !isCaptureDependencyResourceType(resourceType) ||
    !RESPONSE_CACHE_REDIRECT_STATUSES.has(entry?.status)
  ) {
    return null;
  }

  const visited = new Set([sourceUrl]);
  const requestHeaders = await requestHeadersForResponseCache(request);
  if (!await requestCanUseResponseCache(request, responseCache, requestHeaders)) return null;
  let currentUrl = sourceUrl;
  let currentEntry = entry;
  for (let redirects = 0; redirects < MAX_RESPONSE_CACHE_REDIRECTS; redirects += 1) {
    const target = redirectTarget(currentEntry, currentUrl);
    if (target === null) return null;
    if (visited.has(target.href)) {
      throw new Error("browser response cache redirect cycle detected");
    }
    visited.add(target.href);

    const targetCacheKey = responseCacheRequestKey(request, target.href, requestHeaders);
    let targetEntry = responseCache.lookup(targetCacheKey, {requestHeaders});
    if (targetEntry === null) {
      if (cacheOnly) {
        throw new Error(`candidate response cache miss: ${target.href}`);
      } else if (fetchRetainedEntry !== null) {
        targetEntry = await fetchRetainedEntry(target.href);
      } else {
        await gate.acquire();
        const response = await route.fetch({ url: target.href, maxRedirects: 0 });
        targetEntry = await retainedEntryFromResponse(responseCache, targetCacheKey, response, {requestHeaders});
        if (targetEntry === null) {
          if (RESPONSE_CACHE_REDIRECT_STATUSES.has(response.status())) {
            responseCache.recordBypass();
            throw new Error(`browser response cache could not retain redirect target: ${target.href}`);
          }
          responseCache.recordBypass();
          return { finalUrl: target.href, entry: null, response };
        }
      }
      if (targetEntry === null) {
        throw new Error(`browser response cache could not retain redirect target: ${target.href}`);
      }
    }

    currentUrl = target.href;
    currentEntry = targetEntry;
    if (!RESPONSE_CACHE_REDIRECT_STATUSES.has(currentEntry.status)) {
      return { finalUrl: currentUrl, entry: currentEntry, response: null };
    }
  }
  throw new Error("browser response cache redirect limit exceeded");
}

export function evidenceReplaySubresourceFulfillment(resourceType, resolved) {
  if (!resolved?.entry) return null;
  if (resourceType === "stylesheet" && resolved.entry.status === 200) {
    const body = Buffer.from(`@import url(${JSON.stringify(resolved.finalUrl)});\n`);
    return {
      status: 200,
      headers: { "content-type": "text/css; charset=utf-8" },
      body,
    };
  }
  return {
    status: resolved.entry.status,
    headers: resolved.entry.headers,
    body: resolved.entry.body,
  };
}

async function fulfillRetainedEntry(route, { gate, responseCache, request, entry, cacheOnly = false, fetchRetainedEntry = null }) {
  const resolved = await resolveEvidenceReplaySubresourceRedirect({
    route,
    gate,
    responseCache,
    resourceType: request.resourceType(),
    sourceUrl: request.url(),
    entry,
    request,
    cacheOnly,
    fetchRetainedEntry,
  });
  if (resolved === null) {
    await route.fulfill(entry);
    return;
  }
  if (resolved.entry !== null) {
    await route.fulfill(
      evidenceReplaySubresourceFulfillment(request.resourceType(), resolved),
    );
    return;
  }
  await route.fulfill({ response: resolved.response });
}

async function servePublicRoute(route, {gate, responseCache, cacheOnly = false, fetchRetainedEntry = null}) {
  const request = route.request();
  const requestHeaders = await requestHeadersForResponseCache(request);
  if (cacheOnly) {
    if (!responseCache || !await requestCanUseResponseCache(request, responseCache, requestHeaders)) {
      responseCache?.recordBypass();
      throw new Error(`candidate response cache cannot serve ${request.url()}`);
    }
    const cached = responseCache.lookup(responseCacheRequestKey(request, null, requestHeaders), {requestHeaders});
    if (cached === null) throw new Error(`candidate response cache miss: ${request.url()}`);
    await fulfillRetainedEntry(route, { gate, responseCache, request, entry: cached, cacheOnly: true });
    return;
  }
  if (!responseCache || !await requestCanUseResponseCache(request, responseCache, requestHeaders)) {
    responseCache?.recordBypass();
    await gate.acquire();
    await route.continue();
    return;
  }

  const cacheKey = responseCacheRequestKey(request, null, requestHeaders);
  const cached = responseCache.lookup(cacheKey, {requestHeaders});
  if (cached) {
    await fulfillRetainedEntry(route, { gate, responseCache, request, entry: cached, fetchRetainedEntry });
    return;
  }

  if (fetchRetainedEntry !== null) {
    const entry = await fetchRetainedEntry(request.url());
    await fulfillRetainedEntry(route, { gate, responseCache, request, entry, fetchRetainedEntry });
    return;
  }

  await gate.acquire();
  const response = await route.fetch({maxRedirects: 0});
  if (!await responseCanBeCached(response, responseCache)) {
    responseCache.recordBypass();
    await route.fulfill({response});
    return;
  }
  const entry = await retainedEntryFromResponse(responseCache, cacheKey, response, {requestHeaders});
  if (entry === null) {
    responseCache.recordBypass();
    await route.fulfill({response});
    return;
  }
  await fulfillRetainedEntry(route, { gate, responseCache, request, entry });
}

/**
 * @param {import("@playwright/test").BrowserContext} context
 * @param {{
 *   gate: object
 *   exemptOrigins?: string[]
 *   responseCache?: object | null
 *   publicOriginPredicate?: ((value: string, resourceType: string, method: string) => boolean) | null
 *   cacheOnly?: boolean
 * }} [options]
 */
export async function installBrowserRequestGate(context, {gate, exemptOrigins = [], responseCache = null, publicOriginPredicate = null, cacheOnly = false} = {}) {
  if (!gate || typeof gate.acquire !== "function" || typeof gate.deferForRetryAfter !== "function" || typeof gate.failClosed !== "function" || typeof gate.recordLocalExempt !== "function" || typeof gate.recordUnsupportedRequestBlocked !== "function" || typeof gate.recordWebSocketBlocked !== "function") throw new Error("browser request gate is malformed");
  if (!context || typeof context.route !== "function" || typeof context.routeWebSocket !== "function" || typeof context.on !== "function") throw new Error("browser context cannot enforce request-level capture controls");
  if (responseCache !== null && (typeof responseCache.get !== "function" || typeof responseCache.lookup !== "function" || typeof responseCache.store !== "function" || typeof responseCache.withFill !== "function" || typeof responseCache.recordBypass !== "function" || typeof responseCache.snapshot !== "function")) throw new Error("browser response cache is malformed");
  if (publicOriginPredicate !== null && typeof publicOriginPredicate !== "function") throw new Error("browser request-gate public origin predicate is malformed");
  const exempt = normalizedOrigins(exemptOrigins);
  const attributedAborts = new WeakMap();
  if (exempt.size > 0) {
    context.on("request", (request) => {
      try {
        const url = new URL(request.url());
        if (new Set(["http:", "https:"]).has(url.protocol) && exempt.has(url.origin)) {
          gate.recordLocalExempt();
        }
      } catch {
        // Malformed/non-HTTP requests remain the route handler's fail-closed concern.
      }
    });
  }
  const abortWithAttribution = async (route, decision) => {
    const request = route.request();
    attributedAborts.set(request, Object.freeze({
      decision,
      abort_reason: "blockedbyclient",
    }));
    if (!(await abortRoute(route))) attributedAborts.delete(request);
  };
  const fetchRetainedEntry = (route, request, targetUrl = request.url()) => fetchBrowserEvidenceResponse({
    route,
    gate,
    responseCache,
    request,
    targetUrl,
  });
  const routePattern = exempt.size === 0
    ? "**/*"
    : (url) => !exempt.has(url.origin);
  await context.route(routePattern, async (route) => {
    try {
      const url = new URL(route.request().url());
      if (!new Set(["http:", "https:"]).has(url.protocol)) {
        gate.recordUnsupportedRequestBlocked();
        await abortWithAttribution(route, "unsupported_protocol");
        return;
      }
      if (
        publicOriginPredicate !== null &&
        !publicOriginPredicate(
          url,
          route.request().resourceType(),
          route.request().method(),
          typeof route.request().frame === "function" ? route.request().frame()?.url() : null,
        ) &&
        !isCaptureDependencyResourceType(route.request().resourceType())
      ) {
        gate.recordUnsupportedRequestBlocked(url.hostname);
        await abortWithAttribution(
          route,
          "unsupported_public_origin_resource_type",
        );
        return;
      }
      const request = route.request();
      await servePublicRoute(route, {
        gate,
        responseCache,
        cacheOnly,
        fetchRetainedEntry: responseCache?.evidenceReplay
          ? (targetUrl) => fetchRetainedEntry(route, request, targetUrl)
          : null,
      });
    } catch (error) {
      gate.failClosed(error);
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
    let status;
    let retryAfter;
    try {
      status = response.status();
      retryAfter = response.headers()?.["retry-after"];
    } catch {
      gate.failClosed(new Error("browser response headers cannot be inspected for request-gate enforcement"));
      return;
    }
    // Retry-After is a retry demand only on 429/503. Wikidot HTML pages also
    // send it on 200 responses as a cache hint; treating that as a retry
    // demand would stall every browser capture for the hinted interval.
    if (status !== 429 && status !== 503) return;
    if (retryAfter !== undefined) void gate.deferForRetryAfter(retryAfter).catch((error) => gate.failClosed(error));
  });
  await context.routeWebSocket("**/*", () => {
    gate.recordWebSocketBlocked();
    // Do not call connectToServer: Playwright keeps this as an in-page mock and no unmetered socket reaches the network.
  });
  return {
    exempt_origins: [...exempt].sort(),
    response_cache: responseCache,
    classifyRequestFailure(request) {
      return attributedAborts.get(request) ?? null;
    },
  };
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
