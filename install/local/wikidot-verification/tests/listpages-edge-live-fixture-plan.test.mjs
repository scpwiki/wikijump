import assert from "node:assert/strict";
import test from "node:test";

import {
  LISTPAGES_LIVE_FIXTURE_PLAN_SCHEMA,
  buildListPagesEdgeLiveFixturePlan,
} from "../src/listpages-live-fixture-plan.mjs";

test("plans controlled live oracles for legacy and body-edge behavior", () => {
  const plan = buildListPagesEdgeLiveFixturePlan();
  assert.equal(plan.schema, LISTPAGES_LIVE_FIXTURE_PLAN_SCHEMA);
  assert.equal(plan.run_id, "listpages-edge-20260728");
  assert.equal(plan.pages.length, 6);
  assert.equal(plan.captures.length, 2);

  const pages = new Map(plan.pages.map((page) => [page.key, page]));
  assert.match(
    pages.get("edge-default-holder")?.sources.at(-1) ?? "",
    /^\[\[module ListPages [^\n]+\]\]$/u,
  );
  assert.doesNotMatch(
    pages.get("edge-default-holder")?.sources.at(-1) ?? "",
    /\[\[\/module\]\]/u,
  );

  const edgeSource = pages.get("edge-holder")?.sources.at(-1) ?? "";
  for (const expected of [
    "lp-edge-code-body",
    "lp-edge-html-body",
    "lp-edge-summary-section",
    "lp-edge-skip-yes",
    "lp-edge-skip-true",
    "lp-edge-skip-no",
    "lp-edge-skip-false",
    "lp-edge-skip-invalid",
    "lp-edge-reverse-yes",
    "lp-edge-reverse-true",
    "lp-edge-reverse-no",
    "lp-edge-reverse-false",
    "lp-edge-reverse-invalid",
    "lp-edge-reverse-empty",
    "lp-edge-tags-same-implicit-skip",
    "lp-edge-tag-target",
    "lp-edge-link-to-current",
  ]) {
    assert.match(edgeSource, new RegExp(expected, "u"));
  }
});
