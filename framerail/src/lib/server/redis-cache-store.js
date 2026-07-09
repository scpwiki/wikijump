import { Socket } from "node:net"
import { connect as tlsConnect } from "node:tls"

const DEFAULT_REDIS_PORT = 6379
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
    if (this.socket && !this.socket.destroyed) return
    if (this.connecting) {
      await this.connecting
      return
    }

    this.connecting = new Promise((resolve, reject) => {
      const url = new URL(this.redisUrl)
      const port = Number.parseInt(url.port || `${DEFAULT_REDIS_PORT}`, 10)
      const host = url.hostname
      const socket =
        url.protocol === "rediss:"
          ? tlsConnect({ host, port, servername: host })
          : new Socket().connect({ host, port })

      const onConnect = async () => {
        socket.off("error", onError)
        this.socket = socket
        this.buffer = Buffer.alloc(0)
        socket.on("data", (chunk) => this.handleData(chunk))
        socket.on("error", () => this.reset())
        socket.on("close", () => this.reset())

        try {
          for (const command of this.authCommands) {
            await this.command(command)
          }
          resolve()
        } catch (error) {
          this.reset()
          reject(error)
        }
      }
      const onError = (error) => {
        socket.destroy()
        reject(error)
      }

      socket.once(url.protocol === "rediss:" ? "secureConnect" : "connect", onConnect)
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
    if (!this.socket || this.socket.destroyed) {
      throw new Error("Redis connection unavailable")
    }

    return new Promise((resolve, reject) => {
      this.pending.push({ resolve, reject })
      this.socket.write(encodeCommand(parts), "utf8", (error) => {
        if (error) {
          this.pending = this.pending.filter((request) => request.resolve !== resolve)
          reject(error)
        }
      })
    })
  }

  async get(key) {
    return this.command(["GET", key])
  }

  async set(key, value, ttlSeconds) {
    if (Number.isInteger(ttlSeconds) && ttlSeconds > 0) {
      await this.command(["SETEX", key, ttlSeconds, value])
    } else {
      await this.command(["SET", key, value])
    }
    return true
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
