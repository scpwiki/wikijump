import { isHeaderPair } from "./entry.js"
import { serializedByteLength } from "./shared.js"

/** @typedef {[string, string]} HeaderPair */
/** @typedef {"br" | "gzip"} ReplayEncoding */
/**
 * @typedef {object} CachedArticleResponseEntry
 * @property {number} status
 * @property {HeaderPair[]} headers
 * @property {string} body
 */
/**
 * @typedef {object} ReplayVariantInput
 * @property {HeaderPair[]} headers
 * @property {Buffer} bodyBuffer
 */
/** @typedef {{ br?: ReplayVariantInput; gzip?: ReplayVariantInput }} ReplayVariantsInput */
/**
 * @typedef {object} ReplayInput
 * @property {number} [status]
 * @property {HeaderPair[]} [headers]
 * @property {Buffer} [bodyBuffer]
 * @property {ReplayVariantsInput} [variants]
 * @property {boolean} [finalHeaders]
 */
/**
 * @typedef {object} PreparedReplayVariant
 * @property {ReadonlyArray<Readonly<HeaderPair>>} headers
 * @property {ReadonlyArray<string>} nodeRawHeaders
 * @property {Buffer} bodyBuffer
 */
/**
 * @typedef {Readonly<
 *   Partial<Record<ReplayEncoding, PreparedReplayVariant>>
 * >} PreparedReplayVariants
 */
/**
 * @typedef {object} PreparedReplay
 * @property {number} status
 * @property {ReadonlyArray<Readonly<HeaderPair>>} headers
 * @property {ReadonlyArray<string>} nodeRawHeaders
 * @property {Buffer} bodyBuffer
 * @property {PreparedReplayVariants | undefined} variants
 * @property {boolean} finalHeaders
 */

const REPLAY_ENCODINGS = /** @type {const} */ (["br", "gzip"])

/**
 * @param {HeaderPair[]} headers
 * @returns {ReadonlyArray<Readonly<HeaderPair>>}
 */
const freezeHeaderEntries = (headers) => {
  return Object.freeze(
    headers.map(([name, value]) =>
      Object.freeze(/** @type {HeaderPair} */ ([name, value]))
    )
  )
}

/**
 * @param {HeaderPair[]} headers
 * @returns {ReadonlyArray<string>}
 */
const freezeNodeRawHeaders = (headers) => {
  return Object.freeze(headers.flatMap(([name, value]) => [name, value]))
}

/**
 * @param {unknown} variants
 * @returns {PreparedReplayVariants | null | undefined}
 */
const normalizeReplayVariants = (variants) => {
  if (variants === undefined) return undefined
  if (variants === null || typeof variants !== "object") return null

  const replayVariants = /** @type {ReplayVariantsInput} */ (variants)
  /** @type {Partial<Record<ReplayEncoding, PreparedReplayVariant>>} */
  const normalized = {}
  for (const encoding of REPLAY_ENCODINGS) {
    const variant = replayVariants[encoding]
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

/**
 * @param {CachedArticleResponseEntry} entry
 * @param {ReplayInput | undefined} replay
 * @returns {PreparedReplay | null}
 */
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

/**
 * @param {PreparedReplayVariants | undefined} variants
 * @returns {PreparedReplayVariants | undefined}
 */
const copyReplayVariants = (variants) => {
  if (variants === undefined) return undefined
  /** @type {Partial<Record<ReplayEncoding, PreparedReplayVariant>>} */
  const copy = {}
  for (const encoding of REPLAY_ENCODINGS) {
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

/**
 * @param {PreparedReplay} replay
 * @returns {PreparedReplay}
 */
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

/**
 * @param {string} key
 * @param {CachedArticleResponseEntry} entry
 * @param {PreparedReplay | undefined} replay
 */
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
