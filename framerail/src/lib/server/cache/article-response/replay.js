import { isHeaderPair } from "./entry.js"
import { serializedByteLength } from "./shared.js"

const freezeHeaderEntries = (headers) => {
  return Object.freeze(headers.map(([name, value]) => Object.freeze([name, value])))
}

const freezeNodeRawHeaders = (headers) => {
  return Object.freeze(headers.flatMap(([name, value]) => [name, value]))
}

const normalizeReplayVariants = (variants) => {
  if (variants === undefined) return undefined
  if (variants === null || typeof variants !== "object") return null

  const normalized = {}
  for (const encoding of ["br", "gzip"]) {
    const variant = variants[encoding]
    if (variant === undefined) continue
    if (
      !variant ||
      typeof variant !== "object" ||
      !Array.isArray(variant.headers) ||
      !variant.headers.every(isHeaderPair) ||
      !Buffer.isBuffer(variant.bodyBuffer)
    ) {
      return null
    }
    normalized[encoding] = Object.freeze({
      headers: freezeHeaderEntries(variant.headers),
      nodeRawHeaders: freezeNodeRawHeaders(variant.headers),
      bodyBuffer: Buffer.from(variant.bodyBuffer)
    })
  }

  return Object.freeze(normalized)
}

export const normalizeCachedArticleResponseReplay = (entry, replay) => {
  const status = replay?.status ?? entry.status
  const headers = replay?.headers ?? entry.headers
  const bodyBuffer = replay?.bodyBuffer
  const variants = replay?.variants

  if (status !== entry.status) return null
  if (!Array.isArray(headers) || !headers.every(isHeaderPair)) return null
  if (bodyBuffer !== undefined && !Buffer.isBuffer(bodyBuffer)) return null
  const replayBodyBuffer =
    bodyBuffer === undefined ? Buffer.from(entry.body, "utf8") : Buffer.from(bodyBuffer)
  const normalizedVariants = normalizeReplayVariants(variants)
  if (normalizedVariants === null) return null

  return Object.freeze({
    status,
    headers: freezeHeaderEntries(headers.map(([name, value]) => [name, value])),
    nodeRawHeaders: freezeNodeRawHeaders(headers),
    bodyBuffer: replayBodyBuffer,
    variants: normalizedVariants,
    finalHeaders: replay?.finalHeaders === true
  })
}

const copyReplayVariants = (variants) => {
  if (variants === undefined) return undefined
  const copy = {}
  for (const encoding of ["br", "gzip"]) {
    const variant = variants[encoding]
    if (!variant) continue
    copy[encoding] = Object.freeze({
      headers: variant.headers,
      nodeRawHeaders: variant.nodeRawHeaders,
      bodyBuffer: Buffer.from(variant.bodyBuffer)
    })
  }
  return Object.freeze(copy)
}

export const copyCachedArticleResponseReplay = (replay) => {
  return Object.freeze({
    status: replay.status,
    headers: replay.headers,
    nodeRawHeaders: replay.nodeRawHeaders,
    bodyBuffer: Buffer.from(replay.bodyBuffer),
    variants: copyReplayVariants(replay.variants),
    finalHeaders: replay.finalHeaders
  })
}

export const cachedArticleResponseEntryByteLength = (key, entry, replay) => {
  let bytes = serializedByteLength(key) + 8
  bytes += serializedByteLength(entry.body)
  for (const [name, value] of entry.headers) {
    bytes += serializedByteLength(name) + serializedByteLength(value) + 4
  }
  for (const variant of Object.values(replay?.variants ?? {})) {
    bytes += variant.bodyBuffer.byteLength
    for (const [name, value] of variant.headers) {
      bytes += serializedByteLength(name) + serializedByteLength(value) + 4
    }
  }
  return bytes
}
