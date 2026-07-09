import http from "node:http"

import { createArticleResponseFastPathHandler } from "./article-response-fast-path.js"
import { handler } from "./build/handler.js"
import { createMemoryArticleResponseFenceCache } from "./src/lib/server/article-response-cache.js"
import {
  articleResponseCacheStore,
  articleResponseTokenStore
} from "./src/lib/server/article-response-cache-stores.js"

const path = process.env.SOCKET_PATH
const host = process.env.HOST ?? "0.0.0.0"
const port = process.env.PORT ?? "3000"
const fenceCache = createMemoryArticleResponseFenceCache({
  store: articleResponseTokenStore,
  subscriber: articleResponseTokenStore
})
const fastPathHandler = createArticleResponseFastPathHandler({
  responseStore: articleResponseCacheStore,
  tokenStore: articleResponseTokenStore,
  handler,
  fenceCache
})

const server = http.createServer((request, response) => {
  void fastPathHandler(request, response).catch((error) => {
    response.statusCode = 500
    response.end(error instanceof Error ? error.message : "Internal Server Error")
  })
})

server.listen(path ? { path } : { host, port }, () => {
  console.log(`Listening on ${path || `http://${host}:${port}`}`)
})

const closeServer = () => {
  fenceCache.close()
  server.close()
}

process.on("SIGTERM", closeServer)
process.on("SIGINT", closeServer)
