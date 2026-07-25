import { strict as assert } from "node:assert"
import net from "node:net"
import test from "node:test"

import { normalizeCachedArticleResponseEntry } from "../src/lib/server/cache/article-response/entry.js"
import { parseFenceInvalidationMessage } from "../src/lib/server/cache/article-response/fence-message.js"
import { applyFenceInvalidationToSites } from "../src/lib/server/cache/article-response/fence-reducer.js"
import { createArticleResponseFenceState } from "../src/lib/server/cache/article-response/fence-state.js"
import {
  normalizeFenceVersion,
  parsePermissionFence
} from "../src/lib/server/cache/article-response/fence-values.js"
import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousPermissionFenceKeys,
  buildPublicContentFenceKey,
  createMemoryArticleResponseFenceCache,
  readAnonymousArticleResponseCacheFences
} from "../src/lib/server/cache/article-response/fences.js"
import { createByteLimitedLru } from "../src/lib/server/cache/article-response/local-lru.js"
import { normalizeCachedArticleResponseReplay } from "../src/lib/server/cache/article-response/replay.js"
import {
  ARTICLE_RESPONSE_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES,
  buildAnonymousArticleResponseCacheKey,
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  canConsiderAnonymousArticleResponseCache,
  createLocalArticleResponseHotCache,
  createMemoryArticleResponseCacheStore,
  deserializeCachedArticleResponse,
  readAnonymousArticleResponseCache,
  readAnonymousArticleResponseToken,
  readCachedArticleResponse,
  serializeArticleResponseForCache,
  writeAnonymousArticleResponseToken,
  writeAnonymousArticleResponseCache,
  writeCachedArticleResponse
} from "../src/lib/server/cache/article-response/index.js"
import { connectRedisSocket } from "../src/lib/server/cache/redis-connection.js"
import {
  encodeRedisCommand,
  parseRedisResponse
} from "../src/lib/server/cache/redis-protocol.js"
import { createRedisCacheStore } from "../src/lib/server/cache/redis-store.js"
import { RedisFenceInvalidationSubscriber } from "../src/lib/server/cache/redis-subscriber.js"

const REQUEST_HOST = "scp-wiki.example"

const redisArray = (...values) => {
  let response = `*${values.length}\r\n`
  for (const value of values) {
    response += `$${Buffer.byteLength(value, "utf8")}\r\n${value}\r\n`
  }
  return response
}

const waitFor = async (condition, message, timeoutMs = 1500) => {
  const started = Date.now()
  while (!condition()) {
    if (Date.now() - started > timeoutMs) {
      throw new Error(message)
    }
    await new Promise((resolve) => setTimeout(resolve, 10))
  }
}

const getUnusedPort = async () => {
  const server = net.createServer()
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  await new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()))
  })
  return address.port
}

const createPubSubRedisServer = async ({
  channel,
  port = 0,
  onSocket,
  onSubscribe
} = {}) => {
  const sockets = new Set()
  const server = net.createServer((socket) => {
    sockets.add(socket)
    socket.on("close", () => sockets.delete(socket))
    onSocket?.(socket)
    socket.setEncoding("utf8")
    let buffer = ""
    socket.on("data", (chunk) => {
      buffer += chunk
      if (!buffer.includes("SUBSCRIBE")) return
      buffer = ""
      onSubscribe?.(socket)
      socket.write(redisArray("subscribe", channel, "1"))
    })
  })

  await new Promise((resolve) => server.listen(port, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")

  return {
    port: address.port,
    close: async () => {
      for (const socket of sockets) {
        socket.destroy()
      }
      await new Promise((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()))
      })
    }
  }
}

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

test("Redis protocol encodes commands and parses nested replies", () => {
  assert.equal(
    encodeRedisCommand(["GET", "cache-key"]),
    "*2\r\n$3\r\nGET\r\n$9\r\ncache-key\r\n"
  )
  assert.deepEqual(parseRedisResponse(Buffer.from("*2\r\n$5\r\nvalue\r\n:7\r\n")), {
    value: ["value", 7],
    nextOffset: 19
  })
})

test("Redis connection helper opens a plain socket", async () => {
  const server = net.createServer()
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  const socket = await connectRedisSocket(
    `redis://127.0.0.1:${address.port}`,
    "Redis connection timed out"
  )

  try {
    assert.equal(socket.destroyed, false)
  } finally {
    socket.destroy()
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()))
    })
  }
})

test("Redis cache store times out unanswered commands", async () => {
  const sockets = new Set()
  const server = net.createServer((socket) => {
    sockets.add(socket)
    socket.on("close", () => sockets.delete(socket))
    socket.on("data", () => {
      // Keep the socket open while withholding a Redis response.
    })
  })
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  const store = createRedisCacheStore(`redis://127.0.0.1:${address.port}`)

  try {
    await assert.rejects(store.get("deepwell:public-content:site:6000005:version"), {
      message: "Redis command timed out"
    })
  } finally {
    for (const socket of sockets) {
      socket.destroy()
    }
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()))
    })
  }
})

test("Redis cache store reconnects after a command timeout", async () => {
  const sockets = new Set()
  let connections = 0
  const server = net.createServer((socket) => {
    connections += 1
    sockets.add(socket)
    socket.on("close", () => sockets.delete(socket))
    socket.on("data", () => {
      if (connections > 1) socket.write("$5\r\nvalue\r\n")
    })
  })
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  const store = createRedisCacheStore(`redis://127.0.0.1:${address.port}`)

  try {
    await assert.rejects(store.get("cache-key"), { message: "Redis command timed out" })
    assert.equal(await store.get("cache-key"), "value")
    assert.equal(connections, 2)
  } finally {
    for (const socket of sockets) socket.destroy()
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()))
    })
  }
})

test("Redis cache store ignores a superseded socket's close event", async () => {
  const sockets = new Set()
  let connections = 0
  let releaseSecond
  const secondAllowed = new Promise((resolve) => {
    releaseSecond = resolve
  })
  const server = net.createServer((socket) => {
    connections += 1
    const connection = connections
    sockets.add(socket)
    socket.on("close", () => sockets.delete(socket))
    socket.on("data", () => {
      if (connection === 1) socket.write("$5\r\nvalue\r\n")
      else void secondAllowed.then(() => socket.write("$5\r\nvalue\r\n"))
    })
  })
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  const store = createRedisCacheStore(`redis://127.0.0.1:${address.port}`)

  try {
    assert.equal(await store.get("cache-key"), "value")
    const superseded = store.socket
    assert.ok(superseded)

    // Drop the connection the way a command timeout does, then reconnect.
    store.reset()
    const second = store.get("cache-key")
    await waitFor(
      () => store.pending.length > 0,
      "the reconnected command should be in flight"
    )

    // The dead socket's `close` arrives after the new one is serving. It must
    // not reject the command the new connection is holding.
    superseded.emit("close")
    releaseSecond()

    assert.equal(await second, "value")
    assert.equal(connections, 2)
  } finally {
    releaseSecond()
    for (const socket of sockets) socket.destroy()
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()))
    })
  }
})

test("Redis cache store authenticates and selects a database before commands", async () => {
  const sockets = new Set()
  let authReceivedResolve
  const authReceived = new Promise((resolve) => {
    authReceivedResolve = resolve
  })
  let allowAuthResolve
  const allowAuth = new Promise((resolve) => {
    allowAuthResolve = resolve
  })
  let commands = 0
  const server = net.createServer((socket) => {
    sockets.add(socket)
    socket.on("close", () => sockets.delete(socket))
    socket.on("data", (chunk) => {
      commands += 1
      const command = chunk.toString("utf8")
      if (commands === 1) {
        assert.match(command, /AUTH.*cache-password/s)
        authReceivedResolve()
        void allowAuth.then(() => socket.write("+OK\r\n"))
      } else if (commands === 2) {
        assert.match(command, /SELECT.*2/s)
        socket.write("+OK\r\n")
      } else {
        assert.match(command, /GET.*cache-key/s)
        const getCount = command.match(/\r\nGET\r\n/g)?.length ?? 0
        socket.write("$5\r\nvalue\r\n".repeat(getCount))
      }
    })
  })
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  const store = createRedisCacheStore(
    `redis://:cache-password@127.0.0.1:${address.port}/2`
  )

  try {
    const firstRead = store.get("cache-key")
    await authReceived
    const concurrentRead = store.get("cache-key")
    await new Promise((resolve) => setImmediate(resolve))
    const commandsBeforeAuth = commands
    allowAuthResolve()
    assert.equal(
      commandsBeforeAuth,
      1,
      "concurrent command bypassed Redis initialization"
    )
    assert.deepEqual(await Promise.all([firstRead, concurrentRead]), ["value", "value"])
  } finally {
    for (const socket of sockets) {
      socket.destroy()
    }
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()))
    })
  }
})

test("Redis cache store times out unanswered authentication", async () => {
  const sockets = new Set()
  const server = net.createServer((socket) => {
    sockets.add(socket)
    socket.on("close", () => sockets.delete(socket))
    socket.on("data", () => {
      // Keep the socket open while withholding the AUTH response.
    })
  })
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  const store = createRedisCacheStore(`redis://:cache-password@127.0.0.1:${address.port}`)

  try {
    await assert.rejects(store.get("cache-key"), {
      message: "Redis command timed out"
    })
  } finally {
    for (const socket of sockets) {
      socket.destroy()
    }
    await new Promise((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()))
    })
  }
})

test("Redis article response fence subscriber retries after initial subscribe failure", async () => {
  const channel = "test:article-response-fence"
  const port = await getUnusedPort()
  const redisUrl = `redis://127.0.0.1:${port}`
  let disconnects = 0
  let subscribed = 0

  const subscriber = new RedisFenceInvalidationSubscriber(redisUrl, [])
  subscriber.subscribe({
    channel,
    onSubscribed: () => {
      subscribed += 1
    },
    onMessage: () => {},
    onDisconnect: () => {
      disconnects += 1
    },
    onMalformed: () => {}
  })

  await waitFor(() => disconnects > 0, "subscriber did not fail closed")
  const redis = await createPubSubRedisServer({ channel, port })

  try {
    await waitFor(() => subscribed === 1, "subscriber did not retry subscription")
    assert.equal(disconnects >= 1, true)
  } finally {
    subscriber.close()
    await redis.close()
  }
})

test("Redis article response fence subscriber resubscribes after socket close", async () => {
  const channel = "test:article-response-fence"
  let firstSocket = null
  let subscribed = 0
  let disconnects = 0
  const redis = await createPubSubRedisServer({
    channel,
    onSocket: (socket) => {
      firstSocket ??= socket
    }
  })
  const subscriber = new RedisFenceInvalidationSubscriber(
    `redis://127.0.0.1:${redis.port}`,
    []
  )
  subscriber.subscribe({
    channel,
    onSubscribed: () => {
      subscribed += 1
    },
    onMessage: () => {},
    onDisconnect: () => {
      disconnects += 1
    },
    onMalformed: () => {}
  })

  try {
    await waitFor(() => subscribed === 1, "subscriber did not subscribe")
    firstSocket.destroy()
    await waitFor(() => disconnects > 0, "subscriber did not fail closed")
    await waitFor(() => subscribed === 2, "subscriber did not resubscribe")
  } finally {
    subscriber.close()
    await redis.close()
  }
})

test("memory article response fence cache closes subscriber handle", () => {
  let closed = 0
  const fenceCache = createMemoryArticleResponseFenceCache({
    subscriber: {
      subscribe() {
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
