import {
  ARTICLE_RESPONSE_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES,
  ARTICLE_RESPONSE_CACHE_TTL_SECONDS,
  serializedByteLength
} from "./article-response-cache-shared.js"
import { normalizeCachedArticleResponseEntry } from "./article-response-cache-hot.js"

export const serializeArticleResponseForCache = async (response) => {
  return {
    status: response.status,
    headers: [...response.headers.entries()]
      .filter(([name]) => name.toLowerCase() !== "set-cookie")
      .sort(([left], [right]) => left.localeCompare(right)),
    body: await response.clone().text()
  }
}

export const createMemoryArticleResponseCacheStore = ({
  now = () => Date.now(),
  maxEntries = ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  maxBytes = ARTICLE_RESPONSE_CACHE_MAX_BYTES
} = {}) => {
  const entries = new Map()
  let totalBytes = 0
  const maxEntryCount =
    Number.isInteger(maxEntries) && maxEntries > 0
      ? maxEntries
      : ARTICLE_RESPONSE_CACHE_MAX_ENTRIES
  const maxTotalBytes =
    Number.isInteger(maxBytes) && maxBytes > 0
      ? maxBytes
      : ARTICLE_RESPONSE_CACHE_MAX_BYTES

  const deleteEntry = (key) => {
    const entry = entries.get(key)
    if (!entry) return
    totalBytes -= entry.bytes
    entries.delete(key)
  }

  const pruneExpired = (nowMs) => {
    for (const [key, entry] of entries) {
      if (entry.expiresAt <= nowMs) deleteEntry(key)
    }
  }

  const pruneOverflow = () => {
    while (entries.size > maxEntryCount || totalBytes > maxTotalBytes) {
      const oldest = entries.keys().next()
      if (oldest.done) return
      deleteEntry(oldest.value)
    }
  }

  return {
    async get(key) {
      const entry = entries.get(key)
      if (!entry) return null
      if (entry.expiresAt <= now()) {
        deleteEntry(key)
        return null
      }
      return entry.value
    },

    async set(key, value, ttlSeconds = ARTICLE_RESPONSE_CACHE_TTL_SECONDS) {
      const nowMs = now()
      const expiresAt = nowMs + ttlSeconds * 1000

      pruneExpired(nowMs)
      if (expiresAt <= nowMs) {
        deleteEntry(key)
        return false
      }

      const bytes = serializedByteLength(value)
      deleteEntry(key)
      if (bytes > maxTotalBytes) return false

      entries.set(key, { value, expiresAt, bytes })
      totalBytes += bytes
      pruneOverflow()
      return entries.has(key)
    },

    size() {
      return entries.size
    }
  }
}

export const deserializeCachedArticleResponse = (value) => {
  const entry = normalizeCachedArticleResponseEntry(value)
  if (!entry) return null

  return new Response(entry.body, {
    status: entry.status,
    headers: entry.headers
  })
}

export const readCachedArticleResponseEntry = async (store, key) => {
  try {
    const cached = await store.get(key)
    if (typeof cached !== "string") return null

    return normalizeCachedArticleResponseEntry(JSON.parse(cached))
  } catch {
    return null
  }
}

export const readCachedArticleResponse = async (store, key) => {
  return deserializeCachedArticleResponse(
    await readCachedArticleResponseEntry(store, key)
  )
}

export const writeCachedArticleResponse = async (
  store,
  key,
  entry,
  ttlSeconds,
  { maxSerializedBytes = ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES } = {}
) => {
  try {
    const serialized = JSON.stringify(entry)
    if (serializedByteLength(serialized) > maxSerializedBytes) return false

    return (await store.set(key, serialized, ttlSeconds)) !== false
  } catch {
    return false
  }
}
