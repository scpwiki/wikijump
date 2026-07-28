import { buildListPagesLiveFixturePlan } from "./listpages-live-fixture-plan.mjs";

export const LISTPAGES_LIVE_REGRESSION_COVERAGE_SCHEMA =
  "wikijump_listpages_compat.live_regression_coverage.v1";

const PAGE_RS = "deepwell/tests/page.rs";
const LIST_PAGES_RS = "deepwell/tests/list_pages.rs";
const SERVICE_TESTS_RS = "deepwell/src/services/render/service/tests.rs";

function rustTest(path, test) {
  return {
    kind: "rust-test",
    path,
    test,
  };
}

const parentFixture = rustTest(
  PAGE_RS,
  "listpages_parent_selectors_match_the_saved_page_live_fixture",
);
const currentTagsFixture = rustTest(
  PAGE_RS,
  "listpages_current_tag_selectors_match_the_saved_page_live_fixture",
);
const rangeFixture = rustTest(
  PAGE_RS,
  "listpages_range_selectors_match_the_saved_page_live_fixture",
);
const metricFixture = rustTest(
  PAGE_RS,
  "listpages_current_metric_selectors_match_the_saved_page_live_fixture",
);
const paginationFixture = rustTest(
  PAGE_RS,
  "list_pages_saved_view_preserves_live_pagination_path_shapes",
);
const defaultPaginationFixture = rustTest(
  LIST_PAGES_RS,
  "listpages_uses_limit_as_total_and_defaults_pagination_to_twenty",
);
const pagerDomFixture = rustTest(
  PAGE_RS,
  "listpages_perpage_renders_wikidot_pager_controls",
);
const urlTagFixture = rustTest(
  PAGE_RS,
  "list_pages_url_tag_selector_reads_the_url_tag_argument",
);
const urlOffsetFixture = rustTest(
  PAGE_RS,
  "list_pages_url_offset_selector_reads_the_request_route",
);
const orderParseFixture = rustTest(
  SERVICE_TESTS_RS,
  "parses_wikidot_camel_case_list_pages_order_argument",
);
const createdAtOrderFixture = rustTest(
  PAGE_RS,
  "listpages_created_at_order_renders_results",
);
const ratingOrderFixture = rustTest(
  LIST_PAGES_RS,
  "rating_order_listpages_sorts_by_descending_score",
);
const voteOrderFixture = rustTest(
  PAGE_RS,
  "page_query_vote_filter_and_order_use_imported_snapshot_vote_counts",
);
const revisionOrderFixture = rustTest(
  PAGE_RS,
  "listpages_fixture_subset_renders_titles_slugs_order_and_tag_filter",
);
const presentationFixture = rustTest(
  PAGE_RS,
  "listpages_live_evidenced_noop_arguments_render_rows",
);
const structuralVariablesFixture = rustTest(
  PAGE_RS,
  "listpages_structural_identity_and_site_variables_match_live_wikidot",
);
const contentVariablesFixture = rustTest(
  PAGE_RS,
  "listpages_preview_summary_and_content_aliases_match_live_wikidot",
);
const missingFormVariablesFixture = rustTest(
  PAGE_RS,
  "listpages_missing_data_form_variables_stay_literal_without_blocking_rows",
);
const noCommentVariablesFixture = rustTest(
  PAGE_RS,
  "listpages_no_comment_variables_match_live_wikidot",
);
const updatedAuthorAliasesFixture = rustTest(
  PAGE_RS,
  "listpages_legacy_updated_author_aliases_match_live_wikidot",
);
const linkVariableFixture = rustTest(
  PAGE_RS,
  "listpages_link_and_fullname_keep_distinct_wikidot_identities",
);
const variableSubstitutionFixture = rustTest(
  SERVICE_TESTS_RS,
  "substitutes_wikidot_list_pages_author_tool_variables",
);
const childAndRatingPercentFixture = rustTest(
  SERVICE_TESTS_RS,
  "substitutes_wikidot_list_pages_child_count_and_leaves_rating_percent_literal",
);
const revisionVariableFixture = rustTest(
  SERVICE_TESTS_RS,
  "substitutes_wikidot_list_pages_revision_count",
);
const limitVariableFixture = rustTest(
  SERVICE_TESTS_RS,
  "substitutes_wikidot_list_pages_limit_variable",
);

const REGRESSION_COVERAGE = {
  "lp-parent-static": [parentFixture],
  "lp-parent-same": [parentFixture],
  "lp-parent-different": [parentFixture],
  "lp-parent-child": [parentFixture],
  "lp-parent-none": [parentFixture],

  "lp-tags-same": [currentTagsFixture],
  "lp-tags-exact": [currentTagsFixture],

  "lp-range-before": [rangeFixture],
  "lp-range-after": [rangeFixture],
  "lp-range-others": [rangeFixture],
  "lp-range-current": [rangeFixture],

  "lp-rating-current": [metricFixture],
  "lp-rating-not-zero": [metricFixture, ratingOrderFixture],
  "lp-votes-current": [metricFixture],
  "lp-votes-positive": [metricFixture, voteOrderFixture],
  "lp-created-current": [metricFixture],
  "lp-updated-current": [metricFixture],

  "lp-order-created-desc": [createdAtOrderFixture, orderParseFixture],
  "lp-order-created-ascending": [createdAtOrderFixture, orderParseFixture],
  "lp-order-created-invalid-asc": [createdAtOrderFixture, orderParseFixture],
  "lp-order-legacy-created": [createdAtOrderFixture, orderParseFixture],
  "lp-order-title": [revisionOrderFixture, orderParseFixture],
  "lp-order-size": [revisionOrderFixture, orderParseFixture],
  "lp-order-rating": [ratingOrderFixture, orderParseFixture],
  "lp-order-votes": [voteOrderFixture, orderParseFixture],
  "lp-order-revisions": [revisionOrderFixture, orderParseFixture],
  "lp-order-created-by": [revisionOrderFixture, orderParseFixture],
  "lp-order-unknown": [orderParseFixture],

  "lp-pagination-default": [paginationFixture, defaultPaginationFixture],
  "lp-pagination-five": [paginationFixture, pagerDomFixture],
  "lp-pagination-limited": [paginationFixture, pagerDomFixture],

  "lp-limit-negative": [defaultPaginationFixture],
  "lp-limit-float": [defaultPaginationFixture],
  "lp-limit-huge": [defaultPaginationFixture],
  "lp-perpage-zero": [defaultPaginationFixture],
  "lp-perpage-negative": [defaultPaginationFixture],
  "lp-perpage-float": [defaultPaginationFixture],
  "lp-perpage-clamped": [defaultPaginationFixture],
  "lp-offset-negative": [paginationFixture],
  "lp-offset-float": [paginationFixture],
  "lp-offset-huge": [paginationFixture],

  "lp-multiple-a": [paginationFixture],
  "lp-multiple-b": [paginationFixture],
  "lp-prefixed-a": [paginationFixture],
  "lp-prefixed-b": [paginationFixture],
  "lp-prefixed-url-page2": [paginationFixture],
  "lp-prefixed-url-page3": [paginationFixture],

  "lp-url-tags": [urlTagFixture],
  "lp-url-offset": [urlOffsetFixture],
};

for (const blockClass of [
  "lp-var-category",
  "lp-var-created-by-id",
  "lp-var-created-by-unix",
  "lp-var-fullname",
  "lp-var-legacy-full-page-name",
  "lp-var-legacy-page-name",
  "lp-var-legacy-page-unix-name",
  "lp-var-name",
  "lp-var-parent-category",
  "lp-var-parent-fullname",
  "lp-var-parent-name",
  "lp-var-parent-title",
  "lp-var-parent-title-linked",
  "lp-var-site-domain",
  "lp-var-site-name",
  "lp-var-site-title",
  "lp-var-total-or-limit",
  "lp-var-updated-by-id",
  "lp-var-updated-by-unix",
]) {
  REGRESSION_COVERAGE[blockClass] = [structuralVariablesFixture];
}
for (const blockClass of [
  "lp-var-content",
  "lp-var-content-section",
  "lp-var-first-paragraph",
  "lp-var-legacy-body",
  "lp-var-legacy-description",
  "lp-var-legacy-long",
  "lp-var-legacy-short",
  "lp-var-legacy-text",
  "lp-var-preview",
  "lp-var-preview-length",
  "lp-var-summary",
]) {
  REGRESSION_COVERAGE[blockClass] = [contentVariablesFixture];
}
for (const blockClass of [
  "lp-var-form-data",
  "lp-var-form-hint",
  "lp-var-form-label",
  "lp-var-form-raw",
]) {
  REGRESSION_COVERAGE[blockClass] = [missingFormVariablesFixture];
}
for (const blockClass of [
  "lp-var-commented-at",
  "lp-var-commented-by",
  "lp-var-commented-by-id",
  "lp-var-commented-by-linked",
  "lp-var-commented-by-unix",
  "lp-var-comments",
]) {
  REGRESSION_COVERAGE[blockClass] = [noCommentVariablesFixture];
}
for (const blockClass of [
  "lp-var-legacy-author-edited",
  "lp-var-legacy-user-edited",
  "lp-var-updated-by",
  "lp-var-updated-by-linked",
]) {
  REGRESSION_COVERAGE[blockClass] = [updatedAuthorAliasesFixture];
}
for (const blockClass of [
  "lp-var-created-at",
  "lp-var-created-at-format",
  "lp-var-created-by",
  "lp-var-created-by-linked",
  "lp-var-hidden-tags",
  "lp-var-hidden-tags-linked",
  "lp-var-index",
  "lp-var-legacy-author",
  "lp-var-legacy-date",
  "lp-var-legacy-date-edited",
  "lp-var-legacy-linked-title",
  "lp-var-rating",
  "lp-var-rating-votes",
  "lp-var-size",
  "lp-var-tags",
  "lp-var-tags-linked",
  "lp-var-title",
  "lp-var-title-linked",
  "lp-var-total",
  "lp-var-updated-at",
]) {
  REGRESSION_COVERAGE[blockClass] = [variableSubstitutionFixture];
}
REGRESSION_COVERAGE["lp-var-link"] = [linkVariableFixture];
REGRESSION_COVERAGE["lp-var-children"] = [childAndRatingPercentFixture];
REGRESSION_COVERAGE["lp-var-rating-percent"] = [childAndRatingPercentFixture];
REGRESSION_COVERAGE["lp-var-revisions"] = [revisionVariableFixture];
REGRESSION_COVERAGE["lp-var-limit"] = [limitVariableFixture];

const BLOCKER_REFS = {
  "lp-rating-not-zero": ["nonzero rating and vote mutation"],
  "lp-votes-positive": ["nonzero rating and vote mutation"],
  "lp-order-rating": ["nonzero rating and vote mutation"],
  "lp-order-votes": ["nonzero rating and vote mutation"],
  "lp-var-rating-votes": ["nonzero rating and vote mutation"],
  "lp-var-rating-percent": ["nonzero rating and vote mutation"],
  "lp-var-form-label": ["data-form schema mutation"],
  "lp-var-form-hint": ["data-form schema mutation"],
  "lp-var-commented-by-id": ["nonzero comment identity fixture"],
  "lp-var-commented-by-linked": ["nonzero comment identity fixture"],
  "lp-var-commented-by-unix": ["nonzero comment identity fixture"],
};

function extractBlockClassesFromSource(source) {
  const classes = [];
  for (const match of source.matchAll(
    /\[\[div\s+class="([^"]*\blp-case\b[^"]*)"\]\]/giu,
  )) {
    for (const className of match[1].trim().split(/\s+/u)) {
      if (className !== "lp-case") classes.push(className);
    }
  }
  return classes;
}

export function liveFixtureBlockClasses(
  plan = buildListPagesLiveFixturePlan(),
) {
  return [
    ...new Set(
      plan.pages.flatMap((page) =>
        (page.sources ?? []).flatMap(extractBlockClassesFromSource),
      ),
    ),
  ].sort();
}

function liveFixtureBlockProvenance(plan) {
  const classesByPage = new Map(
    plan.pages.map((page) => [
      page.key,
      [
        ...new Set((page.sources ?? []).flatMap(extractBlockClassesFromSource)),
      ].sort(),
    ]),
  );
  const pageByKey = new Map(plan.pages.map((page) => [page.key, page]));
  const provenance = new Map();

  for (const capture of plan.captures) {
    for (const blockClass of classesByPage.get(capture.page) ?? []) {
      if (!provenance.has(blockClass)) {
        provenance.set(blockClass, {
          fixture_pages: [],
          live_capture_case_ids: [],
        });
      }
      const entry = provenance.get(blockClass);
      const page = pageByKey.get(capture.page);
      if (page && !entry.fixture_pages.some((item) => item.key === page.key)) {
        entry.fixture_pages.push({
          key: page.key,
          fullname: page.fullname,
        });
      }
      entry.live_capture_case_ids.push(capture.case_id);
    }
  }

  for (const entry of provenance.values()) {
    entry.fixture_pages.sort((left, right) =>
      left.key.localeCompare(right.key),
    );
    entry.live_capture_case_ids = [
      ...new Set(entry.live_capture_case_ids),
    ].sort();
  }
  return provenance;
}

export function buildListPagesLiveRegressionCoverage(
  plan = buildListPagesLiveFixturePlan(),
) {
  const blockClasses = liveFixtureBlockClasses(plan);
  const provenance = liveFixtureBlockProvenance(plan);
  const coverage = blockClasses.map((blockClass) => {
    const localRegressions = REGRESSION_COVERAGE[blockClass] ?? [];
    const blockerRefs = BLOCKER_REFS[blockClass] ?? [];
    return {
      block_class: blockClass,
      status:
        localRegressions.length > 0 ? "covered" : "missing-local-regression",
      ...(blockerRefs.length > 0
        ? { live_environment_blocker_refs: blockerRefs }
        : {}),
      local_regressions: localRegressions,
      ...(provenance.get(blockClass) ?? {
        fixture_pages: [],
        live_capture_case_ids: [],
      }),
    };
  });
  const missing = coverage
    .filter((entry) => entry.status !== "covered")
    .map((entry) => entry.block_class);

  return {
    schema: LISTPAGES_LIVE_REGRESSION_COVERAGE_SCHEMA,
    summary: {
      block_classes: blockClasses.length,
      covered: coverage.length - missing.length,
      missing_local_regression: missing.length,
      blocker_referenced: coverage.filter(
        (entry) => entry.live_environment_blocker_refs?.length > 0,
      ).length,
    },
    missing_local_regression_block_classes: missing,
    live_environment_blockers: plan.live_environment_blockers,
    coverage,
  };
}
