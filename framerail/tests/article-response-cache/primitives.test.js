import { strict as assert } from "node:assert"
import test from "node:test"

import { createByteLimitedCache } from "../../src/lib/server/cache/article-response/byte-limited-cache.js"
import { normalizeCachedArticleResponseEntry } from "../../src/lib/server/cache/article-response/entry.js"
import { createByteLimitedLru } from "../../src/lib/server/cache/article-response/local-lru.js"
import { normalizeCachedArticleResponseReplay } from "../../src/lib/server/cache/article-response/replay.js"
import {
  ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES,
  createMemoryArticleResponseCacheStore,
  readCachedArticleResponse,
  writeCachedArticleResponse
} from "../../src/lib/server/cache/article-response/index.js"

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

test("byte-limited cache preserves insertion order when reads do not touch", () => {
  let now = 0
  const cache = createByteLimitedCache({
    now: () => now,
    maxEntries: 2,
    maxBytes: 10
  })

  assert.equal(cache.insert("first", "a", 1, 10), true)
  assert.equal(cache.insert("second", "b", 1, 10), true)
  assert.equal(cache.get("first"), "a")
  assert.equal(cache.insert("third", "c", 1, 10), true)
  assert.equal(cache.get("first"), null)
  assert.equal(cache.get("second"), "b")

  now = 11
  assert.equal(cache.get("second"), null)
  assert.equal(cache.get("third"), null)
  assert.equal(cache.size(), 0)
})

test("byte-limited LRU tracks recency, capacity, and expiry", () => {
  let now = 0
  const cache = createByteLimitedLru({
    now: () => now,
    ttlMs: 10,
    maxEntries: 2,
    maxBytes: 10
  })

  assert.equal(cache.insert("first", "a", 1), true)
  assert.equal(cache.insert("second", "b", 1), true)
  assert.equal(cache.get("first"), "a")
  assert.equal(cache.insert("third", "c", 1), true)
  assert.equal(cache.get("second"), null)
  assert.equal(cache.get("first"), "a")

  now = 11
  assert.equal(cache.get("first"), null)
  assert.equal(cache.size(), 1)
  cache.clear()
  assert.equal(cache.size(), 0)
})

test("cached article response entry normalization validates and copies headers", () => {
  const headers = [["content-type", "text/html"]]
  const normalized = normalizeCachedArticleResponseEntry({
    status: 200,
    headers,
    body: "cached body"
  })

  assert.deepEqual(normalized, {
    status: 200,
    headers: [["content-type", "text/html"]],
    body: "cached body"
  })
  headers[0][1] = "text/plain"
  assert.deepEqual(normalized.headers, [["content-type", "text/html"]])
  assert.equal(
    normalizeCachedArticleResponseEntry({ status: 200, headers: ["bad"], body: "x" }),
    null
  )
})

test("cached article response replay normalization prepares immutable transport state", () => {
  const replay = normalizeCachedArticleResponseReplay(
    {
      status: 200,
      headers: [["content-type", "text/html"]],
      body: "cached body"
    },
    {
      status: 200,
      headers: [["x-final", "safe"]],
      bodyBuffer: Buffer.from("cached body")
    }
  )

  assert.equal(replay.status, 200)
  assert.deepEqual(replay.headers, [["x-final", "safe"]])
  assert.deepEqual(replay.nodeRawHeaders, ["x-final", "safe"])
  assert.equal(replay.bodyBuffer.toString("utf8"), "cached body")
  assert.throws(() => replay.headers.push(["x-extra", "nope"]), TypeError)
})
