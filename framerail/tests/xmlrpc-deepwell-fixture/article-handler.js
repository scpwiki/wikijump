import {
  fixtureState,
  hasExactKeys,
  isFixtureSiteId,
  requestContextHeaders
} from "./context.js"
import { pages, toArticleViewResult } from "./data.js"

const LISTPAGES_NAVIGATION_EXTRA = /^p\/[1-9][0-9]*$/u
const SEARCH_SITE_EXTRA = /^q\/.+$/u
const NEW_PAGE_EDIT_EXTRA = /^edit\/true(?:\/.*)?$/u
const DATA_FORM_CREATE_SLUG = "data-form-create-flow:example"
const DATA_FORM_CONTROLS_CREATE_SLUG = "data-form-controls-flow:example"
const DATA_FORM_REGEX_BUDGET_CREATE_SLUG = "data-form-regex-budget-flow:example"
const DATA_FORM_INVALID_REGEX_CREATE_SLUG = "data-form-invalid-regex-flow:example"
const DATA_FORM_EMPTY_SELECT_CREATE_SLUG = "data-form-empty-select-flow:example"
const DATA_FORM_PROPERTIES_CREATE_SLUG = "data-form-properties-flow:example"
const DATA_FORM_CHECKBOX_WIKI_CREATE_SLUG = "data-form-checkbox-wiki-flow:example"
const DATA_FORM_DATE_CREATE_SLUG = "data-form-date-field-flow:example"
const DATA_FORM_DATE_OPTIONS_CREATE_SLUG = "data-form-date-options-flow:example"
const DATA_FORM_PAGEPATH_CREATE_SLUG = "data-form-pagepath-flow:example"
const DATA_FORM_PAGEPATH_ROOT_CREATE_SLUG = "data-form-pagepath-root-flow:example"
const DATA_FORM_EDIT_SLUG = "data-form-edit-flow:example"
const DATA_FORM_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "name",
      label: "Name",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null
    },
    {
      name: "choice",
      label: "Choice",
      hint: "",
      field_type: "select",
      values: [
        { value: "a", label: "Alpha" },
        { value: "b", label: "Beta" }
      ],
      default_value: "b"
    }
  ]
}
const DATA_FORM_DATE_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "date",
      label: "Date value",
      hint: "",
      field_type: "date",
      values: [],
      default_value: null,
      configured_value: null,
      width: 24,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: false,
      options: { dateFormat: "mm/dd/yy", showOn: "button" }
    }
  ]
}
const DATA_FORM_DATE_OPTIONS_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "date-primary",
      label: "Primary date",
      hint: "",
      field_type: "date",
      values: [],
      default_value: null,
      configured_value: null,
      width: 24,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: false,
      options: {
        altField: "input[name=field-alt-date]",
        altFormat: "m/d/yy",
        dateFormat: "mm/dd/yy",
        showOn: "button"
      }
    },
    {
      name: "date-secondary",
      label: "Secondary date",
      hint: "",
      field_type: "date",
      values: [],
      default_value: null,
      configured_value: null,
      width: 24,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: false,
      options: { dateFormat: "DD, d MM yy", showOn: "both", firstDay: 1 }
    },
    {
      name: "alt-date",
      label: "Alternate date",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      configured_value: null,
      width: 10,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: false
    }
  ]
}
const DATA_FORM_PAGEPATH_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "origin",
      label: "Origin",
      hint: "",
      field_type: "pagepath",
      values: [],
      default_value: null,
      configured_value: null,
      options: {},
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: false,
      pagepath_category: "data-form-pagepath-tree",
      pagepath_max_level: 3
    }
  ]
}
const DATA_FORM_PAGEPATH_NODES = {
  origin: [
    { fullname: "data-form-pagepath-tree:_root", name: "_root", parent: null },
    {
      fullname: "data-form-pagepath-tree:alpha",
      name: "alpha",
      parent: "data-form-pagepath-tree:_root"
    },
    {
      fullname: "data-form-pagepath-tree:beta",
      name: "beta",
      parent: "data-form-pagepath-tree:alpha"
    }
  ]
}
const DATA_FORM_PAGEPATH_ROOT_DEFINITION = {
  ...DATA_FORM_PAGEPATH_DEFINITION,
  fields: DATA_FORM_PAGEPATH_DEFINITION.fields.map((field) => ({
    ...field,
    pagepath_category: "data-form-pagepath-root-tree"
  }))
}
const DATA_FORM_CONTROLS_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "plain",
      label: "Plain text",
      hint: "",
      field_type: "text",
      values: [],
      default_value: "**bold** #hash",
      width: 1,
      height: 1,
      match_pattern: null,
      match_error: null
    },
    {
      name: "multi",
      label: "Multi line",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      width: 50,
      height: 3,
      match_pattern: null,
      match_error: null
    },
    {
      name: "matched",
      label: "Matched text",
      hint: "enter a color like \\#468259",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: "/^ok-[0-9]+$/",
      match_error: "Use ok- followed by digits"
    },
    {
      name: "select_one",
      label: "Select one",
      hint: "",
      field_type: "select",
      values: [{ value: "a", label: "Alpha" }],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    },
    {
      name: "select_four",
      label: "Select four",
      hint: "",
      field_type: "select",
      values: [
        { value: "a", label: "Alpha" },
        { value: "b", label: "Beta" },
        { value: "c", label: "Gamma" },
        { value: "d", label: "Delta" }
      ],
      default_value: "c",
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    },
    {
      name: "select_five",
      label: "Select five",
      hint: "",
      field_type: "select",
      values: [
        { value: "0", label: "Zero" },
        { value: "1", label: "One" },
        { value: "2", label: "Two" },
        { value: "3", label: "Three" },
        { value: "4", label: "Four" }
      ],
      default_value: "4",
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    }
  ]
}
const DATA_FORM_REGEX_BUDGET_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "matched",
      label: "Matched text",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: "/^(a+)+$/",
      match_error: "Use only a characters"
    },
    {
      name: "matched_two",
      label: "Second matched text",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: "/^(a+)+$/",
      match_error: "Use only a characters"
    }
  ]
}
const DATA_FORM_INVALID_REGEX_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "matched",
      label: "Matched text",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: "/(/",
      match_error: "Site-authored mismatch message"
    }
  ]
}
const DATA_FORM_EMPTY_SELECT_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "missing_values",
      label: "Missing values",
      hint: "",
      field_type: "select",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    },
    {
      name: "empty_values",
      label: "Empty values",
      hint: "",
      field_type: "select",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    },
    {
      name: "select_one",
      label: "Select one",
      hint: "",
      field_type: "select",
      values: [{ value: "a", label: "Alpha" }],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    },
    {
      name: "select_two",
      label: "Select two",
      hint: "",
      field_type: "select",
      values: [
        { value: "a", label: "Alpha" },
        { value: "b", label: "Beta" }
      ],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    },
    {
      name: "select_five",
      label: "Select five",
      hint: "",
      field_type: "select",
      values: ["a", "b", "c", "d", "e"].map((value) => ({
        value,
        label: value.toUpperCase()
      })),
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null
    }
  ]
}
const DATA_FORM_PROPERTIES_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "base",
      label: "Base label",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: false
    },
    {
      name: "joined",
      label: "Joined label",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: "/^ok$/i",
      match_error: null,
      before: "PRE",
      after: "POST",
      join: true
    },
    {
      name: "extended",
      label: "",
      hint: "  padded # hint  ",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 2,
      match_pattern: "/^a b$/x",
      match_error: "Extended mismatch",
      before: "pre # ",
      after: " post",
      join: false
    },
    {
      name: "duplicate_modifier",
      label: "Duplicate modifier",
      hint: "",
      field_type: "text",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: "/^ok$/ii",
      match_error: "Duplicate mismatch",
      before: "",
      after: "",
      join: false
    },
    {
      name: "choice",
      label: "Choice",
      hint: "ignored select hint",
      field_type: "select",
      values: [
        { value: "a", label: "Alpha" },
        { value: "b", label: "Beta" }
      ],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "PRE",
      after: "POST",
      join: false
    }
  ]
}
const DATA_FORM_CHECKBOX_WIKI_DEFINITION = {
  default_layout: true,
  fields: [
    {
      name: "checkbox_unchecked",
      label: "Unchecked",
      hint: "ignored checkbox hint",
      field_type: "checkbox",
      values: [],
      default_value: null,
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "PRE",
      after: "POST",
      join: false
    },
    {
      name: "checkbox_checked",
      label: "Checked",
      hint: "",
      field_type: "checkbox",
      values: [],
      default_value: "1",
      width: 40,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: true
    },
    {
      name: "wiki",
      label: "Wiki",
      hint: "enter wiki \\#source",
      field_type: "wiki",
      values: [],
      default_value: "**Default**",
      width: 40,
      height: 2,
      match_pattern: null,
      match_error: null,
      before: "**Before**",
      after: "//After//",
      join: false
    },
    {
      name: "wiki_one_line",
      label: "Wiki one line",
      hint: "",
      field_type: "wiki",
      values: [],
      default_value: null,
      width: 20,
      height: 1,
      match_pattern: null,
      match_error: null,
      before: "",
      after: "",
      join: false
    }
  ]
}

const optionValue = (extra, name) => {
  const parts = extra.split("/")
  for (let index = 0; index + 1 < parts.length; index += 2) {
    if (parts[index].toLowerCase() === name.toLowerCase()) {
      return decodeURIComponent(parts[index + 1])
    }
  }
  return null
}

/** @param {{ slug: string; extra: string }} route */
const pageForArticleRoute = (route) => {
  const aliases = {
    "PAGE-WORKFLOW-PROBE": "page-workflow-probe",
    "page_workflow-probe": "page-workflow-probe",
    "page-workflow-probe.": "page-workflow-probe",
    "page-workflow-probe ": "page-workflow-probe"
  }
  const canonicalSlug = aliases[route.slug]
  const page = pages[canonicalSlug ?? route.slug]
  if (!page) return null
  const resolvedPage = canonicalSlug ? { ...page, redirect_page: page.slug } : page
  if (route.slug === "listpages-navigation") {
    if (route.extra !== "" && !LISTPAGES_NAVIGATION_EXTRA.test(route.extra)) {
      return null
    }
    return {
      ...page,
      compiled_body_html: [
        `<span id="listpages-route">${route.extra || "root"}</span>`,
        '<div class="pager">',
        '<span class="target"><a id="listpages-page-one" href="/listpages-navigation/p/1">1</a></span>',
        '<span class="target"><a id="listpages-page-two" href="/listpages-navigation/p/2">2</a></span>',
        "</div>"
      ].join("")
    }
  }
  if (route.slug === "search:site") {
    return SEARCH_SITE_EXTRA.test(route.extra) ? page : null
  }
  if (
    (route.slug === DATA_FORM_EDIT_SLUG ||
      route.slug === DATA_FORM_CONTROLS_CREATE_SLUG ||
      route.slug === DATA_FORM_EMPTY_SELECT_CREATE_SLUG ||
      route.slug === DATA_FORM_DATE_OPTIONS_CREATE_SLUG) &&
    route.extra === "edit"
  ) {
    return page
  }
  // The `/edit` route loads the same page through article_view with the edit
  // extra, so ordinary fixture pages model the editor's page data too.
  if (route.extra === "edit") return page
  return route.extra === "" ? resolvedPage : null
}

const siteView = {
  site: {
    site_id: 6000005,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: null,
    deleted_at: null,
    from_wikidot: false,
    slug: "scp-wiki",
    name: "SCP Foundation",
    tagline: "Secure, Contain, Protect",
    description: "Fixture site",
    locale: "en",
    default_page: "main",
    top_bar_page: null,
    side_bar_page: null,
    preferred_domain: null,
    layout: "wikidot",
    license: "cc-by-sa-3.0"
  },
  site_file_domain: "scp-wiki.wjfiles.localhost",
  license_name: "CC BY-SA 3.0",
  license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
  license_kind: "standard",
  license_html: null,
  user_session: null
}

/** @param {{ slug: string; extra: string }} route */
const missingPageArticleViewResult = (route) => ({
  ...siteView,
  article_page_cache_key: null,
  public_content_cache_fence: null,
  anonymous_permission_cache_fence: null,
  page: {
    type: "missing",
    data: {
      options: {
        edit: NEW_PAGE_EDIT_EXTRA.test(route.extra),
        title: optionValue(route.extra, "title"),
        parent:
          optionValue(route.extra, "parentPage") ?? optionValue(route.extra, "parent"),
        tags: optionValue(route.extra, "tags"),
        no_redirect: false,
        no_render: false,
        debug: false,
        renderer: false,
        comments: false,
        history: false,
        offset: null,
        data: ""
      },
      redirect_page: null,
      wikitext: "",
      compiled_body_html: "",
      compiled_body_styles: [],
      compiled_top_bar_html: null,
      compiled_side_bar_html: null,
      new_page_wikitext: null,
      page_templates: [],
      selected_template_page_id: null,
      data_form:
        route.slug === DATA_FORM_CREATE_SLUG
          ? { definition: DATA_FORM_DEFINITION, values: {}, pagepaths: {} }
          : route.slug === DATA_FORM_DATE_CREATE_SLUG
            ? { definition: DATA_FORM_DATE_DEFINITION, values: {}, pagepaths: {} }
            : route.slug === DATA_FORM_DATE_OPTIONS_CREATE_SLUG
              ? {
                  definition: DATA_FORM_DATE_OPTIONS_DEFINITION,
                  values: {},
                  pagepaths: {}
                }
              : route.slug === DATA_FORM_PAGEPATH_CREATE_SLUG
                ? {
                    definition: DATA_FORM_PAGEPATH_DEFINITION,
                    values: {},
                    pagepaths: DATA_FORM_PAGEPATH_NODES
                  }
                : route.slug === DATA_FORM_PAGEPATH_ROOT_CREATE_SLUG
                  ? {
                      definition: DATA_FORM_PAGEPATH_ROOT_DEFINITION,
                      values: {},
                      pagepaths: { origin: [] }
                    }
                  : route.slug === DATA_FORM_CONTROLS_CREATE_SLUG
                    ? {
                        definition: DATA_FORM_CONTROLS_DEFINITION,
                        values: {},
                        pagepaths: {}
                      }
                    : route.slug === DATA_FORM_REGEX_BUDGET_CREATE_SLUG
                      ? {
                          definition: DATA_FORM_REGEX_BUDGET_DEFINITION,
                          values: {},
                          pagepaths: {}
                        }
                      : route.slug === DATA_FORM_INVALID_REGEX_CREATE_SLUG
                        ? {
                            definition: DATA_FORM_INVALID_REGEX_DEFINITION,
                            values: {},
                            pagepaths: {}
                          }
                        : route.slug === DATA_FORM_EMPTY_SELECT_CREATE_SLUG
                          ? {
                              definition: DATA_FORM_EMPTY_SELECT_DEFINITION,
                              values: {},
                              pagepaths: {}
                            }
                          : route.slug === DATA_FORM_PROPERTIES_CREATE_SLUG
                            ? {
                                definition: DATA_FORM_PROPERTIES_DEFINITION,
                                values: {},
                                pagepaths: {}
                              }
                            : route.slug === DATA_FORM_CHECKBOX_WIKI_CREATE_SLUG
                              ? {
                                  definition: DATA_FORM_CHECKBOX_WIKI_DEFINITION,
                                  values: {},
                                  pagepaths: {}
                                }
                              : null
    }
  }
})

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handleArticleRpc = ({ rpcRequest, request }) => {
  const { articleReadRequests, pageReadRequests } = fixtureState
  const articleRoute =
    rpcRequest.params?.route === null
      ? { slug: "main", extra: "" }
      : rpcRequest.params?.route
  let result

  if (
    rpcRequest.method === "article_view" &&
    ((hasExactKeys(rpcRequest.params, ["locales", "route", "site_id"]) &&
      rpcRequest.params.session_token === undefined) ||
      (hasExactKeys(rpcRequest.params, [
        "locales",
        "route",
        "session_token",
        "site_id"
      ]) &&
        ["fixture-session-token", "fixture-authenticated-session-token"].includes(
          rpcRequest.params.session_token
        ))) &&
    isFixtureSiteId(rpcRequest.params.site_id) &&
    Array.isArray(rpcRequest.params.locales) &&
    (rpcRequest.params.route === null ||
      hasExactKeys(rpcRequest.params.route, ["extra", "slug"])) &&
    typeof articleRoute.slug === "string" &&
    (pageForArticleRoute(articleRoute) ||
      ((articleRoute.slug === DATA_FORM_CREATE_SLUG ||
        articleRoute.slug === DATA_FORM_DATE_CREATE_SLUG ||
        articleRoute.slug === DATA_FORM_DATE_OPTIONS_CREATE_SLUG ||
        articleRoute.slug === DATA_FORM_PAGEPATH_CREATE_SLUG ||
        articleRoute.slug === DATA_FORM_PAGEPATH_ROOT_CREATE_SLUG) &&
        articleRoute.extra === "") ||
      NEW_PAGE_EDIT_EXTRA.test(articleRoute.extra))
  ) {
    articleReadRequests.articleView.push(rpcRequest.params)
    const page = pageForArticleRoute(articleRoute)
    if (page) {
      result = toArticleViewResult(page, rpcRequest.params.site_id)
      if (articleRoute.slug === DATA_FORM_EDIT_SLUG && articleRoute.extra === "edit") {
        result.page.data.options.edit = true
        result.page.data.data_form = {
          definition: DATA_FORM_DEFINITION,
          values: { name: "Probe Name", choice: "a" }
        }
      } else if (
        articleRoute.slug === DATA_FORM_DATE_OPTIONS_CREATE_SLUG &&
        articleRoute.extra === "edit"
      ) {
        result.page.data.options.edit = true
        result.page.data.data_form = {
          definition: DATA_FORM_DATE_OPTIONS_DEFINITION,
          values: {
            "date-primary": "1411099200",
            "date-secondary": "1418360400",
            "alt-date": "12/11/2014"
          }
        }
      } else if (
        articleRoute.slug === DATA_FORM_CONTROLS_CREATE_SLUG &&
        articleRoute.extra === "edit"
      ) {
        result.page.data.options.edit = true
        result.page.data.data_form = {
          definition: DATA_FORM_CONTROLS_DEFINITION,
          values: {
            plain: `O'Brien: # [x] \\ slash "quote"`,
            multi: `first "quoted"\nsecond 'single' \\ end`,
            matched: "ok-42",
            select_one: "a",
            select_four: "c",
            select_five: "2"
          }
        }
      } else if (
        articleRoute.slug === DATA_FORM_EMPTY_SELECT_CREATE_SLUG &&
        articleRoute.extra === "edit"
      ) {
        result.page.data.options.edit = true
        result.page.data.data_form = {
          definition: DATA_FORM_EMPTY_SELECT_DEFINITION,
          values: {
            missing_values: "",
            empty_values: "",
            select_one: "",
            select_two: "",
            select_five: "a"
          }
        }
      } else if (articleRoute.extra === "edit") {
        // Catch-all for ordinary pages opened through the `/edit` route; the
        // DATA_FORM_* arms above must stay first because they also attach
        // their data-form definitions.
        result.page.data.options.edit = true
      }
    } else {
      result = missingPageArticleViewResult(articleRoute)
    }
  } else if (
    rpcRequest.method === "article_view_cache_metadata" &&
    hasExactKeys(rpcRequest.params, ["locales", "route", "session_token", "site_id"]) &&
    isFixtureSiteId(rpcRequest.params.site_id) &&
    rpcRequest.params.session_token === null &&
    Array.isArray(rpcRequest.params.locales) &&
    (rpcRequest.params.route === null ||
      hasExactKeys(rpcRequest.params.route, ["extra", "slug"])) &&
    typeof articleRoute.slug === "string" &&
    pageForArticleRoute(articleRoute)
  ) {
    articleReadRequests.articleViewCacheMetadata.push(rpcRequest.params)
    const page = pageForArticleRoute(articleRoute)
    if (!page) return undefined
    result = {
      article_page_cache_key: `deepwell:article-view:page:v1:site=${rpcRequest.params.site_id}:page=${page.page_id}:rev=${page.revision_id}:updated=0:permission=site=0,user=0:body=fixture`,
      public_content_cache_fence: "0",
      anonymous_permission_cache_fence: "site=0,user=0"
    }
  } else if (
    rpcRequest.method === "page_view" &&
    hasExactKeys(rpcRequest.params, ["locales", "route", "session_token", "site_id"]) &&
    isFixtureSiteId(rpcRequest.params.site_id) &&
    Array.isArray(rpcRequest.params.locales) &&
    rpcRequest.params.session_token === "fixture-session-token" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    isFixtureSiteId(Number(request.headers["x-deepwell-site-id"])) &&
    hasExactKeys(rpcRequest.params.route, ["extra", "slug"]) &&
    typeof rpcRequest.params.route.slug === "string" &&
    rpcRequest.params.route.extra === ""
  ) {
    pageReadRequests.pageView.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    const page = pages[rpcRequest.params.route.slug]
    result = page
      ? page.slug === "private-page"
        ? { type: "forbidden", data: {} }
        : { type: "found", data: { page: { slug: page.slug } } }
      : { type: "missing", data: {} }
  } else {
    return undefined
  }

  return { result }
}
