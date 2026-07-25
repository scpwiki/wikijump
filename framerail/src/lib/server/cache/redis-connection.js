import { Socket } from "node:net"
import { connect as tlsConnect } from "node:tls"

const DEFAULT_REDIS_PORT = 6379
const REDIS_CONNECT_TIMEOUT_MS = 1000

export const connectRedisSocket = (redisUrl, timeoutMessage) =>
  new Promise((resolve, reject) => {
    const url = new URL(redisUrl)
    const port = Number.parseInt(url.port || `${DEFAULT_REDIS_PORT}`, 10)
    const host = url.hostname
    const socket =
      url.protocol === "rediss:"
        ? tlsConnect({ host, port, servername: host })
        : new Socket().connect({ host, port })
    const connectEvent = url.protocol === "rediss:" ? "secureConnect" : "connect"
    const timeout = setTimeout(() => {
      socket.destroy()
      reject(new Error(timeoutMessage))
    }, REDIS_CONNECT_TIMEOUT_MS)

    const cleanup = () => {
      clearTimeout(timeout)
      socket.off(connectEvent, onConnect)
      socket.off("error", onError)
    }
    const onConnect = () => {
      cleanup()
      resolve(socket)
    }
    const onError = (error) => {
      cleanup()
      socket.destroy()
      reject(error)
    }

    socket.once(connectEvent, onConnect)
    socket.once("error", onError)
  })
