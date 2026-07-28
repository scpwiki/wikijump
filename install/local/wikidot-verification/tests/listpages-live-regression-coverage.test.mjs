import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  LISTPAGES_LIVE_REGRESSION_COVERAGE_SCHEMA,
  buildListPagesLiveRegressionCoverage,
  liveFixtureBlockClasses,
} from "../src/listpages-live-regression-coverage.mjs";
import { buildListPagesLiveFixturePlan } from "../src/listpages-live-fixture-plan.mjs";
import { main as buildCoverageCli } from "../scripts/build-listpages-live-regression-coverage.mjs";

test("accounts for every controlled live ListPages fixture block class", () => {
  const plan = buildListPagesLiveFixturePlan();
  const blockClasses = liveFixtureBlockClasses(plan);
  const coverage = buildListPagesLiveRegressionCoverage(plan);

  assert.equal(coverage.schema, LISTPAGES_LIVE_REGRESSION_COVERAGE_SCHEMA);
  assert.equal(blockClasses.length, 118);
  assert.equal(coverage.summary.block_classes, blockClasses.length);
  assert.equal(coverage.summary.covered, blockClasses.length);
  assert.equal(coverage.summary.missing_local_regression, 0);
  assert.deepEqual(coverage.missing_local_regression_block_classes, []);
  assert.deepEqual(
    coverage.coverage.map((entry) => entry.block_class),
    blockClasses,
  );
  assert.ok(
    coverage.coverage.every(
      (entry) =>
        entry.local_regressions.length > 0 &&
        entry.live_capture_case_ids.length > 0 &&
        entry.fixture_pages.length > 0,
    ),
  );
});

test("ties newly discovered saved-fixture classes to concrete Rust regressions", () => {
  const byClass = new Map(
    buildListPagesLiveRegressionCoverage().coverage.map((entry) => [
      entry.block_class,
      entry,
    ]),
  );

  assert.equal(
    byClass.get("lp-offset-huge").local_regressions[0].test,
    "list_pages_saved_view_preserves_live_pagination_path_shapes",
  );
  assert.equal(
    byClass.get("lp-range-before").local_regressions[0].test,
    "listpages_range_selectors_match_the_saved_page_live_fixture",
  );
  assert.equal(
    byClass.get("lp-tags-exact").local_regressions[0].test,
    "listpages_current_tag_selectors_match_the_saved_page_live_fixture",
  );
  assert.equal(
    byClass.get("lp-votes-current").local_regressions[0].test,
    "listpages_current_metric_selectors_match_the_saved_page_live_fixture",
  );
  assert.deepEqual(
    byClass.get("lp-votes-positive").live_environment_blocker_refs,
    ["nonzero rating and vote mutation"],
  );
});

test("writes reusable live regression coverage artifacts without replacement", async () => {
  const tempDir = await fs.mkdtemp(
    path.join(os.tmpdir(), "listpages-live-regression-coverage-"),
  );
  try {
    const output = path.join(tempDir, "coverage.json");
    assert.equal(
      await buildCoverageCli(["node", "script", "--output", output]),
      0,
    );
    const coverage = JSON.parse(await fs.readFile(output, "utf8"));
    assert.equal(coverage.schema, LISTPAGES_LIVE_REGRESSION_COVERAGE_SCHEMA);
    assert.equal(coverage.summary.missing_local_regression, 0);
    await assert.rejects(
      () => buildCoverageCli(["node", "script", "--output", output]),
      /EEXIST/u,
    );
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
});
