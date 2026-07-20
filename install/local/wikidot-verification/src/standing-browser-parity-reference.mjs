import fs from "node:fs/promises";
import path from "node:path";

import {
  STANDING_BROWSER_CAPTURE_SCHEMA,
  STANDING_BROWSER_LIVE_REFERENCE_SCHEMA,
  canaryContractForPair,
  currentCanaryContractSummary,
  evaluateFirstPaintCustomProperties,
  evaluatePresenceProbes,
  isExternalFailure,
  policyAllowsFailure,
  validateLiveCompletionPolicy,
  validateThresholds,
} from "./standing-browser-parity-contract.mjs";
import {
  isPlainObject,
  normalizedUrl,
  requireNonEmptyString,
  requirePlainObject,
  requireSha256,
  sha256File,
  sha256Value,
} from "./standing-browser-parity-util.mjs";

function sameJson(left, right) {
  return sha256Value(left) === sha256Value(right);
}

function normalizedPair(value) {
  const pair = requirePlainObject(value, "canary pair");
  return {
    local_url: normalizedUrl(pair.local_url, "canary pair local_url").href,
    live_url: normalizedUrl(pair.live_url, "canary pair live_url").href,
  };
}

function safeArtifactName(value, label) {
  const name = requireNonEmptyString(value, label);
  if (
    path.basename(name) !== name ||
    /[\\/]/u.test(name) ||
    name === "." ||
    name === ".."
  ) {
    throw new Error(`${label} must be a safe artifact basename`);
  }
  return name;
}

async function verifyScreenshotArtifact(
  root,
  screenshot,
  label,
  expectedFullPage,
) {
  const metadata = requirePlainObject(
    screenshot,
    `live reference ${label} screenshot`,
  );
  const fileName = safeArtifactName(
    metadata.path,
    `live reference ${label} screenshot path`,
  );
  if (metadata.full_page !== expectedFullPage) {
    throw new Error(
      `live reference ${label} screenshot has the wrong full-page value`,
    );
  }
  const expectedSha256 = requireSha256(
    metadata.sha256,
    `live reference ${label} screenshot SHA-256`,
  );
  const filePath = path.join(root, fileName);
  const stat = await fs.lstat(filePath).catch(() => null);
  if (!stat?.isFile() || stat.isSymbolicLink()) {
    throw new Error(`live reference ${label} screenshot is unavailable`);
  }
  const actualSha256 = await sha256File(filePath);
  if (actualSha256 !== expectedSha256) {
    throw new Error(`live reference ${label} screenshot SHA-256 mismatch`);
  }
  return { path: fileName, sha256: actualSha256, full_page: expectedFullPage };
}

function validateObservedFailure(failure, capture, policy) {
  const value = requirePlainObject(failure, "live reference failure");
  normalizedUrl(value.url, "live reference failure.url");
  if (!policyAllowsFailure(policy, value, capture)) {
    throw new Error(
      `live reference contains an unapproved failure: ${value.url}`,
    );
  }
}

function normalizedBrokenImageFailure(image) {
  const value = requirePlainObject(image, "live reference broken image");
  return {
    kind: "broken_image",
    url: normalizedUrl(value.src, "live reference broken image.src").href,
    resource_type: "image",
    error: "image did not decode",
  };
}

function validateRequestGate(value, { minimumPublicRequests = 0 } = {}) {
  const gate = requirePlainObject(value, "live reference request gate");
  if (gate.schema !== "wikijump_full_parity.browser_request_gate.v1") {
    throw new Error("live reference request gate has an unsupported schema");
  }
  if (!Number.isInteger(gate.interval_ms) || gate.interval_ms < 4_000) {
    throw new Error(
      "live reference request gate must preserve the initial 0.25 req/s throttle",
    );
  }
  if (gate.enforcement_failed !== false) {
    throw new Error("live reference request gate enforcement was not clean");
  }
  if (
    !Number.isInteger(gate.public_requests) ||
    gate.public_requests < minimumPublicRequests
  ) {
    throw new Error(
      "live reference request gate did not admit every required live navigation",
    );
  }
  return {
    ...gate,
    config_sha256: requireSha256(
      gate.config_sha256,
      "live reference request gate config SHA-256",
    ),
  };
}

async function validateLiveCapture(capture, pair, root, policy) {
  const value = requirePlainObject(capture, "live reference capture");
  if (value.schema !== STANDING_BROWSER_CAPTURE_SCHEMA) {
    throw new Error(
      `live reference capture has an unsupported schema for ${pair.live_url}`,
    );
  }
  if (value.navigation_status !== 200 || value.capture_error) {
    throw new Error(`live reference is incomplete for ${pair.live_url}`);
  }
  if (
    value.first_paint?.document?.phase !==
    "domcontentloaded_immediate_observation"
  ) {
    throw new Error(
      `live reference lacks the required DOMContentLoaded observation for ${pair.live_url}`,
    );
  }
  if (value.document?.phase !== "settled") {
    throw new Error(
      `live reference lacks the required settled observation for ${pair.live_url}`,
    );
  }
  if (value.document?.resource_completion?.status !== "complete") {
    throw new Error(
      `live reference did not complete load, font, and image observation for ${pair.live_url}`,
    );
  }
  if (
    normalizedUrl(value.input_url, "live reference input_url").href !==
    pair.live_url
  ) {
    throw new Error(`live reference URL mismatch: expected ${pair.live_url}`);
  }
  if (
    normalizedUrl(value.final_url, "live reference final_url").href !==
    pair.live_url
  ) {
    throw new Error(
      `live reference final URL mismatch: expected ${pair.live_url}`,
    );
  }
  const contract = canaryContractForPair(pair);
  const immediateProperties = evaluateFirstPaintCustomProperties(
    value.first_paint?.document?.custom_properties,
    contract.first_paint_custom_properties,
  );
  if (immediateProperties.status !== "pass") {
    throw new Error(
      `live reference fails DOMContentLoaded theme properties for ${pair.live_url}`,
    );
  }
  for (const phase of [value.first_paint?.document, value.document]) {
    const failed = evaluatePresenceProbes(
      phase?.presence_probes,
      contract.presence_probes,
    ).filter((probe) => probe.status !== "pass");
    if (failed.length > 0) {
      throw new Error(
        `live reference fails required browser probes for ${pair.live_url}: ${failed.map((probe) => probe.id).join(", ")}`,
      );
    }
  }
  for (const failure of value.failures ?? []) {
    validateObservedFailure(failure, value, policy);
  }
  if (!Array.isArray(value.broken_images)) {
    throw new Error(
      `live reference lacks broken image observations for ${pair.live_url}`,
    );
  }
  for (const image of value.broken_images) {
    const failure = normalizedBrokenImageFailure(image);
    if (!isExternalFailure(failure, value)) {
      throw new Error(
        `live reference has a broken first-party image: ${failure.url}`,
      );
    }
    if (!policyAllowsFailure(policy, failure, value)) {
      throw new Error(
        `live reference has an unapproved broken external image: ${failure.url}`,
      );
    }
  }
  const artifacts = {
    domcontentloaded_immediate: await verifyScreenshotArtifact(
      root,
      value.first_paint?.screenshot,
      "DOMContentLoaded immediate",
      false,
    ),
    settled_viewport: await verifyScreenshotArtifact(
      root,
      value.settled_viewport_screenshot,
      "settled viewport",
      false,
    ),
    settled_full_page: await verifyScreenshotArtifact(
      root,
      value.screenshot,
      "settled full page",
      true,
    ),
  };
  return {
    capture: value,
    artifacts,
  };
}

export async function validateLiveReferenceRecord({
  capture,
  pair,
  root,
  policy,
}) {
  const checkedPair = normalizedPair(pair);
  const checkedPolicy = validateLiveCompletionPolicy(policy);
  return await validateLiveCapture(capture, checkedPair, root, checkedPolicy);
}

function captureContract({ viewport, thresholds, policy, policySha256 }) {
  const canaries = currentCanaryContractSummary();
  return {
    ...canaries,
    viewport,
    thresholds,
    domcontentloaded_immediate_observation: {
      viewport_screenshot: true,
      custom_properties: [
        "--logo",
        "--header-logo",
        "--header-title",
        "--header-subtitle",
      ],
      pseudo_layout:
        "CDP DOMSnapshot generated-content layout with clipping evidence",
      limitation:
        "This samples DOM/CSS state immediately after DOMContentLoaded; it is not a compositor-filmstrip timestamp.",
    },
    settled_capture: {
      viewport_screenshot: true,
      full_page_screenshot: true,
      pseudo_layout_geometry_comparison: true,
    },
    completion_policy: {
      schema: policy.schema,
      policy_version: policy.policy_version,
      policy_sha256: policySha256,
    },
  };
}

export function buildLiveReferenceLedger({
  records,
  viewport,
  thresholds,
  policy,
  policySha256,
  browserEnvironment,
  requestGate,
  generatedAt = new Date().toISOString(),
}) {
  if (!Array.isArray(records) || records.length === 0) {
    throw new Error("live reference records must be a non-empty array");
  }
  const checkedPolicy = validateLiveCompletionPolicy(policy);
  requireSha256(policySha256, "live completion policy SHA-256");
  const checkedThresholds = validateThresholds(thresholds);
  const checkedBrowser = requirePlainObject(
    browserEnvironment,
    "live reference browser environment",
  );
  requireNonEmptyString(
    checkedBrowser.engine,
    "live reference browser environment.engine",
  );
  requireNonEmptyString(
    checkedBrowser.version,
    "live reference browser environment.version",
  );
  requireSha256(
    checkedBrowser.executable_sha256,
    "live reference browser executable SHA-256",
  );
  const normalizedRecords = records.map((record) => {
    const input = normalizedPair(record.input);
    canaryContractForPair(input);
    return {
      input,
      live: requirePlainObject(record.live, "live reference capture"),
    };
  });
  const liveUrls = normalizedRecords
    .map((record) => record.input.live_url)
    .sort();
  if (new Set(liveUrls).size !== liveUrls.length) {
    throw new Error("live reference contains duplicate canary URLs");
  }
  return {
    schema: STANDING_BROWSER_LIVE_REFERENCE_SCHEMA,
    status: "sealed",
    generated_at: requireNonEmptyString(
      generatedAt,
      "live reference generated_at",
    ),
    capture_contract: captureContract({
      viewport,
      thresholds: checkedThresholds,
      policy: checkedPolicy,
      policySha256,
    }),
    browser: {
      engine: checkedBrowser.engine,
      version: checkedBrowser.version,
      executable_sha256: checkedBrowser.executable_sha256,
    },
    request_gate: validateRequestGate(requestGate, {
      minimumPublicRequests: normalizedRecords.length,
    }),
    records: normalizedRecords,
  };
}

export async function loadSealedLiveReference({
  filePath,
  expectedSha256,
  pairs,
  viewport,
  thresholds,
  policy,
  policySha256,
  policyFilePath,
}) {
  const actualSha256 = await sha256File(filePath);
  if (
    actualSha256 !==
    requireSha256(expectedSha256, "live reference expected SHA-256")
  ) {
    throw new Error("live reference SHA-256 mismatch");
  }
  const reference = JSON.parse(await fs.readFile(filePath, "utf8"));
  if (
    !isPlainObject(reference) ||
    reference.schema !== STANDING_BROWSER_LIVE_REFERENCE_SCHEMA
  ) {
    throw new Error("live reference has an unsupported schema");
  }
  if (reference.status !== "sealed") {
    throw new Error("live reference is not sealed");
  }
  requireNonEmptyString(reference.generated_at, "live reference generated_at");
  validateRequestGate(reference.request_gate, {
    minimumPublicRequests: Array.isArray(reference.records)
      ? reference.records.length
      : 0,
  });
  const checkedPolicy = validateLiveCompletionPolicy(policy);
  const checkedPolicySha256 = requireSha256(
    policySha256,
    "live completion policy SHA-256",
  );
  if (typeof policyFilePath !== "string" || policyFilePath === "") {
    throw new Error("live completion policy file path is required");
  }
  if ((await sha256File(policyFilePath)) !== checkedPolicySha256) {
    throw new Error(
      "live completion policy file does not match its supplied SHA-256",
    );
  }
  const policyFromFile = validateLiveCompletionPolicy(
    JSON.parse(await fs.readFile(policyFilePath, "utf8")),
  );
  if (!sameJson(policyFromFile, checkedPolicy)) {
    throw new Error(
      "live completion policy value does not match its sealed file",
    );
  }
  const expectedContract = captureContract({
    viewport,
    thresholds: validateThresholds(thresholds),
    policy: checkedPolicy,
    policySha256: checkedPolicySha256,
  });
  if (!sameJson(reference.capture_contract, expectedContract)) {
    throw new Error(
      "live reference capture contract does not match this candidate parity run",
    );
  }
  const expectedPairs = pairs.map(normalizedPair);
  const seen = new Map();
  for (const entry of reference.records ?? []) {
    const input = normalizedPair(entry.input);
    if (seen.has(input.live_url)) {
      throw new Error(
        `live reference contains duplicate URL: ${input.live_url}`,
      );
    }
    seen.set(input.live_url, entry);
  }
  if (seen.size !== expectedPairs.length) {
    throw new Error(
      "live reference does not contain exactly the requested canaries",
    );
  }
  const root = path.dirname(filePath);
  const records = [];
  for (const pair of expectedPairs) {
    const entry = seen.get(pair.live_url);
    if (!entry || normalizedPair(entry.input).live_url !== pair.live_url) {
      throw new Error(
        `live reference does not bind the requested pair: ${pair.live_url}`,
      );
    }
    const validated = await validateLiveCapture(
      entry.live,
      pair,
      root,
      checkedPolicy,
    );
    records.push({ input: pair, ...validated });
  }
  return {
    reference,
    sha256: actualSha256,
    policy: checkedPolicy,
    identity: {
      sha256: actualSha256,
      generated_at: requireNonEmptyString(
        reference.generated_at,
        "live reference generated_at",
      ),
      policy_version: checkedPolicy.policy_version,
      policy_sha256: checkedPolicySha256,
      canary_contract_sha256: reference.capture_contract.canary_contract_sha256,
    },
    records,
  };
}
