import { strict as assert } from "node:assert"
import test from "node:test"

import { createMemoryArticleResponseCacheStore } from "../../src/lib/server/cache/article-response/index.js"
import {
  createCountingStore,
  createFastPathFixtureStore,
  fastPathHeaders,
  // eslint-disable-next-line no-redeclare
  fetch,
  seedFastPathStoreEntry,
  withServer
} from "./helpers.js"

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
