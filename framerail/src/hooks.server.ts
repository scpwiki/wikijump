// Hook that runs on every request, including form actions.

import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheMetadata,
  canConsiderAnonymousArticleResponseCache,
  createMemoryArticleResponseCacheStore,
  readAnonymousArticleResponseCacheFences,
  readAnonymousArticleResponseCache,
  readAnonymousArticleResponseToken,
  writeAnonymousArticleResponseToken,
  writeAnonymousArticleResponseCache
} from "$lib/server/article-response-cache"
import { articleViewCacheMetadata } from "$lib/server/deepwell/views"
import {
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import { storeRequestContext } from "$lib/server/load/request-ctx"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { createRedisCacheStore } from "$lib/server/redis-cache-store"
import type { Handle, RequestEvent } from "@sveltejs/kit"

function isLocalEnvironment() {
  return process.env.FRAMERAIL_ENV === "local" || process.env.NODE_ENV === "development"
}

const SECURITY_HEADERS = {
  "cross-origin-opener-policy": "same-origin",
  "permissions-policy": [
    "accelerometer=()",
    "autoplay=()",
    "camera=()",
    "display-capture=()",
    "encrypted-media=()",
    "fullscreen=(self)",
    "geolocation=()",
    "gyroscope=()",
    "magnetometer=()",
    "microphone=()",
    "midi=()",
    "payment=()",
    "publickey-credentials-get=(self)",
    "screen-wake-lock=()",
    "usb=()",
    "web-share=(self)",
    "xr-spatial-tracking=()"
  ].join(", "),
  "referrer-policy": "strict-origin-when-cross-origin",
  "x-content-type-options": "nosniff",
  "x-frame-options": "DENY"
}

const HSTS_HEADER = "max-age=31536000; includeSubDomains"
const SITE_CONTEXT_EXEMPT_PATHS = new Set(["/xml-rpc-api.php"])
const LOCAL_WIKIDOT_INTERWIKI_FRAME_PATHS = new Set([
  "/-/wikidot-interwiki/interwikiFrame.html",
  "/-/wikidot-interwiki/styleFrame.html"
])
const articleResponseCacheStore = createMemoryArticleResponseCacheStore()
const articleResponseTokenStore = createRedisCacheStore()

function shouldSetHsts() {
  return !isLocalEnvironment()
}

function allowsLocalWikidotInterwikiFrame(pathname: string) {
  return isLocalEnvironment() && LOCAL_WIKIDOT_INTERWIKI_FRAME_PATHS.has(pathname)
}

function applySecurityHeaders(response: Response, pathname: string) {
  for (const [header, value] of Object.entries(SECURITY_HEADERS)) {
    response.headers.set(header, value)
  }

  if (shouldSetHsts()) {
    response.headers.set("strict-transport-security", HSTS_HEADER)
  }

  if (allowsLocalWikidotInterwikiFrame(pathname)) {
    response.headers.delete("content-security-policy")
    response.headers.delete("x-frame-options")
  }
}

function getArticleRoute(event: RequestEvent) {
  return event.params.slug || event.params.extra
    ? { slug: event.params.slug, extra: event.params.extra }
    : null
}

function canUseAnonymousArticleResponseCache(
  event: RequestEvent,
  siteId: number,
  siteSlug: string
) {
  return canConsiderAnonymousArticleResponseCache({
    method: event.request.method,
    routeId: event.route.id,
    url: event.url,
    siteId,
    siteSlug,
    route: getArticleRoute(event),
    cookieHeader: event.request.headers.get("cookie")
  })
}

async function readAnonymousArticleResponseCacheForEvent(
  event: RequestEvent,
  siteId: number,
  siteSlug: string
) {
  const route = getArticleRoute(event)
  const requestLocales = getPreloadRequestLocales(event.request)
  const backendLocales = getPreloadBackendLocales(requestLocales)
  const gate = canUseAnonymousArticleResponseCache(event, siteId, siteSlug)

  if (!gate.cacheable) return null

  if (articleResponseTokenStore) {
    try {
      const fences = await readAnonymousArticleResponseCacheFences({
        store: articleResponseTokenStore,
        siteId
      })
      const tokenMetadata = buildAnonymousArticleResponseCacheFences({
        siteId,
        siteSlug,
        route,
        requestLocales,
        backendLocales,
        publicContentFence: fences?.publicContentFence,
        permissionFence: fences?.permissionFence
      })
      const deepwellArticlePageCacheKey = await readAnonymousArticleResponseToken({
        store: articleResponseTokenStore,
        tokenMetadata
      })
      const metadata = buildAnonymousArticleResponseCacheMetadata({
        siteId,
        siteSlug,
        requestLocales,
        backendLocales,
        deepwellArticlePageCacheKey,
        publicContentFence: fences?.publicContentFence,
        permissionFence: fences?.permissionFence
      })

      const cachedResponse = await readAnonymousArticleResponseCache({
        store: articleResponseCacheStore,
        metadata
      })
      if (cachedResponse) return cachedResponse
    } catch {
      // Fall through to Deepwell metadata.
    }
  }

  try {
    const cacheMetadata = await articleViewCacheMetadata(siteId, backendLocales, route)
    const metadata = buildAnonymousArticleResponseCacheMetadata({
      siteId,
      siteSlug,
      requestLocales,
      backendLocales,
      deepwellArticlePageCacheKey: cacheMetadata.article_page_cache_key,
      publicContentFence: cacheMetadata.public_content_cache_fence,
      permissionFence: cacheMetadata.anonymous_permission_cache_fence
    })
    if (!metadata) return null

    return readAnonymousArticleResponseCache({
      store: articleResponseCacheStore,
      metadata
    })
  } catch {
    return null
  }
}

export const handle: Handle = async ({ event, resolve }) => {
  const { request, cookies, locals, params } = event

  if (SITE_CONTEXT_EXEMPT_PATHS.has(event.url.pathname)) {
    const response = await resolve(event)
    applySecurityHeaders(response, event.url.pathname)
    return response
  }

  // Gather common request metadata into a shared context.
  const { siteId, siteSlug } = loadSiteInfo(request.headers)
  const page_slug = params.slug
  const sessionToken = cookies.get("wikijump_token")

  storeRequestContext(locals, sessionToken, siteId, page_slug)

  const cachedResponse = await readAnonymousArticleResponseCacheForEvent(
    event,
    siteId,
    siteSlug
  )
  if (cachedResponse) {
    applySecurityHeaders(cachedResponse, event.url.pathname)
    return cachedResponse
  }

  // Continue processing the request
  const response = await resolve(event)

  applySecurityHeaders(response, event.url.pathname)

  const writeGate = canUseAnonymousArticleResponseCache(event, siteId, siteSlug)
  if (writeGate.cacheable) {
    const wroteResponse = await writeAnonymousArticleResponseCache({
      store: articleResponseCacheStore,
      metadata: locals.anonymousArticleResponseCacheMetadata,
      response
    })
    if (wroteResponse && articleResponseTokenStore) {
      const metadata = locals.anonymousArticleResponseCacheMetadata
      const tokenMetadata = buildAnonymousArticleResponseCacheFences({
        siteId: metadata?.siteId,
        siteSlug: metadata?.siteSlug,
        route: getArticleRoute(event),
        requestLocales: metadata?.requestLocales,
        backendLocales: metadata?.backendLocales,
        publicContentFence: metadata?.publicContentFence,
        permissionFence: metadata?.permissionFence
      })
      await writeAnonymousArticleResponseToken({
        store: articleResponseTokenStore,
        tokenMetadata,
        deepwellArticlePageCacheKey: metadata?.deepwellArticlePageCacheKey
      })
    }
  }

  return response
}
