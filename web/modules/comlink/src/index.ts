import * as Comlink from "comlink"

const decoder = new TextDecoder()
const encoder = new TextEncoder()

/** Convert a string or generic buffer into an `ArrayBuffer` that can be transferred. */
export function encode(buffer: string | ArrayBufferLike | ArrayBufferView) {
  if (typeof buffer === "string") return encoder.encode(buffer).buffer
  if ("buffer" in buffer) return buffer.buffer
  if (buffer instanceof ArrayBuffer) return buffer
  throw new TypeError("Expected a string, ArrayBuffer, or typed array!")
}

/** Decode an `ArrayBuffer` into a string. */
export function decode(buffer: ArrayBuffer) {
  return decoder.decode(buffer)
}

/**
 * Marks a transferable buffer as such, so not to be serialized and
 * deserialized on messaging with the main thread, but rather to transfer
 * ownership of it to the receiving thread.
 */
export function transferBuffer(raw: string | ArrayBufferLike | ArrayBufferView) {
  const encoded = encode(raw)
  return Comlink.transfer(encoded, [encoded])
}

export const transfer = Comlink.transfer

export type {
  Endpoint,
  Local,
  LocalObject,
  ProxyMarked,
  ProxyMethods,
  ProxyOrClone,
  Remote,
  RemoteObject,
  TransferHandler,
  UnproxyOrClone
} from "comlink"
export { Comlink }

// set up the transfer handlers

// we're going to setup a special one for fast serializing strings
Comlink.transferHandlers.set("string", {
  canHandle(value: unknown): value is string {
    // let's not bother with small strings
    return typeof value === "string" && value.length > 512
  },

  serialize(value: string) {
    const encoded = encode(value)
    return [encoded, [encoded]]
  },

  deserialize(value: [ArrayBuffer, Transferable[]]) {
    return decode(value[0])
  }
})

// and another one which handles arrays of strings
Comlink.transferHandlers.set("string[]", {
  canHandle(value: unknown): value is string[] {
    if (!Array.isArray(value)) return false
    let hasLargeString = false
    for (const item of value) {
      if (typeof item !== "string") return false
      if (item.length > 512) hasLargeString = true
    }
    return hasLargeString
  },

  serialize(value: string[]) {
    const encoded = value.map(encode)
    return [encoded, [...encoded]]
  },

  deserialize(value: [ArrayBuffer[], Transferable[]]) {
    return value[0].map(decode)
  }
})
