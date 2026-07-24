import path from "node:path";

import {browserContextOptions, openBrowser} from "./browser-session.mjs";
import {
  prepareThemeArtifactDirectory,
  writePrivateThemeFile,
  writePrivateThemeJson,
  writeThemeViewportArtifacts,
} from "./theme-browser-artifacts.mjs";

export {
  prepareThemeArtifactDirectory,
  writeThemeViewportArtifacts,
} from "./theme-browser-artifacts.mjs";
import {startCaptureEgressProxy} from "./capture-egress-proxy.mjs";
import {collectLayoutShifts, collectTimingDiagnostics, installLayoutShiftObserver, installTimingObserver} from "./layout-diagnostics.mjs";
import {findRawSyntaxLeaks} from "./render-health.mjs";
import {RUN_OWNED_SLUG_PREFIX, assertRunOwnedSlug, themeComputedStyleProperties, validateTargetOrigin, validateThemeComputedStyleContract} from "./theme-localization-e2e.mjs";

export const THEME_BROWSER_CAPTURE_SCHEMA = "wikijump_local_lab.theme_browser_capture.v1";
export const THEME_PERFORMANCE_ATTRIBUTION_SCHEMA = "wikijump_local_lab.theme_performance_attribution.v1";
const DEFAULT_TIMEOUT_MS = 30_000;
const DEFAULT_SETTLE_MS = 250;
const DEFAULT_INTERACTION_TIMEOUT_MS = 1_000;

function safeId(value, label) {
  if (typeof value !== "string" || !/^[a-z0-9][a-z0-9-]{0,63}$/u.test(value)) throw new Error(`${label} is not a safe artifact identifier`);
  return value;
}

export function lcpObservationDeadlineMs(capture, settleMs = DEFAULT_SETTLE_MS) {
  const gate = capture?.web_vitals?.gates?.lcp_ms;
  if (gate?.operator !== "lte" || !Number.isFinite(gate.value) || gate.value <= 0 || !Number.isFinite(settleMs) || settleMs < 0) {
    throw new Error("theme capture requires a positive LCP upper bound and non-negative settle interval");
  }
  return gate.value + settleMs;
}

async function waitForLcpObservationWindow(page, capture, settleMs) {
  const deadlineMs = lcpObservationDeadlineMs(capture, settleMs);
  await page.evaluate(async (deadline) => {
    const remaining = deadline - performance.now();
    if (remaining > 0) await new Promise((resolve) => setTimeout(resolve, remaining));
  }, deadlineMs);
  return deadlineMs;
}

export function validateThemeCaptureTarget({tier, target}) {
  safeId(tier.id, "tier id");
  const prefix = RUN_OWNED_SLUG_PREFIX;
  const suffix = `-${tier.id}`;
  if (typeof tier.run_owned_slug !== "string" || !tier.run_owned_slug.startsWith(prefix) || !tier.run_owned_slug.endsWith(suffix)) throw new Error("tier does not carry a run-owned slug");
  assertRunOwnedSlug(tier.run_owned_slug, tier.run_owned_slug.slice(prefix.length, -suffix.length), tier.id);
  if (!target || !["wikidot", "wikijump"].includes(target.id)) throw new Error("capture target must be wikidot or wikijump");
  const url = new URL(target.url);
  validateTargetOrigin(url.origin, target.id);
  if (url.username || url.password || url.search || url.hash || decodeURIComponent(url.pathname) !== `/${tier.run_owned_slug}`) {
    throw new Error(`capture URL does not identify the run-owned ${tier.id} page`);
  }
  return url.href;
}

export async function installLocalFilePortRoute(context, target) {
  if (target?.id !== "wikijump") return false;
  const targetUrl = new URL(target.url);
  validateTargetOrigin(targetUrl.origin, target.id);
  if (!targetUrl.port) return false;
  const port = targetUrl.port;
  await context.route("https://*.wjfiles.localhost/**", async (route) => {
    const requestUrl = new URL(route.request().url());
    if (requestUrl.port || !/^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.wjfiles\.localhost$/u.test(requestUrl.hostname)) {
      await route.continue();
      return;
    }
    requestUrl.port = port;
    const response = await route.fetch({url: requestUrl.href});
    await route.fulfill({response});
  });
  return true;
}

function performanceObserverBootstrap() {
  const state = {fcp: null, lcp: null, lcpAttribution: null, cls: null, eventEntries: [], eventThresholdMs: 16, observers: [], support: {paint: false, lcp: false, cls: false, event: false}};
  window.__wjThemeMetrics = state;
  const supported = new Set(globalThis.PerformanceObserver?.supportedEntryTypes ?? []);
  const observe = (type, callback, options = {}) => {
    if (!supported.has(type)) return false;
    try {
      const observer = new PerformanceObserver((list) => callback(list.getEntries()));
      observer.observe({type, buffered: true, ...options});
      state.observers.push(observer);
      return true;
    } catch {
      return false;
    }
  };
  state.support.paint = observe("paint", (entries) => {
    const entry = entries.find((candidate) => candidate.name === "first-contentful-paint");
    if (entry) state.fcp = entry.startTime;
  });
  state.support.lcp = observe("largest-contentful-paint", (entries) => {
    if (!entries.length) return;
    const entry = entries.at(-1);
    state.lcp = entry.startTime;
    const element = entry.element;
    const tag = element?.tagName?.toLowerCase?.() ?? null;
    const id = bounded(element?.id, 120);
    const safeId = cssToken(id);
    const classes = [...(element?.classList ?? [])].map(cssToken).filter(Boolean).slice(0, 6);
    const selector = safeId ? `${tag ?? "node"}#${safeId}` : `${tag ?? "node"}${classes.slice(0, 3).map((name) => `.${name}`).join("")}`;
    state.lcpAttribution = {selector_hint: selector.slice(0, 240), tag, id, classes, url: bounded(entry.url, 500), size: Number.isFinite(entry.size) ? entry.size : null};
  });
  state.support.cls = observe("layout-shift", (entries) => {
    if (state.cls === null) state.cls = 0;
    for (const entry of entries) if (!entry.hadRecentInput) state.cls += entry.value;
  });
  if (state.support.cls && state.cls === null) state.cls = 0;
  state.support.event = observe("event", (entries) => {
    for (const entry of entries) state.eventEntries.push({name: entry.name, startTime: entry.startTime, duration: entry.duration, interactionId: entry.interactionId ?? 0});
  }, {durationThreshold: state.eventThresholdMs});

  function bounded(value, maximum) {
    const text = String(value ?? "").replace(/\s+/gu, " ").trim();
    return text ? text.slice(0, maximum) : null;
  }

  function cssToken(value) {
    return bounded(value, 80)?.replace(/\s+/gu, "-").replace(/[^a-zA-Z0-9_-]/gu, "") || null;
  }
}

function startErrorCapture(page) {
  const errors = {console: [], page: [], requests: [], responses: []};
  const append = (list, value) => {
    if (list.length < 200) list.push(value);
  };
  page.on("console", (message) => {
    if (message.type() === "error") append(errors.console, message.text().slice(0, 1_000));
  });
  page.on("pageerror", (error) => append(errors.page, (error.message ?? String(error)).slice(0, 1_000)));
  page.on("requestfailed", (request) => append(errors.requests, {url: request.url(), error: request.failure()?.errorText ?? "unknown"}));
  page.on("response", (response) => {
    if (response.status() >= 400) append(errors.responses, {url: response.url(), status: response.status(), resource_type: response.request().resourceType()});
  });
  return errors;
}

export async function collectComputedStyles(page, contract) {
  validateThemeComputedStyleContract(contract);
  const properties = themeComputedStyleProperties(contract);
  return page.evaluate(({properties, probes}) => probes.map((probe) => {
    const element = document.querySelector(probe.selector);
    if (!element) return {id: probe.id, selector: probe.selector, pseudo: probe.pseudo ?? null, expectation: probe.expectation, status: "missing", properties: null, rect: null};
    const style = getComputedStyle(element, probe.pseudo ?? null);
    const values = Object.fromEntries(properties.map((property) => [property, style.getPropertyValue(property)]));
    const rect = element.getBoundingClientRect();
    return {id: probe.id, selector: probe.selector, pseudo: probe.pseudo ?? null, expectation: probe.expectation, status: "measured", properties: values, rect: {x: rect.x, y: rect.y, width: rect.width, height: rect.height}};
  }), {properties, probes: contract.probes});
}

export async function collectNavigationMetrics(page) {
  return page.evaluate(() => {
    const state = window.__wjThemeMetrics ?? {support: {}};
    const navigation = performance.getEntriesByType("navigation")[0] ?? null;
    const fcp = state.fcp ?? performance.getEntriesByName("first-contentful-paint")[0]?.startTime ?? null;
    const pick = (entry, names) => entry ? Object.fromEntries(names.map((name) => [name.replace(/[A-Z]/gu, (letter) => `_${letter.toLowerCase()}`), entry[name]])) : null;
    const navigationTiming = pick(navigation, ["startTime", "duration", "redirectStart", "redirectEnd", "fetchStart", "domainLookupStart", "domainLookupEnd", "connectStart", "connectEnd", "requestStart", "responseStart", "responseEnd", "domInteractive", "domContentLoadedEventEnd", "loadEventEnd", "transferSize", "encodedBodySize", "decodedBodySize"]);
    return {
      navigation_timing: navigationTiming,
      web_vitals: {
        ttfb_ms: navigation ? navigation.responseStart - navigation.startTime : null,
        fcp_ms: fcp,
        lcp_ms: state.lcp,
        lcp_attribution: state.lcpAttribution,
        cls: state.cls,
        support: state.support,
      },
    };
  });
}

async function visibleLocator(page, selectors) {
  for (const selector of selectors) {
    const locator = page.locator(selector);
    const count = await locator.count();
    for (let index = 0; index < count; index += 1) if (await locator.nth(index).isVisible()) return {selector, index, locator: locator.nth(index)};
  }
  return null;
}

async function prepareInteraction(page, target, postcondition) {
  return page.evaluate(({selector, index, postcondition}) => {
    const element = document.querySelectorAll(selector)[index];
    if (!element) return false;
    const signature = () => {
      if (postcondition === "expanded_state_changes") {
        const root = element.closest("details, .collapsible-block") ?? element.parentElement;
        const folded = root?.querySelector?.(".collapsible-block-folded");
        const unfolded = root?.querySelector?.(".collapsible-block-unfolded");
        return JSON.stringify({open: root?.open ?? null, aria: element.getAttribute("aria-expanded"), root_class: root?.className ?? null, folded: folded ? getComputedStyle(folded).display : null, unfolded: unfolded ? getComputedStyle(unfolded).display : null});
      }
      const root = element.closest(".yui-navset") ?? element.parentElement;
      const all = [...root.querySelectorAll("*")];
      return JSON.stringify([...root.querySelectorAll(".selected, .active, [aria-selected='true']")].map((node) => ({index: all.indexOf(node), tag: node.tagName, id: node.id, class: node.className, aria: node.getAttribute("aria-selected"), href: node.getAttribute("href")})));
    };
    const state = {element, before: signature(), clickedAt: null, eventOffset: window.__wjThemeMetrics?.eventEntries.length ?? 0, signature};
    element.addEventListener("click", () => { state.clickedAt = performance.now(); }, {capture: true, once: true});
    window.__wjThemeInteraction = state;
    return true;
  }, {selector: target.selector, index: target.index, postcondition});
}

async function readInteraction(page, timeoutMs) {
  return page.evaluate(async (timeout) => {
    const state = window.__wjThemeInteraction;
    if (!state?.clickedAt) return {changed: false, visual_response_ms: null, event_timing: null, reason: "click event was not observed"};
    const deadline = state.clickedAt + timeout;
    let changedAt = null;
    while (performance.now() <= deadline) {
      if (state.signature() !== state.before) {
        await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        changedAt = performance.now();
        break;
      }
      await new Promise((resolve) => requestAnimationFrame(resolve));
    }
    await new Promise((resolve) => setTimeout(resolve, 0));
    const metrics = window.__wjThemeMetrics ?? {};
    const entries = (metrics.eventEntries ?? []).slice(state.eventOffset).filter((entry) => entry.interactionId > 0 && entry.startTime >= state.clickedAt - 100);
    const duration = entries.length ? Math.max(...entries.map((entry) => entry.duration)) : null;
    return {changed: changedAt !== null, visual_response_ms: changedAt === null ? null : changedAt - state.clickedAt, event_supported: metrics.support?.event === true, event_threshold_ms: metrics.eventThresholdMs ?? 16, event_timing: duration === null ? null : {duration_ms: duration, interaction_ids: [...new Set(entries.map((entry) => entry.interactionId))]}, reason: changedAt === null ? "postcondition did not change" : null};
  }, timeoutMs);
}

export async function captureInteraction(page, interaction, {timeoutMs = DEFAULT_INTERACTION_TIMEOUT_MS, visualResponseGateMs = null} = {}) {
  let target;
  try {
    target = await visibleLocator(page, interaction.target_selectors);
  } catch (error) {
    return {id: interaction.id, status: "fail", selector: null, visual_response_ms: null, inp_equivalent: {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null}, reason: error.message};
  }
  if (!target) return {id: interaction.id, status: "missing", selector: null, visual_response_ms: null, inp_equivalent: {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null}, reason: "no visible target selector"};
  let prepared;
  try {
    prepared = await prepareInteraction(page, target, interaction.postcondition);
  } catch (error) {
    return {id: interaction.id, status: "fail", selector: target.selector, visual_response_ms: null, inp_equivalent: {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null}, reason: error.message};
  }
  if (!prepared) return {id: interaction.id, status: "missing", selector: target.selector, visual_response_ms: null, inp_equivalent: {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null}, reason: "target detached before measurement"};
  const beforeUrl = new URL(page.url());
  try {
    await target.locator.click({timeout: timeoutMs});
  } catch (error) {
    return {id: interaction.id, status: "fail", selector: target.selector, visual_response_ms: null, inp_equivalent: {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null}, reason: error.message};
  }
  const afterUrl = new URL(page.url());
  if (afterUrl.origin !== beforeUrl.origin || afterUrl.pathname !== beforeUrl.pathname || afterUrl.search !== beforeUrl.search) {
    return {id: interaction.id, status: "fail", selector: target.selector, visual_response_ms: null, inp_equivalent: {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null}, reason: "interaction navigated away from the capture page"};
  }
  let measured;
  try {
    measured = await readInteraction(page, timeoutMs);
  } catch (error) {
    return {id: interaction.id, status: "fail", selector: target.selector, visual_response_ms: null, inp_equivalent: {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null}, reason: error.message};
  }
  return {
    id: interaction.id,
    status: measured.changed ? "measured" : "fail",
    selector: target.selector,
    visual_response_ms: measured.visual_response_ms,
    inp_equivalent: measured.event_timing ? {metric: "single_interaction_event_timing", formal_inp: false, status: "measured", duration_ms: measured.event_timing.duration_ms, interaction_ids: measured.event_timing.interaction_ids} : measured.event_supported && Number.isFinite(visualResponseGateMs) && measured.visual_response_ms <= visualResponseGateMs ? {metric: "single_interaction_event_timing", formal_inp: false, status: "bounded_below_threshold", duration_ms: null, upper_bound_ms: measured.event_threshold_ms, interaction_ids: []} : {metric: "single_interaction_event_timing", formal_inp: false, status: "missing", duration_ms: null, interaction_ids: []},
    reason: measured.reason ?? (measured.event_supported ? "No EventTiming entry at or above the observer threshold" : "PerformanceEventTiming unsupported"),
  };
}

function metricGate(id, actual, specification) {
  if (!Number.isFinite(actual)) return {id, status: "missing", actual: null, expected: specification};
  if (specification?.operator !== "lte" || !Number.isFinite(specification.value)) return {id, status: "fail", actual, expected: specification, reason: "unsupported gate specification"};
  return {id, status: actual <= specification.value ? "pass" : "fail", actual, expected: specification};
}

function boundedMetricGate(id, evidence, specification) {
  if (evidence?.status === "measured") return metricGate(id, evidence.duration_ms, specification);
  if (evidence?.status !== "bounded_below_threshold" || !Number.isFinite(evidence.upper_bound_ms)) return metricGate(id, null, specification);
  if (specification?.operator !== "lte" || !Number.isFinite(specification.value)) return {id, status: "fail", actual: {operator: "lt", value: evidence.upper_bound_ms}, expected: specification, reason: "unsupported gate specification"};
  return {id, status: evidence.upper_bound_ms <= specification.value ? "pass" : "fail", actual: {operator: "lt", value: evidence.upper_bound_ms}, expected: specification};
}

function propertyExpectationPass(actual, specification) {
  if (typeof actual !== "string") return false;
  return specification.operator === "eq" ? actual === specification.value : specification.values.includes(actual);
}

function computedStyleGate(observations, contract) {
  validateThemeComputedStyleContract(contract);
  const results = Array.isArray(observations) ? observations : [];
  const byId = new Map();
  const invalid = [];
  for (const observation of results) {
    if (!observation || typeof observation.id !== "string" || byId.has(observation.id)) {
      invalid.push(observation?.id ?? null);
      continue;
    }
    byId.set(observation.id, observation);
  }
  const plannedIds = new Set(contract.probes.map((probe) => probe.id));
  const unexpected = [...byId.keys()].filter((id) => !plannedIds.has(id));
  const requiredMissing = [];
  const optionalMissing = [];
  const expectedAbsentPresent = [];
  const propertyMismatches = [];
  const probeChecks = contract.probes.map((probe) => {
    const observation = byId.get(probe.id);
    const actual = observation?.status ?? "not_recorded";
    let status = "pass";
    if (!observation || observation.expectation !== probe.expectation || !new Set(["measured", "missing"]).has(actual)) {
      status = "fail";
      invalid.push(probe.id);
    } else if (probe.expectation === "required" && actual === "missing") {
      status = "missing";
      requiredMissing.push(probe.id);
    } else if (probe.expectation === "optional" && actual === "missing") {
      optionalMissing.push(probe.id);
    } else if (probe.expectation === "expected_absent" && actual === "measured") {
      status = "fail";
      expectedAbsentPresent.push(probe.id);
    }
    const propertyChecks = [];
    if (actual === "measured" && probe.expectation !== "expected_absent") {
      for (const [property, specification] of Object.entries(probe.expected_properties ?? {})) {
        const propertyActual = observation.properties?.[property];
        const propertyStatus = propertyExpectationPass(propertyActual, specification) ? "pass" : "fail";
        propertyChecks.push({property, status: propertyStatus, actual: propertyActual ?? null, expected: specification});
        if (propertyStatus === "fail") {
          status = "fail";
          propertyMismatches.push({probe_id: probe.id, property, actual: propertyActual ?? null, expected: specification});
        }
      }
    }
    return {id: probe.id, expectation: probe.expectation, actual, status, property_checks: propertyChecks};
  });
  const uniqueInvalid = [...new Set(invalid)];
  const status = uniqueInvalid.length || unexpected.length || expectedAbsentPresent.length || propertyMismatches.length ? "fail" : requiredMissing.length ? "missing" : "pass";
  return {id: "computed_style_probes", status, required_missing: requiredMissing, optional_missing: optionalMissing, expected_absent_present: expectedAbsentPresent, property_mismatches: propertyMismatches, invalid_observations: uniqueInvalid, unexpected_observations: unexpected, probes: probeChecks};
}

export function evaluateStrictThemeVerdict(result, gates, computedStyleContract) {
  const checks = [];
  checks.push({id: "http_status", status: result.http_status === 200 ? "pass" : Number.isInteger(result.http_status) ? "fail" : "missing", actual: result.http_status});
  checks.push({id: "navigation", status: result.navigation_error ? "fail" : "pass", reason: result.navigation_error ?? null});
  checks.push({id: "final_url", status: result.final_url === result.url ? "pass" : "fail", actual: result.final_url, expected: result.url});
  checks.push({id: "dom_artifact", status: result.dom_status === "captured" ? "pass" : "missing", actual: result.dom_status ?? null});
  checks.push({id: "screenshot_artifact", status: result.screenshot_status === "captured" ? "pass" : "missing", actual: result.screenshot_status ?? null});
  checks.push({id: "capture_errors", status: result.capture_errors?.length ? "fail" : "pass", count: result.capture_errors?.length ?? 0});
  for (const key of ["ttfb_ms", "fcp_ms", "lcp_ms", "cls"]) checks.push(metricGate(key, result.web_vitals?.[key], gates[key]));
  checks.push(computedStyleGate(result.computed_styles, computedStyleContract));
  const browserErrors = [...(result.errors?.console ?? []), ...(result.errors?.page ?? [])];
  const networkErrors = [...(result.errors?.requests ?? []), ...(result.errors?.responses ?? [])];
  checks.push({id: "browser_errors", status: browserErrors.length ? "fail" : "pass", count: browserErrors.length});
  checks.push({id: "network_errors", status: networkErrors.length ? "fail" : "pass", count: networkErrors.length});
  checks.push({id: "raw_syntax", status: result.raw_syntax?.length ? "fail" : "pass", count: result.raw_syntax?.length ?? 0});
  for (const interaction of result.interactions ?? []) {
    checks.push({id: `interaction:${interaction.id}:postcondition`, status: interaction.status === "measured" ? "pass" : interaction.status === "missing" ? "missing" : "fail", reason: interaction.reason ?? null});
    checks.push(metricGate(`interaction:${interaction.id}:visual_response_ms`, interaction.visual_response_ms, gates.visual_response_ms));
    checks.push(boundedMetricGate(`interaction:${interaction.id}:inp_equivalent_ms`, interaction.inp_equivalent, gates.inp_ms));
  }
  const blocking = checks.filter((check) => check.status !== "pass");
  return {status: blocking.length ? "fail" : "pass", checks, failed_gate_ids: blocking.map((check) => check.id), missing_gate_ids: blocking.filter((check) => check.status === "missing").map((check) => check.id)};
}

export async function collectThemePerformanceAttribution(page, webVitals, {maxLayoutShifts = 80, maxSourcesPerShift = 8, maxResources = 200} = {}) {
  const rawLayoutShifts = await collectLayoutShifts(page);
  const entries = (rawLayoutShifts.entries ?? []).slice(0, maxLayoutShifts).map((entry) => ({...entry, sources: (entry.sources ?? []).slice(0, maxSourcesPerShift)}));
  const layoutShifts = {...rawLayoutShifts, entries, truncated: rawLayoutShifts.truncated === true || (rawLayoutShifts.entries?.length ?? 0) > entries.length};
  const resourceTiming = await collectTimingDiagnostics(page, layoutShifts, {maxResources});
  return {schema: THEME_PERFORMANCE_ATTRIBUTION_SCHEMA, lcp_element: webVitals?.lcp_attribution ?? null, layout_shifts: layoutShifts, resource_timing: resourceTiming};
}

export async function captureThemeViewport({context, target, viewport, capture, artifactDir, source, timeoutMs = DEFAULT_TIMEOUT_MS, settleMs = DEFAULT_SETTLE_MS, interactionTimeoutMs = DEFAULT_INTERACTION_TIMEOUT_MS}) {
  artifactDir = await prepareThemeArtifactDirectory(artifactDir);
  const page = await context.newPage();
  const diagnosticErrors = [];
  await page.addInitScript(performanceObserverBootstrap);
  await installTimingObserver(page).catch((error) => diagnosticErrors.push(`timing observer: ${error.message}`));
  await installLayoutShiftObserver(page).catch((error) => diagnosticErrors.push(`layout-shift observer: ${error.message}`));
  const errors = startErrorCapture(page);
  let response = null;
  let navigationError = null;
  let settleStatus = "missing";
  let observationDeadlineMs = null;
  const captureErrors = [];
  try {
    response = await page.goto(target.url, {waitUntil: "domcontentloaded", timeout: timeoutMs});
    try {
      observationDeadlineMs = await waitForLcpObservationWindow(page, capture, settleMs);
      settleStatus = "lcp_observation_window";
    } catch (error) {
      settleStatus = "observation_error";
      captureErrors.push(`LCP observation window: ${error.message}`);
    }
  } catch (error) {
    navigationError = error.message;
  }
  let dom = "";
  let domStatus = "missing";
  try { dom = await page.content(); domStatus = "captured"; } catch (error) { captureErrors.push(`dom: ${error.message}`); }
  const computedStyles = await collectComputedStyles(page, capture.computed_styles).catch((error) => {
    captureErrors.push(`computed styles: ${error.message}`);
    return capture.computed_styles.probes.map((probe) => ({id: probe.id, selector: probe.selector, pseudo: probe.pseudo ?? null, expectation: probe.expectation, status: "missing", properties: null, rect: null}));
  });
  const metrics = await collectNavigationMetrics(page).catch((error) => {
    captureErrors.push(`navigation metrics: ${error.message}`);
    return {navigation_timing: null, web_vitals: {ttfb_ms: null, fcp_ms: null, lcp_ms: null, cls: null, support: {}}};
  });
  const performanceAttribution = await collectThemePerformanceAttribution(page, metrics.web_vitals).catch((error) => {
    diagnosticErrors.push(`performance attribution: ${error.message}`);
    return {schema: THEME_PERFORMANCE_ATTRIBUTION_SCHEMA, lcp_element: metrics.web_vitals?.lcp_attribution ?? null, layout_shifts: {supported: false, entries: [], cls: null, truncated: false}, resource_timing: {supported: false, resources: [], layout_shift_correlations: []}};
  });
  performanceAttribution.errors = diagnosticErrors;
  const screenshotPath = path.join(artifactDir, "screenshot.png");
  let screenshotStatus = "missing";
  try { await writePrivateThemeFile(screenshotPath, await page.screenshot({fullPage: true})); screenshotStatus = "captured"; } catch (error) { captureErrors.push(`screenshot: ${error.message}`); }
  const interactions = [];
  for (const interaction of capture.interactions) interactions.push(await captureInteraction(page, interaction, {timeoutMs: interactionTimeoutMs, visualResponseGateMs: capture.web_vitals.gates.visual_response_ms.value}));
  const finalUrl = page.url();
  const capturedErrors = structuredClone(errors);
  await page.close().catch(() => {});
  const result = {viewport, url: target.url, final_url: finalUrl, http_status: response?.status() ?? null, navigation_error: navigationError, settle_status: settleStatus, observation_deadline_ms: observationDeadlineMs, dom_status: domStatus, screenshot_status: screenshotStatus, capture_errors: captureErrors, diagnostic_errors: diagnosticErrors, dom, computed_styles: computedStyles, navigation_timing: metrics.navigation_timing, web_vitals: metrics.web_vitals, performance_attribution: performanceAttribution, errors: capturedErrors, raw_syntax: findRawSyntaxLeaks({html: dom, source}), interactions};
  result.verdict = evaluateStrictThemeVerdict(result, capture.web_vitals.gates, capture.computed_styles);
  result.artifacts = await writeThemeViewportArtifacts(artifactDir, result);
  const summary = {...result};
  delete summary.dom;
  return summary;
}

export async function captureThemeTierBrowserEvidence({tier, outputDir, source, chromium, browserExecutable, cdpEndpoint, browserSession = null, ignoreHttpsErrors = false, storageStates = {}, openBrowserImpl = openBrowser, captureViewportImpl = captureThemeViewport, startEgressProxyImpl = startCaptureEgressProxy}) {
  if (!tier?.capture || typeof source !== "string" || !source.trim()) throw new Error("tier capture contract and non-empty source are required");
  validateThemeComputedStyleContract(tier.capture.computed_styles, {label: `${tier.id} computed-style contract`});
  const tierId = safeId(tier.id, "tier id");
  const rootDirectory = await prepareThemeArtifactDirectory(outputDir);
  const tierDirectory = await prepareThemeArtifactDirectory(path.join(rootDirectory, tierId));
  const validatedTargets = tier.targets.map((target) => ({...target, url: validateThemeCaptureTarget({tier, target})}));
  const egressProxy = await startEgressProxyImpl({
    allowedLocalOrigins: [...new Set(validatedTargets.map((target) => new URL(target.url).origin))],
  });
  const ownsSession = browserSession === null;
  let session = browserSession;
  const targets = [];
  try {
    session ??= await openBrowserImpl({chromium, cdpEndpoint, browserExecutable, ignoreHttpsErrors, createInitialContexts: false});
    for (const target of validatedTargets) {
      safeId(target.id, "target id");
      const targetDirectory = await prepareThemeArtifactDirectory(path.join(tierDirectory, target.id));
      const targetResult = {id: target.id, url: target.url, viewports: []};
      for (const viewport of tier.capture.viewports) {
        safeId(viewport.id, "viewport id");
        const context = await session.browser.newContext({
          ...browserContextOptions({
            ignoreHttpsErrors,
            storageState: storageStates[target.id] ?? null,
            proxyServer: egressProxy.url,
          }),
          viewport: {width: viewport.width, height: viewport.height},
        });
        try {
          await installLocalFilePortRoute(context, target);
          const viewportDirectory = await prepareThemeArtifactDirectory(path.join(targetDirectory, viewport.id));
          targetResult.viewports.push(await captureViewportImpl({context, target, viewport, capture: tier.capture, artifactDir: viewportDirectory, source}));
        } finally {
          await context.close();
        }
      }
      targetResult.verdict = {status: targetResult.viewports.every((item) => item.verdict.status === "pass") ? "pass" : "fail", failed_viewports: targetResult.viewports.filter((item) => item.verdict.status !== "pass").map((item) => item.viewport.id)};
      targets.push(targetResult);
    }
  } finally {
    if (ownsSession && session) await session.close();
    await egressProxy.close();
  }
  const result = {schema: THEME_BROWSER_CAPTURE_SCHEMA, tier_id: tierId, status: targets.every((target) => target.verdict.status === "pass") ? "pass" : "fail", targets};
  const resultPath = path.join(tierDirectory, "browser-capture.json");
  await writePrivateThemeJson(resultPath, result);
  return {...result, result_path: resultPath};
}
