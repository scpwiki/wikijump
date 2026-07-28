import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  LISTPAGES_NAVIGATION_COVERAGE_SCHEMA,
  buildListPagesNavigationCoverage,
} from "../src/listpages-navigation-coverage.mjs";
import { main as buildCoverageCli } from "../scripts/build-listpages-navigation-coverage.mjs";

function matrixCase(id, urlSuffix) {
  return { id, url_suffix: urlSuffix };
}

async function writeMatrix(root, rows) {
  const filePath = path.join(root, "navigation.jsonl");
  await fs.writeFile(
    filePath,
    rows.map((row) => `${JSON.stringify(row)}\n`).join(""),
  );
  return filePath;
}

test("classifies navigation matrix coverage and fails closed on unknown cases", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-navigation-"),
  );
  try {
    const matrixCasesPath = await writeMatrix(root, [
      matrixCase("lpnav-0003-p-2", "/p/2"),
      matrixCase("lpnav-9999-unclassified", "/unclassified"),
    ]);

    const coverage = await buildListPagesNavigationCoverage({
      matrixCasesPath,
    });

    assert.equal(coverage.schema, LISTPAGES_NAVIGATION_COVERAGE_SCHEMA);
    assert.equal(coverage.summary.navigation_cases, 2);
    assert.equal(coverage.summary.gaps, 1);
    assert.deepEqual(coverage.gap_case_ids, [
      "lpnav-9999-unclassified",
    ]);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("writes navigation coverage artifacts without replacement", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-navigation-"),
  );
  try {
    const matrixCasesPath = await writeMatrix(root, [
      matrixCase("lpnav-0001-root", ""),
    ]);
    const output = path.join(root, "coverage.json");
    assert.equal(
      await buildCoverageCli([
        "node",
        "script",
        "--matrix-cases",
        matrixCasesPath,
        "--output",
        output,
      ]),
      0,
    );
    const coverage = JSON.parse(await fs.readFile(output, "utf8"));
    assert.equal(coverage.schema, LISTPAGES_NAVIGATION_COVERAGE_SCHEMA);
    await assert.rejects(
      () =>
        buildCoverageCli([
          "node",
          "script",
          "--matrix-cases",
          matrixCasesPath,
          "--output",
          output,
        ]),
      /EEXIST/u,
    );
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("classifies every formerly open navigation case as locally regressed", async () => {
  const root = await fs.mkdtemp(
    path.join(os.tmpdir(), "wj-listpages-navigation-"),
  );
  try {
    const matrixCasesPath = await writeMatrix(root, [
      matrixCase("lpnav-0011-p-2-tag-alpha", "/p/2/tag/alpha"),
      matrixCase(
        "lpnav-0013-category-fragment-p-2",
        "/category/fragment/p/2",
      ),
      matrixCase(
        "lpnav-0016-page2-limit-1-page3-limit-2",
        "/page2_limit/1/page3_limit/2",
      ),
      matrixCase("lpnav-0017-q-1", "?q=1"),
      matrixCase("lpnav-0018-p-2-q-1", "/p/2?q=1"),
      matrixCase("lpnav-0019-p-2-fragment", "/p/2#fragment"),
    ]);

    const coverage = await buildListPagesNavigationCoverage({
      matrixCasesPath,
    });

    assert.equal(coverage.summary.gaps, 0);
    assert.deepEqual(coverage.gap_case_ids, []);
    assert.ok(
      coverage.coverage.every((row) => row.local_regressions.length > 0),
    );
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});
