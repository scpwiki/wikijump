import { strict as assert } from "node:assert"
import net from "node:net"
import test from "node:test"

import {
  createRedisCommandState,
  resetRedisCommandState,
  writeRedisCommand
} from "../../src/lib/server/cache/redis-command-state.js"
import { connectRedisSocket } from "../../src/lib/server/cache/redis-connection.js"
import {
  encodeRedisCommand,
  parseRedisResponse
} from "../../src/lib/server/cache/redis-protocol.js"
import { createRedisCacheStore } from "../../src/lib/server/cache/redis-store.js"
import { RedisFenceInvalidationSubscriber } from "../../src/lib/server/cache/redis-subscriber.js"

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
    if (Date.now() - started > timeoutMs) throw new Error(message)
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
      for (const socket of sockets) socket.destroy()
      await new Promise((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()))
      })
    }
  }
}

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

test("Redis command state owns pending request cleanup", async () => {
  let destroyed = false
  let written = ""
  const socket = /** @type {import("node:net").Socket} */ (
    /** @type {unknown} */ ({
      get destroyed() {
        return destroyed
      },
      write(command) {
        written = `${command}`
        return true
      },
      destroy() {
        destroyed = true
        return this
      }
    })
  )
  const state = createRedisCommandState()
  state.socket = socket
  const command = writeRedisCommand({
    state,
    parts: ["GET", "cache-key"],
    timeoutMs: 1000,
    unavailableMessage: "Redis fixture unavailable",
    timeoutMessage: "Redis fixture timed out",
    onTimeout() {
      throw new Error("fixture command should be reset before timeout")
    }
  })

  assert.match(written, /^\*2\r\n\$3\r\nGET\r\n/u)
  assert.equal(state.pending.length, 1)
  resetRedisCommandState(state, "Redis fixture closed")

  await assert.rejects(command, { message: "Redis fixture closed" })
  assert.equal(state.pending.length, 0)
  assert.equal(state.socket, null)
  assert.equal(destroyed, true)
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
    socket.on("data", () => {})
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
    for (const socket of sockets) socket.destroy()
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
    store.reset()
    const second = store.get("cache-key")
    await waitFor(
      () => store.pending.length > 0,
      "the reconnected command should be in flight"
    )
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
    for (const socket of sockets) socket.destroy()
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
    socket.on("data", () => {})
  })
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  const store = createRedisCacheStore(`redis://:cache-password@127.0.0.1:${address.port}`)

  try {
    await assert.rejects(store.get("cache-key"), { message: "Redis command timed out" })
  } finally {
    for (const socket of sockets) socket.destroy()
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
