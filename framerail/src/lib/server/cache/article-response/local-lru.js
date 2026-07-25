/**
 * @template T
 * @typedef {object} LruEntry
 * @property {T} value
 * @property {number} expiresAt
 * @property {number} bytes
 */

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
  /** @type {Map<string, LruEntry<T>>} */
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

    entries.delete(key)
    entries.set(key, entry)
    return entry.value
  }

  /**
   * @param {string} key
   * @param {T} value
   * @param {number} bytes
   */
  const insert = (key, value, bytes) => {
    const nowMs = now()
    pruneExpired(nowMs)
    deleteEntry(key)
    if (bytes > maxBytes) return false

    entries.set(key, {
      value,
      expiresAt: nowMs + ttlMs,
      bytes
    })
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
