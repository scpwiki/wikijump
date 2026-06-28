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
 *   headers: Record<string, string | string[] | undefined>
 *   params: unknown
 * }} RecordedRpcRequest
 */

const PORT = 42747
/** @type {RpcParams | null} */
let lastPageTagsSelectParams = null
/** @type {RpcParams | null} */
let lastPageSelectParams = null
/** @type {Record<string, unknown[]>} */
const pageReadRequests = {
  pageGet: [],
  pageGetDirect: [],
  pageRevisionGet: [],
  pageSelect: [],
  parentRelationshipsGet: [],
  siteGet: []
}
/** @type {Record<string, RecordedRpcRequest[]>} */
const pageWriteRequests = {
  login: [],
  pageCreate: [],
  pageEdit: [],
  pageMove: [],
  parentGetAll: [],
  parentUpdate: [],
  sessionGet: []
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
  }
}
/** @type {Record<string, string>} */
const parentBySlug = {
  "scp-173": "scp-173-parent"
}
let nextPageId = 4000000
let nextRevisionId = 9100000

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
  }

  return result
}

const server = createServer((request, response) => {
  if (request.method === "GET" && request.url === "/last-page-tags-request") {
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(lastPageTagsSelectParams))
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
  if (request.method === "GET" && request.url === "/last-page-write-requests") {
    const snapshot = structuredClone(pageWriteRequests)
    resetPageWriteRequests()
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
      rpcRequest.params.name_or_email === "admin@wikijump" &&
      rpcRequest.params.password === "wikijumpadmin1" &&
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
      lastPageTagsSelectParams = rpcRequest.params
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
      typeof rpcRequest.params.page === "string" &&
      pages[rpcRequest.params.page] &&
      typeof rpcRequest.params.last_revision_id === "number" &&
      typeof rpcRequest.params.revision_comments === "string" &&
      typeof rpcRequest.params.user_id === "number" &&
      typeof rpcRequest.params.ip_address === "string" &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
      request.headers["x-deepwell-site-id"] === "6000005" &&
      request.headers["x-deepwell-page"] === rpcRequest.params.page
    ) {
      pageWriteRequests.pageEdit.push({
        headers: requestContextHeaders(request),
        params: rpcRequest.params
      })
      const page = pages[rpcRequest.params.page]
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
      rpcRequest.method === "parent_get_all" &&
      hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "string"
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
    } else {
      response.writeHead(200, { "content-type": "application/json" }).end(
        JSON.stringify({
          error: {
            code: -32601,
            message: `Unexpected Deepwell fixture request: ${rpcRequest.method}`
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

const resetPageWriteRequests = () => {
  for (const requests of Object.values(pageWriteRequests)) {
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
