import { connectRedisSocket } from "./redis-connection.js"
import { encodeRedisCommand, parseRedisResponse } from "./redis-protocol.js"

const REDIS_COMMAND_TIMEOUT_MS = 1000
const REDIS_SUBSCRIBER_RETRY_DELAY_MS = 100

export class RedisFenceInvalidationSubscriber {
  constructor(redisUrl, authCommands) {
    this.redisUrl = redisUrl
    this.authCommands = authCommands
    this.socket = null
    this.buffer = Buffer.alloc(0)
    this.pending = []
    this.started = false
    this.running = false
    this.retryTimer = null
    this.stopped = false
    this.subscribed = false
    this.channel = null
    this.onSubscribed = null
    this.onMessage = null
    this.onDisconnect = null
    this.onMalformed = null
  }

  subscribe({ channel, onSubscribed, onMessage, onDisconnect, onMalformed }) {
    if (this.started) return
    this.started = true
    this.stopped = false
    this.channel = channel
    this.onSubscribed = onSubscribed
    this.onMessage = onMessage
    this.onDisconnect = onDisconnect
    this.onMalformed = onMalformed
    void this.run()
  }

  async run() {
    if (this.running || this.stopped) return
    this.running = true
    try {
      await this.connect()
      for (const command of this.authCommands) {
        await this.command(command)
      }
      const response = await this.command(["SUBSCRIBE", this.channel])
      if (
        !Array.isArray(response) ||
        response[0] !== "subscribe" ||
        response[1] !== this.channel
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
    if (this.socket && !this.socket.destroyed) return

    const socket = await connectRedisSocket(
      this.redisUrl,
      "Redis subscriber connection timed out"
    )
    this.socket = socket
    this.buffer = Buffer.alloc(0)
    socket.on("data", (chunk) => this.handleData(chunk))
    const resetIfCurrent = () => {
      if (this.socket === socket) this.reset()
    }
    socket.on("error", resetIfCurrent)
    socket.on("close", resetIfCurrent)
  }

  reset() {
    this.subscribed = false
    if (this.socket && !this.socket.destroyed) {
      this.socket.destroy()
    }
    this.socket = null
    this.buffer = Buffer.alloc(0)
    const pending = this.pending
    this.pending = []
    for (const request of pending) {
      request.reject(new Error("Redis subscriber connection closed"))
    }
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

  handleData(chunk) {
    this.buffer = Buffer.concat([this.buffer, chunk])

    while (this.buffer.length > 0) {
      let parsed
      try {
        parsed = parseRedisResponse(this.buffer)
      } catch (error) {
        const request = this.pending.shift()
        if (request) request.reject(error)
        this.onMalformed?.()
        this.buffer = Buffer.alloc(0)
        this.reset()
        return
      }

      if (!parsed) return

      this.buffer = this.buffer.subarray(parsed.nextOffset)
      const request = this.pending.shift()
      if (request) {
        request.resolve(parsed.value)
      } else {
        this.handlePubSubMessage(parsed.value)
      }
    }
  }

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

  async command(parts) {
    await this.connect()
    if (!this.socket || this.socket.destroyed) {
      throw new Error("Redis subscriber connection unavailable")
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending = this.pending.filter((request) => request !== pendingRequest)
        reject(new Error("Redis subscriber command timed out"))
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
}
