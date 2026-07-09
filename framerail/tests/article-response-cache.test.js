import { strict as assert } from "node:assert"
import test from "node:test"

import {
  ARTICLE_RESPONSE_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES,
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheKey,
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  buildAnonymousPermissionFenceKeys,
  buildPublicContentFenceKey,
  canConsiderAnonymousArticleResponseCache,
  createMemoryArticleResponseCacheStore,
  deserializeCachedArticleResponse,
  readAnonymousArticleResponseCacheFences,
  readAnonymousArticleResponseCache,
  readAnonymousArticleResponseToken,
  readCachedArticleResponse,
  serializeArticleResponseForCache,
  writeAnonymousArticleResponseToken,
  writeAnonymousArticleResponseCache,
  writeCachedArticleResponse
} from "../src/lib/server/article-response-cache.js"

test("anonymous article response cache gate allows only plain anonymous article GETs", () => {
  const allowed = canConsiderAnonymousArticleResponseCache({
    method: "GET",
    routeId: "/[slug]/[...extra]",
    url: new URL("https://scp-wiki.example/scp-173"),
    siteId: 6000005,
    siteSlug: "scp-wiki",
    route: { slug: "scp-173", extra: "" },
    cookieHeader: null
  })

  assert.equal(allowed.cacheable, true)

  for (const candidate of [
    { method: "POST" },
    { routeId: "/-/admin" },
    { url: new URL("https://scp-wiki.example/scp-173?x=1") },
    { route: { slug: "scp-173", extra: "comments/show" } },
    { cookieHeader: "wikijump_token=fixture-session" },
    { siteSlug: "" }
  ]) {
    assert.equal(
      canConsiderAnonymousArticleResponseCache({
        method: "GET",
        routeId: "/[slug]/[...extra]",
        url: new URL("https://scp-wiki.example/scp-173"),
        siteId: 6000005,
        siteSlug: "scp-wiki",
        route: { slug: "scp-173", extra: "" },
        cookieHeader: null,
        ...candidate
      }).cacheable,
      false
    )
  }
})

test("anonymous article response cache key requires Deepwell eligibility metadata", () => {
  const deepwellArticlePageCacheKey =
    "deepwell:article-view:page:v1:site=6000005:page=173:rev=9:updated=123:body=aa:top=bb:side=cc:slug=7363702d313733:extra=:locales=6a612d4a502c656e2d55532c656e"
  const metadata = buildAnonymousArticleResponseCacheMetadata({
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestLocales: ["ja-JP", "en-US"],
    backendLocales: ["ja-JP", "en-US", "en"],
    deepwellArticlePageCacheKey
  })

  assert.deepEqual(metadata, {
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestLocales: ["ja-JP", "en-US"],
    backendLocales: ["ja-JP", "en-US", "en"],
    deepwellArticlePageCacheKey,
    publicContentFence: "0",
    permissionFence: "anonymous-page-view-v1"
  })

  assert.match(
    buildAnonymousArticleResponseCacheKey(metadata),
    /^framerail:article-response:v1:site=6000005:slug=7363702d77696b69:requestLocales=6a612d4a502c656e2d5553:backendLocales=6a612d4a502c656e2d55532c656e:content=0:permission=anonymous-page-view-v1:deepwell=[a-f0-9]{64}$/
  )

  assert.equal(
    buildAnonymousArticleResponseCacheMetadata({
      siteId: 6000005,
      siteSlug: "scp-wiki",
      requestLocales: ["en-US"],
      backendLocales: ["en-US", "en"],
      deepwellArticlePageCacheKey: null
    }),
    null
  )
})

test("anonymous article response cache fence helpers read Redis keys with default zero", async () => {
  assert.equal(
    buildPublicContentFenceKey(6000005),
    "deepwell:public-content:site:6000005:version"
  )
  assert.deepEqual(buildAnonymousPermissionFenceKeys(6000005), {
    siteKey: "permission:site:6000005:version",
    userKey: "permission:site:6000005:user:anonymous:version"
  })

  const store = createMemoryArticleResponseCacheStore()
  assert.deepEqual(
    await readAnonymousArticleResponseCacheFences({ store, siteId: 6000005 }),
    {
      publicContentFence: "0",
      permissionFence: "site=0,user=0"
    }
  )

  await store.set(buildPublicContentFenceKey(6000005), "7")
  await store.set("permission:site:6000005:version", "11")
  await store.set("permission:site:6000005:user:anonymous:version", "13")

  assert.deepEqual(
    await readAnonymousArticleResponseCacheFences({ store, siteId: 6000005 }),
    {
      publicContentFence: "7",
      permissionFence: "site=11,user=13"
    }
  )
})

test("anonymous article response cache fence helpers use atomic multi-key reads when available", async () => {
  const store = {
    async mget(keys) {
      assert.deepEqual(keys, [
        buildPublicContentFenceKey(6000005),
        "permission:site:6000005:version",
        "permission:site:6000005:user:anonymous:version"
      ])
      return ["7", "11", "13"]
    },
    async get() {
      throw new Error("non-atomic fence read")
    }
  }

  assert.deepEqual(
    await readAnonymousArticleResponseCacheFences({ store, siteId: 6000005 }),
    {
      publicContentFence: "7",
      permissionFence: "site=11,user=13"
    }
  )
})

test("anonymous article response cache fence helpers fail closed on malformed values", async () => {
  const store = createMemoryArticleResponseCacheStore()
  await store.set(buildPublicContentFenceKey(6000005), "not-a-version")

  assert.equal(
    await readAnonymousArticleResponseCacheFences({ store, siteId: 6000005 }),
    null
  )
})

test("anonymous article response token maps route and fences to Deepwell cache key", async () => {
  const tokenMetadata = buildAnonymousArticleResponseCacheFences({
    siteId: 6000005,
    siteSlug: "scp-wiki",
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
    /^framerail:article-response-token:v1:site=6000005:slug=7363702d77696b69:route=[a-f0-9]{64}:requestLocales=656e2d5553:backendLocales=656e2d55532c656e:content=7:permission=736974653d31312c757365723d3133$/
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
    async set() {
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

test("cached article response writes reject oversized serialized entries", async () => {
  assert.equal(Number.isInteger(ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES), true)
  const store = createMemoryArticleResponseCacheStore()

  assert.equal(
    await writeCachedArticleResponse(
      store,
      "large",
      { status: 200, headers: [], body: "x".repeat(32) },
      60,
      { maxSerializedBytes: 16 }
    ),
    false
  )
  assert.equal(await readCachedArticleResponse(store, "large"), null)
})

test("anonymous article response cache read/write helpers gate final responses", async () => {
  const metadata = buildAnonymousArticleResponseCacheMetadata({
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestLocales: ["en-US"],
    backendLocales: ["en-US", "en"],
    deepwellArticlePageCacheKey:
      "deepwell:article-view:page:v1:site=6000005:page=173:permission=site:3,user:5:body=aa"
  })
  const store = createMemoryArticleResponseCacheStore()

  assert.equal(
    await writeAnonymousArticleResponseCache({
      store,
      metadata: null,
      response: new Response("missing metadata")
    }),
    false
  )
  assert.equal(
    await writeAnonymousArticleResponseCache({
      store,
      metadata,
      response: new Response("not found", { status: 404 })
    }),
    false
  )
  assert.equal(
    await writeAnonymousArticleResponseCache({
      store,
      metadata,
      response: new Response("session", { headers: { "set-cookie": "a=b" } })
    }),
    false
  )

  assert.equal(
    await writeAnonymousArticleResponseCache({
      store,
      metadata,
      response: new Response("cached body", {
        status: 200,
        headers: { "content-type": "text/html" }
      })
    }),
    true
  )

  const cached = await readAnonymousArticleResponseCache({ store, metadata })
  assert.equal(cached.status, 200)
  assert.equal(cached.headers.get("content-type"), "text/html")
  assert.equal(await cached.text(), "cached body")
})
