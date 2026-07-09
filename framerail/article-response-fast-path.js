import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheMetadata,
  readAnonymousArticleResponseCacheEntry,
  readAnonymousArticleResponseCacheFences,
  readAnonymousArticleResponseToken
} from "./src/lib/server/article-response-cache.js"
import { applyStaticSecurityHeadersToNodeResponse } from "./src/lib/server/security-headers.js"
import { parseAcceptLangHeader, withFallbackLocale } from "./src/lib/locales.js"

const FALLBACK_LOCALE = "en"
const SITE_ID_HEADER = "x-wikijump-site-id"
const SITE_SLUG_HEADER = "x-wikijump-site-slug"
const SESSION_COOKIE = "wikijump_token"
const STATIC_APP_ROUTE_SLUGS = new Set([
  "-",
  ".well-known",
  "about",
  "forum",
  "xml-rpc-api.php"
])

const hasSessionCookie = (cookieHeader) => {
  if (!cookieHeader) return false

  return cookieHeader
    .split(";")
    .map((cookie) => cookie.trim())
    .some(
      (cookie) => cookie === SESSION_COOKIE || cookie.startsWith(`${SESSION_COOKIE}=`)
    )
}

const singleHeaderValue = (headers, name) => {
  const value = headers[name]
  if (typeof value === "string") return value
  if (Array.isArray(value) && value.length === 1) return value[0]
  return null
}

const toFetchHeaders = (headers) => {
  const fetchHeaders = new Headers()
  for (const [name, value] of Object.entries(headers)) {
    if (typeof value === "string") {
      fetchHeaders.set(name, value)
    } else if (Array.isArray(value)) {
      fetchHeaders.set(name, value.join(", "))
    }
  }
  return fetchHeaders
}

const articleRouteFromPathname = (pathname) => {
  if (pathname === "/") return null
  if (!pathname.startsWith("/") || pathname.endsWith("/")) return undefined

  const segments = pathname.split("/")
  if (segments.length !== 2 || !segments[1]) return undefined

  try {
    const slug = decodeURIComponent(segments[1])
    if (!slug || slug.includes("/")) return undefined
    if (STATIC_APP_ROUTE_SLUGS.has(slug)) return undefined
    return { slug, extra: "" }
  } catch {
    return undefined
  }
}

export const getArticleResponseFastPathRequest = (request) => {
  if (request.method !== "GET" && request.method !== "HEAD") return null
  if (hasSessionCookie(singleHeaderValue(request.headers, "cookie"))) return null

  let url
  try {
    url = new URL(request.url ?? "", "http://localhost")
  } catch {
    return null
  }
  if (url.search !== "") return null

  const route = articleRouteFromPathname(url.pathname)
  if (route === undefined) return null

  const siteSlug = singleHeaderValue(request.headers, SITE_SLUG_HEADER)
  const siteId = Number.parseInt(
    singleHeaderValue(request.headers, SITE_ID_HEADER) ?? "",
    10
  )
  if (!siteSlug || !Number.isInteger(siteId) || siteId <= 0) return null

  const fetchHeaders = toFetchHeaders(request.headers)
  const requestLocales = parseAcceptLangHeader({ headers: fetchHeaders })
  const backendLocales = withFallbackLocale(requestLocales, FALLBACK_LOCALE)

  return {
    route,
    siteId,
    siteSlug,
    requestLocales,
    backendLocales,
    method: request.method
  }
}

export const readArticleResponseFastPathEntry = async ({ store, request }) => {
  return readArticleResponseFastPathEntryFromStores({
    responseStore: store,
    tokenStore: store,
    request
  })
}

export const readArticleResponseFastPathEntryFromStores = async ({
  responseStore,
  tokenStore,
  request
}) => {
  if (!responseStore || !tokenStore || !request) return null

  try {
    const candidate = getArticleResponseFastPathRequest(request)
    if (!candidate) return null

    const fences = await readAnonymousArticleResponseCacheFences({
      store: tokenStore,
      siteId: candidate.siteId
    })
    const tokenMetadata = buildAnonymousArticleResponseCacheFences({
      siteId: candidate.siteId,
      siteSlug: candidate.siteSlug,
      route: candidate.route,
      requestLocales: candidate.requestLocales,
      backendLocales: candidate.backendLocales,
      publicContentFence: fences?.publicContentFence,
      permissionFence: fences?.permissionFence
    })
    const deepwellArticlePageCacheKey = await readAnonymousArticleResponseToken({
      store: tokenStore,
      tokenMetadata
    })
    const metadata = buildAnonymousArticleResponseCacheMetadata({
      siteId: candidate.siteId,
      siteSlug: candidate.siteSlug,
      requestLocales: candidate.requestLocales,
      backendLocales: candidate.backendLocales,
      deepwellArticlePageCacheKey,
      publicContentFence: fences?.publicContentFence,
      permissionFence: fences?.permissionFence
    })
    const cachedEntry = await readAnonymousArticleResponseCacheEntry({
      store: responseStore,
      metadata
    })

    return cachedEntry?.status === 200 ? cachedEntry : null
  } catch {
    return null
  }
}

export const writeArticleResponseFastPathHit = (request, response, entry) => {
  response.statusCode = entry.status
  for (const [name, value] of entry.headers) {
    response.setHeader(name, value)
  }
  const pathname = new URL(request.url ?? "", "http://localhost").pathname
  applyStaticSecurityHeadersToNodeResponse(response, pathname)

  if (request.method === "HEAD") {
    response.end()
  } else {
    response.end(entry.body)
  }
}

export const createArticleResponseFastPathHandler = ({
  responseStore,
  tokenStore,
  store,
  handler
}) => {
  const resolvedResponseStore = responseStore ?? store
  const resolvedTokenStore = tokenStore ?? store

  return async (request, response) => {
    const cachedEntry = await readArticleResponseFastPathEntryFromStores({
      responseStore: resolvedResponseStore,
      tokenStore: resolvedTokenStore,
      request
    })
    if (cachedEntry) {
      writeArticleResponseFastPathHit(request, response, cachedEntry)
      return
    }

    return handler(request, response)
  }
}
