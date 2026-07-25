export const createByteLimitedLru = ({ now, ttlMs, maxEntries, maxBytes }) => {
  const entries = new Map()
  let totalBytes = 0

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
    while (entries.size > maxEntries || totalBytes > maxBytes) {
      const oldest = entries.keys().next()
      if (oldest.done) return
      deleteEntry(oldest.value)
    }
  }

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
