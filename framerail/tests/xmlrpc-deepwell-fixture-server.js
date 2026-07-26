import { createServer } from "node:http"

import { fixtureState, resetRequestGroups } from "./xmlrpc-deepwell-fixture/context.js"
import { dispatchFixtureRpc } from "./xmlrpc-deepwell-fixture/dispatch.js"
import { sendRpcError } from "./xmlrpc-deepwell-fixture/response.js"

const PORT = Number(process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747")

/**
 * @param {import("node:http").ServerResponse} response
 * @param {unknown} value
 */
const sendJson = (response, value) => {
  response
    .writeHead(200, { "content-type": "application/json" })
    .end(JSON.stringify(value))
}

/**
 * @param {import("node:http").IncomingMessage} request
 * @param {import("node:http").ServerResponse} response
 */
const handleUpload = (request, response) => {
  if (request.method !== "PUT" || !request.url?.startsWith("/upload/")) return false

  const pendingBlobId = decodeURIComponent(request.url.slice("/upload/".length))
  const chunks = []
  request.on("data", (chunk) => chunks.push(Buffer.from(chunk)))
  request.on("end", () => {
    if (request.headers.host !== `127.0.0.1:${PORT}`) {
      response.writeHead(400).end("Unexpected signed upload Host")
      return
    }
    fixtureState.pendingUploads[pendingBlobId] = Buffer.concat(chunks)
    response.writeHead(200).end()
  })
  return true
}

/**
 * @param {import("node:http").IncomingMessage} request
 * @param {import("node:http").ServerResponse} response
 */
const handleDiagnosticRequest = (request, response) => {
  if (request.method !== "GET") return false

  if (request.url === "/last-page-tags-request") {
    sendJson(response, fixtureState.lastPageTagsSelectRequest)
    return true
  }
  if (request.url === "/last-page-select-request") {
    sendJson(response, fixtureState.lastPageSelectParams)
    return true
  }
  if (request.url === "/last-page-read-requests") {
    const snapshot = structuredClone(fixtureState.pageReadRequests)
    resetRequestGroups(fixtureState.pageReadRequests)
    sendJson(response, snapshot)
    return true
  }
  if (request.url === "/last-article-read-requests") {
    const snapshot = structuredClone(fixtureState.articleReadRequests)
    resetRequestGroups(fixtureState.articleReadRequests)
    sendJson(response, snapshot)
    return true
  }
  if (request.url === "/last-page-write-requests") {
    const snapshot = structuredClone(fixtureState.pageWriteRequests)
    resetRequestGroups(fixtureState.pageWriteRequests)
    sendJson(response, snapshot)
    return true
  }
  if (request.url === "/last-file-requests") {
    const snapshot = structuredClone(fixtureState.fileRequests)
    resetRequestGroups(fixtureState.fileRequests)
    sendJson(response, snapshot)
    return true
  }
  return false
}

/**
 * @param {import("node:http").IncomingMessage} request
 * @param {import("node:http").ServerResponse} response
 */
const handleJsonRpc = (request, response) => {
  if (request.method !== "POST" || request.url !== "/jsonrpc") return false

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
      sendRpcError(response, null, -32700, "Parse error")
      return
    }

    if (
      typeof rpcRequest !== "object" ||
      rpcRequest === null ||
      Array.isArray(rpcRequest) ||
      typeof rpcRequest.method !== "string"
    ) {
      sendRpcError(response, null, -32600, "Invalid Request")
      return
    }

    dispatchFixtureRpc({ rpcRequest, request, response, port: PORT })
  })
  return true
}

const server = createServer((request, response) => {
  if (handleUpload(request, response)) return
  if (handleDiagnosticRequest(request, response)) return
  if (handleJsonRpc(request, response)) return
  response.writeHead(404).end()
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
