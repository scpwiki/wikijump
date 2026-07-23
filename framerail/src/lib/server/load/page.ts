import defaults from "$lib/defaults"

import { buildAnonymousArticleResponseCacheMetadata } from "$lib/server/article-response-cache"
import { authGetSession } from "$lib/server/auth/get-session"
import { resolvePageRedirect } from "$lib/server/page-redirect"
import {
  pageDelete,
  pageEdit,
  pageEditPermission,
  pageLayout,
  pageMove,
  pageParentGet,
  pageParentUpdate,
  pageScore,
  pageVoteCast,
  pageVoteList,
  pageVoteRemove
} from "$lib/server/deepwell/page"
import { translate } from "$lib/server/deepwell/translate"
import { articleView, preloadView } from "$lib/server/deepwell/views"
import { buildPageLoadData } from "$lib/server/load/page-data"
import { resolvePageMutationUserId } from "$lib/server/load/local-authoring-actor"
import {
  finalizePreloadData,
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import {
  failForActionError,
  failForMissingSession,
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page-action-shared"
import {
  pageFileEditSchema,
  pageFileMoveSchema,
  pageFileRestoreSchema,
  pageFileUploadSchema
} from "$lib/server/load/page-file-actions"
import { loadSiteInfo } from "$lib/server/load/site-info"
import {
  buildWikidotRequestInfo,
  requestHostFromRequest
} from "$lib/server/wikidot-request-info"
import { DeleteOptions, Layout } from "$lib/types"
import {
  buildWikidotPageActionLabels,
  sourceShowsStandardWikidotPageActions
} from "$lib/wikidot-page-actions"
import { buildWikidotPageInfoText } from "$lib/wikidot-page-info"
import { buildWikidotPageWatchLabel } from "$lib/wikidot-page-watch"
import { toIntlLocales } from "$lib/wikidot-locale"
import { error, redirect } from "@sveltejs/kit"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  array,
  literal,
  nullable,
  object,
  optional,
  string,
  variant,
  enum as vEnum
} from "valibot"

import type { PageView } from "$lib/server/deepwell/views"
import type { Optional, TranslateKeys } from "$lib/types"
import type { Cookies, RequestEvent } from "@sveltejs/kit"
import { getRequestContext, withDefaultPageContext } from "./request-ctx"

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

/* ----- Base ----- */
/* ----- Page Delete ----- */
export async function pageDeleteAction({
  request,
  params,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageDeleteSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const { slug } = params
  const { siteId: requestSiteId, siteSlug } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")
  const ipAddress = getClientAddress()

  try {
    const session = sessionToken ? await authGetSession(sessionToken) : undefined
    const { siteId, pageId, lastRevisionId, option, comments } = form.data
    const userId = resolvePageMutationUserId(
      session?.user_id,
      siteSlug,
      requestSiteId,
      siteId
    )
    if (userId === undefined) {
      return fail(403, {
        form,
        message: "Permission denied."
      })
    }
    if (option === DeleteOptions.Move) {
      const { newSlug } = form.data
      const res = await pageMove(
        {
          siteId,
          pageId,
          userId,
          userIpAddr: ipAddress,
          slug,
          lastRevisionId,
          newSlug,
          revisionComments: comments
        },
        { sessionToken, siteId, page: pageId ?? slug }
      )
      return { form, res, option: DeleteOptions.Move }
    } else {
      const res = await pageDelete(
        {
          siteId,
          pageId,
          userId,
          userIpAddr: ipAddress,
          slug,
          lastRevisionId,
          revisionComments: comments
        },
        { sessionToken, siteId, page: pageId ?? slug }
      )
      return { form, res, option: DeleteOptions.Delete }
    }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

const pageDeleteSchema = variant("option", [
  object({
    ...pageMutationBaseSchema,
    option: literal(DeleteOptions.Move),
    newSlug: string(),
    comments: string()
  }),
  object({
    ...pageMutationBaseSchema,
    option: literal(DeleteOptions.Delete),
    comments: string()
  })
])

/* ----- Page Edit Check Permission ----- */
export async function pageEditPermissionAction({
  request,
  cookies,
  locals
}: RequestEvent) {
  try {
    let requestContext = getRequestContext(locals)

    if (requestContext?.page === undefined) {
      const { siteId } = loadSiteInfo(request.headers)
      const requestLocales = getPreloadRequestLocales(request)
      const backendLocales = getPreloadBackendLocales(requestLocales)
      const sessionToken = cookies.get("wikijump_token")
      const { site } = await preloadView(siteId, backendLocales, sessionToken)
      requestContext = withDefaultPageContext(requestContext, site.default_page)
    }

    const res = await pageEditPermission(requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

/* ----- Page Edit ----- */
export async function pageEditAction({
  request,
  params,
  getClientAddress,
  cookies
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageEditSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const { slug } = params
  const { siteId: requestSiteId, siteSlug } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")
  const ipAddress = getClientAddress()

  try {
    const session = sessionToken ? await authGetSession(sessionToken) : undefined
    const {
      siteId,
      pageId,
      lastRevisionId,
      comments,
      wikitext,
      title,
      altTitle,
      tags: tagsStr,
      layout
    } = form.data
    const userId = resolvePageMutationUserId(
      session?.user_id,
      siteSlug,
      requestSiteId,
      siteId
    )
    if (userId === undefined) {
      return fail(403, {
        form,
        message: "Permission denied."
      })
    }
    const tags = tagsStr.split(" ").filter((tag) => tag.length)
    const res = await pageEdit(
      {
        siteId,
        pageId,
        userId,
        userIpAddr: ipAddress,
        slug,
        lastRevisionId,
        revisionComments: comments,
        wikitext,
        title,
        altTitle,
        tags,
        layout
      },
      { sessionToken, siteId, page: pageId ?? slug }
    )

    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

const pageEditSchema = object({
  ...pageMutationBaseSchema,
  title: string(),
  altTitle: string(),
  wikitext: string(),
  tags: string(),
  comments: string(),
  layout: optional(nullable(vEnum(Layout)))
})

/* ----- Page Layout ----- */
export async function layoutAction({ request, cookies, getClientAddress }: RequestEvent) {
  const form = await superValidate(request, valibot(layoutSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession({ form })
  const ipAddress = getClientAddress()

  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, layout } = form.data
    await pageLayout(siteId, pageId, session?.user_id, ipAddress, layout, {
      sessionToken,
      siteId,
      page: pageId
    })

    return { form }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

const layoutSchema = object({
  ...pageMutationBaseSchema,
  layout: nullable(vEnum(Layout))
})

/* ----- Page Move ----- */
export async function pageMoveAction({
  request,
  cookies,
  params,
  getClientAddress
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageMoveSchema))
  if (!form.valid) {
    return fail(400, { form })
  }
  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession({ form })
  const ipAddress = getClientAddress()
  const { slug } = params

  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, lastRevisionId, newSlug, comments } = form.data
    const res = await pageMove(
      {
        siteId,
        pageId,
        userId: session?.user_id,
        userIpAddr: ipAddress,
        slug,
        lastRevisionId,
        newSlug,
        revisionComments: comments
      },
      { sessionToken, siteId, page: pageId ?? slug }
    )
    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

const pageMoveSchema = object({
  ...pageMutationBaseSchema,
  newSlug: string(),
  comments: string()
})

/* ----- Page Parent Set ----- */
export async function pageParentSetAction({ request, cookies }: RequestEvent) {
  const form = await superValidate(request, valibot(pageParentSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession({ form })
  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, addParents, removeParents } = form.data
    const res = await pageParentUpdate(
      siteId,
      pageId,
      session?.user_id,
      addParents,
      removeParents,
      { sessionToken, siteId, page: pageId }
    )
    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

const pageParentSchema = object({
  ...pageMutationBaseSchema,
  parents: string(),
  addParents: optional(array(string())),
  removeParents: optional(array(string()))
})

/* ----- Page Parent Get ----- */
export async function pageParentGetAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      slug: string
    } = await readActionJson(request)
    const { siteId, pageId, slug } = requestData
    const res = await pageParentGet(siteId, pageId, slug)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

/* ----- Page Vote Get ----- */
export async function pageVoteGetAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await readActionJson(request)
    const { siteId, pageId } = requestData
    const res = await pageVoteList(pageId, { siteId, page: pageId })
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

/* ----- Page Vote Cast ----- */
export async function pageVoteCastAction({ request, cookies }: RequestEvent) {
  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession()
  try {
    const session = await authGetSession(sessionToken)
    const requestData: {
      siteId: number
      pageId: number
      value: number
    } = await readActionJson(request)
    const { siteId, pageId, value } = requestData
    if (!session) {
      return fail(401, { message: "login is required to rate this page" })
    }
    const res = await pageVoteCast(pageId, session.user_id, value, {
      sessionToken,
      siteId,
      page: pageId
    })
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

/* ----- Page Vote Cancel ----- */
export async function pageVoteCancelAction({ request, cookies }: RequestEvent) {
  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession()
  try {
    const session = await authGetSession(sessionToken)
    const requestData: {
      siteId: number
      pageId: number
    } = await readActionJson(request)
    const { siteId, pageId } = requestData
    if (!session) {
      return fail(401, { message: "login is required to rate this page" })
    }
    const res = await pageVoteRemove(pageId, session.user_id, {
      sessionToken,
      siteId,
      page: pageId
    })
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

/* ----- Page Score ----- */
export async function pageScoreAction({ request, params }: RequestEvent) {
  const { slug } = params

  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await readActionJson(request)
    const { siteId, pageId } = requestData
    const res = await pageScore(siteId, pageId, slug)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}
