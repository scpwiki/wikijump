import fs from "node:fs/promises";
import path from "node:path";

import { sha256 } from "./syntax-differential.mjs";

export const LISTPAGES_NAVIGATION_COVERAGE_SCHEMA =
  "wikijump_listpages_compat.navigation_coverage.v1";

const PAGE_RS = "deepwell/tests/page.rs";
const SERVICE_TESTS_RS = "deepwell/src/services/render/service/tests.rs";

function rustTest(path, test) {
  return { kind: "rust-test", path, test };
}

const savedPathFixture = rustTest(
  PAGE_RS,
  "list_pages_saved_view_preserves_live_pagination_path_shapes",
);
const offsetRouteFixture = rustTest(
  PAGE_RS,
  "list_pages_url_offset_selector_reads_the_request_route",
);
const urlTagFixture = rustTest(
  PAGE_RS,
  "list_pages_url_tag_selector_reads_the_url_tag_argument",
);
const pagerShapeFixture = rustTest(
  SERVICE_TESTS_RS,
  "generated_list_pages_pager_preserves_live_url_argument_shape",
);
const prefixFixture = rustTest(
  SERVICE_TESTS_RS,
  "generated_list_pages_pager_uses_url_attr_prefix",
);
const prefixParseFixture = rustTest(
  "deepwell/src/services/render/url_arguments.rs",
  "page_selection_uses_last_positive_matching_prefix",
);
const argumentOrderFixture = rustTest(
  PAGE_RS,
  "list_pages_saved_view_preserves_argument_order_around_the_pager",
);
const prefixedUrlFixture = rustTest(
  PAGE_RS,
  "list_pages_prefixed_url_arguments_address_only_the_matching_module",
);
const browserNavigationFixture = {
  kind: "playwright-test",
  path: "framerail/tests/list-pages-navigation.spec.ts",
  test: "ListPages pagination survives direct loads, reload, and browser history",
};

const COVERAGE = {
  "lpnav-0001-root": {
    status: "covered",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-page-1"],
  },
  "lpnav-0002-p-1": {
    status: "covered-local-live-inferred",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-page-1"],
    note: "Live root capture and local /p/1 assertion cover first-page equivalence.",
  },
  "lpnav-0003-p-2": {
    status: "covered",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-page-2"],
  },
  "lpnav-0004-p-3": {
    status: "covered",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-page-3"],
  },
  "lpnav-0005-p-0": {
    status: "covered",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-zero"],
  },
  "lpnav-0006-p-1": {
    status: "covered",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-negative"],
  },
  "lpnav-0007-p-abc": {
    status: "covered-local-live-semantic",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-text"],
  },
  "lpnav-0008-p-2-5": {
    status: "covered-local-live-semantic",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-text"],
  },
  "lpnav-0009-p-999999999": {
    status: "covered-local-live-semantic",
    local_regressions: [savedPathFixture],
    live_capture_case_ids: ["lp-live-pagination-beyond"],
  },
  "lpnav-0010-tag-alpha-p-2": {
    status: "covered-semantic",
    local_regressions: [offsetRouteFixture, urlTagFixture, pagerShapeFixture],
    live_capture_case_ids: ["lp-live-url-composed"],
    note: "Exact tag value differs; live fixture covers tag + offset + pagination ordering.",
  },
  "lpnav-0011-p-2-tag-alpha": {
    status: "covered",
    local_regressions: [argumentOrderFixture, urlTagFixture],
    live_capture_case_ids: ["lp-live-navigation-p-before-tag"],
  },
  "lpnav-0012-p-2-p-3": {
    status: "covered",
    local_regressions: [savedPathFixture, pagerShapeFixture],
    live_capture_case_ids: ["lp-live-pagination-repeated"],
  },
  "lpnav-0013-category-fragment-p-2": {
    status: "covered",
    local_regressions: [argumentOrderFixture],
    live_capture_case_ids: ["lp-live-navigation-category-before-p"],
  },
  "lpnav-0014-offset-1-p-2": {
    status: "covered",
    local_regressions: [offsetRouteFixture],
    live_capture_case_ids: ["lp-live-url-composed"],
  },
  "lpnav-0015-prefix-p-2": {
    status: "covered-semantic",
    local_regressions: [savedPathFixture, prefixFixture, prefixParseFixture],
    live_capture_case_ids: [
      "lp-live-prefixed-a-page-2",
      "lp-live-prefixed-b-page-2",
    ],
    note: "Exact prefix differs; live and local cover urlAttrPrefix page selection semantics.",
  },
  "lpnav-0016-page2-limit-1-page3-limit-2": {
    status: "covered",
    local_regressions: [prefixedUrlFixture],
    live_capture_case_ids: ["lp-live-navigation-prefixed-limits"],
  },
  "lpnav-0017-q-1": {
    status: "covered",
    local_regressions: [browserNavigationFixture],
    live_capture_case_ids: ["lp-live-pagination-query"],
  },
  "lpnav-0018-p-2-q-1": {
    status: "covered",
    local_regressions: [browserNavigationFixture],
    live_capture_case_ids: ["lp-live-pagination-query"],
  },
  "lpnav-0019-p-2-fragment": {
    status: "covered",
    local_regressions: [browserNavigationFixture],
    live_capture_case_ids: ["lp-live-pagination-page-2"],
    note: "The path behavior is live-captured; the fragment is browser-only, never enters Deepwell route.extra, and is covered through the served-page browser seam.",
  },
};

async function readJsonl(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  if (!text.trim()) return [];
  return text
    .trimEnd()
    .split(/\r?\n/u)
    .map((line) => JSON.parse(line));
}

function isGap(status) {
  return status.startsWith("missing") || status === "covered-live-only";
}

export async function buildListPagesNavigationCoverage({ matrixCasesPath }) {
  const matrixText = await fs.readFile(matrixCasesPath, "utf8");
  const matrixCases = await readJsonl(matrixCasesPath);
  const coverage = matrixCases.map((matrixCase) => {
    const entry = COVERAGE[matrixCase.id] ?? {
      status: "missing-coverage-entry",
      local_regressions: [],
      live_capture_case_ids: [],
    };
    return {
      case_id: matrixCase.id,
      url_suffix: matrixCase.url_suffix,
      status: entry.status,
      local_regressions: entry.local_regressions,
      live_capture_case_ids: entry.live_capture_case_ids,
      ...(entry.note ? { note: entry.note } : {}),
    };
  });

  const statuses = {};
  for (const row of coverage) {
    statuses[row.status] = (statuses[row.status] ?? 0) + 1;
  }
  const gaps = coverage.filter((row) => isGap(row.status));

  return {
    schema: LISTPAGES_NAVIGATION_COVERAGE_SCHEMA,
    generated_at: new Date().toISOString(),
    inputs: {
      matrix_cases_path: matrixCasesPath,
      matrix_cases_sha256: sha256(matrixText),
    },
    coverage,
    gap_case_ids: gaps.map((row) => row.case_id),
    summary: {
      navigation_cases: coverage.length,
      exact_or_semantic_covered: coverage.length - gaps.length,
      gaps: gaps.length,
      statuses,
      exit_code: gaps.length > 0 ? 1 : 0,
    },
  };
}

export async function writeListPagesNavigationCoverage(coverage, outputPath) {
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, `${JSON.stringify(coverage, null, 2)}\n`, {
    encoding: "utf8",
    flag: "wx",
  });
}
