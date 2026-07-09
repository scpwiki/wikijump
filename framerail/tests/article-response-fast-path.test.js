import { strict as assert } from "node:assert"
import http from "node:http"
import test from "node:test"

import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheKey,
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  buildPublicContentFenceKey,
  createLocalArticleResponseHotCache,
  createMemoryArticleResponseFenceCache,
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

const seedFastPathStoreEntry = async (
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

  const { cacheKey, tokenKey } = await seedFastPathStoreEntry(
    { responseStore, tokenStore },
    { route, headers, body }
  )

  return { responseStore, tokenStore, cacheKey, tokenKey }
}

const createCountingStore = (store) => {
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

const createDeferredGetStore = (store, shouldDefer) => {
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

const createTrustedFenceCache = async (store, options = {}) => {
  const fenceCache = createMemoryArticleResponseFenceCache({
    store,
    ...options
  })
  await fenceCache.markSubscribedForTest()
  return fenceCache
}

const withServer = async ({ responseStore, tokenStore }, run, fastPathOptions = {}) => {
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

test("article response fast path local hot hit avoids token and response store reads", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)

  await withServer({ responseStore, tokenStore }, async ({ baseUrl, handlerCalls }) => {
    const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(first.status, 200)
    assert.equal(
      await first.text(),
      "<!doctype html><html><body>cached article</body></html>"
    )
    assert.equal(tokenStore.mgetCalls(), 1)
    assert.equal(tokenStore.getCalls(), 1)
    assert.equal(responseStore.getCalls(), 1)

    const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(second.status, 200)
    assert.equal(
      await second.text(),
      "<!doctype html><html><body>cached article</body></html>"
    )
    assert.equal(handlerCalls(), 0)
    assert.equal(tokenStore.mgetCalls(), 2)
    assert.equal(tokenStore.getCalls(), 1)
    assert.equal(responseStore.getCalls(), 1)
  })
})

test("article response fast path uses trusted shared local hot replay", async () => {
  const stores = await createFastPathFixtureStore()
  const replay = Object.freeze({
    status: 200,
    headers: Object.freeze([Object.freeze(["x-cache-fixture", "shared"])]),
    bodyBuffer: Buffer.from(
      "<!doctype html><html><body>shared hot article</body></html>"
    ),
    finalHeaders: true
  })
  const localHotCache = {
    getSharedReplayForInternalUse(key) {
      assert.equal(key, stores.tokenKey)
      return replay
    },
    getReplay() {
      throw new Error("fast path should not copy local hot replay")
    },
    get() {
      throw new Error("fast path should not fall back to entry copy")
    },
    set() {
      throw new Error("fast path should not refill on a hot replay hit")
    }
  }

  await withServer(
    stores,
    async ({ baseUrl, handlerCalls }) => {
      const response = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })

      assert.equal(response.status, 200)
      assert.equal(response.headers.get("x-cache-fixture"), "shared")
      assert.equal(
        await response.text(),
        "<!doctype html><html><body>shared hot article</body></html>"
      )
      assert.equal(handlerCalls(), 0)
    },
    { localHotCache }
  )
})

test("article response fast path trusted local fence second hot hit does zero store reads", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
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
      assert.equal(tokenStore.mgetCalls(), 1)
      assert.equal(tokenStore.getCalls(), 1)
      assert.equal(responseStore.getCalls(), 1)

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 200)
      assert.equal(
        await second.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(handlerCalls(), 0)
      assert.equal(tokenStore.mgetCalls(), 1)
      assert.equal(tokenStore.getCalls(), 1)
      assert.equal(responseStore.getCalls(), 1)
    },
    { fenceCache, localHotCacheOptions: {} }
  )
})

test("article response fast path untrusted local fence cache falls back to Redis fences", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
  const fenceCache = createMemoryArticleResponseFenceCache({ store: tokenStore })

  await withServer(
    { responseStore, tokenStore },
    async ({ baseUrl, handlerCalls }) => {
      const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(first.status, 200)
      assert.equal(
        await first.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 200)
      assert.equal(
        await second.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(handlerCalls(), 0)
      assert.equal(tokenStore.mgetCalls(), 2)
      assert.equal(tokenStore.getCalls(), 1)
      assert.equal(responseStore.getCalls(), 1)
    },
    { fenceCache, localHotCacheOptions: {} }
  )
})

test("article response fast path local hot cache misses after a fence change", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)

  await withServer({ responseStore, tokenStore }, async ({ baseUrl, handlerCalls }) => {
    const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(first.status, 200)
    assert.equal(
      await first.text(),
      "<!doctype html><html><body>cached article</body></html>"
    )

    await tokenStore.set(buildPublicContentFenceKey(SITE_ID), "8")
    const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(second.status, 209)
    assert.equal(await second.text(), "fallback handler")
    assert.equal(handlerCalls(), 1)
    assert.equal(tokenStore.mgetCalls(), 2)
    assert.equal(tokenStore.getCalls(), 2)
    assert.equal(responseStore.getCalls(), 1)
  })
})

test("article response fast path trusted public content invalidation prevents old hot response", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
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

      await fenceCache.applyMessageForTest(
        JSON.stringify({
          type: "public-content",
          site_id: SITE_ID,
          version: "8"
        })
      )

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 209)
      assert.equal(await second.text(), "fallback handler")
      assert.equal(handlerCalls(), 1)
      assert.equal(tokenStore.mgetCalls(), 1)
      assert.equal(tokenStore.getCalls(), 2)
      assert.equal(responseStore.getCalls(), 1)
    },
    { fenceCache, localHotCacheOptions: {} }
  )
})

test("article response fast path trusted anonymous permission invalidation prevents old hot response", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
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

      await fenceCache.applyMessageForTest(
        JSON.stringify({
          type: "anonymous-permission",
          site_id: SITE_ID,
          site_version: "12",
          user_version: "13"
        })
      )

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 209)
      assert.equal(await second.text(), "fallback handler")
      assert.equal(handlerCalls(), 1)
      assert.equal(tokenStore.mgetCalls(), 1)
      assert.equal(tokenStore.getCalls(), 2)
      assert.equal(responseStore.getCalls(), 1)
    },
    { fenceCache, localHotCacheOptions: {} }
  )
})

test("article response fast path revalidates local fences after invalidation during store reads", async () => {
  for (const scenario of [
    {
      defer: "token",
      message: {
        type: "public-content",
        site_id: SITE_ID,
        version: "8"
      }
    },
    {
      defer: "response",
      message: {
        type: "anonymous-permission",
        site_id: SITE_ID,
        site_version: "12",
        user_version: "13"
      }
    }
  ]) {
    const stores = await createFastPathFixtureStore()
    const tokenStore = createCountingStore(stores.tokenStore)
    const responseStore = createCountingStore(stores.responseStore)
    const deferredTokenStore = createDeferredGetStore(tokenStore, (key) =>
      scenario.defer === "token" ? key === stores.tokenKey : false
    )
    const deferredResponseStore = createDeferredGetStore(responseStore, (key) =>
      scenario.defer === "response" ? key === stores.cacheKey : false
    )
    const deferredStore =
      scenario.defer === "token" ? deferredTokenStore : deferredResponseStore
    const fenceCache = await createTrustedFenceCache(deferredTokenStore)

    await withServer(
      { responseStore: deferredResponseStore, tokenStore: deferredTokenStore },
      async ({ baseUrl, handlerCalls }) => {
        const responsePromise = fetch(`${baseUrl}/scp-173`, {
          headers: fastPathHeaders
        })
        await deferredStore.started
        await fenceCache.applyMessageForTest(JSON.stringify(scenario.message))
        deferredStore.resume()

        const response = await responsePromise
        assert.equal(response.status, 209)
        assert.equal(await response.text(), "fallback handler")
        assert.equal(handlerCalls(), 1)
      },
      { fenceCache, localHotCacheOptions: {} }
    )
  }
})

test("article response fast path malformed fence messages fail closed to Redis fences", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
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

      await fenceCache.applyMessageForTest("{not-json")

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 200)
      assert.equal(
        await second.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(handlerCalls(), 0)
      assert.equal(tokenStore.mgetCalls(), 2)
      assert.equal(tokenStore.getCalls(), 2)
      assert.equal(responseStore.getCalls(), 2)
    },
    { fenceCache, localHotCacheOptions: {} }
  )
})

test("article response fast path local fence disconnect clears hot cache and falls back", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
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

      fenceCache.markDisconnectedForTest()

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 200)
      assert.equal(
        await second.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(handlerCalls(), 0)
      assert.equal(tokenStore.mgetCalls(), 2)
      assert.equal(tokenStore.getCalls(), 2)
      assert.equal(responseStore.getCalls(), 2)
    },
    { fenceCache, localHotCacheOptions: {} }
  )
})

test("article response fast path local hot cache entries expire by TTL", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)
  let now = 0

  await withServer(
    { responseStore, tokenStore },
    async ({ baseUrl, handlerCalls }) => {
      const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(first.status, 200)
      assert.equal(
        await first.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(tokenStore.getCalls(), 1)
      assert.equal(responseStore.getCalls(), 1)

      now = 9
      const stillFresh = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(stillFresh.status, 200)
      assert.equal(
        await stillFresh.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(tokenStore.mgetCalls(), 2)
      assert.equal(tokenStore.getCalls(), 1)
      assert.equal(responseStore.getCalls(), 1)

      now = 11
      const expired = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(expired.status, 200)
      assert.equal(
        await expired.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(handlerCalls(), 0)
      assert.equal(tokenStore.mgetCalls(), 3)
      assert.equal(tokenStore.getCalls(), 2)
      assert.equal(responseStore.getCalls(), 2)
    },
    { localHotCacheOptions: { ttlMs: 10, now: () => now } }
  )
})

test("article response fast path local hot cache evicts least recently used entries and oversized entries", async () => {
  const stores = await createFastPathFixtureStore()
  await seedFastPathStoreEntry(stores, {
    route: { slug: "scp-174", extra: "" },
    deepwellArticlePageCacheKey:
      "deepwell:article-view:page:v1:site=6000005:page=174:rev=9:updated=123:permission=site=11,user=13:body=bb",
    body: "<!doctype html><html><body>cached article 174</body></html>"
  })
  await seedFastPathStoreEntry(stores, {
    route: { slug: "scp-175", extra: "" },
    deepwellArticlePageCacheKey:
      "deepwell:article-view:page:v1:site=6000005:page=175:rev=9:updated=123:permission=site=11,user=13:body=cc",
    body: "<!doctype html><html><body>cached article 175</body></html>"
  })
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)

  await withServer(
    { responseStore, tokenStore },
    async ({ baseUrl }) => {
      const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(first.status, 200)
      assert.equal(
        await first.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )

      const second = await fetch(`${baseUrl}/scp-174`, { headers: fastPathHeaders })
      assert.equal(second.status, 200)
      assert.equal(
        await second.text(),
        "<!doctype html><html><body>cached article 174</body></html>"
      )

      const third = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(third.status, 200)
      assert.equal(
        await third.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(tokenStore.getCalls(), 2)
      assert.equal(responseStore.getCalls(), 2)

      const fourth = await fetch(`${baseUrl}/scp-175`, { headers: fastPathHeaders })
      assert.equal(fourth.status, 200)
      assert.equal(
        await fourth.text(),
        "<!doctype html><html><body>cached article 175</body></html>"
      )

      const fifth = await fetch(`${baseUrl}/scp-174`, { headers: fastPathHeaders })
      assert.equal(fifth.status, 200)
      assert.equal(
        await fifth.text(),
        "<!doctype html><html><body>cached article 174</body></html>"
      )
      assert.equal(tokenStore.getCalls(), 4)
      assert.equal(responseStore.getCalls(), 4)
    },
    { localHotCacheOptions: { maxEntries: 2 } }
  )

  const smallTokenStore = createCountingStore(stores.tokenStore)
  const smallResponseStore = createCountingStore(stores.responseStore)
  await withServer(
    { responseStore: smallResponseStore, tokenStore: smallTokenStore },
    async ({ baseUrl }) => {
      const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(first.status, 200)
      assert.equal(
        await first.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )

      const second = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
      assert.equal(second.status, 200)
      assert.equal(
        await second.text(),
        "<!doctype html><html><body>cached article</body></html>"
      )
      assert.equal(smallTokenStore.getCalls(), 2)
      assert.equal(smallResponseStore.getCalls(), 2)
    },
    { localHotCacheOptions: { maxBytes: 1 } }
  )
})

test("article response fast path replays HEAD from local hot cache", async () => {
  const stores = await createFastPathFixtureStore()
  const tokenStore = createCountingStore(stores.tokenStore)
  const responseStore = createCountingStore(stores.responseStore)

  await withServer({ responseStore, tokenStore }, async ({ baseUrl, handlerCalls }) => {
    const first = await fetch(`${baseUrl}/scp-173`, { headers: fastPathHeaders })
    assert.equal(first.status, 200)
    assert.equal(
      await first.text(),
      "<!doctype html><html><body>cached article</body></html>"
    )

    const second = await fetch(`${baseUrl}/scp-173`, {
      method: "HEAD",
      headers: fastPathHeaders
    })
    assert.equal(second.status, 200)
    assert.equal(second.headers.get("x-cache-fixture"), "hit")
    assert.equal(await second.text(), "")
    assert.equal(handlerCalls(), 0)
    assert.equal(tokenStore.mgetCalls(), 2)
    assert.equal(tokenStore.getCalls(), 1)
    assert.equal(responseStore.getCalls(), 1)
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
