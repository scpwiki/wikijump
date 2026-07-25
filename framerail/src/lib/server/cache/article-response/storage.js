import {
  ARTICLE_RESPONSE_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES,
  ARTICLE_RESPONSE_CACHE_TTL_SECONDS,
  serializedByteLength
} from "./shared.js"
import { createByteLimitedCache } from "./byte-limited-cache.js"
import { normalizeCachedArticleResponseEntry } from "./entry.js"

/**
 * @typedef {object} ArticleResponseCacheStore
 * @property {(key: string) => Promise<unknown>} get
 * @property {(
 *   key: string,
 *   value: string,
 *   ttlSeconds?: number
 * ) => Promise<unknown>} set
 */

/**
 * @param {Response} response
 * @returns {Promise<
 *   NonNullable<ReturnType<typeof normalizeCachedArticleResponseEntry>>
 * >}
 */
export const serializeArticleResponseForCache = async (response) => {
  return {
    status: response.status,
    headers: [...response.headers.entries()]
      .filter(([name]) => name.toLowerCase() !== "set-cookie")
      .sort(([left], [right]) => left.localeCompare(right)),
    body: await response.clone().text()
  }
}

/**
 * @param {{
 *   now?: () => number
 *   maxEntries?: number
 *   maxBytes?: number
 * }} [options]
 * @returns {ArticleResponseCacheStore & { size: () => number }}
 */
export const createMemoryArticleResponseCacheStore = ({
  now = () => Date.now(),
  maxEntries = ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  maxBytes = ARTICLE_RESPONSE_CACHE_MAX_BYTES
} = {}) => {
  const maxEntryCount =
    Number.isInteger(maxEntries) && maxEntries > 0
      ? maxEntries
      : ARTICLE_RESPONSE_CACHE_MAX_ENTRIES
  const maxTotalBytes =
    Number.isInteger(maxBytes) && maxBytes > 0
      ? maxBytes
      : ARTICLE_RESPONSE_CACHE_MAX_BYTES
  /**
   * @type {{
   *   get: (key: string) => string | null
   *   insert: (
   *     key: string,
   *     value: string,
   *     bytes: number,
   *     expiresAt: number
   *   ) => boolean
   *   size: () => number
   * }}
   */
  const entries = createByteLimitedCache({
    now,
    maxEntries: maxEntryCount,
    maxBytes: maxTotalBytes
  })

  return {
    /** @param {string} key */
    async get(key) {
      return entries.get(key)
    },

    /**
     * @param {string} key
     * @param {string} value
     * @param {number} [ttlSeconds]
     */
    async set(key, value, ttlSeconds) {
      const effectiveTtlSeconds = ttlSeconds ?? ARTICLE_RESPONSE_CACHE_TTL_SECONDS
      return entries.insert(
        key,
        value,
        serializedByteLength(value),
        now() + effectiveTtlSeconds * 1000
      )
    },

    size: entries.size
  }
}

/** @param {unknown} value */
export const deserializeCachedArticleResponse = (value) => {
  const entry = normalizeCachedArticleResponseEntry(value)
  if (!entry) return null

  return new Response(entry.body, {
    status: entry.status,
    headers: entry.headers
  })
}

/**
 * @param {ArticleResponseCacheStore} store
 * @param {string} key
 */
export const readCachedArticleResponseEntry = async (store, key) => {
  try {
    const cached = await store.get(key)
    if (typeof cached !== "string") return null

    return normalizeCachedArticleResponseEntry(JSON.parse(cached))
  } catch {
    return null
  }
}

/**
 * @param {ArticleResponseCacheStore} store
 * @param {string} key
 */
export const readCachedArticleResponse = async (store, key) => {
  return deserializeCachedArticleResponse(
    await readCachedArticleResponseEntry(store, key)
  )
}

/**
 * @param {ArticleResponseCacheStore} store
 * @param {string} key
 * @param {NonNullable<
 *   ReturnType<typeof normalizeCachedArticleResponseEntry>
 * >} entry
 * @param {number} ttlSeconds
 * @param {{ maxSerializedBytes?: number }} [options]
 */
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
