import { createByteLimitedCache } from "./byte-limited-cache.js"

/**
 * @template T
 * @param {{
 *   now: () => number
 *   ttlMs: number
 *   maxEntries: number
 *   maxBytes: number
 * }} options
 */
export const createByteLimitedLru = ({ now, ttlMs, maxEntries, maxBytes }) => {
  const cache = createByteLimitedCache({
    now,
    maxEntries,
    maxBytes,
    touchOnRead: true
  })

  return {
    /** @param {string} key */
    get(key) {
      return cache.get(key)
    },
    /**
     * @param {string} key
     * @param {T} value
     * @param {number} bytes
     */
    insert(key, value, bytes) {
      return cache.insert(key, value, bytes, now() + ttlMs)
    },
    /** @param {string} key */
    delete(key) {
      cache.delete(key)
    },
    size() {
      return cache.size()
    },
    clear() {
      cache.clear()
    }
  }
}
