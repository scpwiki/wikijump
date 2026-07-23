import {
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS,
  serializedByteLength
} from "./article-response-cache-shared.js"

const isHeaderPair = (value) => {
  return (
    Array.isArray(value) &&
    value.length === 2 &&
    typeof value[0] === "string" &&
    typeof value[1] === "string"
  )
}

const isCachedArticleResponse = (value) => {
  return (
    value !== null &&
    typeof value === "object" &&
    Number.isInteger(value.status) &&
    value.status >= 200 &&
    value.status <= 599 &&
    Array.isArray(value.headers) &&
    value.headers.every(isHeaderPair) &&
    typeof value.body === "string"
  )
}

export const normalizeCachedArticleResponseEntry = (value) => {
  if (!isCachedArticleResponse(value)) return null

  return {
    status: value.status,
    headers: value.headers.map(([name, headerValue]) => [name, headerValue]),
    body: value.body
  }
}

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

const normalizeCachedArticleResponseReplay = (entry, replay) => {
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

const copyCachedArticleResponseReplay = (replay) => {
  return Object.freeze({
    status: replay.status,
    headers: replay.headers,
    nodeRawHeaders: replay.nodeRawHeaders,
    bodyBuffer: Buffer.from(replay.bodyBuffer),
    variants: copyReplayVariants(replay.variants),
    finalHeaders: replay.finalHeaders
  })
}

const copyCachedArticleResponseEntry = (entry) => {
  const copy = {
    status: entry.status,
    headers: entry.headers.map(([name, value]) => [name, value]),
    body: entry.body
  }
  if (Buffer.isBuffer(entry.bodyBuffer)) {
    copy.bodyBuffer = Buffer.from(entry.bodyBuffer)
  }
  return copy
}

const cachedArticleResponseEntryByteLength = (key, entry, replay) => {
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

export const createLocalArticleResponseHotCache = ({
  now = () => Date.now(),
  ttlMs = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS,
  maxEntries = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES,
  maxBytes = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES
} = {}) => {
  const entries = new Map()
  let totalBytes = 0
  const maxEntryCount =
    Number.isInteger(maxEntries) && maxEntries > 0
      ? maxEntries
      : ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES
  const maxTotalBytes =
    Number.isInteger(maxBytes) && maxBytes > 0
      ? maxBytes
      : ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES
  const entryTtlMs =
    Number.isFinite(ttlMs) && ttlMs > 0 ? ttlMs : ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS

  const deleteEntry = (key) => {
    const entry = entries.get(key)
    if (!entry) return
    totalBytes -= entry.bytes
    entries.delete(key)
  }

  const pruneExpired = (nowMs) => {
    for (const [key, entry] of entries) {
      if (entry.expiresAt <= nowMs) {
        deleteEntry(key)
      }
    }
  }

  const pruneOverflow = () => {
    while (entries.size > maxEntryCount || totalBytes > maxTotalBytes) {
      const oldest = entries.keys().next()
      if (oldest.done) return
      deleteEntry(oldest.value)
    }
  }

  const getRecord = (key) => {
    const entry = entries.get(key)
    if (!entry) return null

    if (entry.expiresAt <= now()) {
      deleteEntry(key)
      return null
    }

    entries.delete(key)
    entries.set(key, entry)
    return entry
  }

  return {
    get(key) {
      const entry = getRecord(key)
      if (!entry) return null
      return copyCachedArticleResponseEntry(entry.value)
    },

    getReplay(key) {
      const entry = getRecord(key)
      if (!entry) return null
      return copyCachedArticleResponseReplay(entry.value.replay)
    },

    getSharedReplayForInternalUse(key) {
      const entry = getRecord(key)
      if (!entry) return null
      return entry.value.replay
    },

    set(key, value, { replay } = {}) {
      if (typeof key !== "string" || key.length === 0) return false

      const normalized = normalizeCachedArticleResponseEntry(value)
      if (!normalized) {
        deleteEntry(key)
        return false
      }
      const bodyBuffer = Buffer.from(normalized.body, "utf8")
      const preparedReplay = normalizeCachedArticleResponseReplay(
        { ...normalized, bodyBuffer },
        replay
      )
      if (!preparedReplay) {
        deleteEntry(key)
        return false
      }

      const nowMs = now()
      const expiresAt = nowMs + entryTtlMs
      const bytes = cachedArticleResponseEntryByteLength(key, normalized, preparedReplay)

      pruneExpired(nowMs)
      deleteEntry(key)
      if (bytes > maxTotalBytes) return false

      const cachedValue = copyCachedArticleResponseEntry({
        ...normalized,
        bodyBuffer
      })
      cachedValue.replay = preparedReplay
      entries.set(key, {
        value: cachedValue,
        expiresAt,
        bytes
      })
      totalBytes += bytes
      pruneOverflow()
      return entries.has(key)
    },

    size() {
      return entries.size
    },

    clear() {
      entries.clear()
      totalBytes = 0
    }
  }
}
