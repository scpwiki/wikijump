import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  THEME_BROWSER_CAPTURE_SCHEMA,
  THEME_PERFORMANCE_ATTRIBUTION_SCHEMA,
  captureInteraction,
  captureThemeTierBrowserEvidence,
  collectComputedStyles,
  collectThemePerformanceAttribution,
  evaluateStrictThemeVerdict,
  installLocalFilePortRoute,
  lcpObservationDeadlineMs,
  validateThemeCaptureTarget,
  writeThemeViewportArtifacts,
} from "../src/theme-browser-capture.mjs";
import {THEME_LOCALIZATION_TIERS, THEME_PERFORMANCE_GATES} from "../src/theme-localization-e2e.mjs";

const COMPUTED_STYLE_CONTRACT = {
  properties: ["display"],
  probes: [
    {id: "header", selector: "#header", expectation: "required"},
    {id: "interwiki_frame", selector: "iframe.scpnet-interwiki-frame", expectation: "optional"},
    {id: "watchers_button", selector: "#watchers-button", expectation: "expected_absent"},
  ],
};

function fixtureTier() {
  const slug = "codex-l10n:capture-run-yossistyle";
  return {
    url: "https://scpaiueouiuiuiui.wikijump.localhost:18443/codex-l10n:test-yossistyle",
    final_url: "https://scpaiueouiuiuiui.wikijump.localhost:18443/codex-l10n:test-yossistyle",
    id: "yossistyle",
    run_owned_slug: slug,
    targets: [
      {id: "wikidot", url: `http://scpaiueouiuiuiui.wikidot.com/${slug}`},
      {id: "wikijump", url: `https://scpaiueouiuiuiui.wikijump.localhost:18443/${slug}`},
    ],
    capture: {
      viewports: [{id: "desktop", width: 1440, height: 1000}, {id: "mobile", width: 390, height: 844}],
      computed_styles: COMPUTED_STYLE_CONTRACT,
      web_vitals: {gates: THEME_PERFORMANCE_GATES},
      interactions: [],
    },
  };
}

function passingViewport(overrides = {}) {
  return {
    http_status: 200,
    navigation_error: null,
    dom_status: "captured",
    screenshot_status: "captured",
    capture_errors: [],
    settle_status: "networkidle",
    web_vitals: {ttfb_ms: 200, fcp_ms: 600, lcp_ms: 900, cls: 0.01},
    computed_styles: [
      {id: "header", expectation: "required", status: "measured"},
      {id: "interwiki_frame", expectation: "optional", status: "missing"},
      {id: "watchers_button", expectation: "expected_absent", status: "missing"},
    ],
    errors: {console: [], page: [], requests: [], responses: []},
    raw_syntax: [],
    interactions: [{id: "toggle", status: "measured", visual_response_ms: 40, inp_equivalent: {formal_inp: false, status: "measured", duration_ms: 80}}],
    ...overrides,
  };
}

test("capture target validation only accepts the exact run-owned sandbox URL", () => {
  const tier = fixtureTier();
  assert.equal(validateThemeCaptureTarget({tier, target: tier.targets[0]}), tier.targets[0].url);
  assert.throws(() => validateThemeCaptureTarget({tier, target: {id: "wikijump", url: `https://scp-wiki.wikijump.localhost/${tier.run_owned_slug}`}}), /hard allowlist/);
  assert.throws(() => validateThemeCaptureTarget({tier, target: {...tier.targets[0], url: `${tier.targets[0].url}?capture=1`}}), /does not identify/);
  assert.throws(() => validateThemeCaptureTarget({tier, target: {...tier.targets[0], url: "http://scpaiueouiuiuiui.wikidot.com/theme:other"}}), /does not identify/);
});

test("local file routing preserves the candidate canary port without widening hosts", async () => {
  let pattern;
  let handler;
  const context = {async route(value, callback) { pattern = value; handler = callback; }};
  const target = {id: "wikijump", url: "https://scpaiueouiuiuiui.wikijump.localhost:18443/codex-l10n:test-yossistyle"};
  assert.equal(await installLocalFilePortRoute(context, target), true);
  assert.equal(pattern, "https://*.wjfiles.localhost/**");

  let fetchOptions;
  let fulfillment;
  const response = {status: 200};
  await handler({request() { return {url() { return "https://scp-wiki.wjfiles.localhost/local--files/theme:ashes-to-ashes/fire.webp"; }}; }, async fetch(options) { fetchOptions = options; return response; }, async fulfill(options) { fulfillment = options; }});
  assert.equal(fetchOptions.url, "https://scp-wiki.wjfiles.localhost:18443/local--files/theme:ashes-to-ashes/fire.webp");
  assert.deepEqual(fulfillment, {response});

  let continuation = "unset";
  await handler({request() { return {url() { return "https://example.com/local--files/theme/fire.webp"; }}; }, async continue(options) { continuation = options; }});
  assert.equal(continuation, undefined);
  assert.equal(await installLocalFilePortRoute(context, {id: "wikidot", url: "http://scpaiueouiuiuiui.wikidot.com/page"}), false);
});

test("strict verdict applies every performance, browser, syntax, and interaction gate", () => {
  const verdict = evaluateStrictThemeVerdict(passingViewport(), THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT);
  assert.equal(verdict.status, "pass");
  assert.deepEqual(verdict.failed_gate_ids, []);
  assert.ok(verdict.checks.some((check) => check.id === "interaction:toggle:inp_equivalent_ms" && check.status === "pass"));
  const styleCheck = verdict.checks.find((check) => check.id === "computed_style_probes");
  assert.deepEqual(styleCheck.optional_missing, ["interwiki_frame"]);
  assert.equal(styleCheck.probes.find((probe) => probe.id === "interwiki_frame").status, "pass");

  const failed = evaluateStrictThemeVerdict(passingViewport({
    web_vitals: {ttfb_ms: 801, fcp_ms: null, lcp_ms: 2501, cls: 0.11},
    computed_styles: [
      {id: "header", expectation: "required", status: "missing"},
      {id: "interwiki_frame", expectation: "optional", status: "missing"},
      {id: "watchers_button", expectation: "expected_absent", status: "missing"},
    ],
    errors: {console: ["boom"], page: [], requests: [{url: "https://bad.invalid"}], responses: []},
    raw_syntax: [{category: "leaked-marker"}],
  }), THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT);
  assert.equal(failed.status, "fail");
  assert.ok(failed.failed_gate_ids.includes("ttfb_ms"));
  assert.ok(failed.failed_gate_ids.includes("browser_errors"));
  assert.ok(failed.failed_gate_ids.includes("raw_syntax"));
  assert.ok(failed.missing_gate_ids.includes("fcp_ms"));
  assert.ok(failed.missing_gate_ids.includes("computed_style_probes"));

  const redirected = evaluateStrictThemeVerdict(passingViewport({final_url: "https://scpaiueouiuiuiui.wikijump.localhost:18443/login", settle_status: "timeout"}), THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT);
  assert.equal(redirected.status, "fail");
  assert.ok(redirected.failed_gate_ids.includes("final_url"));
  assert.ok(!redirected.failed_gate_ids.includes("network_idle_settle"));
});

test("expected-absent probes gate unexpected presence and incomplete observations fail closed", () => {
  const present = passingViewport({computed_styles: [
    {id: "header", expectation: "required", status: "measured"},
    {id: "interwiki_frame", expectation: "optional", status: "missing"},
    {id: "watchers_button", expectation: "expected_absent", status: "measured"},
  ]});
  const presentVerdict = evaluateStrictThemeVerdict(present, THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT);
  const presentCheck = presentVerdict.checks.find((check) => check.id === "computed_style_probes");
  assert.equal(presentVerdict.status, "fail");
  assert.deepEqual(presentCheck.expected_absent_present, ["watchers_button"]);

  const incomplete = passingViewport({computed_styles: [{id: "header", expectation: "required", status: "measured"}]});
  const incompleteCheck = evaluateStrictThemeVerdict(incomplete, THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT).checks.find((check) => check.id === "computed_style_probes");
  assert.equal(incompleteCheck.status, "fail");
  assert.deepEqual(incompleteCheck.invalid_observations.sort(), ["interwiki_frame", "watchers_button"]);
});

test("YOSSISTYLE gates sourced Rate elements while allowing platform-dependent probe gaps", () => {
  const contract = {properties: ["display"], probes: THEME_LOCALIZATION_TIERS.find((tier) => tier.id === "yossistyle").computed_style_probes};
  const observations = (missing) => contract.probes.map((probe) => ({id: probe.id, expectation: probe.expectation, status: missing.has(probe.id) ? "missing" : "measured", properties: Object.fromEntries(Object.entries(probe.expected_properties ?? {}).map(([property, specification]) => [property, specification.value ?? specification.values[0]]))}));
  const wikidot = evaluateStrictThemeVerdict(passingViewport({computed_styles: observations(new Set(["rate_widget", "interwiki_frame", "rate_points"]))}), THEME_PERFORMANCE_GATES, contract);
  const wikijump = evaluateStrictThemeVerdict(passingViewport({computed_styles: observations(new Set(["interwiki_frame", "watchers_button"]))}), THEME_PERFORMANCE_GATES, contract);
  assert.equal(wikidot.status, "fail");
  assert.deepEqual(wikidot.checks.find((check) => check.id === "computed_style_probes").required_missing, ["rate_widget", "rate_points"]);
  assert.equal(wikijump.status, "pass");
  assert.deepEqual(wikijump.checks.find((check) => check.id === "computed_style_probes").optional_missing, ["interwiki_frame", "watchers_button"]);
});

test("computed style expectations fail closed with exact mismatch diagnostics", () => {
  const contract = {properties: ["display"], probes: [{id: "header", selector: "#header", expectation: "required", expected_properties: {display: {operator: "eq", value: "block"}, "font-weight": {operator: "one_of", values: ["700", "bold"]}}}]};
  const result = passingViewport({computed_styles: [{id: "header", expectation: "required", status: "measured", properties: {display: "block", "font-weight": "400"}}]});
  const check = evaluateStrictThemeVerdict(result, THEME_PERFORMANCE_GATES, contract).checks.find((item) => item.id === "computed_style_probes");
  assert.equal(check.status, "fail");
  assert.deepEqual(check.property_mismatches, [{probe_id: "header", property: "font-weight", actual: "400", expected: {operator: "one_of", values: ["700", "bold"]}}]);
  assert.equal(check.probes[0].property_checks[0].status, "pass");
  assert.equal(check.probes[0].property_checks[1].status, "fail");
});

test("computed style collection captures the union of common and expected properties", async () => {
  let argument;
  const page = {async evaluate(_fn, value) { argument = value; return []; }};
  await collectComputedStyles(page, {properties: ["display"], probes: [{id: "header", selector: "#header", expectation: "required", expected_properties: {"margin-left": {operator: "eq", value: "1px"}}}]});
  assert.deepEqual(argument.properties, ["display", "margin-left"]);
});

test("capture observation window follows the LCP gate without waiting for network idle", () => {
  assert.equal(lcpObservationDeadlineMs({web_vitals: {gates: THEME_PERFORMANCE_GATES}}, 250), 2750);
  assert.throws(() => lcpObservationDeadlineMs({web_vitals: {gates: {lcp_ms: {operator: "gte", value: 2500}}}}, 250), /positive LCP upper bound/u);
});

test("supported EventTiming absence below its threshold is bounded and passes", () => {
  const result = passingViewport({interactions: [{id: "toggle", status: "measured", visual_response_ms: 35, inp_equivalent: {formal_inp: false, status: "bounded_below_threshold", duration_ms: null, upper_bound_ms: 16}}]});
  const verdict = evaluateStrictThemeVerdict(result, THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT);
  assert.equal(verdict.status, "pass");
  assert.deepEqual(verdict.checks.find((check) => check.id === "interaction:toggle:inp_equivalent_ms").actual, {operator: "lt", value: 16});
  assert.equal(result.interactions[0].inp_equivalent.formal_inp, false);
});

test("unsupported EventTiming remains missing without invalidating a successful postcondition", () => {
  const result = passingViewport({interactions: [{id: "toggle", status: "measured", visual_response_ms: 35, inp_equivalent: {formal_inp: false, status: "missing", duration_ms: null}, reason: "PerformanceEventTiming unsupported"}]});
  const verdict = evaluateStrictThemeVerdict(result, THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT);
  assert.equal(verdict.status, "fail");
  assert.equal(verdict.checks.find((check) => check.id === "interaction:toggle:postcondition").status, "pass");
  assert.ok(verdict.missing_gate_ids.includes("interaction:toggle:inp_equivalent_ms"));
});

test("missing interaction selectors return explicit missing evidence without clicking", async () => {
  const page = {locator() { return {async count() { return 0; }}; }};
  const result = await captureInteraction(page, {id: "tab", target_selectors: [".missing"], postcondition: "selected_tab_changes"});
  assert.equal(result.status, "missing");
  assert.equal(result.inp_equivalent.status, "missing");
  assert.equal(result.inp_equivalent.formal_inp, false);
});

test("viewport artifact writer emits the complete fixed artifact set", async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), "theme-browser-artifacts-"));
  await fs.writeFile(path.join(directory, "screenshot.png"), "png fixture", {mode: 0o600});
  const result = {...passingViewport(), dom: "<html><body>fixture</body></html>", navigation_timing: {response_start: 100}, performance_attribution: {schema: THEME_PERFORMANCE_ATTRIBUTION_SCHEMA}, verdict: {status: "pass"}};
  const artifacts = await writeThemeViewportArtifacts(directory, result);
  assert.equal(await fs.readFile(artifacts.dom, "utf8"), result.dom);
  assert.equal((await fs.stat(artifacts.screenshot)).isFile(), true);
  assert.equal(JSON.parse(await fs.readFile(artifacts.verdict, "utf8")).status, "pass");
  assert.deepEqual(Object.keys(artifacts).sort(), ["computed_styles", "dom", "interactions", "network_errors", "performance_attribution", "raw_syntax", "screenshot", "verdict", "web_vitals"]);
  for (const filePath of Object.values(artifacts)) assert.equal((await fs.stat(filePath)).mode & 0o077, 0);
  await assert.rejects(writeThemeViewportArtifacts(directory, result), /EEXIST/);
});

test("performance attribution bounds resources and layout-shift sources without adding gates", async () => {
  let call = 0;
  const page = {async evaluate() {
    call += 1;
    if (call === 1) return {supported: true, cls: 0.2, entries: [{value: 0.1, startTime: 100, sources: [{selector_hint: "div#one"}, {selector_hint: "div#two"}]}, {value: 0.1, startTime: 200, sources: []}]};
    return {supported: true, navigation: null, marks: [], resources: [{name: "/one.css", initiatorType: "link", responseEnd: 80}, {name: "/two.js", initiatorType: "script", responseEnd: 90}]};
  }};
  const result = await collectThemePerformanceAttribution(page, {lcp_attribution: {selector_hint: "main#page-content"}}, {maxLayoutShifts: 1, maxSourcesPerShift: 1, maxResources: 1});
  assert.equal(result.schema, THEME_PERFORMANCE_ATTRIBUTION_SCHEMA);
  assert.equal(result.layout_shifts.truncated, true);
  assert.deepEqual(result.layout_shifts.entries[0].sources, [{selector_hint: "div#one"}]);
  assert.equal(result.resource_timing.resources.length, 1);
  assert.equal(result.lcp_element.selector_hint, "main#page-content");
  const verdict = evaluateStrictThemeVerdict(passingViewport({diagnostic_errors: ["resource timing unsupported"]}), THEME_PERFORMANCE_GATES, COMPUTED_STYLE_CONTRACT);
  assert.equal(verdict.status, "pass");
  assert.ok(!verdict.failed_gate_ids.includes("performance_attribution"));
});

test("tier orchestration opens one cold context per target and viewport and is fully mockable", async () => {
  const tier = fixtureTier();
  const outputDir = await fs.mkdtemp(path.join(os.tmpdir(), "theme-browser-capture-"));
  const contextOptions = [];
  const closedContexts = [];
  let closedSession = false;
  const openCalls = [];
  const proxyCalls = [];
  const result = await captureThemeTierBrowserEvidence({
    tier,
    outputDir,
    source: "日本語のテーマ本文",
    chromium: {fixture: true},
    storageStates: {wikidot: "/tmp/wikidot-state.json"},
    async startEgressProxyImpl(options) {
      proxyCalls.push(options);
      return {url: "http://127.0.0.1:27777", async close() {}};
    },
    async openBrowserImpl(options) {
      openCalls.push(options);
      return {
        browser: {
          async newContext(contextOption) {
            const id = contextOptions.length;
            contextOptions.push(contextOption);
            return {id, async route() {}, async close() { closedContexts.push(id); }};
          },
        },
        async close() { closedSession = true; },
      };
    },
    async captureViewportImpl({target, viewport, artifactDir}) {
      assert.ok(artifactDir.endsWith(path.join(tier.id, target.id, viewport.id)));
      return {viewport, verdict: {status: target.id === "wikidot" || viewport.id === "desktop" ? "pass" : "fail"}};
    },
  });

  assert.equal(openCalls.length, 1);
  assert.equal(openCalls[0].createInitialContexts, false);
  assert.deepEqual(proxyCalls, [{allowedLocalOrigins: ["http://scpaiueouiuiuiui.wikidot.com", "https://scpaiueouiuiuiui.wikijump.localhost:18443"]}]);
  assert.equal(contextOptions.length, 4);
  assert.deepEqual(contextOptions.map((options) => options.viewport), [{width: 1440, height: 1000}, {width: 390, height: 844}, {width: 1440, height: 1000}, {width: 390, height: 844}]);
  assert.deepEqual(contextOptions.map((options) => options.proxy), Array.from({length: 4}, () => ({server: "http://127.0.0.1:27777", bypass: "<-loopback>"})));
  assert.equal(contextOptions[0].storageState, "/tmp/wikidot-state.json");
  assert.equal("storageState" in contextOptions[2], false);
  assert.deepEqual(closedContexts, [0, 1, 2, 3]);
  assert.equal(closedSession, true);
  assert.equal(result.schema, THEME_BROWSER_CAPTURE_SCHEMA);
  assert.equal(result.status, "fail");
  assert.deepEqual(result.targets[1].verdict.failed_viewports, ["mobile"]);
  assert.deepEqual(JSON.parse(await fs.readFile(result.result_path, "utf8")).targets[0].verdict, {status: "pass", failed_viewports: []});
  assert.equal((await fs.stat(result.result_path)).mode & 0o077, 0);
  for (const target of tier.targets) for (const viewport of tier.capture.viewports) assert.equal((await fs.stat(path.join(outputDir, tier.id, target.id, viewport.id))).mode & 0o077, 0);
});

test("a supplied run-owned browser session is not reopened or closed by tier capture", async () => {
  const tier = fixtureTier();
  const outputDir = await fs.mkdtemp(path.join(os.tmpdir(), "theme-shared-browser-"));
  let contexts = 0;
  let closes = 0;
  const browserSession = {
    browser: {async newContext() { contexts += 1; return {async route() {}, async close() {}}; }},
    async close() { closes += 1; },
  };
  await captureThemeTierBrowserEvidence({
    tier, outputDir, source: "日本語のテーマ本文", browserSession,
    async openBrowserImpl() { throw new Error("must not reopen shared browser"); },
    async captureViewportImpl({viewport}) { return {viewport, verdict: {status: "pass"}}; },
  });
  assert.equal(contexts, tier.targets.length * tier.capture.viewports.length);
  assert.equal(closes, 0);
});
