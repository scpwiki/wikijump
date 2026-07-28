import assert from "node:assert/strict";
import test from "node:test";

import {
  LISTPAGES_LIVE_FIXTURE_PLAN_SCHEMA,
  buildListPagesLiveFixturePlan,
} from "../src/listpages-live-fixture-plan.mjs";

test("builds a deterministic controlled ListPages live fixture graph", () => {
  const plan = buildListPagesLiveFixturePlan();
  assert.equal(plan.schema, LISTPAGES_LIVE_FIXTURE_PLAN_SCHEMA);
  assert.equal(plan.site, "sandbox-for-codex");
  assert.equal(plan.pages.length, 53);
  assert.equal(plan.captures.length, 30);
  assert.equal(new Set(plan.pages.map((page) => page.key)).size, plan.pages.length);
  assert.equal(
    new Set(plan.pages.map((page) => page.fullname)).size,
    plan.pages.length,
  );
  assert.equal(
    new Set(plan.captures.map((capture) => capture.case_id)).size,
    plan.captures.length,
  );
  assert.ok(plan.pages.every((page) =>
    /^run-owned:lp-campaign-20260727-[a-z0-9][a-z0-9-]*$/u.test(page.fullname)));
  assert.equal(
    plan.pages.filter((page) => page.tags.includes("lp-pagination-20260727")).length,
    23,
  );
  assert.equal(
    plan.pages.flatMap((page) => page.votes ?? []).length,
    0,
  );
  assert.deepEqual(plan.live_environment_blockers, [
    {
      capability: "nonzero rating and vote mutation",
      observed_at: "2026-07-27",
      site: "sandbox-for-codex",
      evidence:
        "RateAction returned status not_ok for both run-owned: and _default run-owned pages when account B attempted to vote.",
      impact:
        "Saved-page fixtures still cover zero/current rating and votes selectors, but cannot derive controlled nonzero rating or rating_votes oracle data from this sandbox run.",
    },
  ]);
  assert.ok(plan.captures.some((capture) =>
    capture.url_suffix === "/tag/lp-pagination-20260727/offset/2/p/2?probe=1"));
  assert.deepEqual(buildListPagesLiveFixturePlan(), plan);
});

test("plans exact live captures for every unresolved path-ordering case", () => {
  const plan = buildListPagesLiveFixturePlan();
  const captures = new Map(
    plan.captures.map((capture) => [capture.case_id, capture]),
  );

  assert.equal(
    captures.get("lp-live-navigation-p-before-tag")?.url_suffix,
    "/p/2/tag/lp-pagination-20260727",
  );
  assert.equal(
    captures.get("lp-live-navigation-category-before-p")?.url_suffix,
    "/category/fragment/p/2",
  );
  assert.equal(
    captures.get("lp-live-navigation-prefixed-limits")?.url_suffix,
    "/page2_limit/1/page3_limit/2",
  );

  const holder = plan.pages.find((page) => page.key === "prefixed-url-holder");
  assert.ok(holder, "the prefixed URL fixture holder must be run-owned");
  assert.match(holder.sources.at(-1), /urlAttrPrefix="page2"/u);
  assert.match(holder.sources.at(-1), /urlAttrPrefix="page3"/u);
  assert.match(holder.sources.at(-1), /limit="@URL\|0"/u);
});

test("plans a saved-page oracle for every documented ListPages template variable family", () => {
  const plan = buildListPagesLiveFixturePlan();
  const pages = new Map(plan.pages.map((page) => [page.key, page]));
  const capture = plan.captures.find(
    (candidate) => candidate.case_id === "lp-live-template-variables",
  );

  assert.equal(capture?.page, "variables-holder");
  assert.deepEqual(capture?.dimensions, [
    "template-variables",
    "aliases",
    "parent",
    "site",
    "content",
  ]);
  assert.match(
    pages.get("variables-holder")?.sources.at(-1) ?? "",
    /created_by_id=%%created_by_id%%/,
  );
  assert.match(
    pages.get("variables-holder")?.sources.at(-1) ?? "",
    /total_or_limit=%%total_or_limit%%/,
  );
  assert.equal(pages.get("variables-target")?.parent, "variables-parent");
});
