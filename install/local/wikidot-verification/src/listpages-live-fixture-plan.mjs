export const LISTPAGES_LIVE_FIXTURE_PLAN_SCHEMA =
  "wikijump_listpages_compat.live_fixture_plan.v1";

const RUN_DATE = "20260727";
const FULLNAME_PREFIX = `run-owned:lp-campaign-${RUN_DATE}-`;
const TAG_SUFFIX = RUN_DATE;

function fullname(name) {
  return `${FULLNAME_PREFIX}${name}`;
}

function tag(name) {
  return `lp-${name}-${TAG_SUFFIX}`;
}

function page(key, {
  account = "A",
  title,
  sources = [key],
  tags = [],
  parent = null,
  votes = [],
  delayAfterSeconds = 0,
} = {}) {
  return {
    key,
    fullname: fullname(key),
    account,
    title: title ?? `ListPages fixture ${key}`,
    sources,
    tags,
    parent,
    votes,
    ...(delayAfterSeconds > 0
      ? { delay_after_seconds: delayAfterSeconds }
      : {}),
  };
}

function listPages(attributes, body = "%%name%%|") {
  return `[[module ListPages ${attributes}]]\n${body}\n[[/module]]`;
}

function caseBlock(className, label, moduleSource) {
  return `[[div class="lp-case ${className}"]]\n${label}\n${moduleSource}\n[[/div]]`;
}

function parentHolderSource() {
  const common = 'category="run-owned" order="name" separate="no" perPage="250"';
  return [
    caseBlock(
      "lp-parent-static",
      "PARENT STATIC",
      listPages(`${common} parent="${fullname("parent-root")}"`),
    ),
    caseBlock(
      "lp-parent-same",
      "PARENT SAME",
      listPages(`${common} parent="="`),
    ),
    caseBlock(
      "lp-parent-different",
      "PARENT DIFFERENT",
      listPages(`${common} parent="-="`),
    ),
    caseBlock(
      "lp-parent-child",
      "PARENT CHILD",
      listPages(`${common} parent="."`),
    ),
    caseBlock(
      "lp-parent-none",
      "PARENT NONE",
      listPages(`${common} parent="-" name="${FULLNAME_PREFIX}parent-*"`),
    ),
  ].join("\n");
}

function tagHolderSource() {
  const common = 'category="run-owned" order="name" range="others" separate="no" perPage="250"';
  return [
    caseBlock(
      "lp-tags-same",
      "TAGS SAME",
      listPages(`${common} tags="="`),
    ),
    caseBlock(
      "lp-tags-exact",
      "TAGS EXACT",
      listPages(`${common} tags="=="`),
    ),
  ].join("\n");
}

function rangeHolderSource() {
  const common = `category="run-owned" tags="+${tag("range")}" order="name" separate="no" perPage="250"`;
  return [
    caseBlock(
      "lp-range-before",
      "RANGE BEFORE",
      listPages(`${common} range="before"`),
    ),
    caseBlock(
      "lp-range-after",
      "RANGE AFTER",
      listPages(`${common} range="after"`),
    ),
    caseBlock(
      "lp-range-others",
      "RANGE OTHERS",
      listPages(`${common} range="others"`),
    ),
    caseBlock(
      "lp-range-current",
      "RANGE CURRENT",
      listPages(`${common} range="."`),
    ),
  ].join("\n");
}

function metricHolderSource() {
  const common = `category="run-owned" tags="+${tag("metric")}" order="name" separate="no" perPage="250"`;
  const body = "%%name%%:%%rating%%:%%rating_votes%%|";
  return [
    caseBlock(
      "lp-rating-current",
      "RATING CURRENT",
      listPages(`${common} rating="="`, body),
    ),
    caseBlock(
      "lp-rating-not-zero",
      "RATING NOT ZERO",
      listPages(`${common} rating="<>0"`, body),
    ),
    caseBlock(
      "lp-votes-current",
      "VOTES CURRENT",
      listPages(`${common} votes="="`, body),
    ),
    caseBlock(
      "lp-votes-positive",
      "VOTES POSITIVE",
      listPages(`${common} votes=">0"`, body),
    ),
    caseBlock(
      "lp-created-current",
      "CREATED CURRENT",
      listPages(`${common} created_at="="`, body),
    ),
    caseBlock(
      "lp-updated-current",
      "UPDATED CURRENT",
      listPages(`${common} updated_at="="`, body),
    ),
  ].join("\n");
}

function orderHolderSource() {
  const common = `category="run-owned" tags="+${tag("order")}" separate="no" perPage="250"`;
  const body =
    "%%name%%:%%title%%:%%created_by%%:%%size%%:%%rating%%:%%rating_votes%%:%%revisions%%|";
  const orders = [
    ["lp-order-created-desc", "ORDER CREATED DESC", "created_at desc"],
    ["lp-order-created-ascending", "ORDER CREATED ASCENDING", "created_at desc desc"],
    ["lp-order-created-invalid-asc", "ORDER CREATED INVALID ASC", "created_at asc"],
    ["lp-order-unknown", "ORDER UNKNOWN", "unknown"],
    ["lp-order-legacy-created", "ORDER LEGACY CREATED", "dateCreatedAsc"],
    ["lp-order-title", "ORDER TITLE", "title"],
    ["lp-order-size", "ORDER SIZE", "size desc"],
    ["lp-order-rating", "ORDER RATING", "rating desc"],
    ["lp-order-votes", "ORDER VOTES", "votes desc"],
    ["lp-order-revisions", "ORDER REVISIONS", "revisions desc"],
    ["lp-order-created-by", "ORDER CREATED BY", "created_by"],
  ];
  return orders.map(([className, label, order]) =>
    caseBlock(
      className,
      label,
      listPages(`${common} order="${order}"`, body),
    )).join("\n");
}

function paginationHolderSource() {
  const common = `category="run-owned" tags="+${tag("pagination")}" order="name" separate="no"`;
  return [
    caseBlock(
      "lp-pagination-default",
      "PAGINATION DEFAULT",
      listPages(common, "D%%index%%:%%name%%|"),
    ),
    caseBlock(
      "lp-pagination-five",
      "PAGINATION FIVE",
      listPages(`${common} perPage="5"`, "F%%index%%:%%name%%|"),
    ),
    caseBlock(
      "lp-pagination-limited",
      "PAGINATION LIMITED",
      listPages(`${common} limit="7" perPage="3"`, "L%%index%%:%%name%%|"),
    ),
  ].join("\n");
}

function numericHolderSource() {
  const common = `category="run-owned" tags="+${tag("pagination")}" order="name" separate="no"`;
  const cases = [
    ["lp-limit-negative", "LIMIT NEGATIVE", 'limit="-1"'],
    ["lp-limit-float", "LIMIT FLOAT", 'limit="2.5"'],
    ["lp-limit-huge", "LIMIT HUGE", 'limit="999999999"'],
    ["lp-perpage-zero", "PERPAGE ZERO", 'perPage="0"'],
    ["lp-perpage-negative", "PERPAGE NEGATIVE", 'perPage="-1"'],
    ["lp-perpage-float", "PERPAGE FLOAT", 'perPage="2.5"'],
    ["lp-perpage-clamped", "PERPAGE CLAMPED", 'perPage="251"'],
    ["lp-offset-negative", "OFFSET NEGATIVE", 'offset="-1"'],
    ["lp-offset-float", "OFFSET FLOAT", 'offset="2.5"'],
    ["lp-offset-huge", "OFFSET HUGE", 'offset="999999999"'],
  ];
  return cases.map(([className, label, argument]) =>
    caseBlock(
      className,
      label,
      listPages(`${common} ${argument}`),
    )).join("\n");
}

function multipleHolderSource() {
  const common = 'category="run-owned" order="name" separate="no" perPage="2"';
  return [
    caseBlock(
      "lp-multiple-a",
      "MULTIPLE A",
      listPages(`${common} tags="+${tag("multi-a")}"`, "A%%index%%:%%name%%|"),
    ),
    caseBlock(
      "lp-multiple-b",
      "MULTIPLE B",
      listPages(`${common} tags="+${tag("multi-b")}"`, "B%%index%%:%%name%%|"),
    ),
    caseBlock(
      "lp-prefixed-a",
      "PREFIXED A",
      listPages(
        `${common} tags="+${tag("multi-a")}" urlAttrPrefix="a"`,
        "PA%%index%%:%%name%%|",
      ),
    ),
    caseBlock(
      "lp-prefixed-b",
      "PREFIXED B",
      listPages(
        `${common} tags="+${tag("multi-b")}" urlAttrPrefix="b"`,
        "PB%%index%%:%%name%%|",
      ),
    ),
  ].join("\n");
}

function urlHolderSource() {
  const common = 'category="run-owned" order="name" separate="no" perPage="5"';
  return [
    caseBlock(
      "lp-url-tags",
      "URL TAGS",
      listPages(`${common} tags="@URL|${tag("pagination")}"`, "T%%index%%:%%name%%|"),
    ),
    caseBlock(
      "lp-url-offset",
      "URL OFFSET",
      listPages(
        `${common} tags="+${tag("pagination")}" offset="@URL|0"`,
        "O%%index%%:%%name%%|",
      ),
    ),
  ].join("\n");
}

function prefixedUrlHolderSource() {
  const common =
    `category="run-owned" tags="+${tag("pagination")}" order="name" separate="no"`;
  return [
    caseBlock(
      "lp-prefixed-url-page2",
      "PREFIXED URL PAGE2",
      listPages(
        `${common} limit="@URL|0" urlAttrPrefix="page2"`,
        "P2%%index%%:%%name%%|",
      ),
    ),
    caseBlock(
      "lp-prefixed-url-page3",
      "PREFIXED URL PAGE3",
      listPages(
        `${common} limit="@URL|0" urlAttrPrefix="page3"`,
        "P3%%index%%:%%name%%|",
      ),
    ),
  ].join("\n");
}

function variablesHolderSource() {
  const common =
    `fullname="${fullname("variables-target")}" separate="no" wrapper="no"`;
  const variables = [
    ["created-at", "created_at", "%%created_at%%"],
    ["created-at-format", "created_at_format", "%%created_at|%Y-%m-%d%%"],
    ["created-by", "created_by", "%%created_by%%"],
    ["created-by-unix", "created_by_unix", "%%created_by_unix%%"],
    ["created-by-id", "created_by_id", "%%created_by_id%%"],
    ["created-by-linked", "created_by_linked", "%%created_by_linked%%"],
    ["updated-at", "updated_at", "%%updated_at%%"],
    ["updated-by", "updated_by", "%%updated_by%%"],
    ["updated-by-unix", "updated_by_unix", "%%updated_by_unix%%"],
    ["updated-by-id", "updated_by_id", "%%updated_by_id%%"],
    ["updated-by-linked", "updated_by_linked", "%%updated_by_linked%%"],
    ["commented-at", "commented_at", "%%commented_at%%"],
    ["commented-by", "commented_by", "%%commented_by%%"],
    ["commented-by-unix", "commented_by_unix", "%%commented_by_unix%%"],
    ["commented-by-id", "commented_by_id", "%%commented_by_id%%"],
    ["commented-by-linked", "commented_by_linked", "%%commented_by_linked%%"],
    ["name", "name", "%%name%%"],
    ["category", "category", "%%category%%"],
    ["fullname", "fullname", "%%fullname%%"],
    ["title", "title", "%%title%%"],
    ["title-linked", "title_linked", "%%title_linked%%"],
    ["parent-name", "parent_name", "%%parent_name%%"],
    ["parent-category", "parent_category", "%%parent_category%%"],
    ["parent-fullname", "parent_fullname", "%%parent_fullname%%"],
    ["parent-title", "parent_title", "%%parent_title%%"],
    [
      "parent-title-linked",
      "parent_title_linked",
      "%%parent_title_linked%%",
    ],
    ["link", "link", "%%link%%"],
    ["content", "content", "%%content%%"],
    ["content-section", "content_1", "%%content{1}%%"],
    ["preview", "preview", "%%preview%%"],
    ["preview-length", "preview_17", "%%preview(17)%%"],
    ["summary", "summary", "%%summary%%"],
    ["first-paragraph", "first_paragraph", "%%first_paragraph%%"],
    ["tags", "tags", "%%tags%%"],
    ["tags-linked", "tags_linked", "%%tags_linked%%"],
    ["hidden-tags", "_tags", "%%_tags%%"],
    ["hidden-tags-linked", "_tags_linked", "%%_tags_linked%%"],
    ["form-data", "form_data", "%%form_data{missing}%%"],
    ["form-raw", "form_raw", "%%form_raw{missing}%%"],
    ["form-label", "form_label", "%%form_label{missing}%%"],
    ["form-hint", "form_hint", "%%form_hint{missing}%%"],
    ["children", "children", "%%children%%"],
    ["comments", "comments", "%%comments%%"],
    ["size", "size", "%%size%%"],
    ["rating", "rating", "%%rating%%"],
    ["rating-votes", "rating_votes", "%%rating_votes%%"],
    ["rating-percent", "rating_percent", "%%rating_percent%%"],
    ["revisions", "revisions", "%%revisions%%"],
    ["index", "index", "%%index%%"],
    ["total", "total", "%%total%%"],
    ["limit", "limit", "%%limit%%"],
    ["total-or-limit", "total_or_limit", "%%total_or_limit%%"],
    ["site-title", "site_title", "%%site_title%%"],
    ["site-name", "site_name", "%%site_name%%"],
    ["site-domain", "site_domain", "%%site_domain%%"],
    ["legacy-linked-title", "linked_title", "%%linked_title%%"],
    ["legacy-page-unix-name", "page_unix_name", "%%page_unix_name%%"],
    ["legacy-full-page-name", "full_page_name", "%%full_page_name%%"],
    ["legacy-page-name", "page_name", "%%page_name%%"],
    ["legacy-author", "author", "%%author%%"],
    ["legacy-author-edited", "author_edited", "%%author_edited%%"],
    ["legacy-user-edited", "user_edited", "%%user_edited%%"],
    ["legacy-date", "date", "%%date%%"],
    ["legacy-date-edited", "date_edited", "%%date_edited%%"],
    ["legacy-description", "description", "%%description%%"],
    ["legacy-short", "short", "%%short%%"],
    ["legacy-text", "text", "%%text%%"],
    ["legacy-long", "long", "%%long%%"],
    ["legacy-body", "body", "%%body%%"],
  ];

  return variables
    .map(([classSuffix, label, variable]) =>
      caseBlock(
        `lp-var-${classSuffix}`,
        label,
        listPages(common, `${label}=${variable}`),
      ))
    .join("\n");
}

function captureCase(caseId, pageKey, urlSuffix = "", dimensions = []) {
  return {
    case_id: caseId,
    page: pageKey,
    url_suffix: urlSuffix,
    dimensions,
  };
}

export function buildListPagesLiveFixturePlan() {
  const pages = [
    page("parent-root"),
    page("parent-holder", {
      title: "ListPages parent holder",
      sources: [parentHolderSource()],
      parent: "parent-root",
    }),
    page("parent-sibling", { parent: "parent-root" }),
    page("parent-child", { parent: "parent-holder" }),
    page("parent-unrelated"),
    page("tag-holder", {
      title: "ListPages tag holder",
      sources: [tagHolderSource()],
      tags: [tag("same-a"), tag("same-b"), "_lp-holder-hidden"],
    }),
    page("tag-exact", {
      tags: [tag("same-a"), tag("same-b"), "_lp-target-hidden"],
    }),
    page("tag-a", { tags: [tag("same-a")] }),
    page("tag-b", { tags: [tag("same-b")] }),
    page("tag-superset", {
      tags: [tag("same-a"), tag("same-b"), tag("same-c")],
    }),
    page("tag-none"),
    page("range-a", { tags: [tag("range")] }),
    page("range-b", { tags: [tag("range")] }),
    page("range-m", {
      title: "ListPages range holder",
      sources: [rangeHolderSource()],
      tags: [tag("range")],
    }),
    page("range-y", { tags: [tag("range")] }),
    page("range-z", { tags: [tag("range")] }),
    page("metric-holder", {
      title: "ListPages metric holder",
      sources: [metricHolderSource()],
    }),
    page("metric-zero", { tags: [tag("metric")] }),
    page("metric-up", {
      tags: [tag("metric")],
    }),
    page("metric-two", {
      tags: [tag("metric")],
    }),
    page("metric-down", {
      tags: [tag("metric")],
    }),
    page("variables-parent", {
      title: "ListPages variables parent",
    }),
    page("variables-target", {
      account: "B",
      title: "ListPages variables target <&>",
      sources: [
        "First paragraph alpha & beta.\n\nSecond paragraph initial.",
        "First paragraph alpha & beta.\n\nSecond paragraph final.\n\nThird paragraph.",
      ],
      tags: [tag("visible"), "_lp-variable-hidden"],
      parent: "variables-parent",
    }),
    page("variables-holder", {
      title: "ListPages variables holder",
      sources: [variablesHolderSource()],
    }),
  ];

  for (let index = 1; index <= 23; index += 1) {
    const number = String(index).padStart(2, "0");
    const tags = [tag("pagination")];
    if (index <= 5) tags.push(tag("order"), tag("multi-a"));
    if (index >= 6 && index <= 12) tags.push(tag("multi-b"));
    const orderFixture = {
      1: {
        title: "Zulu order target",
        sources: ["x"],
      },
      2: {
        title: "Alpha order target",
        sources: ["This source is deliberately longer than target one."],
      },
      3: {
        title: "Mu order target",
        sources: [
          "revision one",
          "revision two is longer",
          "revision three is the final source",
        ],
      },
      4: {
        account: "B",
        title: "Beta order target",
        sources: ["A source created by account B."],
      },
      5: {
        title: "Eta order target",
        sources: ["Medium source text."],
      },
    }[index] ?? {};
    pages.push(page(`page-${number}`, {
      ...orderFixture,
      tags,
      delayAfterSeconds: index <= 5 ? 1.1 : 0,
    }));
  }

  pages.push(
    page("order-holder", {
      title: "ListPages order holder",
      sources: [orderHolderSource()],
    }),
    page("pagination-holder", {
      title: "ListPages pagination holder",
      sources: [paginationHolderSource()],
    }),
    page("numeric-holder", {
      title: "ListPages numeric holder",
      sources: [numericHolderSource()],
    }),
    page("multiple-holder", {
      title: "ListPages multiple holder",
      sources: [multipleHolderSource()],
    }),
    page("url-holder", {
      title: "ListPages URL holder",
      sources: [urlHolderSource()],
    }),
    page("prefixed-url-holder", {
      title: "ListPages prefixed URL holder",
      sources: [prefixedUrlHolderSource()],
    }),
  );

  const captures = [
    captureCase("lp-live-parent-selectors", "parent-holder", "", [
      "parent",
      "current-page",
    ]),
    captureCase("lp-live-tag-selectors", "tag-holder", "", [
      "tags",
      "hidden-tags",
      "current-page",
    ]),
    captureCase("lp-live-range-selectors", "range-m", "", [
      "range",
      "ordering",
      "current-page",
    ]),
    captureCase("lp-live-metric-selectors", "metric-holder", "", [
      "rating",
      "votes",
      "dates",
    ]),
    captureCase("lp-live-ordering", "order-holder", "", [
      "ordering",
      "legacy-aliases",
      "malformed-values",
    ]),
    captureCase("lp-live-pagination-page-1", "pagination-holder", "", [
      "pagination",
      "direct-load",
    ]),
    captureCase("lp-live-pagination-page-2", "pagination-holder", "/p/2", [
      "pagination",
      "direct-load",
    ]),
    captureCase("lp-live-pagination-page-3", "pagination-holder", "/p/3", [
      "pagination",
      "direct-load",
    ]),
    captureCase("lp-live-pagination-final", "pagination-holder", "/p/5", [
      "pagination",
      "boundary",
    ]),
    captureCase("lp-live-pagination-beyond", "pagination-holder", "/p/999", [
      "pagination",
      "boundary",
    ]),
    captureCase("lp-live-pagination-zero", "pagination-holder", "/p/0", [
      "pagination",
      "malformed",
    ]),
    captureCase("lp-live-pagination-negative", "pagination-holder", "/p/-1", [
      "pagination",
      "malformed",
    ]),
    captureCase("lp-live-pagination-text", "pagination-holder", "/p/nope", [
      "pagination",
      "malformed",
    ]),
    captureCase("lp-live-pagination-missing", "pagination-holder", "/p", [
      "pagination",
      "malformed",
    ]),
    captureCase("lp-live-pagination-query", "pagination-holder", "/p/2?probe=1", [
      "pagination",
      "query-string",
    ]),
    captureCase("lp-live-pagination-repeated", "pagination-holder", "/p/2/p/3", [
      "pagination",
      "repeated-parameter",
    ]),
    captureCase(
      "lp-live-navigation-p-before-tag",
      "pagination-holder",
      `/p/2/tag/${tag("pagination")}`,
      ["pagination", "path-ordering"],
    ),
    captureCase(
      "lp-live-navigation-category-before-p",
      "pagination-holder",
      "/category/fragment/p/2",
      ["pagination", "path-ordering"],
    ),
    captureCase("lp-live-numeric-page-1", "numeric-holder", "", [
      "limit",
      "perPage",
      "offset",
      "coercion",
    ]),
    captureCase("lp-live-numeric-page-2", "numeric-holder", "/p/2", [
      "limit",
      "perPage",
      "offset",
      "pagination",
    ]),
    captureCase("lp-live-multiple-page-1", "multiple-holder", "", [
      "multiple-modules",
      "pagination",
    ]),
    captureCase("lp-live-multiple-page-2", "multiple-holder", "/p/2", [
      "multiple-modules",
      "pagination",
    ]),
    captureCase("lp-live-prefixed-a-page-2", "multiple-holder", "/a_p/2", [
      "multiple-modules",
      "urlAttrPrefix",
      "pagination",
    ]),
    captureCase("lp-live-prefixed-b-page-2", "multiple-holder", "/b_p/2", [
      "multiple-modules",
      "urlAttrPrefix",
      "pagination",
    ]),
    captureCase("lp-live-url-default", "url-holder", "", [
      "@URL",
      "fallback",
    ]),
    captureCase(
      "lp-live-url-tag-order",
      "url-holder",
      `/tag/${tag("order")}`,
      ["@URL", "tags"],
    ),
    captureCase(
      "lp-live-url-offset-two",
      "url-holder",
      "/offset/2",
      ["@URL", "offset"],
    ),
    captureCase(
      "lp-live-url-composed",
      "url-holder",
      `/tag/${tag("pagination")}/offset/2/p/2?probe=1`,
      ["@URL", "tags", "offset", "pagination", "query-string"],
    ),
    captureCase(
      "lp-live-navigation-prefixed-limits",
      "prefixed-url-holder",
      "/page2_limit/1/page3_limit/2",
      ["@URL", "urlAttrPrefix", "multiple-modules", "path-ordering"],
    ),
    captureCase("lp-live-template-variables", "variables-holder", "", [
      "template-variables",
      "aliases",
      "parent",
      "site",
      "content",
    ]),
  ];

  return {
    schema: LISTPAGES_LIVE_FIXTURE_PLAN_SCHEMA,
    run_id: `listpages-${RUN_DATE}`,
    site: "sandbox-for-codex",
    pages,
    captures,
    documentation_provenance: [
      {
        path: "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:page-selection/source.wikidot.txt",
        lines: "20-121",
      },
      {
        path: "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-modules:listpages-module/source.wikidot.txt",
        lines: "55-155,326-365,367-386",
      },
    ],
    live_environment_blockers: [
      {
        capability: "nonzero rating and vote mutation",
        observed_at: "2026-07-27",
        site: "sandbox-for-codex",
        evidence:
          "RateAction returned status not_ok for both run-owned: and _default run-owned pages when account B attempted to vote.",
        impact:
          "Saved-page fixtures still cover zero/current rating and votes selectors, but cannot derive controlled nonzero rating or rating_votes oracle data from this sandbox run.",
      },
    ],
  };
}

export function buildListPagesEdgeLiveFixturePlan() {
  const runDate = "20260728";
  const prefix = `run-owned:lp-campaign-${runDate}-edge-`;
  const sameTag = `lp-edge-same-${runDate}`;
  const edgeFullname = (name) => `${prefix}${name}`;
  const edgePage = (key, options = {}) => ({
    key,
    fullname: edgeFullname(key),
    account: options.account ?? "A",
    title: options.title ?? `ListPages edge fixture ${key}`,
    sources: options.sources ?? [key],
    tags: options.tags ?? [],
    parent: options.parent ?? null,
    votes: [],
  });
  const exact = `category="*" fullname="${edgeFullname("target")}"`;
  const common =
    `category="run-owned" tags="+${sameTag}" order="name" separate="no" perPage="250"`;
  const edgeHolderSource = [
    caseBlock(
      "lp-edge-code-body",
      "EDGE CODE BODY",
      listPages(exact, "[[code]]\n%%title%%\n[[/code]]"),
    ),
    caseBlock(
      "lp-edge-html-body",
      "EDGE HTML BODY",
      listPages(exact, "[[html]]<b>%%title%%</b>[[/html]]"),
    ),
    caseBlock(
      "lp-edge-summary-section",
      "EDGE SUMMARY SECTION",
      listPages(
        exact,
        "summary=%%summary%%|description=%%description%%|content1=%%content{1}%%|",
      ),
    ),
    ...[
      ["lp-edge-skip-yes", "yes"],
      ["lp-edge-skip-true", "true"],
      ["lp-edge-skip-no", "no"],
      ["lp-edge-skip-false", "false"],
      ["lp-edge-skip-invalid", "invalid"],
    ].map(([className, value]) =>
      caseBlock(
        className,
        `EDGE SKIP ${value.toUpperCase()}`,
        listPages(`${common} skipCurrent="${value}"`),
      )),
    ...[
      ["lp-edge-reverse-yes", "yes"],
      ["lp-edge-reverse-true", "true"],
      ["lp-edge-reverse-no", "no"],
      ["lp-edge-reverse-false", "false"],
      ["lp-edge-reverse-invalid", "invalid"],
      ["lp-edge-reverse-empty", ""],
    ].map(([className, value]) =>
      caseBlock(
        className,
        `EDGE REVERSE ${value === "" ? "EMPTY" : value.toUpperCase()}`,
        listPages(`${common} reverse="${value}"`),
      )),
    caseBlock(
      "lp-edge-tags-same-implicit-skip",
      "EDGE TAGS SAME IMPLICIT SKIP",
      listPages(
        'category="run-owned" tags="=" order="name" separate="no" perPage="250"',
      ),
    ),
    caseBlock(
      "lp-edge-tag-target",
      "EDGE TAG TARGET",
      listPages(`${exact} tagTarget="edge-tags"`, "%%tags%%"),
    ),
    caseBlock(
      "lp-edge-link-to-current",
      "EDGE LINK TO CURRENT",
      listPages('category="*" link_to="." order="name" separate="no"'),
    ),
    caseBlock(
      "lp-edge-prepend-separate-yes",
      "EDGE PREPEND SEPARATE YES",
      listPages(
        `${exact} separate="yes" prependLine="PREPEND" appendLine="APPEND"`,
      ),
    ),
  ].join("\n");

  return {
    schema: LISTPAGES_LIVE_FIXTURE_PLAN_SCHEMA,
    run_id: `listpages-edge-${runDate}`,
    site: "sandbox-for-codex",
    pages: [
      edgePage("target", {
        title: "ListPages edge target",
        sources: [
          "Summary first section.\n====\nSecond content section.",
        ],
        tags: [sameTag, "lp-edge-visible"],
      }),
      edgePage("other", { tags: [sameTag] }),
      edgePage("third", { tags: [sameTag] }),
      edgePage("linker", {
        sources: [`[[[${edgeFullname("edge-holder")}]]]`],
      }),
      edgePage("edge-holder", {
        title: "ListPages edge holder",
        sources: [edgeHolderSource],
        tags: [sameTag],
      }),
      edgePage("edge-default-holder", {
        title: "ListPages edge default holder",
        sources: [
          `[[module ListPages category="*" fullname="${edgeFullname("target")}"]]`,
        ],
      }),
    ],
    captures: [
      captureCase("lp-live-edge-behaviors", "edge-holder", "", [
        "legacy-arguments",
        "body-preparse",
        "reverse",
        "summary",
        "tags",
        "link_to",
      ]),
      captureCase("lp-live-default-template", "edge-default-holder", "", [
        "default-template",
        "unclosed-module",
      ]),
    ],
    documentation_provenance: [
      {
        path: "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:note-template-in-modules/source.wikidot.txt",
        lines: "1-5",
      },
      {
        path: "/home/roku/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-include:listpages-module-prev/source.wikidot.txt",
        lines: "9-41,101-163",
      },
    ],
    live_environment_blockers: [],
  };
}
