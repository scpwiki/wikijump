import assert from "node:assert/strict";
import test from "node:test";

import { canaryForUrl } from "../src/standing-browser-canaries.mjs";
import {
  DEFAULT_THRESHOLDS,
  compareCaptures,
  evaluateFirstPaintCustomProperties,
  validateLiveCompletionPolicy,
} from "../src/standing-browser-parity-contract.mjs";

function capture(overrides = {}) {
  return {
    input_url: "https://scp-wiki.wikijump.localhost:18443/scp-9506",
    final_url: "https://scp-wiki.wikijump.localhost:18443/scp-9506",
    navigation_status: 200,
    failures: [],
    geometry: {
      "#main-content": {
        count: 1,
        rect: { x: 100, y: 80, width: 900, height: 1200 },
      },
    },
    rendered_images: 2,
    broken_images: [],
    dom_signatures: ["div.block", "img.hero"],
    first_paint: {
      document: { geometry: {}, presence_probes: [], custom_properties: {} },
    },
    document: { presence_probes: [] },
    ...overrides,
  };
}

test("immediate theme properties fail before a settled state can conceal a flash", () => {
  const expectations = canaryForUrl(
    "https://scp-wiki.wikidot.com/scp-9506",
  ).first_paint_custom_properties;
  const passing = evaluateFirstPaintCustomProperties(
    {
      "--logo":
        "url(https://scp-wiki.wjfiles.localhost/local--files/scp-9506/NFSI.png)",
      "--header-logo":
        "url(https://scp-wiki.wjfiles.localhost/local--files/scp-9506/NFSI.png)",
      "--header-title": '"NFSI"',
      "--header-subtitle": '"National Fog Safety Initiative"',
    },
    expectations,
  );
  assert.equal(passing.status, "pass");
  const failing = evaluateFirstPaintCustomProperties(
    {
      "--logo": "",
      "--header-logo": "",
      "--header-title": '"NFSI"',
      "--header-subtitle": '"National Fog Safety Initiative"',
    },
    expectations,
  );
  assert.equal(failing.status, "fail");
});

test("DOMContentLoaded selector geometry is independently blocking", () => {
  const contract = {
    geometry_selectors: ["#main-content"],
    presence_probes: [],
    first_paint_custom_properties: {},
  };
  const local = capture({
    first_paint: {
      document: {
        geometry: {
          "#main-content": {
            count: 1,
            rect: { x: 120, y: 80, width: 900, height: 1200 },
          },
        },
        presence_probes: [],
        custom_properties: {},
      },
    },
  });
  const live = capture({
    input_url: "https://scp-wiki.wikidot.com/scp-9506",
    final_url: "https://scp-wiki.wikidot.com/scp-9506",
    first_paint: {
      document: {
        geometry: {
          "#main-content": {
            count: 1,
            rect: { x: 100, y: 80, width: 900, height: 1200 },
          },
        },
        presence_probes: [],
        custom_properties: {},
      },
    },
  });
  const result = compareCaptures(local, live, DEFAULT_THRESHOLDS, [], contract);
  assert.equal(result.status, "fail");
  assert.ok(
    result.anomalies.some(
      (anomaly) =>
        anomaly.code ===
        "domcontentloaded_immediate_selector_geometry_divergence",
    ),
  );
  assert.equal(result.geometry[0].status, "pass");
});

test("completion policy is sealed and names exact external failures", () => {
  const policy = validateLiveCompletionPolicy({
    schema: "wikijump.standing_browser_live_completion_policy.v1",
    status: "sealed",
    policy_version: "2026-07-20.1",
    allowed_external_failures: [
      {
        kind: "http_error",
        url: "https://cdn.example/advert.css",
        resource_type: "stylesheet",
        status: 404,
      },
    ],
  });
  assert.equal(policy.status, "sealed");
  assert.throws(
    () =>
      validateLiveCompletionPolicy({
        schema: "wikijump.standing_browser_live_completion_policy.v1",
        status: "sealed",
        policy_version: "2026-07-20.1",
        allowed_external_failures: [{ url: "https://cdn.example/advert.css" }],
      }),
    /kind/u,
  );
});
