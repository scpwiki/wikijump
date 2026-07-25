import { encodeRedisCommand } from "./redis-protocol.js"

/**
 * @typedef {object} PendingRedisRequest
 * @property {(value: unknown) => void} resolve
 * @property {(error: Error) => void} reject
 */
/**
 * @typedef {object} RedisCommandState
 * @property {import("node:net").Socket | import("node:tls").TLSSocket | null} socket
 * @property {Buffer} buffer
 * @property {PendingRedisRequest[]} pending
 */

/** @returns {RedisCommandState} */
export const createRedisCommandState = () => ({
  socket: null,
  buffer: Buffer.alloc(0),
  pending: []
})

/**
 * @param {{
 *   state: RedisCommandState
 *   socket: import("node:net").Socket | import("node:tls").TLSSocket
 *   onData: (chunk: Buffer) => void
 *   onDisconnect: () => void
 * }} input
 */
export const attachRedisCommandSocket = ({ state, socket, onData, onDisconnect }) => {
  state.socket = socket
  state.buffer = Buffer.alloc(0)
  socket.on("data", onData)
  const disconnectIfCurrent = () => {
    if (state.socket === socket) onDisconnect()
  }
  socket.on("error", disconnectIfCurrent)
  socket.on("close", disconnectIfCurrent)
}

/**
 * @param {RedisCommandState} state
 * @param {string} closedMessage
 */
export const resetRedisCommandState = (state, closedMessage) => {
  const socket = state.socket
  state.socket = null
  state.buffer = Buffer.alloc(0)
  const pending = state.pending
  state.pending = []
  if (socket && !socket.destroyed) socket.destroy()
  for (const request of pending) {
    request.reject(new Error(closedMessage))
  }
}

/**
 * @param {{
 *   state: RedisCommandState
 *   parts: (string | number)[]
 *   timeoutMs: number
 *   unavailableMessage: string
 *   timeoutMessage: string
 *   onTimeout: () => void
 * }} input
 * @returns {Promise<unknown>}
 */
export const writeRedisCommand = ({
  state,
  parts,
  timeoutMs,
  unavailableMessage,
  timeoutMessage,
  onTimeout
}) => {
  const socket = state.socket
  if (!socket || socket.destroyed) throw new Error(unavailableMessage)

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      state.pending = state.pending.filter((request) => request !== pendingRequest)
      reject(new Error(timeoutMessage))
      onTimeout()
    }, timeoutMs)
    /** @type {PendingRedisRequest} */
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
    state.pending.push(pendingRequest)
    socket.write(encodeRedisCommand(parts), "utf8", (error) => {
      if (!error) return
      state.pending = state.pending.filter((request) => request !== pendingRequest)
      pendingRequest.reject(error)
    })
  })
}
