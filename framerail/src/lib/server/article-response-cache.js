import {
  ARTICLE_RESPONSE_CACHE_TTL_SECONDS,
  hasSessionCookie,
  sha256Hex,
  utf8Hex
} from "./article-response-cache-shared.js"
import { readAnonymousArticleResponseCacheFences } from "./article-response-cache-fences.js"
import {
  readCachedArticleResponse,
  readCachedArticleResponseEntry,
  serializeArticleResponseForCache,
  writeCachedArticleResponse
} from "./article-response-cache-storage.js"

const ARTICLE_ROUTES = new Set(["/", "/[slug]/[...extra]"])
const PERMISSION_FENCE = "anonymous-page-view-v1"
const RESPONSE_CACHE_PREFIX = "framerail:article-response:v1"
const RESPONSE_TOKEN_PREFIX = "framerail:article-response-token:v1"

const isEmptyExtra = (extra) => {
  return extra === undefined || extra === null || extra === ""
}

const reject = (reason) => {
  return { cacheable: false, reason }
}

export const canConsiderAnonymousArticleResponseCache = ({
  method,
  routeId,
  url,
  siteId,
  siteSlug,
  route,
  cookieHeader
}) => {
  if (method !== "GET") return reject("method")
  if (!ARTICLE_ROUTES.has(routeId ?? "")) return reject("route")
  if (url.search !== "") return reject("query")
  if (!Number.isInteger(siteId) || siteId <= 0) return reject("site-id")
  if (!siteSlug) return reject("site-slug")
  if (hasSessionCookie(cookieHeader)) return reject("session-cookie")
  if (!isEmptyExtra(route?.extra)) return reject("extra")

  return { cacheable: true }
}

export const buildAnonymousArticleResponseCacheMetadata = ({
  siteId,
  siteSlug,
  requestHost,
  requestLocales,
  backendLocales,
  deepwellArticlePageCacheKey,
  publicContentFence = "0",
  permissionFence = PERMISSION_FENCE
}) => {
  if (!Number.isInteger(siteId) || siteId <= 0) return null
  if (!siteSlug) return null
  if (typeof requestHost !== "string" || requestHost.length === 0) return null
  if (!Array.isArray(requestLocales) || !Array.isArray(backendLocales)) return null
  if (!deepwellArticlePageCacheKey) return null
  if (!publicContentFence || !permissionFence) return null

  return {
    siteId,
    siteSlug,
    requestHost,
    requestLocales,
    backendLocales,
    deepwellArticlePageCacheKey,
    publicContentFence,
    permissionFence
  }
}

export const buildAnonymousArticleResponseCacheKey = (metadata) => {
  return [
    RESPONSE_CACHE_PREFIX,
    `site=${metadata.siteId}`,
    `slug=${utf8Hex(metadata.siteSlug)}`,
    `host=${utf8Hex(metadata.requestHost)}`,
    `requestLocales=${utf8Hex(metadata.requestLocales.join(","))}`,
    `backendLocales=${utf8Hex(metadata.backendLocales.join(","))}`,
    `content=${metadata.publicContentFence}`,
    `permission=${metadata.permissionFence}`,
    `deepwell=${sha256Hex(metadata.deepwellArticlePageCacheKey)}`
  ].join(":")
}

const normalizeRouteForToken = (route) => {
  if (!route) return { slug: "", extra: "" }
  return { slug: route.slug ?? "", extra: route.extra ?? "" }
}

export const buildAnonymousArticleResponseTokenKey = (metadata) => {
  const route = normalizeRouteForToken(metadata.route)

  return [
    RESPONSE_TOKEN_PREFIX,
    `site=${metadata.siteId}`,
    `slug=${utf8Hex(metadata.siteSlug)}`,
    `host=${utf8Hex(metadata.requestHost)}`,
    `route=${sha256Hex(JSON.stringify(route))}`,
    `requestLocales=${utf8Hex(metadata.requestLocales.join(","))}`,
    `backendLocales=${utf8Hex(metadata.backendLocales.join(","))}`,
    `content=${metadata.publicContentFence}`,
    `permission=${utf8Hex(metadata.permissionFence)}`
  ].join(":")
}

export const readAnonymousArticleResponseCache = async ({ store, metadata }) => {
  if (!metadata) return null
  return readCachedArticleResponse(store, buildAnonymousArticleResponseCacheKey(metadata))
}

export const readAnonymousArticleResponseCacheEntry = async ({ store, metadata }) => {
  if (!metadata) return null
  return readCachedArticleResponseEntry(
    store,
    buildAnonymousArticleResponseCacheKey(metadata)
  )
}

const isTokenValue = (value) => {
  return (
    value !== null &&
    typeof value === "object" &&
    typeof value.articlePageCacheKey === "string" &&
    value.articlePageCacheKey.length > 0
  )
}

export const readAnonymousArticleResponseToken = async ({ store, tokenMetadata }) => {
  if (!tokenMetadata) return null

  try {
    const cached = await store.get(buildAnonymousArticleResponseTokenKey(tokenMetadata))
    if (typeof cached !== "string") return null

    const value = JSON.parse(cached)
    if (!isTokenValue(value)) return null

    return value.articlePageCacheKey
  } catch {
    return null
  }
}

export const writeAnonymousArticleResponseToken = async ({
  store,
  tokenMetadata,
  deepwellArticlePageCacheKey,
  ttlSeconds = ARTICLE_RESPONSE_CACHE_TTL_SECONDS
}) => {
  if (!tokenMetadata || !deepwellArticlePageCacheKey) return false

  try {
    const currentFences = await readAnonymousArticleResponseCacheFences({
      store,
      siteId: tokenMetadata.siteId
    })
    if (
      currentFences?.publicContentFence !== tokenMetadata.publicContentFence ||
      currentFences?.permissionFence !== tokenMetadata.permissionFence
    ) {
      return false
    }

    const value = JSON.stringify({ articlePageCacheKey: deepwellArticlePageCacheKey })
    return (
      (await store.set(
        buildAnonymousArticleResponseTokenKey(tokenMetadata),
        value,
        ttlSeconds
      )) !== false
    )
  } catch {
    return false
  }
}

export const writeAnonymousArticleResponseCache = async ({
  store,
  metadata,
  response,
  ttlSeconds = ARTICLE_RESPONSE_CACHE_TTL_SECONDS
}) => {
  if (!metadata || response.status !== 200 || response.headers.has("set-cookie")) {
    return false
  }

  return writeCachedArticleResponse(
    store,
    buildAnonymousArticleResponseCacheKey(metadata),
    await serializeArticleResponseForCache(response),
    ttlSeconds
  )
}

export {
  ARTICLE_RESPONSE_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_CACHE_MAX_SERIALIZED_BYTES,
  ARTICLE_RESPONSE_CACHE_TTL_SECONDS,
  ARTICLE_RESPONSE_FENCE_INVALIDATION_CHANNEL,
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_BYTES,
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_MAX_ENTRIES,
  ARTICLE_RESPONSE_LOCAL_HOT_CACHE_TTL_MS,
  PUBLIC_CONTENT_FENCE_PREFIX
} from "./article-response-cache-shared.js"
export {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousPermissionFenceKeys,
  buildPublicContentFenceKey,
  createMemoryArticleResponseFenceCache,
  readAnonymousArticleResponseCacheFences
} from "./article-response-cache-fences.js"
export {
  createLocalArticleResponseHotCache,
  normalizeCachedArticleResponseEntry
} from "./article-response-cache-hot.js"
export {
  createMemoryArticleResponseCacheStore,
  deserializeCachedArticleResponse,
  readCachedArticleResponse,
  readCachedArticleResponseEntry,
  serializeArticleResponseForCache,
  writeCachedArticleResponse
} from "./article-response-cache-storage.js"
