import { buildAnonymousArticleResponseCacheMetadata } from "$lib/server/article-response-cache"
import { resolvePageRedirect } from "$lib/server/page-redirect"
import { translate } from "$lib/server/deepwell/translate"
import { articleView } from "$lib/server/deepwell/views"
import { buildPageLoadData } from "$lib/server/load/page-data"
import { buildPageErrorForms, buildPageForms } from "$lib/server/load/page-forms"
import {
  buildPageTranslateKeys,
  buildWikidotPagePresentation
} from "$lib/server/load/page-presentation"
import {
  finalizePreloadData,
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import { loadSiteInfo } from "$lib/server/load/site-info"
import {
  buildWikidotRequestInfo,
  requestHostFromRequest
} from "$lib/server/wikidot-request-info"
import { error, redirect } from "@sveltejs/kit"

import type { PageView } from "$lib/server/deepwell/views"
import type { Optional } from "$lib/types"
import type { Cookies } from "@sveltejs/kit"

export async function loadPage(
  slug: Optional<string>,
  extra: Optional<string>,
  request: Request,
  cookies: Cookies,
  locals?: App.Locals
) {
  // Set up parameters
  const { siteId, siteSlug } = loadSiteInfo(request.headers)
  const route = slug || extra ? { slug, extra } : null
  const sessionToken = cookies.get("wikijump_token")

  const requestLocales = getPreloadRequestLocales(request)
  const backendLocales = getPreloadBackendLocales(requestLocales)
  const articleResponse = await articleView(siteId, backendLocales, route, sessionToken)
  const { page: response, ...preloadResponse } = articleResponse
  const parentData = finalizePreloadData(preloadResponse, requestLocales)
  const locales = parentData.locales
  const siteLocale = parentData.site.locale
  if (locals) locals.siteLocale = siteLocale

  const errorStatus = pageErrorStatus(response.type)
  if (locals && response.type === "found") {
    const requestHost = requestHostFromRequest(request)
    locals.wikidotRequestInfo = buildWikidotRequestInfo({
      domain: requestHost,
      site: parentData.site,
      page: response.data.page
    })
    const metadata = buildAnonymousArticleResponseCacheMetadata({
      siteId,
      siteSlug,
      requestHost,
      requestLocales,
      backendLocales,
      deepwellArticlePageCacheKey: articleResponse.article_page_cache_key,
      publicContentFence: articleResponse.public_content_cache_fence,
      permissionFence: articleResponse.anonymous_permission_cache_fence
    })
    if (metadata) {
      locals.anonymousArticleResponseCacheMetadata = metadata
    }
  }

  const internationalization = await translate(
    locales,
    buildPageTranslateKeys(response, {
      licenseName: parentData.license_name,
      licenseUrl: parentData.license_url,
      locales
    })
  )
  const viewData = {
    ...response.data,
    view: response.type,
    internationalization,
    ...buildWikidotPagePresentation(response, {
      hasSession: !!parentData.user_session,
      siteLocale
    })
  }

  if (errorStatus !== null) {
    const errorForms = await buildPageErrorForms(request, response)
    error(errorStatus, buildPageLoadData(parentData, viewData, errorForms))
  }

  runRedirect(response.data, slug, extra, request.url)

  // Return to page for rendering
  return buildPageLoadData(parentData, viewData, await buildPageForms(request))
}

function pageErrorStatus(type: PageView["type"]): number | null {
  switch (type) {
    case "found":
      return null
    case "missing":
      return 404
    case "permissions":
      return 403
    default:
      return 500
  }
}

function runRedirect(
  viewData: PageView["data"],
  originalSlug: Optional<string>,
  extra: Optional<string>,
  requestUrl: string
): void {
  const resolved = resolvePageRedirect(viewData, originalSlug, extra, requestUrl)
  if (resolved) {
    redirect(resolved.status, resolved.location)
  }
}
