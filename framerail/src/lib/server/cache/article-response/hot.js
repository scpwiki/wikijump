import {
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS
} from "./shared.js"
import {
  copyCachedArticleResponseEntry,
  normalizeCachedArticleResponseEntry
} from "./entry.js"
import { createByteLimitedLru } from "./local-lru.js"
import {
  cachedArticleResponseEntryByteLength,
  copyCachedArticleResponseReplay,
  normalizeCachedArticleResponseReplay
} from "./replay.js"

const positiveIntegerOr = (value, fallback) => {
  return Number.isInteger(value) && value > 0 ? value : fallback
}

const positiveNumberOr = (value, fallback) => {
  return Number.isFinite(value) && value > 0 ? value : fallback
}

export const createLocalArticleResponseHotCache = ({
  now = () => Date.now(),
  ttlMs = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS,
  maxEntries = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES,
  maxBytes = ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES
} = {}) => {
  const records = createByteLimitedLru({
    now,
    ttlMs: positiveNumberOr(ttlMs, ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS),
    maxEntries: positiveIntegerOr(
      maxEntries,
      ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES
    ),
    maxBytes: positiveIntegerOr(maxBytes, ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES)
  })

  return {
    get(key) {
      const value = records.get(key)
      return value ? copyCachedArticleResponseEntry(value) : null
    },

    getReplay(key) {
      const value = records.get(key)
      return value ? copyCachedArticleResponseReplay(value.replay) : null
    },

    getSharedReplayForInternalUse(key) {
      return records.get(key)?.replay ?? null
    },

    store(key, value, { replay } = {}) {
      if (typeof key !== "string" || key.length === 0) return false

      const normalized = normalizeCachedArticleResponseEntry(value)
      if (!normalized) {
        records.delete(key)
        return false
      }
      const bodyBuffer = Buffer.from(normalized.body, "utf8")
      const preparedReplay = normalizeCachedArticleResponseReplay(
        { ...normalized, bodyBuffer },
        replay
      )
      if (!preparedReplay) {
        records.delete(key)
        return false
      }

      const cachedValue = copyCachedArticleResponseEntry({
        ...normalized,
        bodyBuffer
      })
      cachedValue.replay = preparedReplay
      return records.insert(
        key,
        cachedValue,
        cachedArticleResponseEntryByteLength(key, normalized, preparedReplay)
      )
    },

    size: () => records.size(),
    clear: () => records.clear()
  }
}
