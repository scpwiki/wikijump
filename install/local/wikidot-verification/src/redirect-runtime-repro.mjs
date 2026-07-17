import {createHash} from "node:crypto";
import http from "node:http";
import https from "node:https";
import fs from "node:fs/promises";
import net from "node:net";
import path from "node:path";

import {isObject} from "./browser-render-evidence.mjs";
import {
  RUNTIME_IDENTITY_SCHEMA,
  sha256File,
  sha256Value,
  validateRuntimeIdentity,
} from "./local-browser-console-smoke.mjs";

export const REDIRECT_VERDICT_SCHEMA =
  "wikijump_full_parity.redirect_runtime_repro.v1";
const SLUG_PATTERN = /^[A-Za-z0-9_](?:[A-Za-z0-9:_-]{0,254}[A-Za-z0-9_])?$/;
const SHA256_PATTERN = /^[a-f0-9]{64}$/;
const MAX_BODY_BYTES = 64 * 1024;

function canonicalRows(document) {
  if (Array.isArray(document)) return document;
  for (const field of ["rows", "redirects", "results"]) {
    if (Array.isArray(document?.[field])) return document[field];
  }
  throw new Error("redirect authority must be an array or contain rows/redirects/results");
}

function validLocation(value) {
  if (typeof value !== "string" || value.length === 0 || /[\u0000-\u001f\u007f]/u.test(value)) {
    return false;
  }
  if (value.startsWith("/")) return !value.startsWith("//");
  try {
    const url = new URL(value);
    return ["http:", "https:"].includes(url.protocol) && !url.username && !url.password;
  } catch {
    return false;
  }
}

function validateAuthorityRow(row, index) {
  if (!isObject(row)) throw new Error(`authority row ${index} must be an object`);
  const slug = row.fullname;
  if (typeof slug !== "string" || !SLUG_PATTERN.test(slug)) {
    throw new Error(`authority row ${index} fullname is outside the safe slug grammar`);
  }
  const expectedSourceUrl = `https://scp-wiki.wikidot.com/${slug}`;
  if (row.url !== expectedSourceUrl) {
    throw new Error(`authority row ${slug} source URL must be exactly ${expectedSourceUrl}`);
  }
  if (!Number.isInteger(row.status) || row.status < 300 || row.status >= 400) {
    throw new Error(`authority row ${slug} status must be a redirect status`);
  }
  if (!validLocation(row.location)) {
    throw new Error(`authority row ${slug} location is invalid`);
  }
  if (row.expected_destination !== undefined && row.expected_destination !== row.location && row.expected_destination !== row.location.replace(/^\//u, "")) {
    throw new Error(`authority row ${slug} expected_destination disagrees with location`);
  }
  return {
    fixture_id: `EN:${slug}`,
    slug,
    source_url: row.url,
    status: row.status,
    location: row.location,
  };
}

function validateCorpusRow(row, index) {
  if (!isObject(row) || typeof row.fullname !== "string" || !SLUG_PATTERN.test(row.fullname)) {
    throw new Error(`corpus redirect row ${index} is invalid`);
  }
  if (typeof row.destination !== "string" || row.destination.length === 0) {
    throw new Error(`corpus redirect row ${row.fullname} destination is invalid`);
  }
  for (const field of ["source_sha256", "meta_sha256"]) {
    if (!SHA256_PATTERN.test(row[field] ?? "")) {
      throw new Error(`corpus redirect row ${row.fullname} ${field} is invalid`);
    }
  }
  return row;
}

export function validateRedirectInputs({inventoryDocument, authorityDocument, corpusDocument}) {
  const inventory = canonicalRows(inventoryDocument);
  const inventoryMap = new Map();
  for (const [index, row] of inventory.entries()) {
    if (!isObject(row) || typeof row.fixture_id !== "string" || typeof row.slug !== "string") {
      throw new Error(`inventory row ${index} is invalid`);
    }
    if (inventoryMap.has(row.fixture_id)) throw new Error(`duplicate inventory fixture: ${row.fixture_id}`);
    inventoryMap.set(row.fixture_id, row);
  }

  const authority = canonicalRows(authorityDocument).map(validateAuthorityRow);
  const authorityMap = new Map();
  for (const row of authority) {
    if (authorityMap.has(row.fixture_id)) throw new Error(`duplicate redirect authority fixture: ${row.fixture_id}`);
    const inventoryRow = inventoryMap.get(row.fixture_id);
    if (!inventoryRow || inventoryRow.family !== "EN" || inventoryRow.slug !== row.slug) {
      throw new Error(`redirect authority fixture is absent or mismatched in inventory: ${row.fixture_id}`);
    }
    authorityMap.set(row.fixture_id, row);
  }

  const corpus = canonicalRows(corpusDocument).map(validateCorpusRow);
  const corpusMap = new Map();
  for (const row of corpus) {
    const fixtureId = `EN:${row.fullname}`;
    if (corpusMap.has(fixtureId)) throw new Error(`duplicate corpus redirect fixture: ${fixtureId}`);
    corpusMap.set(fixtureId, row);
  }
  const authorityIds = [...authorityMap.keys()].sort();
  const corpusIds = [...corpusMap.keys()].sort();
  if (authorityIds.length !== corpusIds.length || authorityIds.some((id, index) => id !== corpusIds[index])) {
    throw new Error("redirect authority and frozen corpus redirect sets are not exactly equal");
  }
  for (const fixtureId of authorityIds) {
    const authorityRow = authorityMap.get(fixtureId);
    const corpusRow = corpusMap.get(fixtureId);
    const normalizedLocation = authorityRow.location.startsWith("/")
      ? authorityRow.location.slice(1)
      : authorityRow.location;
    if (corpusRow.destination !== authorityRow.location && corpusRow.destination !== normalizedLocation) {
      throw new Error(`redirect destination mismatch for ${fixtureId}`);
    }
  }
  return authorityIds.map((id) => ({
    ...authorityMap.get(id),
    corpus_source_sha256: corpusMap.get(id).source_sha256,
    corpus_meta_sha256: corpusMap.get(id).meta_sha256,
  }));
}

function validateLocalBase(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new Error("local base URL is invalid");
  }
  if (!["http:", "https:"].includes(url.protocol) || url.username || url.password || url.pathname !== "/" || url.search || url.hash) {
    throw new Error("local base URL must be an origin with no credentials, path, query, or fragment");
  }
  if (url.hostname !== "scp-wiki.wikijump.localhost") {
    throw new Error("local base URL hostname must be scp-wiki.wikijump.localhost");
  }
  return url;
}

function validateLoopbackAddress(value) {
  const family = net.isIP(value);
  if (family === 4 && value.startsWith("127.")) return {address: value, family};
  if (family === 6 && value === "::1") return {address: value, family};
  throw new Error("resolved address must be an explicit loopback IP");
}

function headerValues(rawHeaders, name) {
  const values = [];
  for (let index = 0; index < rawHeaders.length; index += 2) {
    if (rawHeaders[index].toLowerCase() === name) values.push(rawHeaders[index + 1]);
  }
  return values;
}

export async function requestRedirectRoute({baseUrl, resolvedAddress, row, timeoutMs, ignoreHttpsErrors, siteId}) {
  const target = new URL(`/${row.slug}`, baseUrl);
  const transport = target.protocol === "https:" ? https : http;
  return await new Promise((resolve, reject) => {
    const request = transport.request({
      protocol: target.protocol,
      hostname: target.hostname,
      port: target.port || undefined,
      path: target.pathname,
      method: "GET",
      headers: {
        accept: "text/html,*/*;q=0.8",
        "user-agent": "wikijump-redirect-repro/1",
        ...(siteId === null ? {} : {
          "x-wikijump-site-id": siteId,
          "x-wikijump-site-slug": "scp-wiki",
        }),
      },
      rejectUnauthorized: !ignoreHttpsErrors,
      servername: target.hostname,
      lookup(_hostname, options, callback) {
        if (options?.all) {
          callback(null, [{address: resolvedAddress.address, family: resolvedAddress.family}]);
        } else {
          callback(null, resolvedAddress.address, resolvedAddress.family);
        }
      },
    });
    request.setTimeout(timeoutMs, () => request.destroy(new Error(`request timed out after ${timeoutMs}ms`)));
    request.once("error", reject);
    request.once("response", (response) => {
      const hash = createHash("sha256");
      let bodyBytes = 0;
      response.on("data", (chunk) => {
        bodyBytes += chunk.length;
        if (bodyBytes > MAX_BODY_BYTES) {
          response.destroy(new Error(`redirect response exceeded ${MAX_BODY_BYTES} bytes`));
          return;
        }
        hash.update(chunk);
      });
      response.once("error", reject);
      response.once("end", () => {
        const locations = headerValues(response.rawHeaders, "location");
        resolve({
          status: response.statusCode ?? null,
          location: locations.length === 1 ? locations[0] : null,
          location_count: locations.length,
          content_type: typeof response.headers["content-type"] === "string" ? response.headers["content-type"] : null,
          body_bytes: bodyBytes,
          body_sha256: hash.digest("hex"),
        });
      });
    });
    request.end();
  });
}

function observationResult(expected, observation) {
  const differences = [];
  if (observation.status !== expected.status) differences.push("status");
  if (observation.location_count !== 1) differences.push("location_count");
  if (observation.location !== expected.location) differences.push("location");
  return {status: differences.length === 0 ? "pass" : "fail", differences};
}

function reproducible(observations) {
  if (observations.length !== 2) return false;
  return sha256Value(observations[0].response) === sha256Value(observations[1].response);
}

async function runPool(rows, workers, operation) {
  const results = new Array(rows.length);
  let next = 0;
  const worker = async () => {
    while (next < rows.length) {
      const index = next++;
      results[index] = await operation(rows[index]);
    }
  };
  await Promise.all(Array.from({length: Math.min(workers, rows.length)}, worker));
  return results;
}

async function writeJsonAtomic(outputPath, value) {
  await fs.mkdir(path.dirname(outputPath), {recursive: true, mode: 0o700});
  const temporary = `${outputPath}.tmp-${process.pid}`;
  await fs.writeFile(temporary, `${JSON.stringify(value, null, 2)}\n`, {mode: 0o600});
  await fs.rename(temporary, outputPath);
}

export async function runRedirectRuntimeRepro({
  inventoryPath,
  authorityPath,
  corpusRedirectsPath,
  runtimeIdentityPath,
  localBase,
  resolvedAddress,
  outputPath,
  timeoutMs,
  workers,
  ignoreHttpsErrors,
  siteId = null,
  requester = requestRedirectRoute,
}) {
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || !Number.isSafeInteger(workers) || workers < 1 || workers > 64 || typeof ignoreHttpsErrors !== "boolean") {
    throw new Error("timeout, workers, and HTTPS policy are invalid");
  }
  const baseUrl = validateLocalBase(localBase);
  const loopback = validateLoopbackAddress(resolvedAddress);
  if (siteId !== null && (typeof siteId !== "string" || !/^[1-9][0-9]{0,18}$/u.test(siteId))) {
    throw new Error("site ID must be a positive decimal string or null");
  }
  const absoluteInputs = [inventoryPath, authorityPath, corpusRedirectsPath, runtimeIdentityPath].map((value) => path.resolve(value));
  if (absoluteInputs.includes(path.resolve(outputPath))) {
    throw new Error("redirect verdict output must not overwrite an input");
  }
  const [inventoryBytes, authorityBytes, corpusBytes, runtimeIdentityBytes] = await Promise.all([
    fs.readFile(inventoryPath),
    fs.readFile(authorityPath),
    fs.readFile(corpusRedirectsPath),
    fs.readFile(runtimeIdentityPath),
  ]);
  const runtimeIdentity = validateRuntimeIdentity(JSON.parse(runtimeIdentityBytes));
  const rows = validateRedirectInputs({
    inventoryDocument: JSON.parse(inventoryBytes),
    authorityDocument: JSON.parse(authorityBytes),
    corpusDocument: JSON.parse(corpusBytes),
  });
  const observationsByFixture = new Map(rows.map((row) => [row.fixture_id, []]));
  for (let pass = 1; pass <= 2; pass += 1) {
    const observations = await runPool(rows, workers, async (row) => {
      try {
        const response = await requester({baseUrl, resolvedAddress: loopback, row, timeoutMs, ignoreHttpsErrors, siteId});
        return {pass, response, ...observationResult(row, response), error: null};
      } catch (error) {
        return {pass, response: null, status: "fail", differences: ["request_error"], error: error.message ?? String(error)};
      }
    });
    for (let index = 0; index < rows.length; index += 1) {
      observationsByFixture.get(rows[index].fixture_id).push(observations[index]);
    }
  }
  const resultRows = rows.map((row) => {
    const observations = observationsByFixture.get(row.fixture_id);
    const isReproducible = observations.every((item) => item.response !== null) && reproducible(observations);
    const status = observations.every((item) => item.status === "pass") && isReproducible ? "pass" : "fail";
    return {...row, observations, reproducible: isReproducible, status};
  });
  const failed = resultRows.filter((row) => row.status !== "pass").map((row) => row.fixture_id);
  const verdict = {
    schema: REDIRECT_VERDICT_SCHEMA,
    status: failed.length === 0 ? "pass" : "fail",
    runtime_identity: runtimeIdentity,
    inputs: {
      inventory: {path: inventoryPath, sha256: sha256Value(inventoryBytes)},
      authority: {path: authorityPath, sha256: sha256Value(authorityBytes)},
      corpus_redirects: {path: corpusRedirectsPath, sha256: sha256Value(corpusBytes)},
      runtime_identity: {path: runtimeIdentityPath, sha256: sha256Value(runtimeIdentityBytes)},
    },
    contract: {
      local_base: baseUrl.origin,
      resolved_address: loopback.address,
      passes: 2,
      workers,
      timeout_ms: timeoutMs,
      ignore_https_errors: ignoreHttpsErrors,
      injected_site_identity: siteId === null ? null : {site_id: siteId, site_slug: "scp-wiki"},
      redirects_followed: false,
      max_body_bytes: MAX_BODY_BYTES,
    },
    expected_count: rows.length,
    observed_count: resultRows.length,
    failed_count: failed.length,
    failed_fixtures: failed,
    rows: resultRows,
  };
  await writeJsonAtomic(outputPath, verdict);
  return verdict;
}

export {RUNTIME_IDENTITY_SCHEMA, sha256File};
