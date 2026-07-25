import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildPublicContentFenceKey,
  createMemoryArticleResponseFenceCache
} from "../../src/lib/server/cache/article-response/index.js"
import {
  SITE_ID,
  createCountingStore,
  createDeferredGetStore,
  createFastPathFixtureStore,
  createTrustedFenceCache,
  fastPathHeaders,
  // eslint-disable-next-line no-redeclare
  fetch,
  withServer
} from "./helpers.js"

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
    store(_key, _value, _options = {}) {
      void _options
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
