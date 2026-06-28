import { createServer } from "node:http"

const PORT = 42747
let lastPageTagsSelectParams = null
let lastPageSelectParams = null
const pageReadRequests = {
  pageGet: [],
  pageGetDirect: [],
  pageRevisionGet: [],
  pageSelect: [],
  parentRelationshipsGet: [],
  siteGet: []
}

const pages = {
  "scp-173": {
    page_id: 3000173,
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

const toPageResult = (page, details) => {
  if (!page) {
    return null
  }

  const result = {
    page_created_at: page.page_created_at,
    page_id: page.page_id,
    page_updated_at: page.page_updated_at,
    page_revision_count: page.page_revision_count,
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
      rpcRequest.params.page_id === pages["scp-173-parent"].page_id &&
      rpcRequest.params.allow_deleted === false &&
      hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
      rpcRequest.params.details.compiled_html === false &&
      rpcRequest.params.details.wikitext === false
    ) {
      pageReadRequests.pageGetDirect.push(rpcRequest.params)
      result = toPageResult(pages["scp-173-parent"], rpcRequest.params.details)
    } else if (
      rpcRequest.method === "page_revision_get" &&
      hasExactKeys(rpcRequest.params, [
        "details",
        "page_id",
        "revision_number",
        "site_id"
      ]) &&
      rpcRequest.params.site_id === 6000005 &&
      rpcRequest.params.page_id === pages["scp-173"].page_id &&
      rpcRequest.params.revision_number === 0 &&
      hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
      rpcRequest.params.details.compiled_html === false &&
      rpcRequest.params.details.wikitext === false
    ) {
      pageReadRequests.pageRevisionGet.push(rpcRequest.params)
      result = {
        revision_number: 0,
        user_id: pages["scp-173"].creator_user_id
      }
    } else if (
      rpcRequest.method === "parent_relationships_get" &&
      hasExactKeys(rpcRequest.params, ["page", "relationship_type", "site_id"]) &&
      rpcRequest.params.site_id === 6000005 &&
      typeof rpcRequest.params.page === "string" &&
      rpcRequest.params.relationship_type === "parents"
    ) {
      pageReadRequests.parentRelationshipsGet.push(rpcRequest.params)
      result =
        rpcRequest.params.page === "scp-173"
          ? [
              {
                child_page_id: pages["scp-173"].page_id,
                parent_page_id: pages["scp-173-parent"].page_id
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
      rpcRequest.method === "page_tags_select" &&
      rpcRequest.params?.site === "scp-wiki" &&
      (rpcRequest.params.categories === undefined ||
        rpcRequest.params.categories === null ||
        (Array.isArray(rpcRequest.params.categories) &&
          rpcRequest.params.categories.length <= 100 &&
          rpcRequest.params.categories.every(
            (category) => typeof category === "string"
          ))) &&
      (rpcRequest.params.pages === undefined ||
        rpcRequest.params.pages === null ||
        (Array.isArray(rpcRequest.params.pages) &&
          rpcRequest.params.pages.length <= 100 &&
          rpcRequest.params.pages.every((page) => typeof page === "string")))
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

const hasExactKeys = (value, keys) => {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    Object.keys(value).sort().join("\n") === keys.slice().sort().join("\n")
  )
}

const resetPageReadRequests = () => {
  for (const requests of Object.values(pageReadRequests)) {
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
