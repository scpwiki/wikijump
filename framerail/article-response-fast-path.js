import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  createLocalArticleResponseHotCache,
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

const fencesMatch = (currentFences, capturedFences) => {
  return (
    currentFences?.publicContentFence === capturedFences?.publicContentFence &&
    currentFences?.permissionFence === capturedFences?.permissionFence
  )
}

const revalidateCapturedFences = async ({ store, fenceCache, siteId, fences }) => {
  if (!fences) return false

  const localValidation = fenceCache?.areFencesCurrent?.({
    siteId,
    publicContentFence: fences.publicContentFence,
    permissionFence: fences.permissionFence
  })
  if (localValidation !== null && localValidation !== undefined) {
    return localValidation === true
  }

  return fencesMatch(
    await readAnonymousArticleResponseCacheFences({
      store,
      siteId
    }),
    fences
  )
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
    pathname: url.pathname,
    siteId,
    siteSlug,
    requestLocales,
    backendLocales,
    method: request.method
  }
}

const finalArticleResponseFastPathHeaders = (entry, pathname) => {
  const headers = new Map(
    entry.headers.map(([name, value]) => [name.toLowerCase(), value])
  )
  const headerTarget = {
    setHeader(name, value) {
      headers.set(name.toLowerCase(), String(value))
    },
    removeHeader(name) {
      headers.delete(name.toLowerCase())
    }
  }

  applyStaticSecurityHeadersToNodeResponse(headerTarget, pathname)
  return [...headers.entries()].sort(([left], [right]) => left.localeCompare(right))
}

const prepareArticleResponseFastPathReplay = (entry, pathname) => {
  return {
    status: entry.status,
    headers: finalArticleResponseFastPathHeaders(entry, pathname),
    bodyBuffer: Buffer.isBuffer(entry.bodyBuffer)
      ? entry.bodyBuffer
      : Buffer.from(entry.body, "utf8"),
    finalHeaders: true
  }
}

export const readArticleResponseFastPathEntry = async ({
  store,
  request,
  localHotCache,
  fenceCache = null
}) => {
  return readArticleResponseFastPathEntryFromStores({
    responseStore: store,
    tokenStore: store,
    request,
    localHotCache,
    fenceCache
  })
}

export const readArticleResponseFastPathEntryFromStores = async ({
  responseStore,
  tokenStore,
  request,
  localHotCache,
  fenceCache = null
}) => {
  if (!responseStore || !tokenStore || !request) return null

  try {
    const candidate = getArticleResponseFastPathRequest(request)
    if (!candidate) return null

    const fences = fenceCache
      ? await fenceCache.readFences({ siteId: candidate.siteId })
      : await readAnonymousArticleResponseCacheFences({
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
    if (!tokenMetadata) return null

    const shouldRevalidateLocalFences =
      fenceCache?.canValidateFencesLocally?.({ siteId: candidate.siteId }) === true
    const tokenKey = buildAnonymousArticleResponseTokenKey(tokenMetadata)
    const hotEntry =
      localHotCache?.getSharedReplayForInternalUse?.(tokenKey) ??
      localHotCache?.getReplay?.(tokenKey) ??
      localHotCache?.get(tokenKey)
    if (hotEntry?.status === 200) {
      if (
        shouldRevalidateLocalFences &&
        !(await revalidateCapturedFences({
          store: tokenStore,
          fenceCache,
          siteId: candidate.siteId,
          fences
        }))
      ) {
        return null
      }
      return hotEntry
    }

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

    if (cachedEntry?.status !== 200) return null
    if (
      shouldRevalidateLocalFences &&
      !(await revalidateCapturedFences({
        store: tokenStore,
        fenceCache,
        siteId: candidate.siteId,
        fences
      }))
    ) {
      return null
    }
    const replay = prepareArticleResponseFastPathReplay(cachedEntry, candidate.pathname)
    localHotCache?.set(tokenKey, cachedEntry, { replay })
    return replay
  } catch {
    return null
  }
}

export const writeArticleResponseFastPathHit = (request, response, entry) => {
  response.statusCode = entry.status
  for (const [name, value] of entry.headers) {
    response.setHeader(name, value)
  }
  if (entry.finalHeaders !== true) {
    const pathname = new URL(request.url ?? "", "http://localhost").pathname
    applyStaticSecurityHeadersToNodeResponse(response, pathname)
  }

  if (request.method === "HEAD") {
    response.end()
  } else {
    response.end(entry.bodyBuffer ?? entry.body)
  }
}

export const createArticleResponseFastPathHandler = ({
  responseStore,
  tokenStore,
  store,
  handler,
  localHotCache,
  localHotCacheOptions,
  fenceCache
}) => {
  const resolvedResponseStore = responseStore ?? store
  const resolvedTokenStore = tokenStore ?? store
  const hotCache =
    localHotCache ??
    (localHotCacheOptions
      ? createLocalArticleResponseHotCache(localHotCacheOptions)
      : createLocalArticleResponseHotCache())
  fenceCache?.attachHotCache?.(hotCache)

  return async (request, response) => {
    const cachedEntry = await readArticleResponseFastPathEntryFromStores({
      responseStore: resolvedResponseStore,
      tokenStore: resolvedTokenStore,
      request,
      localHotCache: hotCache,
      fenceCache
    })
    if (cachedEntry) {
      writeArticleResponseFastPathHit(request, response, cachedEntry)
      return
    }

    return handler(request, response)
  }
}
