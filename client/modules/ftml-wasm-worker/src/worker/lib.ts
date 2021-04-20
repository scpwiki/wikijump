/**
 * @file Exports the various functions misc. functions needed for the worker.
 */

import { Transfer } from "threads/worker"

interface TypedArray extends ArrayBuffer {
  buffer: ArrayBufferLike
}
type TransferInput = string | ArrayBuffer | TypedArray

const decoder = new TextDecoder()
const encoder = new TextEncoder()

export { expose, Transfer } from "threads/worker"

export const encode = (buffer: TransferInput) => {
  if (typeof buffer === "string") return encoder.encode(buffer).buffer
  if ("buffer" in buffer) return buffer.buffer
  if (buffer instanceof ArrayBuffer) return buffer
  throw new TypeError("Expected a string, ArrayBuffer, or typed array!")
}

export const decode = (buffer: ArrayBuffer) => decoder.decode(buffer)
