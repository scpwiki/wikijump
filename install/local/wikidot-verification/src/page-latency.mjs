import fs from "node:fs";
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";

function positiveInteger(value, name) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new Error(`${name} must be a non-negative integer`);
  }
  return parsed;
}

export function parseArgs(argv) {
  const args = {
    url: null,
    compareUrl: null,
    requests: 20,
    warmups: 3,
    headers: {},
    output: null,
    requireStableBody: false,
  };
  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = () => {
      i += 1;
      if (i >= argv.length) throw new Error(`${arg} requires a value`);
      return argv[i];
    };
    if (arg === "--url") args.url = next();
    else if (arg === "--compare-url") args.compareUrl = next();
    else if (arg === "--requests") args.requests = positiveInteger(next(), "--requests");
    else if (arg === "--warmups") args.warmups = positiveInteger(next(), "--warmups");
    else if (arg === "--header") {
      const value = next();
      const separator = value.indexOf(":");
      if (separator <= 0) throw new Error("--header must be NAME:VALUE");
      args.headers[value.slice(0, separator).trim()] = value.slice(separator + 1).trim();
    } else if (arg === "--output") args.output = next();
    else if (arg === "--require-stable-body") args.requireStableBody = true;
    else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  if (!args.help && !args.url) throw new Error("--url is required");
  if (args.requests === 0) throw new Error("--requests must be greater than 0");
  return args;
}

export function percentile(values, percentileValue) {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.max(0, Math.ceil((percentileValue / 100) * sorted.length) - 1);
  return sorted[Math.min(index, sorted.length - 1)];
}

function round(value) {
  return value == null ? null : Number(value.toFixed(3));
}

function statusCounts(samples) {
  const counts = {};
  for (const sample of samples) {
    const key = String(sample.status);
    counts[key] = (counts[key] ?? 0) + 1;
  }
  return counts;
}

export function summarizeSamples(samples) {
  const durations = samples.map((sample) => sample.duration_ms);
  const bytes = samples.map((sample) => sample.bytes);
  const hashes = [...new Set(samples.map((sample) => sample.body_sha256))];
  const sum = durations.reduce((total, value) => total + value, 0);
  return {
    requests: samples.length,
    ok: samples.filter((sample) => sample.ok).length,
    status_counts: statusCounts(samples),
    duration_ms: {
      min: round(Math.min(...durations)),
      mean: round(sum / durations.length),
      p50: round(percentile(durations, 50)),
      p90: round(percentile(durations, 90)),
      p95: round(percentile(durations, 95)),
      p99: round(percentile(durations, 99)),
      max: round(Math.max(...durations)),
    },
    response_bytes: {
      min: Math.min(...bytes),
      max: Math.max(...bytes),
    },
    body_stable: hashes.length === 1,
    body_sha256: hashes.length === 1 ? hashes[0] : null,
    body_sha256_values: hashes,
  };
}

async function oneFetch(fetchImpl, url, headers) {
  const started = performance.now();
  const response = await fetchImpl(url, { headers, redirect: "manual" });
  const body = Buffer.from(await response.arrayBuffer());
  const durationMs = performance.now() - started;
  return {
    status: response.status,
    ok: response.ok,
    duration_ms: durationMs,
    bytes: body.length,
    content_type: response.headers.get("content-type"),
    body_sha256: createHash("sha256").update(body).digest("hex"),
    body,
  };
}

function summarizeComparison(measuredSamples, comparisonSample, compareUrl) {
  const measuredHashes = [...new Set(measuredSamples.map((sample) => sample.body_sha256))];
  return {
    url: compareUrl,
    status: comparisonSample.status,
    bytes: comparisonSample.bytes,
    body_sha256: comparisonSample.body_sha256,
    same_body: measuredHashes.length === 1 && comparisonSample.body_sha256 === measuredHashes[0],
    same_bytes: measuredSamples.length > 0 && measuredSamples.every((sample) => sample.body.equals(comparisonSample.body)),
  };
}

export async function runPageLatency({ url, compareUrl = null, requests = 20, warmups = 3, headers = {}, fetchImpl = fetch } = {}) {
  if (!url) throw new Error("url is required");
  for (let i = 0; i < warmups; i += 1) {
    await oneFetch(fetchImpl, url, headers);
  }
  const samples = [];
  for (let i = 0; i < requests; i += 1) {
    samples.push(await oneFetch(fetchImpl, url, headers));
  }
  const summary = summarizeSamples(samples);
  if (compareUrl) {
    const comparisonSample = await oneFetch(fetchImpl, compareUrl, headers);
    summary.comparison = summarizeComparison(samples, comparisonSample, compareUrl);
  }
  return {
    schema_version: 1,
    measured_at: new Date().toISOString(),
    url,
    warmups,
    summary,
    samples: samples.map((sample, index) => {
      const reportSample = {...sample};
      delete reportSample.body;
      return {index: index + 1, ...reportSample, duration_ms: round(sample.duration_ms)};
    }),
  };
}

export function writeReport(report, output) {
  const json = JSON.stringify(report, null, 2);
  if (output) {
    fs.writeFileSync(output, `${json}\n`);
  }
  return json;
}
