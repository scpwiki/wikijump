import {
  buildAnonymousArticleResponseCacheMetadata,
  buildAnonymousArticleResponseTokenKey,
  readAnonymousArticleResponseCacheEntry,
  readAnonymousArticleResponseToken
} from "./src/lib/server/cache/article-response/index.js"
import {
  buildAnonymousArticleResponseCacheFences,
  readAnonymousArticleResponseCacheFences
} from "./src/lib/server/cache/article-response/fences.js"
import { createLocalArticleResponseHotCache } from "./src/lib/server/cache/article-response/hot.js"
import { hasSessionCookie } from "./src/lib/server/cache/article-response/shared.js"
import { applyStaticSecurityHeadersToNodeResponse } from "./src/lib/server/security-headers.js"
import { parseAcceptLangHeader, withFallbackLocale } from "./src/lib/locales.js"
import { promisify } from "node:util"
import zlib from "node:zlib"

const FALLBACK_LOCALE = "en"
const SITE_ID_HEADER = "x-wikijump-site-id"
const SITE_SLUG_HEADER = "x-wikijump-site-slug"
const MIN_COMPRESSED_REPLAY_BODY_BYTES = 2048
const brotliCompress = promisify(zlib.brotliCompress)
const gzipCompress = promisify(zlib.gzip)
const STATIC_APP_ROUTE_SLUGS = new Set([
  "-",
  ".well-known",
  "about",
  "forum",
  "xml-rpc-api.php"
])
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
  const requestHost = singleHeaderValue(request.headers, "host")?.toLowerCase()
  const siteId = Number.parseInt(
    singleHeaderValue(request.headers, SITE_ID_HEADER) ?? "",
    10
  )
  if (!siteSlug || !requestHost || !Number.isInteger(siteId) || siteId <= 0) return null

  const fetchHeaders = toFetchHeaders(request.headers)
  const requestLocales = parseAcceptLangHeader({ headers: fetchHeaders })
  const backendLocales = withFallbackLocale(requestLocales, FALLBACK_LOCALE)

  return {
    route,
    pathname: url.pathname,
    siteId,
    siteSlug,
    requestHost,
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

const mergeVaryAcceptEncoding = (value) => {
  if (!value) return "Accept-Encoding"
  const parts = value.split(",").map((part) => part.trim())
  if (parts.some((part) => part.toLowerCase() === "accept-encoding")) return value
  return `${value}, Accept-Encoding`
}

const shouldPrepareCompressedReplayVariants = ({ status, headers, bodyBuffer }) => {
  if (status !== 200) return false
  if (bodyBuffer.byteLength < MIN_COMPRESSED_REPLAY_BODY_BYTES) return false

  const headerMap = new Map(headers)
  if (headerMap.has("set-cookie")) return false
  if (headerMap.has("content-encoding")) return false
  if (
    headerMap
      .get("cache-control")
      ?.split(",")
      .some((part) => part.trim().toLowerCase() === "no-transform")
  ) {
    return false
  }
  return headerMap.get("content-type")?.toLowerCase().startsWith("text/html") === true
}

const compressedVariantHeaders = (headers, encoding, bodyBuffer) => {
  const mapped = new Map(headers)
  mapped.set("content-encoding", encoding)
  mapped.set("content-length", String(bodyBuffer.byteLength))
  mapped.set("vary", mergeVaryAcceptEncoding(mapped.get("vary")))
  mapped.delete("transfer-encoding")

  const etag = mapped.get("etag")
  if (etag && !etag.trim().startsWith("W/")) {
    mapped.delete("etag")
  }

  return [...mapped.entries()].sort(([left], [right]) => left.localeCompare(right))
}

const prepareCompressedReplayVariants = async (headers, bodyBuffer) => {
  const [brBodyBuffer, gzipBodyBuffer] = await Promise.all([
    brotliCompress(bodyBuffer, {
      params: {
        [zlib.constants.BROTLI_PARAM_MODE]: zlib.constants.BROTLI_MODE_TEXT,
        [zlib.constants.BROTLI_PARAM_QUALITY]: 5,
        [zlib.constants.BROTLI_PARAM_SIZE_HINT]: bodyBuffer.byteLength
      }
    }),
    gzipCompress(bodyBuffer, { level: 6 })
  ])

  return {
    br: {
      headers: compressedVariantHeaders(headers, "br", brBodyBuffer),
      bodyBuffer: brBodyBuffer
    },
    gzip: {
      headers: compressedVariantHeaders(headers, "gzip", gzipBodyBuffer),
      bodyBuffer: gzipBodyBuffer
    }
  }
}

const prepareArticleResponseFastPathReplay = async (entry, pathname) => {
  const headers = finalArticleResponseFastPathHeaders(entry, pathname)
  const bodyBuffer = Buffer.isBuffer(entry.bodyBuffer)
    ? entry.bodyBuffer
    : Buffer.from(entry.body, "utf8")
  const variants = shouldPrepareCompressedReplayVariants({
    status: entry.status,
    headers,
    bodyBuffer
  })
    ? await prepareCompressedReplayVariants(headers, bodyBuffer)
    : undefined
  const replayHeaders = variants
    ? headers
        .map(([name, value]) =>
          name === "vary" ? [name, mergeVaryAcceptEncoding(value)] : [name, value]
        )
        .concat(
          headers.some(([name]) => name === "vary") ? [] : [["vary", "Accept-Encoding"]]
        )
        .sort(([left], [right]) => left.localeCompare(right))
    : headers

  return {
    status: entry.status,
    headers: replayHeaders,
    nodeRawHeaders: Object.freeze(
      replayHeaders.flatMap(([name, value]) => [name, value])
    ),
    bodyBuffer,
    variants,
    finalHeaders: true
  }
}

const parseEncodingQuality = (value) => {
  const quality = Number.parseFloat(value)
  if (!Number.isFinite(quality)) return 0
  if (quality < 0) return 0
  if (quality > 1) return 1
  return quality
}

const parseAcceptEncoding = (header) => {
  const accepted = { br: null, gzip: null, identity: null, wildcard: null }
  if (!header) return accepted

  for (const part of header.split(",")) {
    const [rawEncoding, ...parameters] = part.split(";")
    const encoding = rawEncoding.trim().toLowerCase()
    if (!encoding) continue
    let quality = 1
    for (const parameter of parameters) {
      const [name, value] = parameter.split("=")
      if (name?.trim().toLowerCase() === "q") {
        quality = parseEncodingQuality(value?.trim())
      }
    }
    if (encoding === "br" || encoding === "gzip" || encoding === "identity") {
      accepted[encoding] = quality
    } else if (encoding === "*") {
      accepted.wildcard = quality
    }
  }

  return accepted
}

const selectReplayVariant = (request, entry) => {
  const variants = entry.variants
  if (!variants?.br && !variants?.gzip) return entry

  const acceptEncoding = singleHeaderValue(request.headers, "accept-encoding")
  if (!acceptEncoding) return entry

  const accepted = parseAcceptEncoding(acceptEncoding)
  const quality = (encoding) => {
    if (accepted[encoding] !== null) return accepted[encoding]
    if (accepted.wildcard !== null && encoding !== "identity") return accepted.wildcard
    return encoding === "identity" ? 1 : 0
  }
  const candidates = [
    variants.br ? { encoding: "br", variant: variants.br, priority: 3 } : null,
    variants.gzip ? { encoding: "gzip", variant: variants.gzip, priority: 2 } : null,
    { encoding: "identity", variant: entry, priority: 1 }
  ].filter(Boolean)

  candidates.sort((left, right) => {
    const qualityDelta = quality(right.encoding) - quality(left.encoding)
    if (qualityDelta !== 0) return qualityDelta
    return right.priority - left.priority
  })

  const selected = candidates[0]
  if (quality(selected.encoding) <= 0) return entry
  if (selected.encoding === "identity") return entry

  return {
    status: entry.status,
    headers: selected.variant.headers,
    nodeRawHeaders: selected.variant.nodeRawHeaders,
    bodyBuffer: selected.variant.bodyBuffer,
    finalHeaders: entry.finalHeaders
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
      requestHost: candidate.requestHost,
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
      requestHost: candidate.requestHost,
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
    const replay = await prepareArticleResponseFastPathReplay(
      cachedEntry,
      candidate.pathname
    )
    localHotCache?.store(tokenKey, cachedEntry, { replay })
    return replay
  } catch {
    return null
  }
}

export const writeArticleResponseFastPathHit = (request, response, entry) => {
  entry = selectReplayVariant(request, entry)
  if (entry.finalHeaders === true && Array.isArray(entry.nodeRawHeaders)) {
    response.writeHead(entry.status, entry.nodeRawHeaders)
    response.end(request.method === "HEAD" ? undefined : (entry.bodyBuffer ?? entry.body))
    return
  }

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
