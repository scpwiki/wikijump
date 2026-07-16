import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import {isObject, rowLocalUrl} from "./browser-render-evidence.mjs";

export const RECORD_SCHEMA = "wikijump_full_parity.local_browser_console_smoke_record.v1";
export const SUMMARY_SCHEMA = "wikijump_full_parity.local_browser_console_smoke_summary.v1";
const LOCAL_HOST = "scp-wiki.wikijump.localhost";

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (isObject(value)) return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

export function validateRuntimeIdentity(identity) {
  if (!isObject(identity) || Object.keys(identity).length === 0) throw new Error("runtime identity must be a non-empty JSON object");
  return identity;
}

export function runtimeIdentityFingerprint(identity) {
  return crypto.createHash("sha256").update(canonicalJson(validateRuntimeIdentity(identity))).digest("hex");
}

export function localSmokeUrl(row) {
  if (!isObject(row) || typeof row.slug !== "string" || row.slug.length === 0 || row.slug.startsWith("/")) {
    throw new Error(`fixture ${row?.fixture_id ?? "unknown"} must have a non-empty relative slug`);
  }
  const expected = new URL(`/${row.slug}`, `https://${LOCAL_HOST}`);
  const configured = rowLocalUrl(row);
  const url = configured ? new URL(configured) : expected;
  if (url.protocol !== "https:" || url.hostname !== LOCAL_HOST || url.port || url.username || url.password || url.search || url.hash || url.href !== expected.href) {
    throw new Error(`fixture ${row.fixture_id} local URL must be exactly ${expected.href}`);
  }
  return url.href;
}

export function classifyFailureUrl(value) {
  try {
    const hostname = new URL(value).hostname.toLowerCase();
    return hostname === "localhost" || hostname.endsWith(".localhost") ? "fail" : "external_unclassified";
  } catch {
    return "fail";
  }
}

function messageRecord(message) {
  const location = typeof message.location === "function" ? message.location() : {};
  const url = location?.url || null;
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

function failed(record) {
  return record.classification === "fail";
}

export async function captureLocalSmoke(page, row, {timeoutMs, settleMs}) {
  const url = localSmokeUrl(row);
  const consoleErrors = [];
  const pageErrors = [];
  const requestFailures = [];
  const httpErrorResponses = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(messageRecord(message));
  });
  page.on("pageerror", (error) => pageErrors.push({message: error.message ?? String(error), classification: "fail"}));
  page.on("requestfailed", (request) => requestFailures.push(requestRecord(request)));
  page.on("response", (response) => {
    if (response.status() >= 400) httpErrorResponses.push(responseRecord(response));
  });

  let response = null;
  let navigationError = null;
  try {
    response = await page.goto(url, {timeout: timeoutMs, waitUntil: "domcontentloaded"});
    if (settleMs > 0) await page.waitForTimeout(settleMs);
  } catch (error) {
    navigationError = error.message ?? String(error);
  }
  const pageContentCount = await page.locator("#page-content").count().catch(() => 0);
  const visibleBrokenImages = await page.evaluate(() =>
    Array.from(document.images)
      .filter((image) => {
        const style = window.getComputedStyle(image);
        const rect = image.getBoundingClientRect();
        return !image.hidden && style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0 && (!image.complete || image.naturalWidth === 0);
      })
      .map((image) => ({url: image.currentSrc || image.src || null})),
  ).catch(() => []);
  for (const image of visibleBrokenImages) image.classification = classifyFailureUrl(image.url);
  const finalUrl = page.url();
  const status = response?.status() ?? null;
  const fatal = navigationError !== null || status === null || status >= 400 || pageContentCount !== 1 || classifyFailureUrl(finalUrl) !== "fail" || consoleErrors.some(failed) || pageErrors.length > 0 || requestFailures.some(failed) || httpErrorResponses.some(failed) || visibleBrokenImages.some(failed);
  return {
    schema: RECORD_SCHEMA,
    fixture_id: row.fixture_id,
    slug: row.slug,
    url,
    status,
    final_url: finalUrl,
    page_content_count: pageContentCount,
    visible_broken_images: visibleBrokenImages,
    console_errors: consoleErrors,
    page_errors: pageErrors,
    request_failures: requestFailures,
    http_error_responses: httpErrorResponses,
    navigation_error: navigationError,
    result: fatal ? "fail" : "pass",
  };
}

export async function inspectLedger(outputPath, identityFingerprint) {
  let contents;
  try {
    contents = await fs.readFile(outputPath, "utf8");
  } catch (error) {
    if (error.code === "ENOENT") return {records: [], observed: [], duplicate: [], ledgerErrors: []};
    throw error;
  }
  const records = [];
  const ledgerErrors = [];
  for (const [index, line] of contents.split("\n").entries()) {
    if (!line.trim()) continue;
    try {
      const record = JSON.parse(line);
      if (record.schema !== RECORD_SCHEMA || typeof record.fixture_id !== "string") throw new Error("invalid record schema or fixture_id");
      if (record.runtime_identity_sha256 !== identityFingerprint) throw new Error("runtime identity does not match this run");
      records.push(record);
    } catch (error) {
      ledgerErrors.push(`line ${index + 1}: ${error.message}`);
    }
  }
  const counts = new Map();
  for (const record of records) counts.set(record.fixture_id, (counts.get(record.fixture_id) ?? 0) + 1);
  return {records, observed: [...counts.keys()], duplicate: [...counts].filter(([, count]) => count > 1).map(([id]) => id), ledgerErrors};
}

export function buildSmokeSummary({expected, records, runtimeIdentity, inventory, shardManifest, shardId, ledgerErrors = []}) {
  const counts = new Map();
  for (const record of records) counts.set(record.fixture_id, (counts.get(record.fixture_id) ?? 0) + 1);
  const observed = [...counts.keys()];
  const expectedSet = new Set(expected);
  const missing = expected.filter((id) => !counts.has(id));
  const duplicate = [...counts].filter(([, count]) => count > 1).map(([id]) => id);
  const unexpected = observed.filter((id) => !expectedSet.has(id));
  const failedFixtures = [...new Set(records.filter((record) => record.result !== "pass").map((record) => record.fixture_id))];
  const status = missing.length || duplicate.length || unexpected.length || failedFixtures.length || ledgerErrors.length ? "fail" : "pass";
  return {schema: SUMMARY_SCHEMA, status, inventory, shard_manifest: shardManifest, shard_id: shardId, runtime_identity: runtimeIdentity, expected, observed, missing, duplicate, unexpected, failed_fixtures: failedFixtures, ledger_errors: ledgerErrors};
}

async function writeSummary(outputPath, summary) {
  const summaryPath = `${outputPath}.summary.json`;
  const temporaryPath = `${summaryPath}.tmp-${process.pid}`;
  await fs.writeFile(temporaryPath, `${JSON.stringify(summary, null, 2)}\n`, "utf8");
  await fs.rename(temporaryPath, summaryPath);
  return summaryPath;
}

export async function runLocalBrowserSmoke({chromium, rows, outputPath, runtimeIdentity, inventoryPath, shardManifestPath, shardId, browserExecutable, workers, timeoutMs, settleMs, ignoreHttpsErrors}) {
  validateRuntimeIdentity(runtimeIdentity);
  const expected = rows.map((row) => row.fixture_id);
  if (new Set(expected).size !== expected.length) throw new Error("selected fixture IDs must be unique");
  for (const row of rows) {
    if (row.family !== "EN" || !row.fixture_id.startsWith("EN:")) throw new Error(`fixture ${row.fixture_id} is outside the EN smoke scope`);
    localSmokeUrl(row);
  }
  await fs.mkdir(path.dirname(outputPath), {recursive: true});
  const fingerprint = runtimeIdentityFingerprint(runtimeIdentity);
  const prior = await inspectLedger(outputPath, fingerprint);
  let records = prior.records;
  let summary = buildSmokeSummary({expected, records, runtimeIdentity, inventory: inventoryPath, shardManifest: shardManifestPath, shardId, ledgerErrors: prior.ledgerErrors});
  if (prior.duplicate.length || prior.ledgerErrors.length || summary.unexpected.length) {
    const summaryPath = await writeSummary(outputPath, summary);
    return {summary, summaryPath};
  }
  const remaining = rows.filter((row) => !new Set(prior.observed).has(row.fixture_id));
  if (remaining.length === 0) return {summary, summaryPath: await writeSummary(outputPath, summary)};

  let browser = null;
  try {
    browser = await chromium.launch({...(browserExecutable ? {executablePath: browserExecutable} : {})});
  } catch (error) {
    await browser?.close().catch(() => {});
    summary = buildSmokeSummary({expected, records, runtimeIdentity, inventory: inventoryPath, shardManifest: shardManifestPath, shardId, ledgerErrors: [`browser initialization: ${error.message ?? String(error)}`]});
    return {summary, summaryPath: await writeSummary(outputPath, summary)};
  }
  const handle = await fs.open(outputPath, "a");
  let nextIndex = 0;
  let appendQueue = Promise.resolve();
  const append = (record) => {
    appendQueue = appendQueue.then(async () => {
      await handle.write(`${JSON.stringify(record)}\n`);
      await handle.sync();
      records = [...records, record];
    });
    return appendQueue;
  };
  try {
    const worker = async () => {
      while (nextIndex < remaining.length) {
        const row = remaining[nextIndex++];
        let rowContext = null;
        let page = null;
        let record;
        try {
          rowContext = await browser.newContext({ignoreHTTPSErrors: ignoreHttpsErrors});
          page = await rowContext.newPage();
          record = await captureLocalSmoke(page, row, {timeoutMs, settleMs});
        } catch (error) {
          record = {schema: RECORD_SCHEMA, fixture_id: row.fixture_id, slug: row.slug, url: localSmokeUrl(row), status: null, final_url: null, page_content_count: 0, visible_broken_images: [], console_errors: [], page_errors: [{message: error.message ?? String(error), classification: "fail"}], request_failures: [], http_error_responses: [], navigation_error: error.message ?? String(error), result: "fail"};
        } finally {
          await page?.close().catch(() => {});
          await rowContext?.close().catch(() => {});
        }
        record.runtime_identity_sha256 = fingerprint;
        await append(record);
      }
    };
    await Promise.all(Array.from({length: Math.min(workers, remaining.length)}, worker));
  } finally {
    await appendQueue;
    await handle.close();
    await browser.close().catch(() => {});
  }
  summary = buildSmokeSummary({expected, records, runtimeIdentity, inventory: inventoryPath, shardManifest: shardManifestPath, shardId});
  return {summary, summaryPath: await writeSummary(outputPath, summary)};
}
