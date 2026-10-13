import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const REPOSITORY_ROOT = fileURLToPath(new URL("../../../..", import.meta.url));

const LINE_BUDGETS = new Map([
  [
    "install/local/wikidot-verification/scripts/build-compatibility-surface-inventory.mjs",
    2_750,
  ],
  [
    "install/local/wikidot-verification/src/compatibility-inventory/ftml-raw-surface-manifest.mjs",
    300,
  ],
  [
    "install/local/wikidot-verification/src/listpages-preview-classification.mjs",
    2_300,
  ],
  ["deepwell/src/services/data_form.rs", 1_450],
  ["deepwell/src/services/render/runtime_modules.rs", 1_950],
  ["deepwell/src/services/render/list_pages/mod.rs", 1_500],
  ["deepwell/src/services/render/list_pages/bounded_expansion.rs", 1_000],
  ["deepwell/src/services/render/list_pages/scanner.rs", 2_700],
  ["deepwell/src/services/render/list_pages/rendering.rs", 300],
  ["deepwell/src/services/render/list_pages/substitution.rs", 1_600],
  ["deepwell/src/services/render/list_pages/delayed.rs", 1_600],
  ["deepwell/src/services/page_query/service.rs", 2_500],
  ["deepwell/src/endpoints/page.rs", 1_700],
  ["framerail/src/lib/server/ajax-module-connector.js", 1_500],
  ["deepwell/src/services/render/service.rs", 4_000],
]);

// Directory-scoped line budgets. Each row names its directory, per-file maximum,
// and the exact file filter the former per-directory test applied. One test walks
// this table so a new split directory is a one-row addition rather than another
// copy of the same readdir/filter/offender assertion. The filter is part of the
// row so consolidating cannot widen a budget or admit a previously excluded file.
const DIRECTORY_LINE_BUDGETS = [
  {
    directory: "deepwell/src/services/render/runtime_modules",
    maximum: 700,
    include: (name) => name.endsWith(".rs"),
  },
  {
    directory: "deepwell/src/services/data_form",
    maximum: 700,
    include: (name) => name.endsWith(".rs"),
  },
  {
    directory: "deepwell/src/services/render/service",
    maximum: 1_000,
    include: (name) => name.endsWith(".rs") && name !== "tests.rs",
  },
  {
    directory: "deepwell/src/services/render/list_pages/scanner",
    maximum: 900,
    include: (name) =>
      name.endsWith(".rs") &&
      !name.endsWith("_tests.rs") &&
      name !== "tests.rs",
  },
  {
    directory: "deepwell/src/services/render/list_pages/rendering",
    maximum: 1_500,
    include: (name) => name.endsWith(".rs"),
  },
  {
    directory: "deepwell/src/services/render/list_pages/substitution",
    maximum: 500,
    include: (name) => name.endsWith(".rs"),
  },
  {
    directory: "deepwell/src/services/render/list_pages/substitution/runtime",
    maximum: 950,
    include: (name) => name.endsWith(".rs"),
  },
  {
    directory: "deepwell/src/services/render/list_pages/delayed",
    maximum: 700,
    include: (name) => name.endsWith(".rs"),
  },
  {
    directory: "deepwell/src/services/page_query/service",
    maximum: 700,
    include: (name) => name.endsWith(".rs"),
  },
  {
    directory: "install/local/wikidot-verification/src/listpages-preview-classification",
    maximum: 1_000,
    include: (name) => name.endsWith(".mjs"),
  },
  {
    directory: "install/local/wikidot-verification/src/compatibility-inventory",
    maximum: 700,
    include: (name) => name.endsWith(".mjs"),
  },
];

function lineCount(relativePath) {
  const text = fs.readFileSync(path.join(REPOSITORY_ROOT, relativePath), "utf8");
  return text === "" ? 0 : text.split("\n").length - Number(text.endsWith("\n"));
}

test("compatibility-sensitive production entrypoints stay within their source-size budgets", () => {
  const offenders = [];
  for (const [relativePath, maximum] of LINE_BUDGETS) {
    const actual = lineCount(relativePath);
    if (actual > maximum) {
      offenders.push({ file: relativePath, actual, maximum });
    }
  }
  assert.deepEqual(offenders, []);
});

test("split implementation directories stay within their source-size budgets", () => {
  const offenders = [];
  for (const { directory, maximum, include } of DIRECTORY_LINE_BUDGETS) {
    const absolute = path.join(REPOSITORY_ROOT, directory);
    for (const name of fs.readdirSync(absolute).filter(include)) {
      const relativePath = `${directory}/${name}`;
      const actual = lineCount(relativePath);
      if (actual > maximum) {
        offenders.push({ file: relativePath, actual, maximum });
      }
    }
  }
  offenders.sort((left, right) => right.actual - left.actual);
  assert.deepEqual(offenders, []);
});
