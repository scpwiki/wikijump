import { strict as assert } from "node:assert"
import test from "node:test"

import { parseFenceInvalidationMessage } from "../../src/lib/server/cache/article-response/fence-message.js"
import { applyFenceInvalidationToSites } from "../../src/lib/server/cache/article-response/fence-reducer.js"
import { createArticleResponseFenceState } from "../../src/lib/server/cache/article-response/fence-state.js"
import {
  normalizeFenceVersion,
  parsePermissionFence
} from "../../src/lib/server/cache/article-response/fence-values.js"
import {
  buildAnonymousPermissionFenceKeys,
  buildPublicContentFenceKey,
  readAnonymousArticleResponseCacheFences
} from "../../src/lib/server/cache/article-response/fences.js"
import {
  buildAnonymousArticleResponseCacheKey,
  buildAnonymousArticleResponseCacheMetadata,
  canConsiderAnonymousArticleResponseCache,
  createMemoryArticleResponseCacheStore
} from "../../src/lib/server/cache/article-response/index.js"

const REQUEST_HOST = "scp-wiki.example"

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
    {
      requestUrl: new URL(
        "https://scp-wiki.example/scp-173/__data.json?x-sveltekit-invalidated=01"
      )
    },
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
    requestHost: REQUEST_HOST,
    requestLocales: ["ja-JP", "en-US"],
    backendLocales: ["ja-JP", "en-US", "en"],
    deepwellArticlePageCacheKey
  })

  assert.deepEqual(metadata, {
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestHost: REQUEST_HOST,
    requestLocales: ["ja-JP", "en-US"],
    backendLocales: ["ja-JP", "en-US", "en"],
    deepwellArticlePageCacheKey,
    publicContentFence: "0",
    permissionFence: "anonymous-page-view-v1"
  })

  assert.match(
    buildAnonymousArticleResponseCacheKey(metadata),
    /^framerail:article-response:v1:site=6000005:slug=7363702d77696b69:host=7363702d77696b692e6578616d706c65:requestLocales=6a612d4a502c656e2d5553:backendLocales=6a612d4a502c656e2d55532c656e:content=0:permission=anonymous-page-view-v1:deepwell=[a-f0-9]{64}$/
  )

  assert.equal(
    buildAnonymousArticleResponseCacheMetadata({
      siteId: 6000005,
      siteSlug: "scp-wiki",
      requestHost: REQUEST_HOST,
      requestLocales: ["en-US"],
      backendLocales: ["en-US", "en"],
      deepwellArticlePageCacheKey: null
    }),
    null
  )
})

test("article response fence values normalize stored versions", () => {
  assert.equal(normalizeFenceVersion(undefined), "0")
  assert.equal(normalizeFenceVersion("17"), "17")
  assert.equal(normalizeFenceVersion("invalid"), null)
  assert.deepEqual(parsePermissionFence("site=11,user=13"), {
    sitePermissionFence: "11",
    userPermissionFence: "13"
  })
  assert.equal(parsePermissionFence("site=11"), null)
})

test("article response fence invalidation messages are validated before use", () => {
  assert.deepEqual(
    parseFenceInvalidationMessage(
      JSON.stringify({ type: "public-content", site_id: 6000005, version: "8" })
    ),
    { type: "public-content", siteId: 6000005, version: "8" }
  )
  assert.deepEqual(
    parseFenceInvalidationMessage(
      JSON.stringify({
        type: "anonymous-permission",
        site_id: 6000005,
        site_version: "12",
        user_version: "13"
      })
    ),
    {
      type: "anonymous-permission",
      siteId: 6000005,
      siteVersion: "12",
      userVersion: "13"
    }
  )
  assert.deepEqual(
    parseFenceInvalidationMessage(
      JSON.stringify({
        type: "user-permission",
        site_id: 6000005,
        user_id: 123,
        version: "19"
      })
    ),
    { type: "user-permission" }
  )
  assert.equal(parseFenceInvalidationMessage("{not-json"), null)
  assert.equal(
    parseFenceInvalidationMessage(
      JSON.stringify({ type: "public-content", site_id: 6000005, version: "bad" })
    ),
    null
  )
})

test("article response fence reducer updates only advancing versions", () => {
  const sites = new Map([
    [
      6000005,
      {
        publicContentFence: "7",
        sitePermissionFence: "11",
        userPermissionFence: "13"
      }
    ]
  ])
  let invalidations = 0
  const clearHotResponses = () => {
    invalidations += 1
  }

  applyFenceInvalidationToSites({
    sites,
    message: { type: "public-content", siteId: 6000005, version: "7" },
    clearHotResponses
  })
  assert.equal(invalidations, 0)

  applyFenceInvalidationToSites({
    sites,
    message: {
      type: "anonymous-permission",
      siteId: 6000005,
      siteVersion: "12",
      userVersion: "13"
    },
    clearHotResponses
  })
  assert.deepEqual(sites.get(6000005), {
    publicContentFence: "7",
    sitePermissionFence: "12",
    userPermissionFence: "13"
  })
  assert.equal(invalidations, 1)
})

test("article response fence state tracks trusted local versions", () => {
  let invalidations = 0
  const state = createArticleResponseFenceState({
    clearHotResponses: () => {
      invalidations += 1
    }
  })

  assert.equal(state.isTrusted(), false)
  state.markTrusted()
  assert.equal(state.isTrusted(), true)
  assert.deepEqual(
    state.seedSite({
      siteId: 6000005,
      seedRevision: state.revision(),
      fences: { publicContentFence: "7", permissionFence: "site=11,user=13" }
    }),
    {
      publicContentFence: "7",
      sitePermissionFence: "11",
      userPermissionFence: "13"
    }
  )
  assert.equal(
    state.areFencesCurrent({
      siteId: 6000005,
      publicContentFence: "7",
      permissionFence: "site=11,user=13"
    }),
    true
  )

  state.applyMessage({ type: "public-content", siteId: 6000005, version: "8" })
  assert.deepEqual(state.readFences(6000005), {
    publicContentFence: "8",
    permissionFence: "site=11,user=13"
  })
  assert.equal(invalidations, 1)

  state.applyMessage({
    type: "anonymous-permission",
    siteId: 6000005,
    siteVersion: "12",
    userVersion: "13"
  })
  assert.deepEqual(state.readFences(6000005), {
    publicContentFence: "8",
    permissionFence: "site=12,user=13"
  })
  assert.equal(invalidations, 2)

  state.poison()
  assert.equal(state.isTrusted(), false)
  assert.equal(state.readFences(6000005), null)
  assert.equal(invalidations, 3)
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
