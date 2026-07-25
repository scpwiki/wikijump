import { strict as assert } from "node:assert"
import test from "node:test"

import {
  buildAnonymousArticleResponseCacheMetadata,
  createLocalArticleResponseHotCache,
  createMemoryArticleResponseCacheStore,
  readAnonymousArticleResponseCache,
  writeAnonymousArticleResponseCache
} from "../../src/lib/server/cache/article-response/index.js"

const REQUEST_HOST = "scp-wiki.example"

test("local article response hot cache keeps an isolated body replay copy", () => {
  const hotCache = createLocalArticleResponseHotCache()

  const entry = {
    status: 200,
    headers: [["content-type", "text/html"]],
    body: "<!doctype html><html><body>cached body</body></html>"
  }
  assert.equal(hotCache.store("token", entry), true)
  entry.headers[0][1] = "text/plain"
  entry.body = "mutated"

  const cached = hotCache.get("token")
  assert.equal(cached.status, 200)
  assert.deepEqual(cached.headers, [["content-type", "text/html"]])
  assert.equal(cached.body, "<!doctype html><html><body>cached body</body></html>")
})

test("local article response hot cache reuses immutable prepared replay state", () => {
  const hotCache = createLocalArticleResponseHotCache()
  const headers = [["x-final", "safe"]]
  const bodyBuffer = Buffer.from("cached body")

  assert.equal(
    hotCache.store(
      "token",
      {
        status: 200,
        headers: [["content-type", "text/html"]],
        body: "cached body"
      },
      {
        replay: {
          status: 200,
          headers,
          bodyBuffer
        }
      }
    ),
    true
  )

  headers[0][1] = "poisoned"
  bodyBuffer.write("poison")

  const firstReplay = hotCache.getReplay("token")
  const secondReplay = hotCache.getReplay("token")
  assert.notEqual(firstReplay, secondReplay)
  assert.equal(firstReplay.status, 200)
  assert.deepEqual(firstReplay.headers, [["x-final", "safe"]])
  assert.deepEqual(firstReplay.nodeRawHeaders, ["x-final", "safe"])
  assert.equal(firstReplay.bodyBuffer.toString("utf8"), "cached body")
  assert.throws(() => firstReplay.headers.push(["x-extra", "nope"]), TypeError)
  assert.throws(() => {
    firstReplay.headers[0][1] = "mutated"
  }, TypeError)
  assert.throws(() => firstReplay.nodeRawHeaders.push("x-extra", "nope"), TypeError)
  assert.throws(() => {
    firstReplay.nodeRawHeaders[1] = "mutated"
  }, TypeError)

  const publicCopy = hotCache.get("token")
  publicCopy.headers[0][1] = "mutated"
  publicCopy.bodyBuffer.write("mutated")
  assert.deepEqual(hotCache.getReplay("token").headers, [["x-final", "safe"]])
  assert.deepEqual(hotCache.getReplay("token").nodeRawHeaders, ["x-final", "safe"])
  assert.equal(hotCache.getReplay("token").bodyBuffer.toString("utf8"), "cached body")
})

test("local article response hot cache getReplay body mutation does not poison later reads", () => {
  const hotCache = createLocalArticleResponseHotCache()

  assert.equal(
    hotCache.store(
      "token",
      {
        status: 200,
        headers: [["content-type", "text/html"]],
        body: "cached body"
      },
      {
        replay: {
          status: 200,
          headers: [["x-final", "safe"]],
          bodyBuffer: Buffer.from("cached body")
        }
      }
    ),
    true
  )

  const replay = hotCache.getReplay("token")
  replay.bodyBuffer.write("poison")

  assert.equal(hotCache.getReplay("token").bodyBuffer.toString("utf8"), "cached body")
  assert.equal(hotCache.get("token").bodyBuffer.toString("utf8"), "cached body")
})

test("local article response hot cache exposes trusted shared replay without copying", () => {
  const hotCache = createLocalArticleResponseHotCache()

  assert.equal(
    hotCache.store(
      "token",
      {
        status: 200,
        headers: [["content-type", "text/html"]],
        body: "cached body"
      },
      {
        replay: {
          status: 200,
          headers: [["x-final", "safe"]],
          bodyBuffer: Buffer.from("cached body")
        }
      }
    ),
    true
  )

  const firstSharedReplay = hotCache.getSharedReplayForInternalUse("token")
  const secondSharedReplay = hotCache.getSharedReplayForInternalUse("token")
  const publicReplay = hotCache.getReplay("token")

  assert.equal(firstSharedReplay, secondSharedReplay)
  assert.equal(firstSharedReplay.bodyBuffer, secondSharedReplay.bodyBuffer)
  assert.notEqual(publicReplay, firstSharedReplay)
  assert.notEqual(publicReplay.bodyBuffer, firstSharedReplay.bodyBuffer)
  assert.deepEqual(firstSharedReplay.headers, [["x-final", "safe"]])
  assert.deepEqual(firstSharedReplay.nodeRawHeaders, ["x-final", "safe"])
  assert.equal(firstSharedReplay.bodyBuffer.toString("utf8"), "cached body")
  assert.throws(() => firstSharedReplay.headers.push(["x-extra", "nope"]), TypeError)
  assert.throws(() => {
    firstSharedReplay.headers[0][1] = "mutated"
  }, TypeError)
  assert.throws(() => firstSharedReplay.nodeRawHeaders.push("x-extra", "nope"), TypeError)
  assert.throws(() => {
    firstSharedReplay.nodeRawHeaders[1] = "mutated"
  }, TypeError)
})

test("local article response hot cache protects public replay variant buffers", () => {
  const hotCache = createLocalArticleResponseHotCache()
  const brBody = Buffer.from("brotli replay")
  const gzipBody = Buffer.from("gzip replay")

  assert.equal(
    hotCache.store(
      "token",
      {
        status: 200,
        headers: [["content-type", "text/html"]],
        body: "cached body"
      },
      {
        replay: {
          status: 200,
          headers: [["content-type", "text/html"]],
          bodyBuffer: Buffer.from("cached body"),
          variants: {
            br: {
              headers: [["content-encoding", "br"]],
              bodyBuffer: brBody
            },
            gzip: {
              headers: [["content-encoding", "gzip"]],
              bodyBuffer: gzipBody
            }
          }
        }
      }
    ),
    true
  )

  brBody.write("poison")
  gzipBody.write("poison")

  const firstReplay = hotCache.getReplay("token")
  const secondReplay = hotCache.getReplay("token")
  assert.equal(firstReplay.variants.br.bodyBuffer.toString("utf8"), "brotli replay")
  assert.equal(firstReplay.variants.gzip.bodyBuffer.toString("utf8"), "gzip replay")
  assert.notEqual(firstReplay.variants.br.bodyBuffer, secondReplay.variants.br.bodyBuffer)
  firstReplay.variants.br.bodyBuffer.write("mutate")
  assert.equal(
    hotCache.getReplay("token").variants.br.bodyBuffer.toString("utf8"),
    "brotli replay"
  )
  assert.throws(() => {
    firstReplay.variants.br.headers[0][1] = "mutated"
  }, TypeError)
})

test("local article response hot cache shares internal replay variant buffers", () => {
  const hotCache = createLocalArticleResponseHotCache()

  assert.equal(
    hotCache.store(
      "token",
      {
        status: 200,
        headers: [["content-type", "text/html"]],
        body: "cached body"
      },
      {
        replay: {
          status: 200,
          headers: [["content-type", "text/html"]],
          bodyBuffer: Buffer.from("cached body"),
          variants: {
            br: {
              headers: [["content-encoding", "br"]],
              bodyBuffer: Buffer.from("brotli replay")
            }
          }
        }
      }
    ),
    true
  )

  const firstSharedReplay = hotCache.getSharedReplayForInternalUse("token")
  const secondSharedReplay = hotCache.getSharedReplayForInternalUse("token")
  assert.equal(firstSharedReplay.variants.br, secondSharedReplay.variants.br)
  assert.equal(
    firstSharedReplay.variants.br.bodyBuffer,
    secondSharedReplay.variants.br.bodyBuffer
  )
  assert.equal(firstSharedReplay.variants.br.bodyBuffer.toString("utf8"), "brotli replay")
})

test("local article response hot cache byte accounting includes replay variants", () => {
  const key = "token"
  const entry = {
    status: 200,
    headers: [["content-type", "text/html"]],
    body: "cached body"
  }
  const identityBytes =
    Buffer.byteLength(key, "utf8") +
    8 +
    Buffer.byteLength(entry.body, "utf8") +
    Buffer.byteLength("content-type", "utf8") +
    Buffer.byteLength("text/html", "utf8") +
    4
  const hotCache = createLocalArticleResponseHotCache({
    maxBytes: identityBytes + 100
  })

  assert.equal(
    hotCache.store(key, entry, {
      replay: {
        status: 200,
        headers: [["content-type", "text/html"]],
        bodyBuffer: Buffer.from(entry.body),
        variants: {
          br: {
            headers: [["content-encoding", "br"]],
            bodyBuffer: Buffer.alloc(128)
          }
        }
      }
    }),
    false
  )
  assert.equal(hotCache.size(), 0)
})

test("anonymous article response cache read/write helpers gate final responses", async () => {
  const metadata = buildAnonymousArticleResponseCacheMetadata({
    siteId: 6000005,
    siteSlug: "scp-wiki",
    requestHost: REQUEST_HOST,
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
