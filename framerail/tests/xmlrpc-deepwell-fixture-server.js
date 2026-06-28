import { createServer } from "node:http"

const PORT = 42747
let lastPageTagsSelectParams = null

const server = createServer((request, response) => {
  if (request.method === "GET" && request.url === "/last-page-tags-request") {
    response
      .writeHead(200, { "content-type": "application/json" })
      .end(JSON.stringify(lastPageTagsSelectParams))
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

server.listen(PORT, "127.0.0.1", () => {
  console.log(`XML-RPC Deepwell fixture listening on 127.0.0.1:${PORT}`)
})

process.on("SIGTERM", () => {
  server.close(() => process.exit(0))
})

process.on("SIGINT", () => {
  server.close(() => process.exit(0))
})
