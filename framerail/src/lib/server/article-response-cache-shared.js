import { createHash } from "node:crypto"

export const ARTICLE_RESPONSE_CACHE_MAX_ENTRIES = 1024
export const ARTICLE_RESPONSE_CACHE_MAX_BYTES = 32 * 1024 * 1024
export const ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES = 1024 * 1024
export const ARTICLE_RESPONSE_CACHE_TTL_SECONDS = 60
export const ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS = 5000
export const ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES = 1024
export const ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES = 8 * 1024 * 1024
export const PUBLIC_CONTENT_FENCE_PREFIX = "deepwell:public-content:site"
export const ARTICLE_RESPONSE_FENCE_INVALIDATION_CHANNEL =
  "wikijump:article-response-fence-invalidation:v1"
const SESSION_COOKIE = "wikijump_token"

export const hasSessionCookie = (cookieHeader) => {
  if (!cookieHeader) return false

  return cookieHeader
    .split(";")
    .map((cookie) => cookie.trim())
    .some(
      (cookie) => cookie === SESSION_COOKIE || cookie.startsWith(`${SESSION_COOKIE}=`)
    )
}

export const utf8Hex = (value) => {
  return Buffer.from(value, "utf8").toString("hex")
}

export const sha256Hex = (value) => {
  return createHash("sha256").update(value).digest("hex")
}

export const serializedByteLength = (value) => {
  return Buffer.byteLength(value, "utf8")
}
