import { strict as assert } from "node:assert"
import http from "node:http"
import test from "node:test"

import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheKey,
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  buildPublicContentFenceKey,
  createMemoryArticleResponseCacheStore
} from "../src/lib/server/article-response-cache.js"
import { createArticleResponseFastPathHandler } from "../article-response-fast-path.js"

const SITE_ID = 6000005
const SITE_SLUG = "scp-wiki"
const REQUEST_LOCALES = ["ja-JP", "en-US"]
const BACKEND_LOCALES = ["ja-JP", "en-US", "en"]
const PUBLIC_CONTENT_FENCE = "7"
const PERMISSION_FENCE = "site=11,user=13"
const DEEPWELL_ARTICLE_PAGE_CACHE_KEY =
  "deepwell:article-view:page:v1:site=6000005:page=173:rev=9:updated=123:permission=site=11,user=13:body=aa"

const createFastPathFixtureStore = async ({
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

  const tokenMetadata = buildAnonymousArticleResponseCacheFences({
    siteId: SITE_ID,
    siteSlug: SITE_SLUG,
    route,
    requestLocales: REQUEST_LOCALES,
    backendLocales: BACKEND_LOCALES,
    publicContentFence: PUBLIC_CONTENT_FENCE,
    permissionFence: PERMISSION_FENCE
  })
  await tokenStore.set(
    buildAnonymousArticleResponseTokenKey(tokenMetadata),
    JSON.stringify({ articlePageCacheKey: DEEPWELL_ARTICLE_PAGE_CACHE_KEY })
  )

  const metadata = buildAnonymousArticleResponseCacheMetadata({
    siteId: SITE_ID,
    siteSlug: SITE_SLUG,
    requestLocales: REQUEST_LOCALES,
    backendLocales: BACKEND_LOCALES,
    deepwellArticlePageCacheKey: DEEPWELL_ARTICLE_PAGE_CACHE_KEY,
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

  return { responseStore, tokenStore, cacheKey }
}

const withServer = async ({ responseStore, tokenStore }, run) => {
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
    handler
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

const fastPathHeaders = {
  "accept-language": "ja-JP,en-US;q=0.8",
  "x-wikijump-site-id": String(SITE_ID),
  "x-wikijump-site-slug": SITE_SLUG
}

test("article response fast path serves a hot anonymous article hit without calling handler", async () => {
  const stores = await createFastPathFixtureStore()

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })

    assert.equal(response.status, 200)
    assert.equal(response.headers.get("content-type"), "text/html; charset=utf-8")
    assert.equal(response.headers.get("x-cache-fixture"), "hit")
    assert.equal(
      await response.text(),
      "<!doctype html><html><body>cached article</body></html>"
    )
    assert.equal(handlerCalls(), 0)
  })
})

test("article response fast path sends no cached body for HEAD hits", async () => {
  const stores = await createFastPathFixtureStore()

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, {
      method: "HEAD",
      headers: fastPathHeaders
    })

    assert.equal(response.status, 200)
    assert.equal(response.headers.get("x-cache-fixture"), "hit")
    assert.equal(await response.text(), "")
    assert.equal(handlerCalls(), 0)
  })
})

test("article response fast path handlers keep stores isolated", async () => {
  const firstStores = await createFastPathFixtureStore({
    body: "<!doctype html><html><body>first store article</body></html>"
  })
  const secondStores = await createFastPathFixtureStore({
    body: "<!doctype html><html><body>second store article</body></html>"
  })

  await withServer(firstStores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(response.status, 200)
    assert.equal(
      await response.text(),
      "<!doctype html><html><body>first store article</body></html>"
    )
    assert.equal(handlerCalls(), 0)
  })

  await withServer(secondStores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(response.status, 200)
    assert.equal(
      await response.text(),
      "<!doctype html><html><body>second store article</body></html>"
    )
    assert.equal(handlerCalls(), 0)
  })
})

test("article response fast path falls through when cache entries miss", async () => {
  const responseStore = createMemoryArticleResponseCacheStore()
  const tokenStore = createMemoryArticleResponseCacheStore()

  await withServer({ responseStore, tokenStore }, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })

    assert.equal(response.status, 209)
    assert.equal(await response.text(), "fallback handler")
    assert.equal(handlerCalls(), 1)
  })
})

test("article response fast path falls through for seeded non-article app routes", async () => {
  const stores = await createFastPathFixtureStore({
    route: { slug: "about", extra: "" },
    body: "<!doctype html><html><body>cached about poison</body></html>"
  })

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/about`, { headers: fastPathHeaders })

    assert.equal(response.status, 209)
    assert.equal(await response.text(), "fallback handler")
    assert.equal(handlerCalls(), 1)
  })
})

test("article response fast path enforces static security headers on replay", async () => {
  const csp = "script-src 'nonce-cached-nonce'"
  const cachedBody =
    '<!doctype html><html><body><script nonce="cached-nonce"></script></body></html>'
  const stores = await createFastPathFixtureStore({
    headers: [
      ["content-security-policy", csp],
      ["content-type", "text/html; charset=utf-8"],
      ["cross-origin-opener-policy", "unsafe-none"],
      ["x-cache-fixture", "hit"],
      ["x-frame-options", "SAMEORIGIN"]
    ],
    body: cachedBody
  })

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })

    assert.equal(response.status, 200)
    assert.equal(response.headers.get("content-security-policy"), csp)
    assert.equal(response.headers.get("cross-origin-opener-policy"), "same-origin")
    assert.equal(
      response.headers.get("permissions-policy"),
      "accelerometer=(), autoplay=(), camera=(), display-capture=(), encrypted-media=(), fullscreen=(self), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), midi=(), payment=(), publickey-credentials-get=(self), screen-wake-lock=(), usb=(), web-share=(self), xr-spatial-tracking=()"
    )
    assert.equal(
      response.headers.get("referrer-policy"),
      "strict-origin-when-cross-origin"
    )
    assert.equal(
      response.headers.get("strict-transport-security"),
      "max-age=31536000; includeSubDomains"
    )
    assert.equal(response.headers.get("x-content-type-options"), "nosniff")
    assert.equal(response.headers.get("x-frame-options"), "DENY")
    assert.equal(await response.text(), cachedBody)
    assert.equal(handlerCalls(), 0)
  })
})

test("article response fast path falls through for unsafe requests", async () => {
  const stores = await createFastPathFixtureStore()
  const candidates = [
    ["/scp-173?module=forum", { headers: fastPathHeaders }],
    ["/scp-173/edit", { headers: fastPathHeaders }],
    ["/scp-173", { method: "POST", headers: fastPathHeaders }],
    ["/scp-173", { headers: { ...fastPathHeaders, cookie: "wikijump_token=s" } }],
    ["/scp-173", { headers: { "x-wikijump-site-id": String(SITE_ID) } }]
  ]

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    for (const [path, init] of candidates) {
      const response = await fetch(`${baseUrl}${path}`, init)
      assert.equal(response.status, 209)
      assert.equal(await response.text(), "fallback handler")
    }
    assert.equal(handlerCalls(), candidates.length)
  })
})

test("article response fast path fails closed on malformed cached response values", async () => {
  const stores = await createFastPathFixtureStore()
  await stores.responseStore.set(stores.cacheKey, "{not-json")

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })

    assert.equal(response.status, 209)
    assert.equal(await response.text(), "fallback handler")
    assert.equal(handlerCalls(), 1)
  })
})
