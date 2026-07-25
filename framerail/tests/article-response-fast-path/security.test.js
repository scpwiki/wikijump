import { strict as assert } from "node:assert"
import test from "node:test"

import { createLocalArticleResponseHotCache } from "../../src/lib/server/cache/article-response/index.js"
import {
  SITE_ID,
  createCountingStore,
  createFastPathFixtureStore,
  createTrustedFenceCache,
  fastPathHeaders,
  // eslint-disable-next-line no-redeclare
  fetch,
  withServer
} from "./helpers.js"

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

test("article response fast path lets static security headers override mixed-case cached names", async () => {
  const stores = await createFastPathFixtureStore({
    headers: [
      ["content-type", "text/html; charset=utf-8"],
      ["X-Frame-Options", "SAMEORIGIN"],
      ["x-cache-fixture", "hit"]
    ]
  })

  await withServer(stores, async ({ baseUrl, handlerCalls }) => {
    const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })

    assert.equal(response.status, 200)
    assert.equal(response.headers.get("x-frame-options"), "DENY")
    assert.equal(handlerCalls(), 0)
  })
})

test("article response fast path replays prepared static security headers from local hot cache", async () => {
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
  const hotCache = createLocalArticleResponseHotCache()

  await withServer(
    stores,
    async ({ baseUrl, handlerCalls }) => {
      const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(first.status, 200)
      assert.equal(await first.text(), cachedBody)

      const replay = hotCache.getReplay(stores.tokenKey)
      assert.equal(replay.status, 200)
      assert.deepEqual(replay.headers, [
        ["content-security-policy", csp],
        ["content-type", "text/html; charset=utf-8"],
        ["cross-origin-opener-policy", "same-origin"],
        [
          "permissions-policy",
          "accelerometer=(), autoplay=(), camera=(), display-capture=(), encrypted-media=(), fullscreen=(self), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), midi=(), payment=(), publickey-credentials-get=(self), screen-wake-lock=(), usb=(), web-share=(self), xr-spatial-tracking=()"
        ],
        ["referrer-policy", "strict-origin-when-cross-origin"],
        ["strict-transport-security", "max-age=31536000; includeSubDomains"],
        ["x-cache-fixture", "hit"],
        ["x-content-type-options", "nosniff"],
        ["x-frame-options", "DENY"]
      ])
      assert.deepEqual(replay.nodeRawHeaders, replay.headers.flat())

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 200)
      assert.equal(second.headers.get("content-security-policy"), csp)
      assert.equal(second.headers.get("cross-origin-opener-policy"), "same-origin")
      assert.equal(second.headers.get("x-frame-options"), "DENY")
      assert.equal(await second.text(), cachedBody)
      assert.equal(handlerCalls(), 0)
    },
    { localHotCache: hotCache }
  )
})

test("article response fast path fence invalidation clears prepared replay entries", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
  const hotCache = createLocalArticleResponseHotCache()
  const fenceCache = await createTrustedFenceCache(tokenStore)

  await withServer(
    { responseStore, tokenStore },
    async ({ baseUrl, handlerCalls }) => {
      const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(first.status, 200)
      assert.equal(
        await first.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(hotCache.size(), 1)
      assert.equal(hotCache.getReplay(stores.tokenKey).status, 200)

      await fenceCache.applyMessageForTest(
        JSON.stringify({
          type: "public-content",
          site_id: SITE_ID,
          version: "8"
        })
      )
      assert.equal(hotCache.size(), 0)
      assert.equal(hotCache.getReplay(stores.tokenKey), null)

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 209)
      assert.equal(await second.text(), "fallback handler")
      assert.equal(handlerCalls(), 1)
    },
    { fenceCache, localHotCache: hotCache }
  )
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
