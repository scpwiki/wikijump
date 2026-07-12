import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  THEME_BROWSER_CAPTURE_SCHEMA,
  captureInteraction,
  captureThemeTierBrowserEvidence,
  evaluateStrictThemeVerdict,
  lcpObservationDeadlineMs,
  validateThemeCaptureTarget,
  writeThemeViewportArtifacts,
} from "../src/theme-browser-capture.mjs";
import {THEME_PERFORMANCE_GATES} from "../src/theme-localization-e2e.mjs";

function fixtureTier() {
  const slug = "theme:codex-l10n-capture-run-yossistyle";
  return {
    url: "https://scpaiueouiuiuiui.wikijump.localhost:18443/theme:codex-l10n-test-yossistyle",
    final_url: "https://scpaiueouiuiuiui.wikijump.localhost:18443/theme:codex-l10n-test-yossistyle",
    id: "yossistyle",
    run_owned_slug: slug,
    targets: [
      {id: "wikidot", url: `http://scpaiueouiuiuiui.wikidot.com/${slug}`},
      {id: "wikijump", url: `https://scpaiueouiuiuiui.wikijump.localhost:18443/${slug}`},
    ],
    capture: {
      viewports: [{id: "desktop", width: 1440, height: 1000}, {id: "mobile", width: 390, height: 844}],
      computed_styles: {properties: ["display"], probes: [{id: "header", selector: "#header"}]},
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
    computed_styles: [{id: "header", status: "measured"}],
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

test("strict verdict applies every performance, browser, syntax, and interaction gate", () => {
  const verdict = evaluateStrictThemeVerdict(passingViewport(), THEME_PERFORMANCE_GATES);
  assert.equal(verdict.status, "pass");
  assert.deepEqual(verdict.failed_gate_ids, []);
  assert.ok(verdict.checks.some((check) => check.id === "interaction:toggle:inp_equivalent_ms" && check.status === "pass"));

  const failed = evaluateStrictThemeVerdict(passingViewport({
    web_vitals: {ttfb_ms: 801, fcp_ms: null, lcp_ms: 2501, cls: 0.11},
    computed_styles: [{id: "header", status: "missing"}],
    errors: {console: ["boom"], page: [], requests: [{url: "https://bad.invalid"}], responses: []},
    raw_syntax: [{category: "leaked-marker"}],
  }), THEME_PERFORMANCE_GATES);
  assert.equal(failed.status, "fail");
  assert.ok(failed.failed_gate_ids.includes("ttfb_ms"));
  assert.ok(failed.failed_gate_ids.includes("browser_errors"));
  assert.ok(failed.failed_gate_ids.includes("raw_syntax"));
  assert.ok(failed.missing_gate_ids.includes("fcp_ms"));
  assert.ok(failed.missing_gate_ids.includes("computed_style_probes"));

  const redirected = evaluateStrictThemeVerdict(passingViewport({final_url: "https://scpaiueouiuiuiui.wikijump.localhost:18443/login", settle_status: "timeout"}), THEME_PERFORMANCE_GATES);
  assert.equal(redirected.status, "fail");
  assert.ok(redirected.failed_gate_ids.includes("final_url"));
  assert.ok(!redirected.failed_gate_ids.includes("network_idle_settle"));
});

test("capture observation window follows the LCP gate without waiting for network idle", () => {
  assert.equal(lcpObservationDeadlineMs({web_vitals: {gates: THEME_PERFORMANCE_GATES}}, 250), 2750);
  assert.throws(() => lcpObservationDeadlineMs({web_vitals: {gates: {lcp_ms: {operator: "gte", value: 2500}}}}, 250), /positive LCP upper bound/u);
});

test("a single interaction without PerformanceEventTiming fails closed and is not called formal INP", () => {
  const result = passingViewport({interactions: [{id: "toggle", status: "missing", visual_response_ms: 35, inp_equivalent: {formal_inp: false, status: "missing", duration_ms: null}, reason: "PerformanceEventTiming interaction entry missing"}]});
  const verdict = evaluateStrictThemeVerdict(result, THEME_PERFORMANCE_GATES);
  assert.equal(verdict.status, "fail");
  assert.ok(verdict.missing_gate_ids.includes("interaction:toggle:inp_equivalent_ms"));
  assert.equal(result.interactions[0].inp_equivalent.formal_inp, false);
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
  await fs.writeFile(path.join(directory, "screenshot.png"), "png fixture");
  const result = {...passingViewport(), dom: "<html><body>fixture</body></html>", navigation_timing: {response_start: 100}, verdict: {status: "pass"}};
  const artifacts = await writeThemeViewportArtifacts(directory, result);
  assert.equal(await fs.readFile(artifacts.dom, "utf8"), result.dom);
  assert.equal((await fs.stat(artifacts.screenshot)).isFile(), true);
  assert.equal(JSON.parse(await fs.readFile(artifacts.verdict, "utf8")).status, "pass");
  assert.deepEqual(Object.keys(artifacts).sort(), ["computed_styles", "dom", "interactions", "network_errors", "raw_syntax", "screenshot", "verdict", "web_vitals"]);
});

test("tier orchestration opens one cold context per target and viewport and is fully mockable", async () => {
  const tier = fixtureTier();
  const outputDir = await fs.mkdtemp(path.join(os.tmpdir(), "theme-browser-capture-"));
  const contextOptions = [];
  const closedContexts = [];
  let closedSession = false;
  const openCalls = [];
  const result = await captureThemeTierBrowserEvidence({
    tier,
    outputDir,
    source: "日本語のテーマ本文",
    chromium: {fixture: true},
    storageStates: {wikidot: "/tmp/wikidot-state.json"},
    async openBrowserImpl(options) {
      openCalls.push(options);
      return {
        browser: {
          async newContext(contextOption) {
            const id = contextOptions.length;
            contextOptions.push(contextOption);
            return {id, async close() { closedContexts.push(id); }};
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
  assert.equal(contextOptions.length, 4);
  assert.deepEqual(contextOptions.map((options) => options.viewport), [{width: 1440, height: 1000}, {width: 390, height: 844}, {width: 1440, height: 1000}, {width: 390, height: 844}]);
  assert.equal(contextOptions[0].storageState, "/tmp/wikidot-state.json");
  assert.equal("storageState" in contextOptions[2], false);
  assert.deepEqual(closedContexts, [0, 1, 2, 3]);
  assert.equal(closedSession, true);
  assert.equal(result.schema, THEME_BROWSER_CAPTURE_SCHEMA);
  assert.equal(result.status, "fail");
  assert.deepEqual(result.targets[1].verdict.failed_viewports, ["mobile"]);
  assert.deepEqual(JSON.parse(await fs.readFile(result.result_path, "utf8")).targets[0].verdict, {status: "pass", failed_viewports: []});
});
