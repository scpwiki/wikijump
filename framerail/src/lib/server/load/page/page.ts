import { buildAnonymousArticleResponseCacheMetadata } from "$lib/server/cache/article-response/index.js"
import {
  resolveShellLayoutValue,
  WIKIDOT_LAYOUT
} from "$lib/layout/wikidot-shell-decision"
import { articleRouteWithQueryNoRender } from "$lib/server/page-norender-query"
import { resolvePageRedirect } from "$lib/server/page-redirect"
import { pageDeletedGet } from "$lib/server/deepwell/page"
import { pageErrorStatus } from "$lib/server/load/page/page-error-status.js"
import { translate } from "$lib/server/deepwell/translate"
import { articleView } from "$lib/server/deepwell/views"
import { buildPageLoadData } from "$lib/server/load/page-data"
import { buildPageErrorForms, buildPageForms } from "$lib/server/load/page/page-forms"
import {
  buildPageTranslateKeys,
  buildWikidotPagePresentation
} from "$lib/server/load/page/page-presentation"
import {
  finalizePreloadData,
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { getRequestContext } from "$lib/server/request-context"
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
  const articleRoute = articleRouteWithQueryNoRender(route, new URL(request.url).search)
  const articleResponse = await articleView(
    siteId,
    backendLocales,
    articleRoute,
    sessionToken
  )
  const { page: articlePage, ...preloadResponse } = articleResponse
  const response = gateMissingPageCreateSource(articlePage)
  const parentData = finalizePreloadData(preloadResponse, requestLocales)
  const locales = parentData.locales
  const siteLocale = parentData.site.locale
  if (locals) locals.siteLocale = siteLocale

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
  const presentation = buildWikidotPagePresentation(response, {
    hasSession: !!parentData.user_session,
    siteLocale
  })

  if (response.type !== "found") {
    const errorForms = await buildPageErrorForms(request, response)
    let restoreAvailable = false
    if (response.type === "missing" && sessionToken) {
      try {
        const requestContext = locals ? getRequestContext(locals) : undefined
        const deletedPages = await pageDeletedGet({
          ...(requestContext ?? {}),
          siteId,
          page: slug ?? parentData.site.default_page
        })
        restoreAvailable = deletedPages.length > 0
      } catch {
        // A failed or unauthorized recoverability lookup must never expose
        // deleted-page state. The restore action will recheck on activation.
      }
    }
    const errorViewData = {
      ...response.data,
      view: response.type,
      restore_available: restoreAvailable,
      internationalization,
      ...presentation
    }
    const errorData = buildPageLoadData(parentData, errorViewData, errorForms)
    if (locals) {
      locals.wikidotDocument = resolveShellLayoutValue(errorData) === WIKIDOT_LAYOUT
    }
    error(pageErrorStatus(response.type, slug), errorData)
  }

  const viewData = {
    ...response.data,
    view: response.type,
    internationalization,
    ...presentation
  }
  runRedirect(response.data, slug, extra, request.url)

  // Return to page for rendering
  const loadData = buildPageLoadData(parentData, viewData, await buildPageForms(request))
  if (locals) {
    locals.wikidotDocument = resolveShellLayoutValue(loadData) === WIKIDOT_LAYOUT
  }
  return loadData
}

/**
 * Template wikitext, data-form definitions, and template selections are
 * create source. They reach the browser only for a viewer Deepwell reports
 * may create the missing page. Fail closed: a missing or false
 * `can_create` strips them.
 */
function gateMissingPageCreateSource(response: PageView): PageView {
  if (response.type !== "missing" || response.data.can_create === true) return response
  return {
    ...response,
    data: {
      ...response.data,
      new_page_wikitext: null,
      page_templates: [],
      selected_template_page_id: null,
      data_form: null
    }
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
