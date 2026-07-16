import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import {isObject, rowLocalUrl} from "./browser-render-evidence.mjs";

export const RECORD_SCHEMA = "wikijump_full_parity.local_browser_console_smoke_record.v2";
export const SUMMARY_SCHEMA = "wikijump_full_parity.local_browser_console_smoke_summary.v2";
export const RUNTIME_IDENTITY_SCHEMA = "wikijump_full_parity.local_browser_runtime_identity.v1";
const LOCAL_ORIGIN = "https://scp-wiki.wikijump.localhost";
const SLUG_PATTERN = /^[A-Za-z0-9_](?:[A-Za-z0-9:_-]{0,254}[A-Za-z0-9_])?$/;
const SHA_PATTERN = /^[a-f0-9]{64}$/;
const GIT_SHA_PATTERN = /^[a-f0-9]{40}$/;
const CLASSIFICATIONS = new Set(["fail", "external_unclassified", "unknown"]);

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (isObject(value)) return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

export function sha256Value(value) {
  return crypto.createHash("sha256").update(typeof value === "string" || Buffer.isBuffer(value) ? value : canonicalJson(value)).digest("hex");
}

export async function sha256File(filePath) {
  return sha256Value(await fs.readFile(filePath));
}

export function validateRuntimeIdentity(identity) {
  if (!isObject(identity) || identity.schema !== RUNTIME_IDENTITY_SCHEMA) throw new Error(`runtime identity must use schema ${RUNTIME_IDENTITY_SCHEMA}`);
  for (const field of ["wikijump_sha", "ftml_sha"]) if (!GIT_SHA_PATTERN.test(identity[field] ?? "")) throw new Error(`runtime identity ${field} must be a 40-character lowercase Git SHA`);
  for (const field of ["deepwell_binary_or_image_sha256", "framerail_assets_sha256"]) if (!SHA_PATTERN.test(identity[field] ?? "")) throw new Error(`runtime identity ${field} must be a lowercase SHA-256`);
  for (const field of ["rustc_vv", "profile", "render_run_id"]) if (typeof identity[field] !== "string" || identity[field].trim() === "") throw new Error(`runtime identity ${field} must be a non-empty string`);
  for (const marker of ["rustc ", "binary: rustc", "commit-hash:", "commit-date:", "host:", "release:", "LLVM version:"]) if (!identity.rustc_vv.includes(marker)) throw new Error(`runtime identity rustc_vv is missing ${marker}`);
  if (!Array.isArray(identity.features) || identity.features.some((item) => typeof item !== "string" || item === "") || new Set(identity.features).size !== identity.features.length || [...identity.features].sort().some((item, index) => item !== identity.features[index])) throw new Error("runtime identity features must be a sorted unique string array");
  return identity;
}

export function localSmokeUrl(row) {
  if (!isObject(row) || typeof row.slug !== "string" || !SLUG_PATTERN.test(row.slug)) throw new Error(`fixture ${row?.fixture_id ?? "unknown"} slug is outside the safe ASCII grammar`);
  const expected = `${LOCAL_ORIGIN}/${row.slug}`;
  const configured = rowLocalUrl(row);
  if (configured && configured !== expected) throw new Error(`fixture ${row.fixture_id} local URL must be exactly ${expected}`);
  let url;
  try {
    url = new URL(configured || expected);
  } catch {
    throw new Error(`fixture ${row.fixture_id} local URL must be exactly ${expected}`);
  }
  if (url.href !== expected || url.protocol !== "https:" || url.hostname !== "scp-wiki.wikijump.localhost" || url.port || url.username || url.password || url.search || url.hash) throw new Error(`fixture ${row.fixture_id} local URL must be exactly ${expected}`);
  return expected;
}

function validWjfilesHost(hostname) {
  return /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.wjfiles\.localhost$/.test(hostname);
}

export function classifyFailureUrl(value) {
  if (typeof value !== "string" || value === "") return "unknown";
  if (value.startsWith("blob:")) return classifyFailureUrl(value.slice(5));
  try {
    const url = new URL(value);
    const hostname = url.hostname.toLowerCase();
    const exactApp = value.startsWith(`${LOCAL_ORIGIN}/`) && !url.username && !url.password;
    const exactWjfiles = /^https?:\/\/[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.wjfiles\.localhost(?:\/|$)/.test(value) && validWjfilesHost(hostname) && !url.username && !url.password;
    const loopback = (hostname === "127.0.0.1" || hostname === "::1" || hostname === "[::1]") && !url.username && !url.password;
    if (exactApp || exactWjfiles || ((url.protocol === "https:" || url.protocol === "http:") && loopback)) return "fail";
    return ["http:", "https:", "ws:", "wss:"].includes(url.protocol) ? "external_unclassified" : "unknown";
  } catch {
    return "unknown";
  }
}

export function preflightEnShardManifest(inventory, manifest, shardId) {
  if (!isObject(manifest) || manifest.schema !== "wikijump_full_parity.corpus_shard_manifest.v1" || !Array.isArray(manifest.shards)) throw new Error("invalid corpus shard manifest");
  const enRows = inventory.filter((row) => row.family === "EN" && row.fixture_id.startsWith("EN:"));
  const rowMap = new Map(enRows.map((row) => [row.fixture_id, row]));
  const shardIds = new Set();
  const owners = new Map();
  for (const shard of manifest.shards) {
    if (!isObject(shard) || typeof shard.shard_id !== "string" || !Array.isArray(shard.fixture_ids)) throw new Error("manifest shards require shard_id and fixture_ids");
    if (shardIds.has(shard.shard_id)) throw new Error(`duplicate shard_id: ${shard.shard_id}`);
    shardIds.add(shard.shard_id);
    for (const fixtureId of shard.fixture_ids) {
      if (!rowMap.has(fixtureId)) throw new Error(`manifest fixture is not in the EN inventory: ${fixtureId}`);
      if (owners.has(fixtureId)) throw new Error(`fixture belongs to multiple shards: ${fixtureId}`);
      owners.set(fixtureId, shard.shard_id);
    }
  }
  const missing = [...rowMap.keys()].filter((fixtureId) => !owners.has(fixtureId));
  if (missing.length) throw new Error(`EN inventory fixtures are missing from the shard union: ${missing.join(", ")}`);
  const selected = manifest.shards.find((shard) => shard.shard_id === shardId);
  if (!selected) throw new Error(`shard not found: ${shardId}`);
  if (selected.fixture_ids.length === 0) throw new Error(`shard ${shardId} selects no fixtures`);
  return selected.fixture_ids.map((fixtureId) => rowMap.get(fixtureId));
}

function messageRecord(message) {
  const url = (typeof message.location === "function" ? message.location() : {})?.url || null;
  return {message: message.text(), url, classification: classifyFailureUrl(url)};
}

function requestRecord(request) {
  const url = request.url();
  return {url, resource_type: request.resourceType(), error_text: request.failure()?.errorText ?? "unknown", classification: classifyFailureUrl(url)};
}

function responseRecord(response) {
  const url = response.url();
  return {url, status: response.status(), resource_type: response.request().resourceType(), classification: classifyFailureUrl(url)};
}

function recordResult(record) {
  const coreFailure = record.navigation_error !== null || !Number.isInteger(record.status) || record.status < 200 || record.status >= 300 || record.final_url !== record.url || record.page_content_count !== 1 || record.redirect_chain.some((entry) => entry.url !== record.url);
  if (coreFailure || [...record.console_errors, ...record.page_errors, ...record.request_failures, ...record.http_error_responses, ...record.visible_broken_images].some((entry) => entry.classification === "fail")) return "fail";
  return record.console_errors.length || record.page_errors.length || record.request_failures.length || record.http_error_responses.length || record.visible_broken_images.length ? "review" : "pass";
}

function detachListeners(page, listeners) {
  if (typeof page.off !== "function") return;
  for (const [event, handler] of Object.entries(listeners)) page.off(event, handler);
}

export async function captureLocalSmoke(page, row, {timeoutMs, settleMs}) {
  const url = localSmokeUrl(row);
  const consoleErrors = [];
  const pageErrors = [];
  const requestFailures = [];
  const httpErrorResponses = [];
  const redirectChain = [];
  const listeners = {
    console: (message) => { if (message.type() === "error") consoleErrors.push(messageRecord(message)); },
    pageerror: (error) => pageErrors.push({message: error.message ?? String(error), url: null, classification: "unknown"}),
    requestfailed: (request) => requestFailures.push(requestRecord(request)),
    response: (response) => {
      const request = response.request();
      if (typeof request.isNavigationRequest === "function" && request.isNavigationRequest() && request.frame() === page.mainFrame()) redirectChain.push({url: response.url(), status: response.status()});
      if (response.status() >= 400) httpErrorResponses.push(responseRecord(response));
    },
  };
  for (const [event, handler] of Object.entries(listeners)) page.on(event, handler);
  let response = null;
  let navigationError = null;
  let pageContentCount = 0;
  const visibleBrokenImages = [];
  try {
    try {
      response = await page.goto(url, {timeout: timeoutMs, waitUntil: "domcontentloaded"});
      if (settleMs > 0) await page.waitForTimeout(settleMs);
    } catch (error) {
      navigationError = error.message ?? String(error);
    }
    try {
      pageContentCount = await page.locator("#page-content").count();
    } catch (error) {
      pageErrors.push({message: `page-content inspection failed: ${error.message ?? String(error)}`, url: null, context_url: url, classification: "fail"});
    }
    const frames = typeof page.frames === "function" ? page.frames() : [page];
    for (const frame of frames) {
      try {
        const images = await frame.evaluate(() => Array.from(document.images).filter((image) => {
          const style = window.getComputedStyle(image);
          const rect = image.getBoundingClientRect();
          return !image.hidden && style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0 && (!image.complete || image.naturalWidth === 0);
        }).map((image) => ({url: image.currentSrc || image.src || null})));
        for (const image of images) visibleBrokenImages.push({...image, frame_url: typeof frame.url === "function" ? frame.url() : null, classification: classifyFailureUrl(image.url)});
      } catch (error) {
        pageErrors.push({message: `image inspection failed: ${error.message ?? String(error)}`, url: null, context_url: typeof frame.url === "function" ? frame.url() : null, classification: "fail"});
      }
    }
  } finally {
    detachListeners(page, listeners);
  }
  const status = response?.status() ?? null;
  const finalUrl = page.url();
  if (redirectChain.length === 0 && response) redirectChain.push({url: typeof response.url === "function" ? response.url() : finalUrl, status});
  const record = {schema: RECORD_SCHEMA, fixture_id: row.fixture_id, slug: row.slug, url, status, final_url: finalUrl, redirect_chain: redirectChain, page_content_count: pageContentCount, visible_broken_images: visibleBrokenImages, console_errors: consoleErrors, page_errors: pageErrors, request_failures: requestFailures, http_error_responses: httpErrorResponses, navigation_error: navigationError, result: null};
  record.result = recordResult(record);
  return record;
}

function validClassifiedArray(value, fields) {
  return Array.isArray(value) && value.every((entry) => isObject(entry) && CLASSIFICATIONS.has(entry.classification) && fields.every((field) => Object.hasOwn(entry, field)) && (entry.url === null ? entry.classification === "unknown" || entry.classification === "fail" : entry.classification === classifyFailureUrl(entry.url)));
}

function validateLedgerRecord(record, row, fingerprint, resumableOnly) {
  if (!isObject(record) || record.schema !== RECORD_SCHEMA || !row) throw new Error("record schema or fixture_id is not in the selected row contract");
  if (record.run_fingerprint_sha256 !== fingerprint) throw new Error("run fingerprint does not match");
  const url = localSmokeUrl(row);
  if (record.fixture_id !== row.fixture_id || record.slug !== row.slug || record.url !== url || typeof record.final_url !== "string" && record.final_url !== null) throw new Error("record does not match fixture slug/url contract");
  if (record.status !== null && !Number.isInteger(record.status) || !Number.isInteger(record.page_content_count) || record.page_content_count < 0 || typeof record.navigation_error !== "string" && record.navigation_error !== null) throw new Error("record status, page_content_count, or navigation_error is invalid");
  if (!Array.isArray(record.redirect_chain) || record.redirect_chain.some((entry) => !isObject(entry) || typeof entry.url !== "string" || !Number.isInteger(entry.status))) throw new Error("record redirect_chain is invalid");
  if (!validClassifiedArray(record.console_errors, ["message", "url"]) || !validClassifiedArray(record.page_errors, ["message", "url"]) || !validClassifiedArray(record.request_failures, ["url", "resource_type", "error_text"]) || !validClassifiedArray(record.http_error_responses, ["url", "resource_type", "status"]) || !validClassifiedArray(record.visible_broken_images, ["url", "frame_url"])) throw new Error("record observation arrays are incomplete or misclassified");
  if ([...record.console_errors, ...record.page_errors].some((entry) => typeof entry.message !== "string" || entry.url !== null && typeof entry.url !== "string") || record.request_failures.some((entry) => typeof entry.url !== "string" || typeof entry.resource_type !== "string" || typeof entry.error_text !== "string") || record.http_error_responses.some((entry) => typeof entry.url !== "string" || typeof entry.resource_type !== "string" || !Number.isInteger(entry.status) || entry.status < 400) || record.visible_broken_images.some((entry) => entry.url !== null && typeof entry.url !== "string" || entry.frame_url !== null && typeof entry.frame_url !== "string")) throw new Error("record observation array field types are invalid");
  if (record.status !== null && (record.redirect_chain.length === 0 || record.redirect_chain.at(-1).url !== record.final_url || record.redirect_chain.at(-1).status !== record.status)) throw new Error("record redirect_chain does not terminate at the final response");
  if (record.result !== recordResult(record)) throw new Error("record result is inconsistent with observations");
  if (resumableOnly && (record.status < 200 || record.status >= 300 || record.final_url !== url || record.result !== "pass" || record.page_content_count !== 1)) throw new Error("only complete exact-URL 2xx pass records are resumable");
}

async function readLedger(outputPath, repairTail) {
  let handle;
  try {
    handle = await fs.open(outputPath, repairTail ? "r+" : "r");
  } catch (error) {
    if (error.code === "ENOENT") return {text: "", truncatedTail: false};
    throw error;
  }
  try {
    let bytes = await handle.readFile();
    let truncatedTail = false;
    if (repairTail && bytes.length > 0 && bytes.at(-1) !== 0x0a) {
      const lastNewline = bytes.lastIndexOf(0x0a);
      const size = lastNewline < 0 ? 0 : lastNewline + 1;
      await handle.truncate(size);
      await handle.sync();
      bytes = bytes.subarray(0, size);
      truncatedTail = true;
    }
    return {text: new TextDecoder("utf-8", {fatal: true}).decode(bytes), truncatedTail};
  } finally {
    await handle.close();
  }
}

export async function inspectLedger(outputPath, fingerprint, rows, {resumableOnly = true, repairTail = true} = {}) {
  const {text, truncatedTail} = await readLedger(outputPath, repairTail);
  const rowMap = new Map(rows.map((row) => [row.fixture_id, row]));
  const records = [];
  const ledgerErrors = [];
  for (const [index, line] of text.split("\n").entries()) {
    if (!line) continue;
    try {
      const record = JSON.parse(line);
      validateLedgerRecord(record, rowMap.get(record?.fixture_id), fingerprint, resumableOnly);
      records.push(record);
    } catch (error) {
      ledgerErrors.push(`line ${index + 1}: ${error.message}`);
    }
  }
  const counts = new Map();
  for (const record of records) counts.set(record.fixture_id, (counts.get(record.fixture_id) ?? 0) + 1);
  return {records, observed: [...counts.keys()], duplicate: [...counts].filter(([, count]) => count > 1).map(([id]) => id), ledgerErrors, truncatedTail};
}

export function buildSmokeSummary({expected, records, runtimeIdentity, runContract, inventory, shardManifest, shardId, ledgerErrors = [], truncatedTail = false}) {
  const counts = new Map();
  for (const record of records) counts.set(record.fixture_id, (counts.get(record.fixture_id) ?? 0) + 1);
  const observed = [...counts.keys()];
  const expectedSet = new Set(expected);
  const missing = expected.filter((id) => !counts.has(id));
  const duplicate = [...counts].filter(([, count]) => count > 1).map(([id]) => id);
  const unexpected = observed.filter((id) => !expectedSet.has(id));
  const failedFixtures = [...new Set(records.filter((record) => record.result !== "pass").map((record) => record.fixture_id))];
  const status = missing.length || duplicate.length || unexpected.length || failedFixtures.length || ledgerErrors.length ? "fail" : "pass";
  return {schema: SUMMARY_SCHEMA, status, inventory, shard_manifest: shardManifest, shard_id: shardId, runtime_identity: runtimeIdentity, run_contract: runContract, run_fingerprint_sha256: runContract ? sha256Value(runContract) : null, expected, observed, missing, duplicate, unexpected, failed_fixtures: failedFixtures, ledger_errors: ledgerErrors, truncated_unterminated_tail: truncatedTail};
}

async function bounded(promise, timeoutMs, label) {
  let timer;
  try {
    return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(`${label} timed out after ${timeoutMs}ms`)), timeoutMs); })]);
  } finally {
    clearTimeout(timer);
  }
}

async function acquireOutputLock(outputPath) {
  const lockPath = `${outputPath}.lock`;
  let handle;
  try {
    handle = await fs.open(lockPath, "wx", 0o600);
  } catch (error) {
    if (error.code === "EEXIST") throw new Error(`output owner lock already exists: ${lockPath}`);
    throw error;
  }
  try {
    await handle.writeFile(`${JSON.stringify({pid: process.pid, acquired_at: new Date().toISOString()})}\n`);
    await handle.sync();
  } catch (error) {
    await handle.close().catch(() => {});
    await fs.unlink(lockPath).catch(() => {});
    throw error;
  }
  return async () => {
    await handle.close().catch(() => {});
    await fs.unlink(lockPath);
  };
}

async function writeSummary(outputPath, summary) {
  const summaryPath = `${outputPath}.summary.json`;
  const temporaryPath = `${summaryPath}.tmp-${process.pid}`;
  await fs.writeFile(temporaryPath, `${JSON.stringify(summary, null, 2)}\n`, {encoding: "utf8", mode: 0o600});
  await fs.rename(temporaryPath, summaryPath);
  return summaryPath;
}

export async function runLocalBrowserSmoke({chromium, rows, outputPath, runtimeIdentity, inventoryPath, inventorySha256, shardManifestPath, shardManifestSha256, shardId, browserExecutable, browserExecutableSha256, browserArgs = [], workers, timeoutMs, settleMs, ignoreHttpsErrors}) {
  validateRuntimeIdentity(runtimeIdentity);
  if (!browserExecutable || !SHA_PATTERN.test(browserExecutableSha256 ?? "") || !SHA_PATTERN.test(inventorySha256 ?? "") || !SHA_PATTERN.test(shardManifestSha256 ?? "")) throw new Error("browser executable and all input byte SHA-256 identities are required");
  if (!Number.isSafeInteger(workers) || workers < 1 || workers > 64 || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || !Number.isSafeInteger(settleMs) || settleMs < 0 || typeof ignoreHttpsErrors !== "boolean") throw new Error("capture config requires 1-64 workers, positive timeout, non-negative settle, and explicit HTTPS policy");
  if (!Array.isArray(browserArgs) || browserArgs.length > 32 || browserArgs.some((arg) => typeof arg !== "string" || arg.length === 0 || arg.length > 4096 || arg.includes("\0"))) throw new Error("browser args must be an array of at most 32 non-empty bounded strings");
  const expected = rows.map((row) => row.fixture_id);
  if (new Set(expected).size !== expected.length) throw new Error("selected fixture IDs must be unique");
  for (const row of rows) {
    if (row.family !== "EN" || !row.fixture_id.startsWith("EN:")) throw new Error(`fixture ${row.fixture_id} is outside the EN smoke scope`);
    localSmokeUrl(row);
  }
  await fs.mkdir(path.dirname(outputPath), {recursive: true});
  const releaseLock = await acquireOutputLock(outputPath);
  const closeTimeoutMs = Math.max(1_000, Math.min(timeoutMs, 10_000));
  let browser = null;
  let outputHandle = null;
  let runContract = null;
  const operationErrors = [];
  let firstInspection = {records: [], observed: [], duplicate: [], ledgerErrors: [], truncatedTail: false};
  try {
    try {
      browser = await chromium.launch({executablePath: browserExecutable, args: browserArgs});
      const browserVersion = await Promise.resolve(browser.version());
      if (typeof browserVersion !== "string" || browserVersion.trim() === "") throw new Error("browser version identity is unavailable");
      runContract = {
        schema: "wikijump_full_parity.local_browser_console_smoke_run_contract.v1",
        runtime_identity: runtimeIdentity,
        inventory_sha256: inventorySha256,
        shard_manifest_sha256: shardManifestSha256,
        shard_id: shardId,
        selected_row_contract_sha256: sha256Value(rows.map((row) => ({fixture_id: row.fixture_id, family: row.family, slug: row.slug, url: localSmokeUrl(row)}))),
        capture_config: {workers, timeout_ms: timeoutMs, settle_ms: settleMs, ignore_https_errors: ignoreHttpsErrors, browser_args: browserArgs},
        browser: {version: browserVersion, executable: browserExecutable, executable_sha256: browserExecutableSha256},
      };
      const fingerprint = sha256Value(runContract);
      firstInspection = await inspectLedger(outputPath, fingerprint, rows, {resumableOnly: true, repairTail: true});
      if (!firstInspection.ledgerErrors.length && !firstInspection.duplicate.length) {
        const observed = new Set(firstInspection.observed);
        const remaining = rows.filter((row) => !observed.has(row.fixture_id));
        if (remaining.length) {
          outputHandle = await fs.open(outputPath, "a", 0o600);
          let nextIndex = 0;
          let appendQueue = Promise.resolve();
          const append = (record) => {
            appendQueue = appendQueue.then(async () => {
              await bounded(outputHandle.appendFile(`${JSON.stringify(record)}\n`), timeoutMs, "ledger append");
              await bounded(outputHandle.sync(), timeoutMs, "ledger fsync");
            });
            return appendQueue;
          };
          const worker = async () => {
            while (nextIndex < remaining.length) {
              const row = remaining[nextIndex++];
              let context = null;
              let page = null;
              let record;
              try {
                record = await bounded((async () => {
                  context = await browser.newContext({ignoreHTTPSErrors: ignoreHttpsErrors});
                  page = await context.newPage();
                  return await captureLocalSmoke(page, row, {timeoutMs, settleMs});
                })(), timeoutMs + settleMs + 5_000, `fixture ${row.fixture_id}`);
              } catch (error) {
                const url = localSmokeUrl(row);
                record = {schema: RECORD_SCHEMA, fixture_id: row.fixture_id, slug: row.slug, url, status: null, final_url: page?.url?.() ?? null, redirect_chain: [], page_content_count: 0, visible_broken_images: [], console_errors: [], page_errors: [{message: error.message ?? String(error), url, classification: "fail"}], request_failures: [], http_error_responses: [], navigation_error: error.message ?? String(error), result: "fail"};
              } finally {
                await bounded(Promise.resolve(page?.removeAllListeners?.()), closeTimeoutMs, "listener freeze").catch((error) => operationErrors.push(error.message));
                await bounded(page?.close() ?? Promise.resolve(), closeTimeoutMs, "page close").catch((error) => operationErrors.push(error.message));
                await bounded(context?.close() ?? Promise.resolve(), closeTimeoutMs, "context close").catch((error) => operationErrors.push(error.message));
              }
              record.run_fingerprint_sha256 = fingerprint;
              await append(record);
            }
          };
          const workerResults = await Promise.allSettled(Array.from({length: Math.min(workers, remaining.length)}, worker));
          for (const result of workerResults) if (result.status === "rejected") operationErrors.push(result.reason?.message ?? String(result.reason));
          await appendQueue.catch((error) => operationErrors.push(error.message ?? String(error)));
        }
      }
    } catch (error) {
      operationErrors.push(error.message ?? String(error));
    }

    if (outputHandle) {
      await bounded(outputHandle.sync(), timeoutMs, "final ledger fsync").catch((error) => operationErrors.push(error.message));
      try {
        await bounded(outputHandle.close(), closeTimeoutMs, "ledger close");
        outputHandle = null;
      } catch (error) {
        operationErrors.push(error.message);
      }
    }
    if (browser) {
      try {
        await bounded(browser.close(), closeTimeoutMs, "browser close");
        browser = null;
      } catch (error) {
        operationErrors.push(error.message);
      }
    }
    const fingerprint = runContract ? sha256Value(runContract) : null;
    let finalInspection = firstInspection;
    if (fingerprint) {
      try {
        finalInspection = await inspectLedger(outputPath, fingerprint, rows, {resumableOnly: false, repairTail: true});
      } catch (error) {
        operationErrors.push(`final ledger inspection: ${error.message ?? String(error)}`);
        finalInspection = {records: [], observed: [], duplicate: [], ledgerErrors: [], truncatedTail: false};
      }
    }
    const summary = buildSmokeSummary({expected, records: finalInspection.records, runtimeIdentity, runContract, inventory: inventoryPath, shardManifest: shardManifestPath, shardId, ledgerErrors: [...firstInspection.ledgerErrors, ...finalInspection.ledgerErrors, ...operationErrors], truncatedTail: firstInspection.truncatedTail || finalInspection.truncatedTail});
    return {summary, summaryPath: await writeSummary(outputPath, summary)};
  } finally {
    await Promise.allSettled([
      bounded(outputHandle?.close() ?? Promise.resolve(), closeTimeoutMs, "ledger cleanup"),
      bounded(browser?.close() ?? Promise.resolve(), closeTimeoutMs, "browser cleanup"),
    ]);
    await releaseLock();
  }
}
