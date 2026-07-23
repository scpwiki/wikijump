import defaults from "$lib/defaults"

import { buildAnonymousArticleResponseCacheMetadata } from "$lib/server/article-response-cache"
import { resolvePageRedirect } from "$lib/server/page-redirect"
import { translate } from "$lib/server/deepwell/translate"
import { articleView } from "$lib/server/deepwell/views"
import { buildPageLoadData } from "$lib/server/load/page-data"
import {
  layoutSchema,
  pageDeleteSchema,
  pageEditSchema,
  pageMoveSchema
} from "$lib/server/load/page-edit-actions"
import {
  finalizePreloadData,
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import {
  pageFileEditSchema,
  pageFileMoveSchema,
  pageFileRestoreSchema,
  pageFileUploadSchema
} from "$lib/server/load/page-file-actions"
import { pageParentSchema } from "$lib/server/load/page-relation-actions"
import { pageRestoreSchema } from "$lib/server/load/page-revision-actions"
import { loadSiteInfo } from "$lib/server/load/site-info"
import {
  buildWikidotRequestInfo,
  requestHostFromRequest
} from "$lib/server/wikidot-request-info"
import {
  buildWikidotPageActionLabels,
  sourceShowsStandardWikidotPageActions
} from "$lib/wikidot-page-actions"
import { buildWikidotPageInfoText } from "$lib/wikidot-page-info"
import { buildWikidotPageWatchLabel } from "$lib/wikidot-page-watch"
import { toIntlLocales } from "$lib/wikidot-locale"
import { error, redirect } from "@sveltejs/kit"
import { superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"

import type { PageView } from "$lib/server/deepwell/views"
import type { Optional, TranslateKeys } from "$lib/types"
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

  // Process response, performing redirects etc
  const { data: responseData, type: responseType } = response

  let errorStatus = null

  switch (responseType) {
    case "found":
      break
    case "missing":
      errorStatus = 404
      break
    case "permissions":
      errorStatus = 403
      break
    default:
      // Unexpected response type!
      // There is an inconsistency between here / DEEPWELL
      errorStatus = 500
  }

  if (locals && responseType === "found") {
    const requestHost = requestHostFromRequest(request)
    locals.wikidotRequestInfo = buildWikidotRequestInfo({
      domain: requestHost,
      site: parentData.site,
      page: responseData.page
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

  let translateKeys: TranslateKeys = {
    ...defaults.translateKeys,

    // Page actions
    "save": {},
    "cancel": {},

    // Page edit
    "title": {},
    "alt-title": {},
    "tags": {},
    "wiki-page-revision-comments": {},
    "wiki-page-layout": {},
    "wiki-page-layout.default": {},
    "wiki-page-layout.wikidot": {},
    "wiki-page-layout.wikijump": {},

    "footer-license-unless": {
      license: parentData.license_name,
      "license_url": parentData.license_url
    }
  }

  if (errorStatus === null && responseType === "found") {
    // Calculate difference of days since latest page edit
    const updatedAt = Date.parse(
      responseData.page.updated_at ?? responseData.page.created_at
    )
    const daysDiff = Math.floor((Date.now() - updatedAt) / 1000 / 86400)

    translateKeys = {
      ...translateKeys,

      // Page actions
      "edit": {},
      "delete": {},
      "history": {},
      "move": {},
      "view": {},
      "vote": {},
      "layout": {},
      "parents": {},
      "options": {},
      "confirm": {},

      // Page history
      "wiki-page-revision": {
        revision: responseData.page_revision.revision_number
      },
      "wiki-page-last-edit": {
        date: new Date(updatedAt).toLocaleString(toIntlLocales(locales)),
        days: daysDiff
      },
      "wiki-page-revision-history": {},
      "wiki-page-revision-number": {},
      "wiki-page-revision-created-at": {},
      "wiki-page-revision-user": {},
      "wiki-page-revision-rollback": {},
      "wiki-page-revision-type": {},
      "wiki-page-revision-type.create": {},
      "wiki-page-revision-type.regular": {},
      "wiki-page-revision-type.move": {},
      "wiki-page-revision-type.delete": {},
      "wiki-page-revision-type.rollback": {},
      "wiki-page-revision-type.undelete": {},
      "wiki-page-revision-type.undo": {},

      // Page vote
      "wiki-page-vote": {},
      "wiki-page-vote.list": {},
      "wiki-page-vote.set": {},
      "wiki-page-vote.remove": {},
      "wiki-page-vote.score": {},

      // Page files
      "files": {},
      "upload": {},
      "restore": {},
      "wiki-page-file": {},
      "wiki-page-file-no-files": {},
      "wiki-page-file-upload.select": {},
      "wiki-page-file-upload.name": {},
      "wiki-page-file.name": {},
      "wiki-page-file.created-at": {},
      "wiki-page-file.updated-at": {},
      "wiki-page-file.mime": {},
      "wiki-page-file.size": {},
      "wiki-page-file.page": {},
      "wiki-page-file-move-destination-page": {},
      "wiki-page-file-revision-type": {},
      "wiki-page-file-revision-type.create": {},
      "wiki-page-file-revision-type.regular": {},
      "wiki-page-file-revision-type.move": {},
      "wiki-page-file-revision-type.delete": {},
      "wiki-page-file-revision-type.rollback": {},
      "wiki-page-file-revision-type.undelete": {},
      "wiki-page-file-revision-type.undo": {},
      "wiki-page-file-restore.new-page": {},
      "wiki-page-file-restore.new-name": {},

      // Misc
      "wiki-page-edit": {},
      "wiki-page-parent": {},
      "wiki-page-delete": {},
      "wiki-page-move": {},
      "wiki-page-move.new-slug": {},
      "wiki-page-no-render": {},
      "wiki-page-source": {},
      "wiki-page-view-source": {}
    }
  } else {
    translateKeys = {
      ...translateKeys,

      // Page actions
      "restore": {},
      "wiki-page-restore": {},
      "wiki-page-restore.select": {},
      "wiki-page-create": {},
      "wiki-page-deleted": {
        // To be determined lazily
        datetime: "{$datetime}"
      }
    }
  }

  const internationalization = await translate(locales, translateKeys)
  let wikidotPageInfo: string | null = null
  let wikidotPageActions: ReturnType<typeof buildWikidotPageActionLabels> | null = null
  let wikidotPageWatch: ReturnType<typeof buildWikidotPageWatchLabel> = null

  if (errorStatus === null && responseType === "found") {
    const wikidotSnapshot = responseData.wikidot_snapshot

    if (
      responseData.page.from_wikidot &&
      wikidotSnapshot?.source_revision_count !== undefined &&
      wikidotSnapshot?.source_updated_at
    ) {
      wikidotPageInfo = buildWikidotPageInfoText({
        revision: wikidotSnapshot.source_revision_count,
        updatedAt: wikidotSnapshot.source_updated_at,
        locale: siteLocale
      })
    }

    if (responseData.page.from_wikidot) {
      const sourceShowsStandardActions = sourceShowsStandardWikidotPageActions(
        wikidotSnapshot?.source_site
      )

      wikidotPageActions = buildWikidotPageActionLabels({
        rating: wikidotSnapshot?.imported_rating ?? null,
        comments: wikidotSnapshot?.comments ?? null,
        locale: siteLocale,
        showRate: sourceShowsStandardActions && responseData.page_rating.enabled,
        showDiscuss:
          sourceShowsStandardActions &&
          (responseData.page_discussion.enabled ||
            responseData.page.discussion_thread_id !== null)
      })

      wikidotPageWatch = buildWikidotPageWatchLabel({
        sourceSite: wikidotSnapshot?.source_site,
        hasSession: !!parentData.user_session,
        locale: siteLocale
      })
    }
  }

  const forms = {
    pageDeleteForm: await superValidate(request, valibot(pageDeleteSchema)),
    pageEditForm: await superValidate(request, valibot(pageEditSchema)),
    fileUploadForm: await superValidate(request, valibot(pageFileUploadSchema)),
    fileEditForm: await superValidate(request, valibot(pageFileEditSchema)),
    fileMoveForm: await superValidate(request, valibot(pageFileMoveSchema)),
    fileRestoreForm: await superValidate(request, valibot(pageFileRestoreSchema)),
    layoutForm: await superValidate(request, valibot(layoutSchema)),
    pageMoveForm: await superValidate(request, valibot(pageMoveSchema)),
    pageParentForm: await superValidate(request, valibot(pageParentSchema)),
    // added here for type checking
    pageRestoreForm: await superValidate(request, valibot(pageRestoreSchema))
  }

  const missingPageEditForm = await superValidate(request, valibot(pageEditSchema))
  if (responseType === "missing" && responseData.new_page_wikitext !== null) {
    missingPageEditForm.data.wikitext = responseData.new_page_wikitext
  }
  const errorForms = {
    pageEditForm: missingPageEditForm,
    pageRestoreForm: await superValidate(request, valibot(pageRestoreSchema))
  }

  const viewData = {
    ...responseData,
    view: responseType,
    internationalization,
    wikidot_page_info: wikidotPageInfo,
    wikidot_page_actions: wikidotPageActions,
    wikidot_page_watch: wikidotPageWatch
  }

  if (errorStatus !== null) {
    error(errorStatus, buildPageLoadData(parentData, viewData, errorForms))
  }

  runRedirect(responseData, slug, extra, request.url)

  // Return to page for rendering
  return buildPageLoadData(parentData, viewData, forms)
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
