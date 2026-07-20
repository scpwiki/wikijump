import {
  COMMON_GEOMETRY_SELECTORS,
  STANDING_BROWSER_CANARY_SCHEMA,
  assertThemeFamilyCoverage,
  canaryForUrl,
  firstPaintPropertyNames,
  standingBrowserCanaryContractSha256,
} from "./standing-browser-canaries.mjs";
import {
  comparePseudoLayouts,
  evaluatePseudoLayout,
} from "./standing-browser-pseudo-layout.mjs";
import {
  isPlainObject,
  normalizedUrl,
  requireNonEmptyString,
  requirePlainObject,
  sortedUniqueStrings,
} from "./standing-browser-parity-util.mjs";

export const STANDING_BROWSER_CAPTURE_SCHEMA =
  "wikijump_local_lab.standing_browser_parity_capture.v2";
export const STANDING_BROWSER_PARITY_SCHEMA =
  "wikijump_local_lab.standing_browser_parity_run.v2";
export const STANDING_BROWSER_LIVE_REFERENCE_SCHEMA =
  "wikijump_local_lab.standing_browser_live_reference.v2";
export const STANDING_BROWSER_LIVE_POLICY_SCHEMA =
  "wikijump.standing_browser_live_completion_policy.v1";
export const STANDING_CANDIDATE_PARITY_IDENTITY_SCHEMA =
  "wikijump.standing_candidate_parity_identity.v1";
export const STANDING_CANDIDATE_PARITY_RECEIPT_SCHEMA =
  "wikijump.standing_candidate_parity_receipt.v1";

export const DEFAULT_THRESHOLDS = Object.freeze({
  geometry_position_px: 8,
  geometry_size_px: 12,
  image_count_delta: 0,
  dom_multiset_distance_ratio: 0.15,
});

function finiteNonNegative(value, name) {
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`${name} must be a non-negative finite number`);
  }
  return value;
}

export function validateThresholds(value = DEFAULT_THRESHOLDS) {
  const thresholds = requirePlainObject(value, "thresholds");
  const result = {};
  for (const key of Object.keys(DEFAULT_THRESHOLDS)) {
    result[key] = finiteNonNegative(thresholds[key], `thresholds.${key}`);
  }
  if (Object.keys(thresholds).some((key) => !(key in DEFAULT_THRESHOLDS))) {
    throw new Error("thresholds contains an unknown field");
  }
  return Object.freeze(result);
}

function failureKey(failure) {
  return JSON.stringify([
    failure?.kind ?? null,
    failure?.url ?? null,
    failure?.status ?? null,
    failure?.resource_type ?? null,
    failure?.error ?? null,
  ]);
}

export function isExternalFailure(failure, capture) {
  let parsed;
  try {
    parsed = new URL(failure?.url);
  } catch {
    return false;
  }
  if (!new Set(["http:", "https:"]).has(parsed.protocol)) return false;
  const hostnames = new Set();
  for (const value of [
    capture?.input_url,
    capture?.final_url ?? capture?.input_url,
  ]) {
    try {
      hostnames.add(new URL(value).hostname.toLowerCase());
    } catch {
      return false;
    }
  }
  return !hostnames.has(parsed.hostname.toLowerCase());
}

function multiset(items) {
  const result = new Map();
  for (const item of items ?? []) {
    result.set(item, (result.get(item) ?? 0) + 1);
  }
  return result;
}

export function multisetDistance(leftItems, rightItems) {
  const left = multiset(leftItems);
  const right = multiset(rightItems);
  const keys = new Set([...left.keys(), ...right.keys()]);
  let difference = 0;
  let total = 0;
  for (const key of keys) {
    difference += Math.abs((left.get(key) ?? 0) - (right.get(key) ?? 0));
    total += Math.max(left.get(key) ?? 0, right.get(key) ?? 0);
  }
  return {
    different_elements: difference,
    union_elements: total,
    ratio: total === 0 ? 0 : difference / total,
  };
}

export function normalizeCssValue(value) {
  return String(value ?? "")
    .trim()
    .replace(/https?:\/\/[^/'")\s]+/gu, "")
    .replace(/\s+/gu, " ");
}

export function matchesPropertyExpectation(value, expectation) {
  if (!isPlainObject(expectation)) return false;
  if (expectation.operator === "eq") return value === expectation.value;
  if (expectation.operator === "contains") {
    return typeof value === "string" && value.includes(expectation.value);
  }
  return false;
}

export function evaluateFirstPaintCustomProperties(
  customProperties,
  expectations,
) {
  const checks = Object.entries(expectations ?? {})
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([property, expectation]) => {
      const actual = customProperties?.[property] ?? null;
      return {
        property,
        actual,
        expected: expectation,
        status: matchesPropertyExpectation(actual, expectation)
          ? "pass"
          : "fail",
      };
    });
  return {
    status: checks.every((check) => check.status === "pass") ? "pass" : "fail",
    checks,
  };
}

export function evaluatePresenceProbes(probes, requirements) {
  const byId = new Map((probes ?? []).map((probe) => [probe.id, probe]));
  return (requirements ?? []).map((requirement) => {
    const observed = byId.get(requirement.id);
    const minimumCount = requirement.minimum_count ?? 1;
    const count = observed?.count ?? 0;
    const renderedCount = observed?.rendered_count ?? 0;
    const countPasses = count >= minimumCount;
    const pseudoLayout = evaluatePseudoLayout(observed, requirement);
    const renderedPasses = requirement.pseudo_layout
      ? pseudoLayout.status === "pass"
      : !requirement.require_rendered || renderedCount >= minimumCount;
    return {
      id: requirement.id,
      selector: requirement.selector,
      pseudo: requirement.pseudo ?? null,
      minimum_count: minimumCount,
      require_rendered: Boolean(requirement.require_rendered),
      observed_count: count,
      observed_rendered_count: renderedCount,
      pseudo_layout: pseudoLayout,
      status: countPasses && renderedPasses ? "pass" : "fail",
    };
  });
}

function geometryFor(capture, selector) {
  if (capture?.geometry?.[selector]) return capture.geometry[selector];
  if (capture?.required_geometry_by_selector?.[selector]) {
    return capture.required_geometry_by_selector[selector];
  }
  if (capture?.required_geometry) {
    return (
      capture.required_geometry.find(
        (geometry) => geometry.selector === selector,
      ) ?? null
    );
  }
  return (
    capture?.document?.required_geometry?.find(
      (geometry) => geometry.selector === selector,
    ) ?? null
  );
}

function rounded(value) {
  return Math.round(Number(value) * 100) / 100;
}

export function compareGeometry(
  local,
  live,
  selectors,
  thresholds,
  prefix = "",
) {
  const geometry = [];
  const anomalies = [];
  const allSelectors = new Set([
    ...selectors,
    ...Object.keys(
      local?.required_geometry_by_selector ?? local?.geometry ?? {},
    ),
    ...Object.keys(live?.required_geometry_by_selector ?? live?.geometry ?? {}),
  ]);
  for (const selector of allSelectors) {
    const localGeometry = geometryFor(local, selector);
    const liveGeometry = geometryFor(live, selector);
    if (
      !localGeometry ||
      !liveGeometry ||
      localGeometry.count !== liveGeometry.count ||
      localGeometry.count !== 1
    ) {
      const detail = {
        selector,
        local_count: localGeometry?.count ?? 0,
        live_count: liveGeometry?.count ?? 0,
      };
      geometry.push({ ...detail, status: "fail" });
      anomalies.push({ code: `${prefix}selector_count_divergence`, detail });
      continue;
    }
    const delta = Object.fromEntries(
      ["x", "y", "width", "height"].map((key) => [
        key,
        rounded(localGeometry.rect[key] - liveGeometry.rect[key]),
      ]),
    );
    const status =
      Math.abs(delta.x) <= thresholds.geometry_position_px &&
      Math.abs(delta.y) <= thresholds.geometry_position_px &&
      Math.abs(delta.width) <= thresholds.geometry_size_px &&
      Math.abs(delta.height) <= thresholds.geometry_size_px
        ? "pass"
        : "fail";
    const row = {
      selector,
      local: localGeometry.rect,
      live: liveGeometry.rect,
      delta,
      status,
    };
    geometry.push(row);
    if (status === "fail") {
      anomalies.push({
        code: `${prefix}selector_geometry_divergence`,
        detail: row,
      });
    }
  }
  return { geometry, anomalies };
}

function propertyObservations(local, live, contract) {
  const localProperties = local.first_paint?.document?.custom_properties ?? {};
  const liveProperties = live.first_paint?.document?.custom_properties ?? {};
  return Object.entries(contract?.first_paint_custom_properties ?? {})
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([property, expected]) => {
      const localValue = localProperties[property] ?? null;
      const liveValue = liveProperties[property] ?? null;
      const normalizedLocal = normalizeCssValue(localValue);
      const normalizedLive = normalizeCssValue(liveValue);
      return {
        property,
        expected,
        local: localValue,
        live: liveValue,
        normalized_local: normalizedLocal,
        normalized_live: normalizedLive,
        local_expected: matchesPropertyExpectation(localValue, expected),
        live_expected: matchesPropertyExpectation(liveValue, expected),
        status:
          matchesPropertyExpectation(localValue, expected) &&
          matchesPropertyExpectation(liveValue, expected) &&
          normalizedLocal === normalizedLive
            ? "pass"
            : "fail",
      };
    });
}

function probeObservations(
  localPhase,
  livePhase,
  contract,
  thresholds,
  { comparePseudoGeometry = false } = {},
) {
  const localProbes = new Map(
    (localPhase?.presence_probes ?? []).map((probe) => [probe.id, probe]),
  );
  const liveProbes = new Map(
    (livePhase?.presence_probes ?? []).map((probe) => [probe.id, probe]),
  );
  return (contract?.presence_probes ?? []).map((requirement) => {
    const local = localProbes.get(requirement.id) ?? null;
    const live = liveProbes.get(requirement.id) ?? null;
    const minimumCount = requirement.minimum_count ?? 1;
    const countMatches =
      local?.count === live?.count && (local?.count ?? 0) >= minimumCount;
    const pseudoLayout = requirement.pseudo_layout
      ? comparePseudoLayouts(local, live, requirement, thresholds, {
          compareGeometry: comparePseudoGeometry,
        })
      : null;
    const renderedCountsMatch = requirement.pseudo_layout
      ? true
      : (local?.rendered_count ?? 0) === (live?.rendered_count ?? 0);
    const renderedRequirementPasses = requirement.pseudo_layout
      ? pseudoLayout.status === "pass"
      : !requirement.require_rendered ||
        ((local?.rendered_count ?? 0) >= minimumCount &&
          (live?.rendered_count ?? 0) >= minimumCount);
    const properties = (requirement.comparison_properties ?? []).map(
      (property) => {
        const localValue = local?.style?.[property] ?? null;
        const liveValue = live?.style?.[property] ?? null;
        const normalizedLocal = normalizeCssValue(localValue);
        const normalizedLive = normalizeCssValue(liveValue);
        let status = normalizedLocal === normalizedLive ? "pass" : "fail";
        if (
          (property === "width" || property === "height") &&
          /px$/u.test(normalizedLocal) &&
          /px$/u.test(normalizedLive)
        ) {
          status =
            Math.abs(
              Number.parseFloat(normalizedLocal) -
                Number.parseFloat(normalizedLive),
            ) <= thresholds.geometry_size_px
              ? "pass"
              : "fail";
        }
        return {
          property,
          local: localValue,
          live: liveValue,
          normalized_local: normalizedLocal,
          normalized_live: normalizedLive,
          status,
        };
      },
    );
    return {
      id: requirement.id,
      selector: requirement.selector,
      pseudo: requirement.pseudo ?? null,
      local_count: local?.count ?? 0,
      live_count: live?.count ?? 0,
      local_rendered_count: local?.rendered_count ?? 0,
      live_rendered_count: live?.rendered_count ?? 0,
      properties,
      pseudo_layout: pseudoLayout,
      status:
        countMatches &&
        renderedCountsMatch &&
        renderedRequirementPasses &&
        properties.every((property) => property.status === "pass")
          ? "pass"
          : "fail",
    };
  });
}

export function compareCaptures(
  local,
  live,
  thresholds = DEFAULT_THRESHOLDS,
  requiredSelectors = COMMON_GEOMETRY_SELECTORS,
  contract = null,
) {
  const checkedThresholds = validateThresholds(thresholds);
  const anomalies = [];
  const liveFailureKeys = new Set(
    (live.failures ?? [])
      .filter((failure) => isExternalFailure(failure, live))
      .map(failureKey),
  );
  const classifiedFailures = (local.failures ?? []).map((failure) => {
    const classification =
      isExternalFailure(failure, local) &&
      liveFailureKeys.has(failureKey(failure))
        ? "parity_matched"
        : "local_only";
    if (classification === "local_only") {
      anomalies.push({ code: "local_only_request_failure", detail: failure });
    }
    return { ...failure, classification };
  });
  const localFailureKeys = new Set(
    (local.failures ?? [])
      .filter((failure) => isExternalFailure(failure, local))
      .map(failureKey),
  );
  const liveOnlyFailures = (live.failures ?? []).filter(
    (failure) => !localFailureKeys.has(failureKey(failure)),
  );
  if (local.navigation_status !== 200) {
    anomalies.push({
      code: "local_main_response_not_200",
      detail: local.navigation_status,
    });
  }
  if (local.capture_error) {
    anomalies.push({
      code: "local_capture_error",
      detail: local.capture_error,
    });
  }
  if (live.navigation_status !== 200) {
    anomalies.push({
      code: "live_main_response_not_200",
      detail: live.navigation_status,
    });
  }
  if (live.capture_error) {
    anomalies.push({ code: "live_capture_error", detail: live.capture_error });
  }
  const selectors = contract?.geometry_selectors ?? requiredSelectors;
  const settledGeometry = compareGeometry(
    local,
    live,
    selectors,
    checkedThresholds,
  );
  anomalies.push(...settledGeometry.anomalies);
  const immediateGeometry = compareGeometry(
    local.first_paint?.document,
    live.first_paint?.document,
    contract?.geometry_selectors ?? [],
    checkedThresholds,
    "domcontentloaded_immediate_",
  );
  anomalies.push(...immediateGeometry.anomalies);
  const imageCountDelta =
    (local.rendered_images ?? 0) - (live.rendered_images ?? 0);
  if (Math.abs(imageCountDelta) > checkedThresholds.image_count_delta) {
    anomalies.push({
      code: "rendered_image_count_divergence",
      detail: {
        local: local.rendered_images ?? 0,
        live: live.rendered_images ?? 0,
        delta: imageCountDelta,
      },
    });
  }
  const classifiedBrokenImages = (local.broken_images ?? []).map((image) => {
    const classification =
      /^https?:\/\//u.test(image.src) &&
      isExternalFailure({ url: image.src }, local) &&
      (live.broken_images ?? []).some(
        (candidate) => candidate.src === image.src,
      )
        ? "parity_matched"
        : "local_only";
    if (classification === "local_only") {
      anomalies.push({ code: "local_broken_image", detail: image });
    }
    return { ...image, classification };
  });
  const localBrokenSources = new Set(
    (local.broken_images ?? []).map((image) => image.src),
  );
  const liveOnlyBrokenImages = (live.broken_images ?? []).filter(
    (image) => !localBrokenSources.has(image.src),
  );
  const dom = multisetDistance(
    local.dom_signatures ?? [],
    live.dom_signatures ?? [],
  );
  if (dom.ratio > checkedThresholds.dom_multiset_distance_ratio) {
    anomalies.push({ code: "dom_structure_divergence", detail: dom });
  }
  const immediateProperties = propertyObservations(local, live, contract);
  for (const property of immediateProperties) {
    if (property.status === "fail") {
      anomalies.push({
        code: "domcontentloaded_immediate_custom_property_divergence",
        detail: property,
      });
    }
  }
  const immediateProbes = probeObservations(
    local.first_paint?.document,
    live.first_paint?.document,
    contract,
    checkedThresholds,
    { comparePseudoGeometry: true },
  );
  for (const probe of immediateProbes) {
    if (probe.status === "fail") {
      anomalies.push({
        code: "domcontentloaded_immediate_probe_divergence",
        detail: probe,
      });
    }
  }
  const settledProbes = probeObservations(
    local.document,
    live.document,
    contract,
    checkedThresholds,
    { comparePseudoGeometry: true },
  );
  for (const probe of settledProbes) {
    if (probe.status === "fail") {
      anomalies.push({ code: "settled_probe_divergence", detail: probe });
    }
  }
  return {
    status: anomalies.length === 0 ? "pass" : "fail",
    classified_failures: classifiedFailures,
    live_only_failures: liveOnlyFailures,
    classified_broken_images: classifiedBrokenImages,
    live_only_broken_images: liveOnlyBrokenImages,
    geometry: settledGeometry.geometry,
    domcontentloaded_immediate_geometry: immediateGeometry.geometry,
    image_counts: {
      local: local.rendered_images ?? 0,
      live: live.rendered_images ?? 0,
      delta: imageCountDelta,
    },
    dom_multiset_distance: dom,
    domcontentloaded_immediate_custom_properties: immediateProperties,
    domcontentloaded_immediate_probes: immediateProbes,
    settled_probes: settledProbes,
    anomalies,
  };
}

function validateFailureAllowance(value, index) {
  const rule = requirePlainObject(value, `allowed_external_failures[${index}]`);
  const url = normalizedUrl(
    rule.url,
    `allowed_external_failures[${index}].url`,
  );
  const kind = requireNonEmptyString(
    rule.kind,
    `allowed_external_failures[${index}].kind`,
  );
  if (!new Set(["http_error", "request_failed", "broken_image"]).has(kind)) {
    throw new Error(`allowed_external_failures[${index}].kind is unsupported`);
  }
  const resourceType = requireNonEmptyString(
    rule.resource_type,
    `allowed_external_failures[${index}].resource_type`,
  );
  const result = { url: url.href, kind, resource_type: resourceType };
  if (kind === "http_error") {
    if (
      !Number.isInteger(rule.status) ||
      rule.status < 100 ||
      rule.status > 599
    ) {
      throw new Error(
        `allowed_external_failures[${index}].status must be an HTTP status`,
      );
    }
    if ("error" in rule) {
      throw new Error(
        `allowed_external_failures[${index}] cannot combine http_error with error`,
      );
    }
    result.status = rule.status;
  } else {
    if ("status" in rule) {
      throw new Error(
        `allowed_external_failures[${index}] cannot combine ${kind} with status`,
      );
    }
    result.error = requireNonEmptyString(
      rule.error,
      `allowed_external_failures[${index}].error`,
    );
  }
  if (
    !isExternalFailure(
      { url: url.href },
      {
        input_url: "https://scp-wiki.wikidot.com/",
        final_url: "https://scp-wiki.wikidot.com/",
      },
    )
  ) {
    throw new Error(
      `allowed_external_failures[${index}].url must name an external HTTP(S) resource`,
    );
  }
  return result;
}

export function validateLiveCompletionPolicy(value) {
  const policy = requirePlainObject(value, "live completion policy");
  if (policy.schema !== STANDING_BROWSER_LIVE_POLICY_SCHEMA) {
    throw new Error(
      `live completion policy must use ${STANDING_BROWSER_LIVE_POLICY_SCHEMA}`,
    );
  }
  const policyVersion = requireNonEmptyString(
    policy.policy_version,
    "live completion policy.policy_version",
  );
  if (policy.status !== "sealed") {
    throw new Error("live completion policy.status must be sealed");
  }
  if (!Array.isArray(policy.allowed_external_failures)) {
    throw new Error(
      "live completion policy.allowed_external_failures must be an array",
    );
  }
  const allowed = policy.allowed_external_failures.map(
    validateFailureAllowance,
  );
  const keys = allowed.map(failureKey);
  sortedUniqueStrings(
    [...keys].sort(),
    "live completion policy allowed failure keys",
  );
  return Object.freeze({
    schema: STANDING_BROWSER_LIVE_POLICY_SCHEMA,
    status: "sealed",
    policy_version: policyVersion,
    allowed_external_failures: Object.freeze(
      [...allowed].sort((left, right) =>
        failureKey(left).localeCompare(failureKey(right)),
      ),
    ),
  });
}

export function policyAllowsFailure(policy, failure, capture) {
  if (!isExternalFailure(failure, capture)) return false;
  return policy.allowed_external_failures.some(
    (rule) => failureKey(rule) === failureKey(failure),
  );
}

export function currentCanaryContractSummary() {
  return {
    canary_schema: STANDING_BROWSER_CANARY_SCHEMA,
    canary_contract_sha256: standingBrowserCanaryContractSha256(),
    theme_family_coverage: assertThemeFamilyCoverage(),
  };
}

export function canaryContractForPair(pair) {
  const live = normalizedUrl(pair.live_url, "pair.live_url");
  const local = normalizedUrl(pair.local_url, "pair.local_url");
  const contract = canaryForUrl(live.href);
  if (!contract || canaryForUrl(local.href)?.slug !== contract.slug) {
    throw new Error(`pair must name a declared standing canary: ${live.href}`);
  }
  for (const property of firstPaintPropertyNames(contract)) {
    if (!contract.first_paint_custom_properties[property]) {
      throw new Error(
        `canary has an invalid immediate-observation property: ${property}`,
      );
    }
  }
  return contract;
}
