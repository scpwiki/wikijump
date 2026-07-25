import {
  attachRedisCommandSocket,
  createRedisCommandState,
  resetRedisCommandState,
  writeRedisCommand
} from "./redis-command-state.js"
import { connectRedisSocket } from "./redis-connection.js"
import { parseRedisResponse } from "./redis-protocol.js"
import { RedisFenceInvalidationSubscriber } from "./redis-subscriber.js"

const REDIS_COMMAND_TIMEOUT_MS = 1000

class RedisCacheStore {
  /** @param {string} redisUrl */
  constructor(redisUrl) {
    this.redisUrl = redisUrl
    this.commandState = createRedisCommandState()
    /** @type {Promise<void> | null} */
    this.connecting = null
    this.authCommands = this.buildAuthCommands(redisUrl)
  }

  get socket() {
    return this.commandState.socket
  }

  get pending() {
    return this.commandState.pending
  }

  /**
   * @param {string} redisUrl
   * @returns {(string | number)[][]}
   */
  buildAuthCommands(redisUrl) {
    const url = new URL(redisUrl)
    /** @type {(string | number)[][]} */
    const commands = []
    const password = decodeURIComponent(url.password)
    if (password) {
      const username = decodeURIComponent(url.username)
      commands.push(username ? ["AUTH", username, password] : ["AUTH", password])
    }

    const database = url.pathname.replace(/^\//, "")
    if (database) commands.push(["SELECT", database])
    return commands
  }

  async connect() {
    if (this.connecting) {
      await this.connecting
      return
    }
    const currentSocket = this.commandState.socket
    if (currentSocket && !currentSocket.destroyed) return

    this.connecting = (async () => {
      const socket = await connectRedisSocket(this.redisUrl, "Redis connection timed out")
      attachRedisCommandSocket({
        state: this.commandState,
        socket,
        onData: (chunk) => this.handleData(chunk),
        onDisconnect: () => this.reset()
      })

      try {
        for (const command of this.authCommands) await this.writeCommand(command)
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
    resetRedisCommandState(this.commandState, "Redis connection closed")
  }

  /** @param {Buffer} chunk */
  handleData(chunk) {
    const state = this.commandState
    state.buffer = Buffer.concat([state.buffer, chunk])

    while (state.pending.length > 0) {
      let parsed
      try {
        parsed = parseRedisResponse(state.buffer)
      } catch (error) {
        const request = state.pending.shift()
        request?.reject(error instanceof Error ? error : new Error(String(error)))
        state.buffer = Buffer.alloc(0)
        this.reset()
        return
      }
      if (!parsed) return

      state.buffer = state.buffer.subarray(parsed.nextOffset)
      state.pending.shift()?.resolve(parsed.value)
    }
  }

  /**
   * @param {(string | number)[]} parts
   * @returns {Promise<unknown>}
   */
  async command(parts) {
    await this.connect()
    return this.writeCommand(parts)
  }

  /**
   * @param {(string | number)[]} parts
   * @returns {Promise<unknown>}
   */
  writeCommand(parts) {
    return writeRedisCommand({
      state: this.commandState,
      parts,
      timeoutMs: REDIS_COMMAND_TIMEOUT_MS,
      unavailableMessage: "Redis connection unavailable",
      timeoutMessage: "Redis command timed out",
      onTimeout: () => this.reset()
    })
  }

  /** @param {string} key */
  async get(key) {
    return this.command(["GET", key])
  }

  /** @param {string[]} keys */
  async mget(keys) {
    if (!Array.isArray(keys) || keys.length === 0) return []
    const values = await this.command(["MGET", ...keys])
    return Array.isArray(values) ? values : []
  }

  /**
   * @param {string} key
   * @param {string} value
   * @param {number} [ttlSeconds]
   */
  async set(key, value, ttlSeconds) {
    if (Number.isInteger(ttlSeconds) && Number(ttlSeconds) > 0) {
      await this.command(["SETEX", key, Number(ttlSeconds), value])
    } else {
      await this.command(["SET", key, value])
    }
    return true
  }

  /** @param {import("./redis-subscriber.js").SubscriptionCallbacks} callbacks */
  subscribe(callbacks) {
    const subscriber = new RedisFenceInvalidationSubscriber(
      this.redisUrl,
      this.authCommands
    )
    subscriber.subscribe(callbacks)
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
