/**
 * @typedef {{
 *   compiled_html?: boolean
 *   wikitext?: boolean
 * }} PageDetails
 *
 *
 * @typedef {{
 *   compiled_body_html: string
 *   compiled_body_styles?: string[]
 *   creator_user_id: number
 *   page_created_at: string
 *   page_id: number
 *   page_revision_count: number
 *   page_updated_at: string | null
 *   rating: number
 *   revision_created_at: string
 *   revision_id: number
 *   revision_user_id: number
 *   slug: string
 *   tags: string[]
 *   title: string
 *   wikitext: string
 * }} FixturePage
 *
 *
 * @typedef {{
 *   content: string
 *   created_at: string
 *   created_by: string
 *   html: string
 *   id: number
 *   reply_to: number | null
 *   title: string
 * }} FixtureForumPost
 */

/** @type {Record<string, FixturePage>} */
export const pages = {
  main: {
    page_id: 3000001,
    revision_id: 9000001,
    page_created_at: "2008-07-19T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2008-07-19T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Main",
    slug: "main",
    tags: [],
    rating: 0,
    wikitext: "Main",
    compiled_body_html: "<p>Main</p>"
  },
  "scp-173": {
    page_id: 3000173,
    revision_id: 9000173,
    page_created_at: "2008-07-26T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 3,
    revision_created_at: "2008-07-26T00:00:00Z",
    revision_user_id: 456,
    creator_user_id: 123,
    title: "SCP-173",
    slug: "scp-173",
    tags: ["scp", "euclid"],
    rating: 173,
    wikitext: "**Item #:** SCP-173",
    compiled_body_html: "<p><strong>Item #:</strong> SCP-173</p>"
  },
  "scp-173-parent": {
    page_id: 3000172,
    revision_id: 9000172,
    page_created_at: "2008-07-25T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2008-07-25T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "SCP Foundation",
    slug: "scp-173-parent",
    tags: ["hub"],
    rating: 1,
    wikitext: "Parent",
    compiled_body_html: "<p>Parent</p>"
  },
  "private-page": {
    page_id: 3000199,
    revision_id: 9000199,
    page_created_at: "2026-07-01T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-01T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Private Page",
    slug: "private-page",
    tags: ["private"],
    rating: 0,
    wikitext: "Private page body marker.",
    compiled_body_html: "<p>Private page body marker.</p>"
  },
  "xmlrpc-post-page": {
    page_id: 3000300,
    revision_id: 9000300,
    page_created_at: "2026-06-20T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-06-20T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "XML-RPC Post Page",
    slug: "xmlrpc-post-page",
    tags: ["fixture"],
    rating: 5,
    wikitext: "XML-RPC post fixture page.",
    compiled_body_html: "<p>XML-RPC post fixture page.</p>"
  },
  "theme:yossistyle": {
    page_id: 3000310,
    revision_id: 9000310,
    page_created_at: "2026-07-13T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-13T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "YOSSISTYLE",
    slug: "theme:yossistyle",
    tags: ["theme"],
    rating: 0,
    wikitext:
      "[[module CSS]]\n#header h2 span { margin-left: 1px; }\n[[/module]]\nXML-RPC theme body marker.",
    compiled_body_html: "<p>XML-RPC theme body marker.</p>",
    compiled_body_styles: ["#header h2 span { margin-left: 1px; }"]
  },
  "wikidot-tabview": {
    page_id: 3000320,
    revision_id: 9000320,
    page_created_at: "2026-07-13T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-13T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Tabview",
    slug: "wikidot-tabview",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      "[[tabview]]\n[[tab First]]First panel[[/tab]]\n[[tab Second]]Second panel[[/tab]]\n[[/tabview]]",
    compiled_body_html:
      '<div class="yui-navset"><ul class="yui-nav"><li class="selected"><a href="javascript:;">First</a></li><li><a href="javascript:;">Second</a></li></ul><div class="yui-content"><div style="display: block;"><p>First panel</p></div><div style="display:none"><p>Second panel</p></div></div></div><script type="text/javascript"></script>'
  },
  "wikidot-collapsible": {
    page_id: 3000330,
    revision_id: 9000330,
    page_created_at: "2026-07-22T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-22T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Collapsible",
    slug: "wikidot-collapsible",
    tags: ["fixture"],
    rating: 0,
    wikitext:
      '[[collapsible show="+ Show" hide="- Hide" hideLocation="both"]]Folded body[[/collapsible]]\n[[collapsible folded="no" show="+ Open" hide="- Close"]]Open body[[/collapsible]]',
    compiled_body_html:
      '<div id="folded-collapsible" class="collapsible-block"><div class="collapsible-block-folded"><a class="collapsible-block-link" href="javascript:;">+&nbsp;Show</a></div><div class="collapsible-block-unfolded" style="display:none"><div class="collapsible-block-unfolded-link"><a class="collapsible-block-link" href="javascript:;">-&nbsp;Hide</a></div><div class="collapsible-block-content"><p>Folded body</p></div><div class="collapsible-block-unfolded-link"><a class="collapsible-block-link" href="javascript:;">-&nbsp;Hide</a></div></div></div><div id="open-collapsible" class="collapsible-block"><div class="collapsible-block-folded" style="display:none"><a class="collapsible-block-link" href="javascript:;">+&nbsp;Open</a></div><div class="collapsible-block-unfolded"><div class="collapsible-block-unfolded-link"><a class="collapsible-block-link" href="javascript:;">-&nbsp;Close</a></div><div class="collapsible-block-content"><p>Open body</p></div></div></div><details id="native-collapsible"><summary>Native summary</summary><p>Native body</p></details>'
  },
  "wikidot-code-highlighting": {
    page_id: 3000350,
    revision_id: 9000350,
    page_created_at: "2026-07-23T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-23T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Wikidot Code Highlighting",
    slug: "wikidot-code-highlighting",
    tags: ["fixture"],
    rating: 0,
    wikitext: '[[code type="css"]]\n#header h2 span { color: red; }\n[[/code]]',
    compiled_body_html:
      '<div class="code" data-wj-language="css"><pre><code>#header h2 span { color: red; }</code></pre></div>'
  },
  "page-workflow-probe": {
    page_id: 3000340,
    revision_id: 9000340,
    page_created_at: "2026-07-23T00:00:00Z",
    page_updated_at: null,
    page_revision_count: 1,
    revision_created_at: "2026-07-23T00:00:00Z",
    revision_user_id: 123,
    creator_user_id: 123,
    title: "Page Workflow Probe",
    slug: "page-workflow-probe",
    tags: ["fixture"],
    rating: 0,
    wikitext: "Page workflow probe",
    compiled_body_html: "<p>Page workflow probe</p>"
  }
}

/** @param {FixturePage} page */
export const toArticleViewResult = (page) => ({
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
  user_session: null,
  article_page_cache_key: `deepwell:article-view:page:v1:site=6000005:page=${page.page_id}:rev=${page.revision_id}:updated=0:permission=site=0,user=0:body=fixture`,
  public_content_cache_fence: "0",
  anonymous_permission_cache_fence: "site=0,user=0",
  page: {
    type: "found",
    data: {
      options: {
        edit: false,
        title: null,
        parent: null,
        tags: null,
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
      wikitext: page.wikitext,
      compiled_body_html: page.compiled_body_html,
      compiled_body_styles: page.compiled_body_styles ?? [],
      compiled_top_bar_html: null,
      compiled_side_bar_html: null,
      page: {
        page_id: page.page_id,
        created_at: page.page_created_at,
        updated_at: page.page_updated_at,
        deleted_at: null,
        from_wikidot: false,
        site_id: 6000005,
        latest_revision_id: page.revision_id,
        page_category_id: 1,
        slug: page.slug,
        discussion_thread_id: null,
        layout: "wikidot"
      },
      page_revision: {
        revision_id: page.revision_id,
        revision_type: "create",
        created_at: page.revision_created_at,
        updated_at: null,
        revision_number: page.page_revision_count - 1,
        page_id: page.page_id,
        site_id: 6000005,
        user_id: page.revision_user_id,
        from_wikidot: false,
        changes: [],
        wikitext_hash: [],
        compiled_body_html_hash: [],
        compiled_top_bar_html_hash: null,
        compiled_side_bar_html_hash: null,
        compiled_at: page.revision_created_at,
        compiled_generator: "fixture",
        comments: "",
        hidden: [],
        title: page.title,
        alt_title: null,
        slug: page.slug,
        tags: page.tags
      },
      wikidot_snapshot: null,
      wikidot_breadcrumbs: [],
      attributions: []
    }
  }
})

/** @type {Record<string, FixtureForumPost[]>} */
export const forumPostsByPage = {
  "xmlrpc-post-page": [
    {
      id: 7000300,
      reply_to: null,
      title: "XML-RPC comment proof",
      content: "XML-RPC page comment proof body.",
      html: "<p>XML-RPC page comment proof body.</p>",
      created_by: "administrator",
      created_at: "2026-06-21T00:00:00Z"
    }
  ]
}

/** @type {Record<string, string>} */
export const parentBySlug = {
  "scp-173": "scp-173-parent"
}

/**
 * @param {FixturePage | null} page
 * @param {PageDetails} details
 * @returns {Record<string, unknown> | null}
 */
export const toPageResult = (page, details) => {
  if (!page) return null

  /** @type {Record<string, unknown>} */
  const result = {
    page_created_at: page.page_created_at,
    page_id: page.page_id,
    page_updated_at: page.page_updated_at,
    page_revision_count: page.page_revision_count,
    revision_id: page.revision_id,
    revision_created_at: page.revision_created_at,
    revision_user_id: page.revision_user_id,
    title: page.title,
    slug: page.slug,
    tags: page.tags,
    rating: page.rating
  }

  if (details.wikitext) result.wikitext = page.wikitext
  if (details.compiled_html) {
    result.compiled_body_html = page.compiled_body_html
    result.compiled_body_styles = page.compiled_body_styles ?? []
  }
  return result
}
