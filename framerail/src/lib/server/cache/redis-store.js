import { Socket } from "node:net"
import { connect as tlsConnect } from "node:tls"

const DEFAULT_REDIS_PORT = 6379
const REDIS_CONNECT_TIMEOUT_MS = 1000
const REDIS_COMMAND_TIMEOUT_MS = 1000
const REDIS_SUBSCRIBER_RETRY_DELAY_MS = 100
const CRLF = "\r\n"

const encodeCommand = (parts) => {
  let command = `*${parts.length}${CRLF}`
  for (const part of parts) {
    const value = `${part}`
    command += `$${Buffer.byteLength(value, "utf8")}${CRLF}${value}${CRLF}`
  }
  return command
}

const parseLine = (buffer, offset) => {
  const end = buffer.indexOf(CRLF, offset, "utf8")
  if (end === -1) return null
  return {
    value: buffer.toString("utf8", offset, end),
    nextOffset: end + CRLF.length
  }
}

const parseResponse = (buffer, offset = 0) => {
  if (offset >= buffer.length) return null

  const type = String.fromCharCode(buffer[offset])
  const payloadOffset = offset + 1
  const line = parseLine(buffer, payloadOffset)
  if (!line) return null

  if (type === "+") {
    return { value: line.value, nextOffset: line.nextOffset }
  }
  if (type === "-") {
    throw new Error(line.value)
  }
  if (type === ":") {
    return { value: Number.parseInt(line.value, 10), nextOffset: line.nextOffset }
  }
  if (type === "$") {
    const length = Number.parseInt(line.value, 10)
    if (length === -1) {
      return { value: null, nextOffset: line.nextOffset }
    }
    if (!Number.isInteger(length) || length < 0) {
      throw new Error("invalid Redis bulk string length")
    }
    const end = line.nextOffset + length
    const nextOffset = end + CRLF.length
    if (buffer.length < nextOffset) return null
    return {
      value: buffer.toString("utf8", line.nextOffset, end),
      nextOffset
    }
  }
  if (type === "*") {
    const length = Number.parseInt(line.value, 10)
    if (length === -1) {
      return { value: null, nextOffset: line.nextOffset }
    }
    if (!Number.isInteger(length) || length < 0) {
      throw new Error("invalid Redis array length")
    }

    const values = []
    let nextOffset = line.nextOffset
    for (let index = 0; index < length; index += 1) {
      const parsed = parseResponse(buffer, nextOffset)
      if (!parsed) return null
      values.push(parsed.value)
      nextOffset = parsed.nextOffset
    }

    return { value: values, nextOffset }
  }
  throw new Error("unsupported Redis response type")
}

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

    this.connecting = new Promise((resolve, reject) => {
      const url = new URL(this.redisUrl)
      const port = Number.parseInt(url.port || `${DEFAULT_REDIS_PORT}`, 10)
      const host = url.hostname
      const socket =
        url.protocol === "rediss:"
          ? tlsConnect({ host, port, servername: host })
          : new Socket().connect({ host, port })
      const connectEvent = url.protocol === "rediss:" ? "secureConnect" : "connect"
      const timeout = setTimeout(() => {
        socket.destroy()
        reject(new Error("Redis connection timed out"))
      }, REDIS_CONNECT_TIMEOUT_MS)

      const cleanup = () => {
        clearTimeout(timeout)
        socket.off(connectEvent, onConnect)
        socket.off("error", onError)
      }
      const onConnect = async () => {
        cleanup()
        this.socket = socket
        this.buffer = Buffer.alloc(0)
        socket.on("data", (chunk) => this.handleData(chunk))
        socket.on("error", () => this.reset())
        socket.on("close", () => this.reset())

        try {
          for (const command of this.authCommands) {
            await this.writeCommand(command)
          }
          resolve()
        } catch (error) {
          this.reset()
          reject(error)
        }
      }
      const onError = (error) => {
        cleanup()
        socket.destroy()
        reject(error)
      }

      socket.once(connectEvent, onConnect)
      socket.once("error", onError)
    }).finally(() => {
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
        parsed = parseResponse(this.buffer)
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
      this.socket.write(encodeCommand(parts), "utf8", (error) => {
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

class RedisFenceInvalidationSubscriber {
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

    await new Promise((resolve, reject) => {
      const url = new URL(this.redisUrl)
      const port = Number.parseInt(url.port || `${DEFAULT_REDIS_PORT}`, 10)
      const host = url.hostname
      const socket =
        url.protocol === "rediss:"
          ? tlsConnect({ host, port, servername: host })
          : new Socket().connect({ host, port })
      const connectEvent = url.protocol === "rediss:" ? "secureConnect" : "connect"
      const timeout = setTimeout(() => {
        socket.destroy()
        reject(new Error("Redis subscriber connection timed out"))
      }, REDIS_CONNECT_TIMEOUT_MS)

      const cleanup = () => {
        clearTimeout(timeout)
        socket.off(connectEvent, onConnect)
        socket.off("error", onError)
      }
      const onConnect = () => {
        cleanup()
        this.socket = socket
        this.buffer = Buffer.alloc(0)
        socket.on("data", (chunk) => this.handleData(chunk))
        socket.on("error", () => this.reset())
        socket.on("close", () => this.reset())
        resolve()
      }
      const onError = (error) => {
        cleanup()
        socket.destroy()
        reject(error)
      }

      socket.once(connectEvent, onConnect)
      socket.once("error", onError)
    })
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
        parsed = parseResponse(this.buffer)
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
      this.socket.write(encodeCommand(parts), "utf8", (error) => {
        if (error) {
          this.pending = this.pending.filter((request) => request !== pendingRequest)
          pendingRequest.reject(error)
        }
      })
    })
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
