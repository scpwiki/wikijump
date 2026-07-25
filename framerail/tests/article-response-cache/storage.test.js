import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildAnonymousArticleResponseCacheFences,
  buildPublicContentFenceKey,
  createMemoryArticleResponseFenceCache
} from "../../src/lib/server/cache/article-response/fences.js"
import {
  ARTICLE_RESPONSE_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  buildAnonymousArticleResponseCacheKey,
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  createLocalArticleResponseHotCache,
  createMemoryArticleResponseCacheStore,
  deserializeCachedArticleResponse,
  readAnonymousArticleResponseToken,
  readCachedArticleResponse,
  serializeArticleResponseForCache,
  writeAnonymousArticleResponseToken,
  writeCachedArticleResponse
} from "../../src/lib/server/cache/article-response/index.js"

const REQUEST_HOST = "scp-wiki.example"

test("memory article response fence cache closes subscriber handle", () => {
  let closed = 0
  const fenceCache = createMemoryArticleResponseFenceCache({
    subscriber: {
      subscribe(callbacks) {
        assert.equal(callbacks.channel, "wikijump:article-response-fence-invalidation:v1")
        return {
          close() {
            closed += 1
          }
        }
      }
    }
  })

  fenceCache.close()

  assert.equal(closed, 1)
})

test("memory article response fence cache does not store a seed raced by invalidation", async () => {
  let resumeSeed
  let seedStartedResolve
  const seedStarted = new Promise((resolve) => {
    seedStartedResolve = resolve
  })
  const staleSeed = ["7", "11", "13"]
  const currentSeed = ["8", "11", "13"]
  let calls = 0
  const store = {
    async mget(keys) {
      assert.equal(keys.length, 3)
      calls += 1
      if (calls === 1) {
        seedStartedResolve()
        await new Promise((resume) => {
          resumeSeed = resume
        })
        return staleSeed
      }
      return currentSeed
    }
  }
  const fenceCache = createMemoryArticleResponseFenceCache({ store })
  await fenceCache.markSubscribedForTest()

  const seedingRead = fenceCache.readFences({ siteId: 6000005 })
  await seedStarted
  await fenceCache.applyMessageForTest(
    JSON.stringify({ type: "public-content", site_id: 6000005, version: "8" })
  )
  resumeSeed()

  assert.equal(await seedingRead, null)
  assert.deepEqual(await fenceCache.readFences({ siteId: 6000005 }), {
    publicContentFence: "8",
    permissionFence: "site=11,user=13"
  })
})

test("memory article response fence cache ignores non-anonymous user permission messages", async () => {
  let reads = 0
  const store = {
    async mget(keys) {
      assert.equal(keys.length, 3)
      reads += 1
      return ["7", "11", "13"]
    }
  }
  const hotCache = createLocalArticleResponseHotCache()
  assert.equal(
    hotCache.store("token", {
      status: 200,
      headers: [["content-type", "text/html"]],
      body: "<!doctype html><html><body>cached body</body></html>"
    }),
    true
  )
  const fenceCache = createMemoryArticleResponseFenceCache({ store })
  fenceCache.attachHotCache(hotCache)
  await fenceCache.markSubscribedForTest()

  assert.deepEqual(await fenceCache.readFences({ siteId: 6000005 }), {
    publicContentFence: "7",
    permissionFence: "site=11,user=13"
  })
  await fenceCache.applyMessageForTest(
    JSON.stringify({
      type: "user-permission",
      site_id: 6000005,
      user_id: 123,
      version: "19"
    })
  )

  assert.deepEqual(await fenceCache.readFences({ siteId: 6000005 }), {
    publicContentFence: "7",
    permissionFence: "site=11,user=13"
  })
  assert.equal(reads, 1)
  assert.equal(hotCache.size(), 1)
})

test("anonymous article response token maps route and fences to Deepwell cache key", async () => {
  const tokenMetadata = buildAnonymousArticleResponseCacheFences({
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestHost: REQUEST_HOST,
    route: { slug: "scp-173", extra: "" },
    requestLocales: ["en-US"],
    backendLocales: ["en-US", "en"],
    publicContentFence: "7",
    permissionFence: "site=11,user=13"
  })
  const tokenKey = buildAnonymousArticleResponseTokenKey(tokenMetadata)
  const store = createMemoryArticleResponseCacheStore()
  await store.set(buildPublicContentFenceKey(6000005), "7")
  await store.set("permission:site:6000005:version", "11")
  await store.set("permission:site:6000005:user:anonymous:version", "13")
  const deepwellArticlePageCacheKey =
    "deepwell:article-view:page:v1:site=6000005:page=173:rev=9:updated=123:permission=site=11,user=13:body=aa"

  assert.match(
    tokenKey,
    /^framerail:article-response-token:v1:site=6000005:slug=7363702d77696b69:host=7363702d77696b692e6578616d706c65:route=[a-f0-9]{64}:requestLocales=656e2d5553:backendLocales=656e2d55532c656e:content=7:permission=736974653d31312c757365723d3133$/
  )
  assert.equal(
    await writeAnonymousArticleResponseToken({
      store,
      tokenMetadata,
      deepwellArticlePageCacheKey
    }),
    true
  )
  assert.equal(
    await readAnonymousArticleResponseToken({ store, tokenMetadata }),
    deepwellArticlePageCacheKey
  )
})

test("anonymous article response token write skips when captured fences are stale", async () => {
  const tokenMetadata = buildAnonymousArticleResponseCacheFences({
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestHost: REQUEST_HOST,
    route: { slug: "scp-173", extra: "" },
    requestLocales: ["en-US"],
    backendLocales: ["en-US", "en"],
    publicContentFence: "7",
    permissionFence: "site=11,user=13"
  })
  const store = createMemoryArticleResponseCacheStore()
  await store.set(buildPublicContentFenceKey(6000005), "8")
  await store.set("permission:site:6000005:version", "11")
  await store.set("permission:site:6000005:user:anonymous:version", "13")

  assert.equal(
    await writeAnonymousArticleResponseToken({
      store,
      tokenMetadata,
      deepwellArticlePageCacheKey:
        "deepwell:article-view:page:v1:site=6000005:page=173:rev=9:updated=123:permission=site=11,user=13:body=aa"
    }),
    false
  )
  assert.equal(await readAnonymousArticleResponseToken({ store, tokenMetadata }), null)
})

test("anonymous article response token reads fail closed on malformed values", async () => {
  const tokenMetadata = buildAnonymousArticleResponseCacheFences({
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestHost: REQUEST_HOST,
    route: { slug: "scp-173", extra: "" },
    requestLocales: ["en-US"],
    backendLocales: ["en-US", "en"],
    publicContentFence: "7",
    permissionFence: "site=11,user=13"
  })
  const store = createMemoryArticleResponseCacheStore()
  await store.set(buildAnonymousArticleResponseTokenKey(tokenMetadata), "{not-json")

  assert.equal(await readAnonymousArticleResponseToken({ store, tokenMetadata }), null)
})

test("anonymous article response cache key varies by Deepwell cache key", () => {
  const baseMetadata = {
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestHost: REQUEST_HOST,
    requestLocales: ["en-US"],
    backendLocales: ["en-US", "en"]
  }
  const first = buildAnonymousArticleResponseCacheMetadata({
    ...baseMetadata,
    deepwellArticlePageCacheKey:
      "deepwell:article-view:page:v1:site=6000005:page=173:rev=9:updated=123:permission=site:3:user:anonymous:body=aa:top=bb:side=cc"
  })
  const second = buildAnonymousArticleResponseCacheMetadata({
    ...baseMetadata,
    deepwellArticlePageCacheKey:
      "deepwell:article-view:page:v1:site=6000005:page=173:rev=10:updated=456:permission=site:3:user:anonymous:body=dd:top=ee:side=ff"
  })

  assert.notEqual(
    buildAnonymousArticleResponseCacheKey(first),
    buildAnonymousArticleResponseCacheKey(second)
  )
})

test("anonymous article response cache and token keys vary by request host", () => {
  const baseMetadata = {
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestLocales: ["en-US"],
    backendLocales: ["en-US", "en"],
    deepwellArticlePageCacheKey:
      "deepwell:article-view:page:v1:site=6000005:page=173:rev=9:updated=123:permission=site:3:user:anonymous:body=aa"
  }
  const first = buildAnonymousArticleResponseCacheMetadata({
    ...baseMetadata,
    requestHost: "scp-wiki.example"
  })
  const second = buildAnonymousArticleResponseCacheMetadata({
    ...baseMetadata,
    requestHost: "scp-wiki.alt-example"
  })

  assert.notEqual(
    buildAnonymousArticleResponseCacheKey(first),
    buildAnonymousArticleResponseCacheKey(second)
  )

  const baseFences = {
    siteId: 6000005,
    siteSlug: "scp-wiki",
    route: { slug: "scp-173", extra: "" },
    requestLocales: ["en-US"],
    backendLocales: ["en-US", "en"],
    publicContentFence: "7",
    permissionFence: "site=11,user=13"
  }
  assert.notEqual(
    buildAnonymousArticleResponseTokenKey(
      buildAnonymousArticleResponseCacheFences({
        ...baseFences,
        requestHost: "scp-wiki.example"
      })
    ),
    buildAnonymousArticleResponseTokenKey(
      buildAnonymousArticleResponseCacheFences({
        ...baseFences,
        requestHost: "scp-wiki.alt-example"
      })
    )
  )
})

test("anonymous article response cache serializes final response headers", async () => {
  const response = new Response("<!doctype html><html><body>cached</body></html>", {
    status: 200,
    headers: {
      "content-type": "text/html",
      "cross-origin-opener-policy": "same-origin",
      "x-frame-options": "DENY"
    }
  })

  const serialized = await serializeArticleResponseForCache(response)

  assert.deepEqual(serialized, {
    status: 200,
    headers: [
      ["content-type", "text/html"],
      ["cross-origin-opener-policy", "same-origin"],
      ["x-frame-options", "DENY"]
    ],
    body: "<!doctype html><html><body>cached</body></html>"
  })

  const restored = deserializeCachedArticleResponse(serialized)
  assert.equal(restored.status, 200)
  assert.equal(restored.headers.get("x-frame-options"), "DENY")
  assert.equal(await restored.text(), "<!doctype html><html><body>cached</body></html>")
})

test("anonymous article response cache store helpers fail closed", async () => {
  const malformedStore = {
    async get() {
      return "{not json"
    },
    async set(key, value, ttlSeconds) {
      assert.equal(key, "key")
      assert.equal(typeof value, "string")
      assert.equal(ttlSeconds, 60)
      throw new Error("redis unavailable")
    }
  }

  assert.equal(await readCachedArticleResponse(malformedStore, "key"), null)
  assert.equal(
    await writeCachedArticleResponse(
      malformedStore,
      "key",
      { status: 200, headers: [], body: "body" },
      60
    ),
    false
  )
})

test("memory article response cache evicts oldest entries above max size", async () => {
  assert.equal(Number.isInteger(ARTICLE_RESPONSE_CACHE_MAX_ENTRIES), true)
  const store = createMemoryArticleResponseCacheStore({ maxEntries: 2 })

  await store.set("first", "a")
  await store.set("second", "b")
  await store.set("third", "c")

  assert.equal(await store.get("first"), null)
  assert.equal(await store.get("second"), "b")
  assert.equal(await store.get("third"), "c")
  assert.equal(store.size(), 2)
})

test("memory article response cache evicts oldest entries above max bytes", async () => {
  assert.equal(Number.isInteger(ARTICLE_RESPONSE_CACHE_MAX_BYTES), true)
  const store = createMemoryArticleResponseCacheStore({
    maxEntries: 10,
    maxBytes: 4
  })

  await store.set("first", "aa")
  await store.set("second", "bb")
  await store.set("third", "c")

  assert.equal(await store.get("first"), null)
  assert.equal(await store.get("second"), "bb")
  assert.equal(await store.get("third"), "c")
  assert.equal(store.size(), 2)
})

test("memory article response cache rejects entries larger than max bytes", async () => {
  const store = createMemoryArticleResponseCacheStore({
    maxEntries: 10,
    maxBytes: 4
  })

  await store.set("too-large", "abcde")

  assert.equal(await store.get("too-large"), null)
  assert.equal(store.size(), 0)
})

test("memory article response cache prunes expired entries on write", async () => {
  let now = 0
  const store = createMemoryArticleResponseCacheStore({
    now: () => now,
    maxEntries: 2
  })

  await store.set("fresh", "a", 60)
  await store.set("expired", "b", 1)
  now = 2000
  await store.set("new", "c", 60)

  assert.equal(await store.get("fresh"), "a")
  assert.equal(await store.get("expired"), null)
  assert.equal(await store.get("new"), "c")
  assert.equal(store.size(), 2)
})
