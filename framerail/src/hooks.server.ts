// Hook that runs on every request, including form actions.

import {
  buildAnonymousArticleResponseCacheFences,
  buildAnonymousArticleResponseCacheMetadata,
  canConsiderAnonymousArticleResponseCache,
  readAnonymousArticleResponseCacheFences,
  readAnonymousArticleResponseCache,
  readAnonymousArticleResponseToken,
  writeAnonymousArticleResponseToken,
  writeAnonymousArticleResponseCache
} from "$lib/server/article-response-cache"
import { getArticleResponseCacheStores } from "$lib/server/article-response-cache-runtime"
import { articleViewCacheMetadata } from "$lib/server/deepwell/views"
import {
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import { storeRequestContext } from "$lib/server/request-context"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { applyStaticSecurityHeaders } from "$lib/server/security-headers"
import {
  injectWikidotRequestInfo,
  requestHostFromRequest
} from "$lib/server/wikidot-request-info"
import type { Handle, RequestEvent } from "@sveltejs/kit"

const SITE_CONTEXT_EXEMPT_PATHS = new Set(["/xml-rpc-api.php"])

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
  siteSlug: string,
  responseStore: ReturnType<typeof getArticleResponseCacheStores>["responseStore"],
  tokenStore: ReturnType<typeof getArticleResponseCacheStores>["tokenStore"]
) {
  const route = getArticleRoute(event)
  const requestLocales = getPreloadRequestLocales(event.request)
  const backendLocales = getPreloadBackendLocales(requestLocales)
  const gate = canUseAnonymousArticleResponseCache(event, siteId, siteSlug)

  if (!gate.cacheable) return null
  const requestHost = requestHostFromRequest(event.request)

  if (tokenStore) {
    try {
      const fences = await readAnonymousArticleResponseCacheFences({
        store: tokenStore,
        siteId
      })
      const tokenMetadata = buildAnonymousArticleResponseCacheFences({
        siteId,
        siteSlug,
        requestHost,
        route,
        requestLocales,
        backendLocales,
        publicContentFence: fences?.publicContentFence,
        permissionFence: fences?.permissionFence
      })
      const deepwellArticlePageCacheKey = await readAnonymousArticleResponseToken({
        store: tokenStore,
        tokenMetadata
      })
      const metadata = buildAnonymousArticleResponseCacheMetadata({
        siteId,
        siteSlug,
        requestHost,
        requestLocales,
        backendLocales,
        deepwellArticlePageCacheKey,
        publicContentFence: fences?.publicContentFence,
        permissionFence: fences?.permissionFence
      })

      const cachedResponse = await readAnonymousArticleResponseCache({
        store: responseStore,
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
      requestHost,
      requestLocales,
      backendLocales,
      deepwellArticlePageCacheKey: cacheMetadata.article_page_cache_key,
      publicContentFence: cacheMetadata.public_content_cache_fence,
      permissionFence: cacheMetadata.anonymous_permission_cache_fence
    })
    if (!metadata) return null

    return readAnonymousArticleResponseCache({
      store: responseStore,
      metadata
    })
  } catch {
    return null
  }
}

export const handle: Handle = async ({ event, resolve }) => {
  const { request, cookies, locals, params } = event
  const { responseStore, tokenStore } = getArticleResponseCacheStores()
  const resolveWithWikidotRequestInfo = () =>
    resolve(event, {
      transformPageChunk: ({ html }) =>
        injectWikidotRequestInfo(html, locals.wikidotRequestInfo, locals.siteLocale)
    })

  if (SITE_CONTEXT_EXEMPT_PATHS.has(event.url.pathname)) {
    const response = await resolveWithWikidotRequestInfo()
    applyStaticSecurityHeaders(response, event.url.pathname)
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
    siteSlug,
    responseStore,
    tokenStore
  )
  if (cachedResponse) {
    applyStaticSecurityHeaders(cachedResponse, event.url.pathname, siteSlug)
    return cachedResponse
  }

  const response = await resolveWithWikidotRequestInfo()

  applyStaticSecurityHeaders(response, event.url.pathname, siteSlug)

  const writeGate = canUseAnonymousArticleResponseCache(event, siteId, siteSlug)
  if (writeGate.cacheable) {
    const wroteResponse = await writeAnonymousArticleResponseCache({
      store: responseStore,
      metadata: locals.anonymousArticleResponseCacheMetadata,
      response
    })
    if (wroteResponse && tokenStore) {
      const metadata = locals.anonymousArticleResponseCacheMetadata
      const tokenMetadata = buildAnonymousArticleResponseCacheFences({
        siteId: metadata?.siteId,
        siteSlug: metadata?.siteSlug,
        requestHost: metadata?.requestHost,
        route: getArticleRoute(event),
        requestLocales: metadata?.requestLocales,
        backendLocales: metadata?.backendLocales,
        publicContentFence: metadata?.publicContentFence,
        permissionFence: metadata?.permissionFence
      })
      await writeAnonymousArticleResponseToken({
        store: tokenStore,
        tokenMetadata,
        deepwellArticlePageCacheKey: metadata?.deepwellArticlePageCacheKey
      })
    }
  }

  return response
}
