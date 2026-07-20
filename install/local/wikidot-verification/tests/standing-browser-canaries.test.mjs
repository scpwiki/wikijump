import assert from "node:assert/strict";
import test from "node:test";

import {
  REQUIRED_THEME_FAMILIES,
  STANDING_BROWSER_CANARIES,
  assertThemeFamilyCoverage,
  canaryForUrl,
  defaultCanaryPairs,
  standingBrowserCanaryContractSha256,
} from "../src/standing-browser-canaries.mjs";

test("standing canaries cover each production theme family and the owner-observed SCP-9506 defects", () => {
  assert.deepEqual(
    assertThemeFamilyCoverage(),
    [...REQUIRED_THEME_FAMILIES].sort(),
  );
  const scp9506 = STANDING_BROWSER_CANARIES.find(
    (canary) => canary.slug === "scp-9506" && canary.theme_family === "basalt",
  );
  assert.ok(scp9506);
  const probes = Object.fromEntries(
    scp9506.presence_probes.map((probe) => [probe.id, probe]),
  );
  for (const id of [
    "header_logo",
    "header_title",
    "header_subtitle",
    "navigation_tab_bar",
    "navigation_tab_links",
  ]) {
    assert.ok(probes[id], `SCP-9506 must probe ${id}`);
  }
  assert.equal(probes.header_logo.selector, "#header h1 a");
  assert.equal(probes.header_subtitle.pseudo, "::after");
  assert.equal(probes.navigation_tab_bar.require_rendered, true);
  assert.equal(probes.navigation_tab_links.minimum_count, 6);
  assert.deepEqual(Object.keys(scp9506.first_paint_custom_properties).sort(), [
    "--header-logo",
    "--header-subtitle",
    "--header-title",
    "--logo",
  ]);
  const basaltTheme = STANDING_BROWSER_CANARIES.find(
    (canary) => canary.slug === "theme:basalt",
  );
  assert.ok(basaltTheme?.geometry_selectors.includes(".yui-navset"));
  assert.ok(
    basaltTheme?.presence_probes.some(
      (probe) => probe.selector === ".yui-navset" && probe.require_rendered,
    ),
  );
});

test("canary URLs bind local and live origins without hidden defaults", () => {
  const pairs = defaultCanaryPairs({
    localOrigin: "https://scp-wiki.wikijump.localhost:18443",
    liveOrigin: "https://scp-wiki.wikidot.com",
  });
  assert.equal(pairs.length, STANDING_BROWSER_CANARIES.length);
  assert.equal(
    pairs.find((pair) => pair.canary_slug === "theme:basalt").local_url,
    "https://scp-wiki.wikijump.localhost:18443/theme:basalt",
  );
  assert.equal(
    canaryForUrl("https://scp-wiki.wikidot.com/scp-9506").theme_family,
    "basalt",
  );
});

test("canary contract hash changes when a behavior-bearing probe changes", () => {
  const original = standingBrowserCanaryContractSha256();
  const changed = STANDING_BROWSER_CANARIES.map((canary) =>
    canary.slug === "theme:basalt"
      ? {
          ...canary,
          presence_probes: [
            ...canary.presence_probes,
            { id: "additional_tab", selector: ".yui-nav", minimum_count: 1 },
          ],
        }
      : canary,
  );
  assert.match(original, /^[0-9a-f]{64}$/u);
  assert.notEqual(original, standingBrowserCanaryContractSha256(changed));
});
