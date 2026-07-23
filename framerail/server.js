import http from "node:http"
import { pathToFileURL } from "node:url"

import { createArticleResponseFastPathHandler } from "./article-response-fast-path.js"
import { createMemoryArticleResponseFenceCache } from "./src/lib/server/article-response-cache.js"
import { configureArticleResponseCacheStores } from "./src/lib/server/article-response-cache-runtime.js"
import { createArticleResponseCacheStores } from "./src/lib/server/article-response-cache-stores.js"

export const createFramerailHttpServer = ({
  fastPathHandler,
  fenceCache,
  closeResources = () => {}
}) => {
  const server = http.createServer((request, response) => {
    void fastPathHandler(request, response).catch((error) => {
      response.statusCode = 500
      response.end(error instanceof Error ? error.message : "Internal Server Error")
    })
  })

  const closeServer = () => {
    fenceCache.close()
    closeResources()
    server.close()
  }

  return { server, closeServer }
}

export const startFramerailServer = async () => {
  const { handler } = await import("./build/handler.js")
  const path = process.env.SOCKET_PATH
  const host = process.env.HOST ?? "0.0.0.0"
  const port = process.env.PORT ?? "3000"
  const { responseStore, tokenStore } = createArticleResponseCacheStores()
  const resetRuntimeStores = configureArticleResponseCacheStores({
    responseStore,
    tokenStore
  })
  const fenceCache = createMemoryArticleResponseFenceCache({
    store: tokenStore,
    subscriber: tokenStore
  })
  const fastPathHandler = createArticleResponseFastPathHandler({
    responseStore,
    tokenStore,
    handler,
    fenceCache
  })
  const lifecycle = createFramerailHttpServer({
    fastPathHandler,
    fenceCache,
    closeResources: () => {
      resetRuntimeStores()
      tokenStore?.reset?.()
    }
  })

  lifecycle.server.listen(path ? { path } : { host, port }, () => {
    console.log(`Listening on ${path || `http://${host}:${port}`}`)
  })
  process.on("SIGTERM", lifecycle.closeServer)
  process.on("SIGINT", lifecycle.closeServer)

  return lifecycle
}

const entryPoint = process.argv[1]
if (entryPoint && import.meta.url === pathToFileURL(entryPoint).href) {
  void startFramerailServer().catch((error) => {
    console.error(error)
    process.exitCode = 1
  })
}
