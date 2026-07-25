import { createServer } from "node:http"

/**
 * @typedef {Record<string, any>} RpcParams
 *
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
 *   content: Buffer
 *   file_created_at: string
 *   file_id: number
 *   file_updated_at: string | null
 *   mime: string
 *   name: string
 *   revision_comments: string
 *   revision_created_at: string
 *   revision_id: number
 *   revision_user_id: number
 *   size: number
 * }} FixtureFile
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
 *
 *
 * @typedef {{
 *   headers: Record<string, string | string[] | undefined>
 *   params: unknown
 * }} RecordedRpcRequest
 */

const PORT = Number(process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747")
/** @type {RecordedRpcRequest | null} */
let lastPageTagsSelectRequest = null
/** @type {RpcParams | null} */
let lastPageSelectParams = null
/** @type {Record<string, unknown[]>} */
const pageReadRequests = {
  forumPostPageSummary: [],
  pageGet: [],
  pageGetDirect: [],
  pageRevisionGet: [],
  pageView: [],
  pageSelect: [],
  parentRelationshipsGet: [],
  siteGet: []
}
const articleReadRequests = {
  articleView: [],
  articleViewCacheMetadata: []
}
/** @type {Record<string, RecordedRpcRequest[]>} */
const pageWriteRequests = {
  login: [],
  pageCreate: [],
  pageEdit: [],
  pageRollback: [],
  pageMove: [],
  parentGetAll: [],
  parentUpdate: [],
  sessionGet: [],
  userGet: [],
  voteSet: []
}
/** @type {Record<string, RecordedRpcRequest[]>} */
const fileRequests = {
  blobUpload: [],
  fileCreate: [],
  fileEdit: [],
  fileGet: [],
  fileRestore: [],
  pageGetFiles: []
}
/** @type {Record<string, Buffer>} */
const pendingUploads = {}
/** @type {Record<number, Record<string, FixtureFile>>} */
const filesByPageId = {}

const MIN_I64 = -(1n << 63n)
const MAX_I64 = (1n << 63n) - 1n

/** @type {(value: string) => boolean} */
const isSignedI64String = (value) => {
  if (!/^-?\d+$/.test(value)) {
    return false
  }
  const parsed = BigInt(value)
  return parsed >= MIN_I64 && parsed <= MAX_I64
}

/** @type {Record<string, FixturePage>} */
const pages = {
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
const toArticleViewResult = (page) => ({
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
const forumPostsByPage = {
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
const parentBySlug = {
  "scp-173": "scp-173-parent"
}
let nextPageId = 4000000
let nextRevisionId = 9100000
let nextFileId = 5000000
let nextPendingBlobId = 1

/**
 * @param {FixturePage | null} page
 * @param {PageDetails} details
 * @returns {Record<string, unknown> | null}
 */
const toPageResult = (page, details) => {
  if (!page) {
    return null
  }

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

  if (details.wikitext) {
    result.wikitext = page.wikitext
  }
  if (details.compiled_html) {
    result.compiled_body_html = page.compiled_body_html
    result.compiled_body_styles = page.compiled_body_styles ?? []
  }

  return result
}

const server = createServer((request, response) => {
  if (request.method === "PUT" && request.url?.startsWith("/upload/")) {
    const pendingBlobId = decodeURIComponent(request.url.slice("/upload/".length))
    const chunks = []
    request.on("data", (chunk) => {
      chunks.push(Buffer.from(chunk))
    })
    request.on("end", () => {
      if (request.headers.host !== `127.0.0.1:${PORT}`) {
        response.writeHead(400).end("Unexpected signed upload Host")
        return
      }
      pendingUploads[pendingBlobId] = Buffer.concat(chunks)
      response.writeHead(200).end()
    })
    return
  }

  if (request.method === "GET" && request.url === "/last-page-tags-request") {
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(lastPageTagsSelectRequest))
    return
  }
  if (request.method === "GET" && request.url === "/last-page-select-request") {
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(lastPageSelectParams))
    return
  }
  if (request.method === "GET" && request.url === "/last-page-read-requests") {
    const snapshot = structuredClone(pageReadRequests)
    resetPageReadRequests()
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(snapshot))
    return
  }
  if (request.method === "GET" && request.url === "/last-article-read-requests") {
    const snapshot = structuredClone(articleReadRequests)
    resetArticleReadRequests()
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(snapshot))
    return
  }
  if (request.method === "GET" && request.url === "/last-page-write-requests") {
    const snapshot = structuredClone(pageWriteRequests)
    resetPageWriteRequests()
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(snapshot))
    return
  }
  if (request.method === "GET" && request.url === "/last-file-requests") {
    const snapshot = structuredClone(fileRequests)
    resetFileRequests()
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(snapshot))
    return
  }

  if (request.method !== "POST" || request.url !== "/jsonrpc") {
    response.writeHead(404).end()
    return
  }

  let body = ""
  request.setEncoding("utf8")
  request.on("data", (chunk) => {
    body += chunk
  })
  request.on("end", () => {
    /** @type {any} */
    let rpcRequest
    try {
      rpcRequest = JSON.parse(body)
    } catch {
      response.writeHead(200, { "content-type": "application/json" }).end(
        JSON.stringify({
          error: {
            code: -32700,
            message: "Parse error"
          },
          id: null,
          jsonrpc: "2.0"
        })
      )
      return
    }

    if (
      typeof rpcRequest !== "object" ||
      rpcRequest === null ||
      Array.isArray(rpcRequest) ||
      typeof rpcRequest.method !== "string"
    ) {
      response.writeHead(200, { "content-type": "application/json" }).end(
        JSON.stringify({
          error: {
            code: -32600,
            message: "Invalid Request"
          },
          id: null,
          jsonrpc: "2.0"
        })
      )
      return
    }

    let result

    if (
      rpcRequest.method === "login" &&
      hasExactKeys(rpcRequest.params, [
        "ip_address",
        "name_or_email",
        "password",
        "user_agent"
      ]) &&
      rpcRequest.params.name_or_email === process.env.XML_RPC_WRITE_USERNAME &&
      rpcRequest.params.password === process.env.XML_RPC_WRITE_PASSWORD &&
      typeof rpcRequest.params.ip_address === "string" &&
      rpcRequest.params.user_agent === "wikijump-xmlrpc-api/0.1"
    ) {
      pageWriteRequests.login.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      result = { needs_mfa: false, session_token: "fixture-session-token" }
    } else if (
      rpcRequest.method === "session_get" &&
      Array.isArray(rpcRequest.params) &&
      rpcRequest.params.length === 1 &&
      rpcRequest.params[0] === "fixture-session-token"
    ) {
      pageWriteRequests.sessionGet.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      result = { user_id: 123 }
    } else if (
      rpcRequest.method === "user_get" &&
      hasExactKeys(rpcRequest.params, ["user"]) &&
      ((rpcRequest.params.user === 123 &&
        request.headers["x-deepwell-session-token"] === "fixture-session-token") ||
        rpcRequest.params.user === "rokurokubi")
    ) {
      pageWriteRequests.userGet.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      result = {
        aliases: [],
        user_id: 123,
        name: "Rokurokubi",
        slug: "rokurokubi"
      }
    } else if (
      rpcRequest.method === "category_get_all" &&
      rpcRequest.params?.site === "scp-wiki"
    ) {
      result = [{ slug: "_default" }, { slug: "nav" }]
    } else if (
      rpcRequest.method === "site_get" &&
      hasExactKeys(rpcRequest.params, ["site"]) &&
      (rpcRequest.params.site === "scp-wiki" || rpcRequest.params.site === "missing-site")
    ) {
      pageReadRequests.siteGet.push(rpcRequest.params)
      result = rpcRequest.params.site === "scp-wiki" ? { site_id: 6000005 } : null
    } else if (
      rpcRequest.method === "article_view" &&
      ((hasExactKeys(rpcRequest.params, ["locales", "route", "site_id"]) &&
        rpcRequest.params.session_token === undefined) ||
        (hasExactKeys(rpcRequest.params, [
          "locales",
          "route",
          "session_token",
          "site_id"
        ]) &&
          rpcRequest.params.session_token === "fixture-session-token")) &&
      rpcRequest.params.site_id === 6000005 &&
      Array.isArray(rpcRequest.params.locales) &&
      hasExactKeys(rpcRequest.params.route, ["extra", "slug"]) &&
      typeof rpcRequest.params.route.slug === "string" &&
      rpcRequest.params.route.extra === "" &&
      pages[rpcRequest.params.route.slug]
    ) {
      articleReadRequests.articleView.push(rpcRequest.params)
      result = toArticleViewResult(pages[rpcRequest.params.route.slug])
    } else if (
      rpcRequest.method === "article_view_cache_metadata" &&
      hasExactKeys(rpcRequest.params, ["locales", "route", "session_token", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      rpcRequest.params.session_token === null &&
      Array.isArray(rpcRequest.params.locales) &&
      hasExactKeys(rpcRequest.params.route, ["extra", "slug"]) &&
      typeof rpcRequest.params.route.slug === "string" &&
      rpcRequest.params.route.extra === "" &&
      pages[rpcRequest.params.route.slug]
    ) {
      articleReadRequests.articleViewCacheMetadata.push(rpcRequest.params)
      const page = pages[rpcRequest.params.route.slug]
      result = {
        article_page_cache_key: `deepwell:article-view:page:v1:site=6000005:page=${page.page_id}:rev=${page.revision_id}:updated=0:permission=site=0,user=0:body=fixture`,
        public_content_cache_fence: "0",
        anonymous_permission_cache_fence: "site=0,user=0"
      }
    } else if (
      rpcRequest.method === "translate" &&
      hasExactKeys(rpcRequest.params, ["locales", "messages", "strip_message_keys"]) &&
      Array.isArray(rpcRequest.params.locales) &&
      typeof rpcRequest.params.messages === "object" &&
      rpcRequest.params.messages !== null &&
      Array.isArray(rpcRequest.params.strip_message_keys)
    ) {
      result = Object.fromEntries(
        Object.keys(rpcRequest.params.messages).map((key) => [key, key])
      )
    } else if (
      rpcRequest.method === "page_view" &&
      hasExactKeys(rpcRequest.params, ["locales", "route", "session_token", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      Array.isArray(rpcRequest.params.locales) &&
      rpcRequest.params.session_token === "fixture-session-token" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
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
          : {
              type: "found",
              data: {
                page: { slug: page.slug }
              }
            }
        : { type: "missing", data: {} }
    } else if (
      rpcRequest.method === "page_get" &&
      hasExactKeys(rpcRequest.params, ["details", "page", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "string" &&
      hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
      typeof rpcRequest.params.details.compiled_html === "boolean" &&
      typeof rpcRequest.params.details.wikitext === "boolean"
    ) {
      pageReadRequests.pageGet.push(rpcRequest.params)
      const page = pages[rpcRequest.params.page] ?? null
      result = toPageResult(page, rpcRequest.params.details)
    } else if (
      rpcRequest.method === "page_get_direct" &&
      hasExactKeys(rpcRequest.params, [
        "allow_deleted",
        "details",
        "page_id",
        "site_id"
      ]) &&
      rpcRequest.params.site_id === 6000005 &&
      pageById(rpcRequest.params.page_id) &&
      rpcRequest.params.allow_deleted === false &&
      hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
      rpcRequest.params.details.compiled_html === false &&
      rpcRequest.params.details.wikitext === false
    ) {
      pageReadRequests.pageGetDirect.push(rpcRequest.params)
      const page = pageById(rpcRequest.params.page_id)
      result = toPageResult(page, rpcRequest.params.details)
    } else if (
      rpcRequest.method === "page_revision_get" &&
      hasExactKeys(rpcRequest.params, [
        "details",
        "page_id",
        "revision_number",
        "site_id"
      ]) &&
      rpcRequest.params.site_id === 6000005 &&
      pageById(rpcRequest.params.page_id) &&
      rpcRequest.params.revision_number === 0 &&
      hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
      rpcRequest.params.details.compiled_html === false &&
      rpcRequest.params.details.wikitext === false
    ) {
      pageReadRequests.pageRevisionGet.push(rpcRequest.params)
      const page = pageById(rpcRequest.params.page_id)
      result = {
        revision_number: 0,
        user_id: page?.creator_user_id
      }
    } else if (
      rpcRequest.method === "parent_relationships_get" &&
      hasExactKeys(rpcRequest.params, ["page", "relationship_type", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "string" &&
      rpcRequest.params.relationship_type === "parents"
    ) {
      pageReadRequests.parentRelationshipsGet.push(rpcRequest.params)
      const parentSlug = parentBySlug[rpcRequest.params.page]
      const child = pages[rpcRequest.params.page]
      const parent = parentSlug ? pages[parentSlug] : null
      result =
        child && parent
          ? [
              {
                child_page_id: child.page_id,
                parent_page_id: parent.page_id
              }
            ]
          : []
    } else if (
      rpcRequest.method === "forum_post_page_summary" &&
      hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "string"
    ) {
      pageReadRequests.forumPostPageSummary.push(rpcRequest.params)
      const posts = forumPostsByPage[rpcRequest.params.page] ?? []
      const latest = posts.at(-1)
      result = {
        comments: posts.length,
        commented_at: latest?.created_at ?? null,
        commented_by: latest?.created_by ?? null
      }
    } else if (
      rpcRequest.method === "page_select" &&
      hasExactKeys(rpcRequest.params, ["parent", "site"]) &&
      rpcRequest.params.site === "scp-wiki" &&
      rpcRequest.params.parent === "scp-173"
    ) {
      pageReadRequests.pageSelect.push(rpcRequest.params)
      result = ["scp-173-child-a", "scp-173-child-b"]
    } else if (
      rpcRequest.method === "page_select" &&
      hasExactKeys(rpcRequest.params, ["parent", "site"]) &&
      rpcRequest.params.site === "scp-wiki" &&
      typeof rpcRequest.params.parent === "string"
    ) {
      pageReadRequests.pageSelect.push(rpcRequest.params)
      result = []
    } else if (
      rpcRequest.method === "page_tags_select" &&
      rpcRequest.params?.site === "scp-wiki" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      (rpcRequest.params.categories === undefined ||
        rpcRequest.params.categories === null ||
        (Array.isArray(rpcRequest.params.categories) &&
          rpcRequest.params.categories.length <= 100 &&
          rpcRequest.params.categories.every(
            /** @param {unknown} category */
            (category) => typeof category === "string"
          ))) &&
      (rpcRequest.params.pages === undefined ||
        rpcRequest.params.pages === null ||
        (Array.isArray(rpcRequest.params.pages) &&
          rpcRequest.params.pages.length <= 100 &&
          rpcRequest.params.pages.every(
            /** @param {unknown} page */
            (page) => typeof page === "string"
          )))
    ) {
      lastPageTagsSelectRequest = {
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      }
      result = ["_cc", "tale"]
    } else if (
      rpcRequest.method === "page_select" &&
      rpcRequest.params?.site === "scp-wiki" &&
      rpcRequest.params?.pagetype === "normal" &&
      Array.isArray(rpcRequest.params.categories) &&
      rpcRequest.params.categories.length === 1 &&
      rpcRequest.params.categories[0] === "_default" &&
      rpcRequest.params?.created_by === "-1" &&
      rpcRequest.params?.rating === ">=0" &&
      rpcRequest.params?.order === "created_at desc"
    ) {
      lastPageSelectParams = rpcRequest.params
      result = ["scp-173", "scp-anthology-2024", "scp-8566"]
    } else if (
      rpcRequest.method === "forum_post_select" &&
      rpcRequest.params?.site_id === 6000005 &&
      (rpcRequest.params.page === undefined ||
        typeof rpcRequest.params.page === "string") &&
      (rpcRequest.params.reply_to === undefined ||
        typeof rpcRequest.params.reply_to === "string") &&
      (rpcRequest.params.created_by === undefined ||
        typeof rpcRequest.params.created_by === "string")
    ) {
      if (
        rpcRequest.params.reply_to !== undefined &&
        rpcRequest.params.reply_to !== "-" &&
        !isSignedI64String(rpcRequest.params.reply_to)
      ) {
        response.writeHead(200, { "content-type": "application/json" }).end(
          JSON.stringify({
            error: {
              code: -32602,
              message: "Unexpected fixture forum_post_select params"
            },
            id: rpcRequest.id,
            jsonrpc: "2.0"
          })
        )
        return
      }

      const posts =
        rpcRequest.params.page === undefined
          ? Object.values(forumPostsByPage).flat()
          : (forumPostsByPage[rpcRequest.params.page] ?? [])
      result = posts
        .filter((post) => {
          if (rpcRequest.params.reply_to === undefined) {
            return true
          }
          if (rpcRequest.params.reply_to === "-") {
            return post.reply_to === null
          }
          return String(post.reply_to) === rpcRequest.params.reply_to
        })
        .filter(
          (post) =>
            rpcRequest.params.created_by === undefined ||
            post.created_by === rpcRequest.params.created_by
        )
        .map((post) => post.id)
    } else if (
      rpcRequest.method === "forum_post_get" &&
      rpcRequest.params?.site_id === 6000005 &&
      Array.isArray(rpcRequest.params.posts) &&
      rpcRequest.params.posts.every(
        /** @param {unknown} post */
        (post) => typeof post === "string"
      )
    ) {
      if (
        rpcRequest.params.posts.length > 10 ||
        rpcRequest.params.posts.some((post) => !isSignedI64String(post))
      ) {
        response.writeHead(200, { "content-type": "application/json" }).end(
          JSON.stringify({
            error: {
              code: -32602,
              message: "Unexpected fixture forum_post_get params"
            },
            id: rpcRequest.id,
            jsonrpc: "2.0"
          })
        )
        return
      }

      const postsById = new Map(
        Object.entries(forumPostsByPage).flatMap(([page, posts]) =>
          posts.map((post) => [String(post.id), { ...post, fullname: page }])
        )
      )
      result = rpcRequest.params.posts.flatMap((post) => {
        const fixturePost = postsById.get(post)
        return fixturePost === undefined ? [] : [fixturePost]
      })
    } else if (
      rpcRequest.method === "page_create" &&
      hasExactKeys(rpcRequest.params, [
        "alt_title",
        "ip_address",
        "layout",
        "revision_comments",
        "site_id",
        "slug",
        "title",
        "user_id",
        "wikitext"
      ]) &&
      rpcRequest.params.site_id === 6000005 &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      typeof rpcRequest.params.slug === "string"
    ) {
      pageWriteRequests.pageCreate.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const slug = rpcRequest.params.slug
      const revisionId = nextRevisionId++
      pages[slug] = {
        page_id: nextPageId++,
        revision_id: revisionId,
        page_created_at: "2026-06-29T00:00:00Z",
        page_updated_at: null,
        page_revision_count: 1,
        revision_created_at: "2026-06-29T00:00:00Z",
        revision_user_id: rpcRequest.params.user_id,
        creator_user_id: rpcRequest.params.user_id,
        title: rpcRequest.params.title,
        slug,
        tags: [],
        rating: 0,
        wikitext: rpcRequest.params.wikitext,
        compiled_body_html: `<p>${rpcRequest.params.wikitext}</p>`
      }
      result = { page_id: pages[slug].page_id, revision_id: revisionId, slug }
    } else if (
      rpcRequest.method === "page_edit" &&
      rpcRequest.params.site_id === 6000005 &&
      ((typeof rpcRequest.params.page === "string" && pages[rpcRequest.params.page]) ||
        (typeof rpcRequest.params.page === "number" &&
          pageById(rpcRequest.params.page))) &&
      typeof rpcRequest.params.last_revision_id === "number" &&
      typeof rpcRequest.params.revision_comments === "string" &&
      typeof rpcRequest.params.user_id === "number" &&
      typeof rpcRequest.params.ip_address === "string" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] ===
        (typeof rpcRequest.params.page === "string"
          ? rpcRequest.params.page
          : pageById(rpcRequest.params.page)?.slug)
    ) {
      pageWriteRequests.pageEdit.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const page =
        typeof rpcRequest.params.page === "string"
          ? pages[rpcRequest.params.page]
          : pageById(rpcRequest.params.page)
      if (typeof rpcRequest.params.wikitext === "string") {
        page.wikitext = rpcRequest.params.wikitext
        page.compiled_body_html = `<p>${rpcRequest.params.wikitext}</p>`
      }
      if (typeof rpcRequest.params.title === "string") {
        page.title = rpcRequest.params.title
      }
      if (Array.isArray(rpcRequest.params.tags)) {
        page.tags = rpcRequest.params.tags
      }
      page.page_updated_at = "2026-06-29T00:01:00Z"
      page.revision_created_at = "2026-06-29T00:01:00Z"
      page.revision_user_id = rpcRequest.params.user_id
      page.revision_id = nextRevisionId++
      page.page_revision_count += 1
      result = {
        revision_id: page.revision_id,
        revision_number: page.page_revision_count - 1
      }
    } else if (
      rpcRequest.method === "page_rollback" &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "number" &&
      pageById(rpcRequest.params.page) &&
      typeof rpcRequest.params.last_revision_id === "number" &&
      typeof rpcRequest.params.revision_number === "number" &&
      typeof rpcRequest.params.revision_comments === "string" &&
      rpcRequest.params.user_id === 123 &&
      typeof rpcRequest.params.ip_address === "string" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page)?.slug
    ) {
      pageWriteRequests.pageRollback.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const page = pageById(rpcRequest.params.page)
      page.revision_id = nextRevisionId++
      page.page_revision_count += 1
      result = {
        revision_id: page.revision_id,
        revision_number: page.page_revision_count - 1
      }
    } else if (
      rpcRequest.method === "vote_set" &&
      hasExactKeys(rpcRequest.params, ["page_id", "user_id", "value"]) &&
      pageById(rpcRequest.params.page_id) &&
      rpcRequest.params.user_id === 123 &&
      (rpcRequest.params.value === -1 || rpcRequest.params.value === 1) &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page_id)?.slug
    ) {
      pageWriteRequests.voteSet.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      result = {
        page_vote_id: 7000001,
        page_id: rpcRequest.params.page_id,
        user_id: rpcRequest.params.user_id,
        value: rpcRequest.params.value
      }
    } else if (
      rpcRequest.method === "parent_get_all" &&
      hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "string" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === rpcRequest.params.page
    ) {
      pageWriteRequests.parentGetAll.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      result = parentBySlug[rpcRequest.params.page]
        ? [parentBySlug[rpcRequest.params.page]]
        : []
    } else if (
      rpcRequest.method === "parent_update" &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.child === "string" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === rpcRequest.params.child
    ) {
      pageWriteRequests.parentUpdate.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      if (Array.isArray(rpcRequest.params.add) && rpcRequest.params.add.length > 0) {
        parentBySlug[rpcRequest.params.child] = rpcRequest.params.add[0]
      }
      if (Array.isArray(rpcRequest.params.remove)) {
        for (const parent of rpcRequest.params.remove) {
          if (parentBySlug[rpcRequest.params.child] === parent) {
            delete parentBySlug[rpcRequest.params.child]
          }
        }
      }
      result = { added: [], removed: [] }
    } else if (
      rpcRequest.method === "page_move" &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "string" &&
      pages[rpcRequest.params.page] &&
      typeof rpcRequest.params.new_slug === "string" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === rpcRequest.params.page
    ) {
      pageWriteRequests.pageMove.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const page = pages[rpcRequest.params.page]
      delete pages[rpcRequest.params.page]
      page.slug = rpcRequest.params.new_slug
      page.revision_id = nextRevisionId++
      page.page_revision_count += 1
      pages[page.slug] = page
      if (parentBySlug[rpcRequest.params.page]) {
        parentBySlug[page.slug] = parentBySlug[rpcRequest.params.page]
        delete parentBySlug[rpcRequest.params.page]
      }
      result = { new_slug: page.slug, old_slug: rpcRequest.params.page }
    } else if (
      rpcRequest.method === "page_get_files" &&
      hasExactKeys(rpcRequest.params, ["deleted", "page_id", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page_id === "number" &&
      rpcRequest.params.deleted === false
    ) {
      fileRequests.pageGetFiles.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      result = Object.values(filesByPageId[rpcRequest.params.page_id] ?? {}).map(
        toFileResultWithoutData
      )
    } else if (
      rpcRequest.method === "file_get" &&
      hasExactKeys(rpcRequest.params, ["details", "file", "page_id", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page_id === "number" &&
      typeof rpcRequest.params.file === "string" &&
      hasExactKeys(rpcRequest.params.details, ["data"]) &&
      typeof rpcRequest.params.details.data === "boolean"
    ) {
      fileRequests.fileGet.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const file =
        filesByPageId[rpcRequest.params.page_id]?.[rpcRequest.params.file] ?? null
      result = file ? toFileResult(file, Boolean(rpcRequest.params.details.data)) : null
    } else if (
      rpcRequest.method === "blob_upload" &&
      hasExactKeys(rpcRequest.params, ["blob_size", "user_id"]) &&
      rpcRequest.params.user_id === 123 &&
      typeof rpcRequest.params.blob_size === "number" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005"
    ) {
      fileRequests.blobUpload.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const pendingBlobId = `fixture-blob-${nextPendingBlobId++}`
      result = {
        pending_blob_id: pendingBlobId,
        presign_url: `http://127.0.0.1:${PORT}/upload/${encodeURIComponent(pendingBlobId)}`
      }
    } else if (
      rpcRequest.method === "file_create" &&
      hasExactKeys(rpcRequest.params, [
        "bypass_filter",
        "ip_address",
        "name",
        "page_id",
        "revision_comments",
        "site_id",
        "uploaded_blob_id",
        "user_id"
      ]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page_id === "number" &&
      pageById(rpcRequest.params.page_id) &&
      typeof rpcRequest.params.name === "string" &&
      typeof rpcRequest.params.uploaded_blob_id === "string" &&
      pendingUploads[rpcRequest.params.uploaded_blob_id] &&
      typeof rpcRequest.params.revision_comments === "string" &&
      rpcRequest.params.user_id === 123 &&
      typeof rpcRequest.params.ip_address === "string" &&
      rpcRequest.params.bypass_filter === true &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page_id)?.slug
    ) {
      fileRequests.fileCreate.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const pageFiles = (filesByPageId[rpcRequest.params.page_id] ??= {})
      if (pageFiles[rpcRequest.params.name]) {
        response.writeHead(200, { "content-type": "application/json" }).end(
          JSON.stringify({
            error: {
              code: -32602,
              message: "Unexpected fixture duplicate file_create target"
            },
            id: rpcRequest.id,
            jsonrpc: "2.0"
          })
        )
        return
      }
      const content = pendingUploads[rpcRequest.params.uploaded_blob_id]
      delete pendingUploads[rpcRequest.params.uploaded_blob_id]
      const file = createFixtureFile(
        rpcRequest.params.name,
        content,
        rpcRequest.params.revision_comments,
        rpcRequest.params.user_id,
        false
      )
      pageFiles[file.name] = file
      result = {
        file_id: file.file_id,
        revision_id: file.revision_id
      }
    } else if (
      rpcRequest.method === "file_edit" &&
      hasExactKeys(rpcRequest.params, [
        "bypass_filter",
        "file_id",
        "ip_address",
        "last_revision_id",
        "page_id",
        "revision_comments",
        "site_id",
        "uploaded_blob_id",
        "user_id"
      ]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page_id === "number" &&
      typeof rpcRequest.params.file_id === "number" &&
      typeof rpcRequest.params.last_revision_id === "number" &&
      typeof rpcRequest.params.uploaded_blob_id === "string" &&
      pendingUploads[rpcRequest.params.uploaded_blob_id] &&
      typeof rpcRequest.params.revision_comments === "string" &&
      rpcRequest.params.user_id === 123 &&
      typeof rpcRequest.params.ip_address === "string" &&
      rpcRequest.params.bypass_filter === true &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page_id)?.slug
    ) {
      const pageFiles = filesByPageId[rpcRequest.params.page_id] ?? {}
      const existing = Object.values(pageFiles).find(
        (file) =>
          file.file_id === rpcRequest.params.file_id &&
          file.revision_id === rpcRequest.params.last_revision_id
      )
      if (!existing) {
        response.writeHead(200, { "content-type": "application/json" }).end(
          JSON.stringify({
            error: {
              code: -32602,
              message: "Unexpected fixture file_edit target"
            },
            id: rpcRequest.id,
            jsonrpc: "2.0"
          })
        )
        return
      }
      fileRequests.fileEdit.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const content = pendingUploads[rpcRequest.params.uploaded_blob_id]
      delete pendingUploads[rpcRequest.params.uploaded_blob_id]
      updateFixtureFile(
        existing,
        content,
        rpcRequest.params.revision_comments,
        rpcRequest.params.user_id
      )
      result = {
        file_id: existing.file_id,
        revision_id: existing.revision_id
      }
    } else if (
      rpcRequest.method === "file_restore" &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page_id === "number" &&
      pageById(rpcRequest.params.page_id) &&
      typeof rpcRequest.params.file_id === "number" &&
      typeof rpcRequest.params.revision_comments === "string" &&
      rpcRequest.params.user_id === 123 &&
      typeof rpcRequest.params.ip_address === "string" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page_id)?.slug
    ) {
      fileRequests.fileRestore.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      result = {
        file_id: rpcRequest.params.file_id,
        page_id: rpcRequest.params.page_id,
        revision_id: nextRevisionId++
      }
    } else {
      const requestShape =
        rpcRequest.method === "article_view"
          ? ` ${JSON.stringify({
              paramKeys: Object.keys(rpcRequest.params ?? {}).sort(),
              route: rpcRequest.params?.route,
              sessionTokenType:
                rpcRequest.params?.session_token === null
                  ? "null"
                  : typeof rpcRequest.params?.session_token,
              siteId: rpcRequest.params?.site_id,
              headerSiteId: request.headers["x-deepwell-site-id"],
              hasHeaderSessionToken: Boolean(request.headers["x-deepwell-session-token"])
            })}`
          : ""
      response.writeHead(200, { "content-type": "application/json" }).end(
        JSON.stringify({
          error: {
            code: -32601,
            message: `Unexpected Deepwell fixture request: ${rpcRequest.method}${requestShape}`
          },
          id: rpcRequest.id,
          jsonrpc: "2.0"
        })
      )
      return
    }

    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify({ id: rpcRequest.id, jsonrpc: "2.0", result }))
  })
})

/**
 * @param {unknown} value
 * @param {string[]} keys
 */
const hasExactKeys = (value, keys) => {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    Object.keys(value).sort().join("\n") === keys.slice().sort().join("\n")
  )
}

/**
 * @param {number} pageId
 * @returns {FixturePage | null}
 */
const pageById = (pageId) =>
  Object.values(pages).find((page) => page.page_id === pageId) ?? null

/**
 * @param {FixtureFile} file
 * @returns {Record<string, unknown>}
 */
const toFileResultWithoutData = (file) => toFileResult(file, false)

/**
 * @param {FixtureFile} file
 * @param {boolean} includeData
 * @returns {Record<string, unknown>}
 */
const toFileResult = (file, includeData) => {
  /** @type {Record<string, unknown>} */
  const result = {
    file_id: file.file_id,
    file_created_at: file.file_created_at,
    file_updated_at: file.file_updated_at,
    revision_id: file.revision_id,
    revision_created_at: file.revision_created_at,
    revision_user_id: file.revision_user_id,
    name: file.name,
    mime: file.mime,
    size: file.size,
    revision_comments: file.revision_comments
  }
  if (includeData) {
    result.data = Array.from(file.content)
  }
  return result
}

/**
 * @param {string} name
 * @param {Buffer} content
 * @param {string} revisionComments
 * @param {number} userId
 * @param {boolean} updated
 * @returns {FixtureFile}
 */
const createFixtureFile = (name, content, revisionComments, userId, updated) => ({
  file_id: nextFileId++,
  file_created_at: "2026-06-29T00:02:00Z",
  file_updated_at: updated ? "2026-06-29T00:03:00Z" : null,
  revision_id: nextRevisionId++,
  revision_created_at: updated ? "2026-06-29T00:03:00Z" : "2026-06-29T00:02:00Z",
  revision_user_id: userId,
  name,
  content,
  mime: "text/plain",
  size: content.length,
  revision_comments: revisionComments
})

/**
 * @param {FixtureFile} file
 * @param {Buffer} content
 * @param {string} revisionComments
 * @param {number} userId
 */
const updateFixtureFile = (file, content, revisionComments, userId) => {
  file.content = content
  file.file_updated_at = "2026-06-29T00:03:00Z"
  file.revision_id = nextRevisionId++
  file.revision_created_at = "2026-06-29T00:03:00Z"
  file.revision_user_id = userId
  file.size = content.length
  file.revision_comments = revisionComments
}

/** @param {import("node:http").IncomingMessage} request */
const requestContextHeaders = (request) => ({
  page: request.headers["x-deepwell-page"],
  sessionToken: request.headers["x-deepwell-session-token"],
  siteId: request.headers["x-deepwell-site-id"]
})

const resetPageReadRequests = () => {
  for (const requests of Object.values(pageReadRequests)) {
    requests.length = 0
  }
}

const resetArticleReadRequests = () => {
  for (const requests of Object.values(articleReadRequests)) {
    requests.length = 0
  }
}

const resetPageWriteRequests = () => {
  for (const requests of Object.values(pageWriteRequests)) {
    requests.length = 0
  }
}

const resetFileRequests = () => {
  for (const requests of Object.values(fileRequests)) {
    requests.length = 0
  }
}

server.listen(PORT, "127.0.0.1", () => {
  console.log(`XML-RPC Deepwell fixture listening on 127.0.0.1:${PORT}`)
})

process.on("SIGTERM", () => {
  server.close(() => process.exit(0))
})

process.on("SIGINT", () => {
  server.close(() => process.exit(0))
})
