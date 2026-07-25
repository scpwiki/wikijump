import { strict as assert } from "node:assert"
import { promisify } from "node:util"
import http from "node:http"
import zlib from "node:zlib"

import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheKey,
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  buildPublicContentFenceKey,
  createMemoryArticleResponseFenceCache,
  createMemoryArticleResponseCacheStore
} from "../../src/lib/server/cache/article-response/index.js"
import { createArticleResponseFastPathHandler } from "../../article-response-fast-path.js"

export const SITE_ID = 6000005
export const SITE_SLUG = "scp-wiki"
export const REQUEST_HOST = "scp-wiki.example"
export const REQUEST_LOCALES = ["ja-JP", "en-US"]
export const BACKEND_LOCALES = ["ja-JP", "en-US", "en"]
export const PUBLIC_CONTENT_FENCE = "7"
export const PERMISSION_FENCE = "site=11,user=13"
export const DEEPWELL_ARTICLE_PAGE_CACHE_KEY =
  "deepwell:article-view:page:v1:site=6000005:page=173:rev=9:updated=123:permission=site=11,user=13:body=aa"
export const brotliDecompress = promisify(zlib.brotliDecompress)
export const gunzip = promisify(zlib.gunzip)

export const seedFastPathStoreEntry = async (
  { responseStore, tokenStore },
  {
    route = { slug: "scp-173", extra: "" },
    deepwellArticlePageCacheKey = DEEPWELL_ARTICLE_PAGE_CACHE_KEY,
    headers = [
      ["content-type", "text/html; charset=utf-8"],
      ["x-cache-fixture", "hit"]
    ],
    body = "<!doctype html><html><body>cached article</body></html>"
  } = {}
) => {
  const tokenMetadata = buildAnonymousArticleResponseCacheFences({
    siteId: SITE_ID,
    siteSlug: SITE_SLUG,
    requestHost: REQUEST_HOST,
    route,
    requestLocales: REQUEST_LOCALES,
    backendLocales: BACKEND_LOCALES,
    publicContentFence: PUBLIC_CONTENT_FENCE,
    permissionFence: PERMISSION_FENCE
  })
  const tokenKey = buildAnonymousArticleResponseTokenKey(tokenMetadata)
  await tokenStore.set(
    tokenKey,
    JSON.stringify({ articlePageCacheKey: deepwellArticlePageCacheKey })
  )

  const metadata = buildAnonymousArticleResponseCacheMetadata({
    siteId: SITE_ID,
    siteSlug: SITE_SLUG,
    requestHost: REQUEST_HOST,
    requestLocales: REQUEST_LOCALES,
    backendLocales: BACKEND_LOCALES,
    deepwellArticlePageCacheKey,
    publicContentFence: PUBLIC_CONTENT_FENCE,
    permissionFence: PERMISSION_FENCE
  })
  const cacheKey = buildAnonymousArticleResponseCacheKey(metadata)
  await responseStore.set(
    cacheKey,
    JSON.stringify({
      status: 200,
      headers,
      body
    })
  )

  return { responseStore, tokenStore, cacheKey, tokenKey }
}

export const createFastPathFixtureStore = async ({
  route = { slug: "scp-173", extra: "" },
  headers = [
    ["content-type", "text/html; charset=utf-8"],
    ["x-cache-fixture", "hit"]
  ],
  body = "<!doctype html><html><body>cached article</body></html>"
} = {}) => {
  const tokenStore = createMemoryArticleResponseCacheStore()
  const responseStore = createMemoryArticleResponseCacheStore()
  await tokenStore.set(buildPublicContentFenceKey(SITE_ID), PUBLIC_CONTENT_FENCE)
  await tokenStore.set(`permission:site:${SITE_ID}:version`, "11")
  await tokenStore.set(`permission:site:${SITE_ID}:user:anonymous:version`, "13")

  const { cacheKey, tokenKey } = await seedFastPathStoreEntry(
    { responseStore, tokenStore },
    { route, headers, body }
  )

  return { responseStore, tokenStore, cacheKey, tokenKey }
}

export const createCountingStore = (store) => {
  let getCalls = 0
  let mgetCalls = 0

  return {
    async get(key) {
      getCalls += 1
      return store.get(key)
    },
    async mget(keys) {
      mgetCalls += 1
      return Promise.all(keys.map((key) => store.get(key)))
    },
    async set(key, value, ttlSeconds) {
      return store.set(key, value, ttlSeconds)
    },
    getCalls() {
      return getCalls
    },
    mgetCalls() {
      return mgetCalls
    }
  }
}

export const createDeferredGetStore = (store, shouldDefer) => {
  let resolveStarted
  let resume
  const started = new Promise((resolve) => {
    resolveStarted = resolve
  })
  const ready = new Promise((resolve) => {
    resume = resolve
  })
  let deferred = false

  return {
    async get(key) {
      if (!deferred && shouldDefer(key)) {
        deferred = true
        resolveStarted(key)
        await ready
      }
      return store.get(key)
    },
    async mget(keys) {
      return store.mget(keys)
    },
    async set(key, value, ttlSeconds) {
      return store.set(key, value, ttlSeconds)
    },
    getCalls() {
      return store.getCalls?.() ?? 0
    },
    mgetCalls() {
      return store.mgetCalls?.() ?? 0
    },
    started,
    resume
  }
}

export const createTrustedFenceCache = async (store, options = {}) => {
  const fenceCache = createMemoryArticleResponseFenceCache({
    store,
    ...options
  })
  await fenceCache.markSubscribedForTest()
  return fenceCache
}

export const withServer = async (
  { responseStore, tokenStore },
  run,
  fastPathOptions = {}
) => {
  let handlerCalls = 0
  const handler = (request, response) => {
    handlerCalls += 1
    response.statusCode = 209
    response.setHeader("content-type", "text/plain")
    response.end("fallback handler")
  }
  const fastPathHandler = createArticleResponseFastPathHandler({
    responseStore,
    tokenStore,
    handler,
    ...fastPathOptions
  })
  const server = http.createServer((request, response) => {
    void fastPathHandler(request, response)
  })

  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  try {
    return await run({
      baseUrl: `http://127.0.0.1:${address.port}`,
      handlerCalls: () => handlerCalls
    })
  } finally {
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()))
    })
  }
}

export const requestRaw = (url, { method = "GET", headers = {} } = {}) => {
  return new Promise((resolve, reject) => {
    const request = http.request(url, { method, headers }, (response) => {
      const chunks = []
      response.on("data", (chunk) => chunks.push(chunk))
      response.on("end", () => {
        resolve({
          status: response.statusCode,
          headers: response.headers,
          body: Buffer.concat(chunks)
        })
      })
    })
    request.on("error", reject)
    request.end()
  })
}

// Node's built-in fetch rewrites Host, so these tests use http.request instead.
// eslint-disable-next-line no-redeclare
export const fetch = async (url, { method = "GET", headers = {} } = {}) => {
  const response = await requestRaw(url, { method, headers })
  const responseHeaders = new Headers()
  for (const [name, value] of Object.entries(response.headers)) {
    if (Array.isArray(value)) {
      for (const item of value) responseHeaders.append(name, item)
    } else if (value !== undefined) {
      responseHeaders.append(name, value)
    }
  }
  return new Response(response.body, {
    status: response.status,
    headers: responseHeaders
  })
}

export const largeHtmlBody = () => {
  return `<!doctype html><html><body>${"<p>cached article paragraph</p>".repeat(120)}</body></html>`
}

export const fastPathHeaders = {
  host: REQUEST_HOST.toUpperCase(),
  "accept-language": "ja-JP,en-US;q=0.8",
  "x-wikijump-site-id": String(SITE_ID),
  "x-wikijump-site-slug": SITE_SLUG
}

export const createRecordingResponse = () => {
  const calls = []
  return {
    calls,
    statusCode: undefined,
    writeHead(status, headers) {
      calls.push(["writeHead", status, headers])
    },
    setHeader(name, value) {
      calls.push(["setHeader", name, value])
    },
    removeHeader(name) {
      calls.push(["removeHeader", name])
    },
    end(body) {
      calls.push(["end", body])
    }
  }
}
