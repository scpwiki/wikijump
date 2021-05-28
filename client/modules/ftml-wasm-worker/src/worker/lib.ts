/* Exports the various functions misc. functions needed for the worker. */

import { Transfer, TransferDescriptor } from "threads"

const decoder = new TextDecoder()
const encoder = new TextEncoder()

export { expose, Transfer as transferMultiple } from "threads/worker"

export function encode(buffer: string | ArrayBufferLike | ArrayBufferView) {
  if (typeof buffer === "string") return encoder.encode(buffer).buffer
  if ("buffer" in buffer) return buffer.buffer
  if (buffer instanceof ArrayBuffer) return buffer
  throw new TypeError("Expected a string, ArrayBuffer, or typed array!")
}

export function decode(buffer: ArrayBuffer) {
  return decoder.decode(buffer)
}

export function transfer(raw: string | ArrayBufferLike | ArrayBufferView) {
  return Transfer(encode(raw)) as TransferDescriptor<ArrayBuffer>
}
