import { connectRedisSocket } from "./redis-connection.js"
import { encodeRedisCommand, parseRedisResponse } from "./redis-protocol.js"
import { RedisFenceInvalidationSubscriber } from "./redis-subscriber.js"

const REDIS_COMMAND_TIMEOUT_MS = 1000

class RedisCacheStore {
  constructor(redisUrl) {
    this.redisUrl = redisUrl
    this.socket = null
    this.buffer = Buffer.alloc(0)
    this.pending = []
    this.connecting = null
    this.authCommands = this.buildAuthCommands(redisUrl)
  }

  buildAuthCommands(redisUrl) {
    const url = new URL(redisUrl)
    const commands = []
    const password = decodeURIComponent(url.password)
    if (password) {
      const username = decodeURIComponent(url.username)
      commands.push(username ? ["AUTH", username, password] : ["AUTH", password])
    }

    const database = url.pathname.replace(/^\//, "")
    if (database) {
      commands.push(["SELECT", database])
    }

    return commands
  }

  async connect() {
    if (this.connecting) {
      await this.connecting
      return
    }
    if (this.socket && !this.socket.destroyed) return

    this.connecting = (async () => {
      const socket = await connectRedisSocket(this.redisUrl, "Redis connection timed out")
      this.socket = socket
      this.buffer = Buffer.alloc(0)
      socket.on("data", (chunk) => this.handleData(chunk))
      const resetIfCurrent = () => {
        if (this.socket === socket) this.reset()
      }
      socket.on("error", resetIfCurrent)
      socket.on("close", resetIfCurrent)

      try {
        for (const command of this.authCommands) {
          await this.writeCommand(command)
        }
      } catch (error) {
        this.reset()
        throw error
      }
    })().finally(() => {
      this.connecting = null
    })

    await this.connecting
  }

  reset() {
    if (this.socket && !this.socket.destroyed) {
      this.socket.destroy()
    }
    this.socket = null
    this.buffer = Buffer.alloc(0)
    const pending = this.pending
    this.pending = []
    for (const request of pending) {
      request.reject(new Error("Redis connection closed"))
    }
  }

  handleData(chunk) {
    this.buffer = Buffer.concat([this.buffer, chunk])

    while (this.pending.length > 0) {
      let parsed
      try {
        parsed = parseRedisResponse(this.buffer)
      } catch (error) {
        const request = this.pending.shift()
        request.reject(error)
        this.buffer = Buffer.alloc(0)
        this.reset()
        return
      }

      if (!parsed) return

      this.buffer = this.buffer.subarray(parsed.nextOffset)
      const request = this.pending.shift()
      request.resolve(parsed.value)
    }
  }

  async command(parts) {
    await this.connect()
    return this.writeCommand(parts)
  }

  writeCommand(parts) {
    if (!this.socket || this.socket.destroyed) {
      throw new Error("Redis connection unavailable")
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending = this.pending.filter((request) => request !== pendingRequest)
        reject(new Error("Redis command timed out"))
        this.reset()
      }, REDIS_COMMAND_TIMEOUT_MS)
      const pendingRequest = {
        resolve: (value) => {
          clearTimeout(timeout)
          resolve(value)
        },
        reject: (error) => {
          clearTimeout(timeout)
          reject(error)
        }
      }
      this.pending.push(pendingRequest)
      this.socket.write(encodeRedisCommand(parts), "utf8", (error) => {
        if (error) {
          this.pending = this.pending.filter((request) => request !== pendingRequest)
          pendingRequest.reject(error)
        }
      })
    })
  }

  async get(key) {
    return this.command(["GET", key])
  }

  async mget(keys) {
    if (!Array.isArray(keys) || keys.length === 0) return []
    const values = await this.command(["MGET", ...keys])
    return Array.isArray(values) ? values : []
  }

  async set(key, value, ttlSeconds) {
    if (Number.isInteger(ttlSeconds) && ttlSeconds > 0) {
      await this.command(["SETEX", key, ttlSeconds, value])
    } else {
      await this.command(["SET", key, value])
    }
    return true
  }

  subscribe({ channel, onSubscribed, onMessage, onDisconnect, onMalformed }) {
    const subscriber = new RedisFenceInvalidationSubscriber(
      this.redisUrl,
      this.authCommands
    )
    subscriber.subscribe({ channel, onSubscribed, onMessage, onDisconnect, onMalformed })
    return subscriber
  }
}

export const createRedisCacheStore = (redisUrl = process.env.REDIS_URL) => {
  if (!redisUrl) return null
  try {
    const url = new URL(redisUrl)
    if (url.protocol !== "redis:" && url.protocol !== "rediss:") return null
    return new RedisCacheStore(redisUrl)
  } catch {
    return null
  }
}
