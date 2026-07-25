/**
 * @template T
 * @typedef {object} ByteLimitedCacheEntry
 * @property {T} value
 * @property {number} expiresAt
 * @property {number} bytes
 */

/**
 * @template T
 * @param {{
 *   now: () => number
 *   maxEntries: number
 *   maxBytes: number
 *   touchOnRead?: boolean
 * }} options
 */
export const createByteLimitedCache = ({
  now,
  maxEntries,
  maxBytes,
  touchOnRead = false
}) => {
  /** @type {Map<string, ByteLimitedCacheEntry<T>>} */
  const entries = new Map()
  let totalBytes = 0

  /** @param {string} key */
  const deleteEntry = (key) => {
    const entry = entries.get(key)
    if (!entry) return
    totalBytes -= entry.bytes
    entries.delete(key)
  }

  /** @param {number} nowMs */
  const pruneExpired = (nowMs) => {
    for (const [key, entry] of entries) {
      if (entry.expiresAt <= nowMs) deleteEntry(key)
    }
  }

  const pruneOverflow = () => {
    while (entries.size > maxEntries || totalBytes > maxBytes) {
      const oldest = entries.keys().next()
      if (oldest.done) return
      deleteEntry(oldest.value)
    }
  }

  /**
   * @param {string} key
   * @returns {T | null}
   */
  const get = (key) => {
    const entry = entries.get(key)
    if (!entry) return null
    if (entry.expiresAt <= now()) {
      deleteEntry(key)
      return null
    }
    if (touchOnRead) {
      entries.delete(key)
      entries.set(key, entry)
    }
    return entry.value
  }

  /**
   * @param {string} key
   * @param {T} value
   * @param {number} bytes
   * @param {number} expiresAt
   */
  const insert = (key, value, bytes, expiresAt) => {
    const nowMs = now()
    pruneExpired(nowMs)
    deleteEntry(key)
    if (expiresAt <= nowMs || bytes > maxBytes) return false

    entries.set(key, { value, expiresAt, bytes })
    totalBytes += bytes
    pruneOverflow()
    return entries.has(key)
  }

  return {
    get,
    insert,
    delete: deleteEntry,
    size: () => entries.size,
    clear() {
      entries.clear()
      totalBytes = 0
    }
  }
}
