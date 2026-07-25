/** @typedef {[string, string]} HeaderPair */
/**
 * @typedef {object} CachedArticleResponseEntry
 * @property {number} status
 * @property {HeaderPair[]} headers
 * @property {string} body
 * @property {Buffer} [bodyBuffer]
 */

/**
 * @param {unknown} value
 * @returns {value is HeaderPair}
 */
export const isHeaderPair = (value) => {
  return (
    Array.isArray(value) &&
    value.length === 2 &&
    typeof value[0] === "string" &&
    typeof value[1] === "string"
  )
}

/**
 * @param {unknown} value
 * @returns {value is CachedArticleResponseEntry}
 */
const isCachedArticleResponse = (value) => {
  if (value === null || typeof value !== "object") return false
  const candidate = /** @type {Partial<CachedArticleResponseEntry>} */ (value)
  return (
    Number.isInteger(candidate.status) &&
    Number(candidate.status) >= 200 &&
    Number(candidate.status) <= 599 &&
    Array.isArray(candidate.headers) &&
    candidate.headers.every(isHeaderPair) &&
    typeof candidate.body === "string"
  )
}

/**
 * @param {unknown} value
 * @returns {CachedArticleResponseEntry | null}
 */
export const normalizeCachedArticleResponseEntry = (value) => {
  if (!isCachedArticleResponse(value)) return null

  return {
    status: value.status,
    headers: value.headers.map(([name, headerValue]) => [name, headerValue]),
    body: value.body
  }
}

/**
 * @param {CachedArticleResponseEntry} entry
 * @returns {CachedArticleResponseEntry}
 */
export const copyCachedArticleResponseEntry = (entry) => {
  /** @type {CachedArticleResponseEntry} */
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
