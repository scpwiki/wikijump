import {
  attachRedisCommandSocket,
  createRedisCommandState,
  resetRedisCommandState,
  writeRedisCommand
} from "./redis-command-state.js"
import { connectRedisSocket } from "./redis-connection.js"
import { parseRedisResponse } from "./redis-protocol.js"

const REDIS_COMMAND_TIMEOUT_MS = 1000
const REDIS_SUBSCRIBER_RETRY_DELAY_MS = 100

/** @typedef {string | number | null | unknown[]} RedisValue */
/**
 * @typedef {object} SubscriptionCallbacks
 * @property {string} channel
 * @property {() => void} [onSubscribed]
 * @property {(message: string) => void} [onMessage]
 * @property {() => void} [onDisconnect]
 * @property {() => void} [onMalformed]
 */

export class RedisFenceInvalidationSubscriber {
  /**
   * @param {string} redisUrl
   * @param {(string | number)[][]} authCommands
   */
  constructor(redisUrl, authCommands) {
    this.redisUrl = redisUrl
    this.authCommands = authCommands
    this.commandState = createRedisCommandState()
    this.started = false
    this.running = false
    /** @type {NodeJS.Timeout | null} */
    this.retryTimer = null
    this.stopped = false
    this.subscribed = false
    /** @type {string | null} */
    this.channel = null
    /** @type {(() => void) | null} */
    this.onSubscribed = null
    /** @type {((message: string) => void) | null} */
    this.onMessage = null
    /** @type {(() => void) | null} */
    this.onDisconnect = null
    /** @type {(() => void) | null} */
    this.onMalformed = null
  }

  /** @param {SubscriptionCallbacks} callbacks */
  subscribe({ channel, onSubscribed, onMessage, onDisconnect, onMalformed }) {
    if (this.started) return
    this.started = true
    this.stopped = false
    this.channel = channel
    this.onSubscribed = onSubscribed ?? null
    this.onMessage = onMessage ?? null
    this.onDisconnect = onDisconnect ?? null
    this.onMalformed = onMalformed ?? null
    void this.run()
  }

  async run() {
    if (this.running || this.stopped) return
    this.running = true
    try {
      await this.connect()
      for (const command of this.authCommands) await this.command(command)
      const channel = this.channel
      if (!channel) {
        this.reset()
        return
      }
      const response = await this.command(["SUBSCRIBE", channel])
      if (
        !Array.isArray(response) ||
        response[0] !== "subscribe" ||
        response[1] !== channel
      ) {
        this.onMalformed?.()
        this.reset()
        return
      }
      this.subscribed = true
      this.onSubscribed?.()
    } catch {
      this.reset()
    } finally {
      this.running = false
      this.scheduleRetry()
    }
  }

  scheduleRetry() {
    if (this.stopped || !this.started || this.subscribed || this.retryTimer) return
    this.retryTimer = setTimeout(() => {
      this.retryTimer = null
      void this.run()
    }, REDIS_SUBSCRIBER_RETRY_DELAY_MS)
    this.retryTimer.unref?.()
  }

  async connect() {
    const currentSocket = this.commandState.socket
    if (currentSocket && !currentSocket.destroyed) return

    const socket = await connectRedisSocket(
      this.redisUrl,
      "Redis subscriber connection timed out"
    )
    attachRedisCommandSocket({
      state: this.commandState,
      socket,
      onData: (chunk) => this.handleData(chunk),
      onDisconnect: () => this.reset()
    })
  }

  reset() {
    this.subscribed = false
    resetRedisCommandState(this.commandState, "Redis subscriber connection closed")
    this.onDisconnect?.()
    this.scheduleRetry()
  }

  close() {
    this.stopped = true
    this.started = false
    if (this.retryTimer) {
      clearTimeout(this.retryTimer)
      this.retryTimer = null
    }
    this.reset()
  }

  /** @param {Buffer} chunk */
  handleData(chunk) {
    const state = this.commandState
    state.buffer = Buffer.concat([state.buffer, chunk])

    while (state.buffer.length > 0) {
      let parsed
      try {
        parsed = parseRedisResponse(state.buffer)
      } catch (error) {
        state.pending
          .shift()
          ?.reject(error instanceof Error ? error : new Error(String(error)))
        this.onMalformed?.()
        state.buffer = Buffer.alloc(0)
        this.reset()
        return
      }
      if (!parsed) return

      state.buffer = state.buffer.subarray(parsed.nextOffset)
      const request = state.pending.shift()
      if (request) request.resolve(parsed.value)
      else this.handlePubSubMessage(parsed.value)
    }
  }

  /** @param {RedisValue} value */
  handlePubSubMessage(value) {
    if (
      Array.isArray(value) &&
      value[0] === "message" &&
      value[1] === this.channel &&
      typeof value[2] === "string"
    ) {
      this.onMessage?.(value[2])
      return
    }
    this.onMalformed?.()
  }

  /**
   * @param {(string | number)[]} parts
   * @returns {Promise<unknown>}
   */
  async command(parts) {
    await this.connect()
    return writeRedisCommand({
      state: this.commandState,
      parts,
      timeoutMs: REDIS_COMMAND_TIMEOUT_MS,
      unavailableMessage: "Redis subscriber connection unavailable",
      timeoutMessage: "Redis subscriber command timed out",
      onTimeout: () => this.reset()
    })
  }
}
