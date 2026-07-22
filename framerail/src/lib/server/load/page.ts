import defaults from "$lib/defaults"

import { buildAnonymousArticleResponseCacheMetadata } from "$lib/server/article-response-cache"
import { authGetSession } from "$lib/server/auth/getSession"
import { resolvePageRedirect } from "$lib/server/page-redirect"
import {
  pageDelete,
  pageDeletedGet,
  pageEdit,
  pageEditPermission,
  pageHistory,
  pageLayout,
  pageMove,
  pageParentGet,
  pageParentUpdate,
  pageRestore,
  pageRevision,
  pageRollback,
  pageScore,
  pageVoteCast,
  pageVoteList,
  pageVoteRemove
} from "$lib/server/deepwell/page"
import {
  pageFileCreate,
  pageFileDelete,
  pageFileEdit,
  pageFileHistory,
  pageFileList,
  pageFileMove,
  pageFileRestore,
  pageFileRollback
} from "$lib/server/deepwell/pageFile"
import { translate } from "$lib/server/deepwell/translate"
import { articleView, preloadView } from "$lib/server/deepwell/views"
import { buildPageLoadData } from "$lib/server/load/page-data"
import { resolvePageMutationUserId } from "$lib/server/load/local-authoring-actor"
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
import { type DeepwellError, DeleteOptions, Layout } from "$lib/types"
import {
  buildWikidotPageActionLabels,
  sourceShowsStandardWikidotPageActions
} from "$lib/wikidot-page-actions"
import { buildWikidotPageInfoText } from "$lib/wikidot-page-info"
import { buildWikidotPageWatchLabel } from "$lib/wikidot-page-watch"
import { toIntlLocales } from "$lib/wikidot-locale"
import { error, redirect } from "@sveltejs/kit"
import { fail, superValidate, withFiles } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  array,
  file,
  literal,
  nullable,
  number,
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

const DEEPWELL_PERMISSION_DENIED = 3106

function failForDeepwellError(error: DeepwellError, body: Record<string, unknown> = {}) {
  return fail(error.code === DEEPWELL_PERMISSION_DENIED ? 403 : 500, {
    ...body,
    message: error.message,
    code: error.code,
    data: error.data
  })
}

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

  // Process response, performing redirects etc
  const { data: responseData, type: responseType } = response

  const checkRedirect = true
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
        showRate: sourceShowsStandardActions,
        showDiscuss: sourceShowsStandardActions && wikidotSnapshot?.comments !== 0
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

  // TODO remove checkRedirect when errorStatus is fixed
  if (checkRedirect) {
    runRedirect(responseData, slug, extra, request.url)
  }

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
const baseSchema = {
  pageId: number(),
  siteId: number(),
  lastRevisionId: number()
}

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
        siteId,
        pageId,
        userId,
        ipAddress,
        slug,
        lastRevisionId,
        newSlug,
        comments,
        { sessionToken, siteId, page: pageId ?? slug }
      )
      return { form, res, option: DeleteOptions.Move }
    } else {
      const res = await pageDelete(
        siteId,
        pageId,
        userId,
        ipAddress,
        slug,
        lastRevisionId,
        comments,
        { sessionToken, siteId, page: pageId ?? slug }
      )
      return { form, res, option: DeleteOptions.Delete }
    }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageDeleteSchema = variant("option", [
  object({
    ...baseSchema,
    option: literal(DeleteOptions.Move),
    newSlug: string(),
    comments: string()
  }),
  object({
    ...baseSchema,
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
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
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
      siteId,
      pageId,
      userId,
      ipAddress,
      slug,
      lastRevisionId,
      comments,
      wikitext,
      title,
      altTitle,
      tags,
      layout,
      { sessionToken, siteId, page: pageId ?? slug }
    )

    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageEditSchema = object({
  ...baseSchema,
  title: string(),
  altTitle: string(),
  wikitext: string(),
  tags: string(),
  comments: string(),
  layout: optional(nullable(vEnum(Layout)))
})

/* ----- Page File ----- */
export async function pageFileListAction({ request }: RequestEvent) {
  try {
    const requestData: { siteId: number; pageId: number; deleted: Optional<boolean> } =
      await request.json()

    const { siteId, pageId, deleted } = requestData

    const res = await pageFileList(siteId, pageId, deleted)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page File Upload ----- */
export async function pageFileUploadAction({
  request,
  cookies,
  getClientAddress
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageFileUploadSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const { siteId, pageId, file, name, comments } = form.data
    const res = await pageFileCreate(
      siteId,
      pageId,
      session?.user_id,
      name === "" ? undefined : name,
      file,
      comments,
      getClientAddress(),
      { sessionToken, siteId, page: pageId }
    )

    return withFiles({ form, res })
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageFileUploadSchema = object({
  ...baseSchema,
  file: file(),
  name: string(),
  comments: string()
})

/* ----- Page File Delete ----- */
export async function pageFileDeleteAction({ request, cookies }: RequestEvent) {
  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const requestData: {
      siteId: number
      pageId: number
      fileId: number
      lastRevisionId: number
      comments: Optional<string>
    } = await request.json()

    const { siteId, pageId, fileId, lastRevisionId, comments } = requestData

    const res = await pageFileDelete(
      siteId,
      pageId,
      session?.user_id,
      fileId,
      lastRevisionId,
      comments ?? "",
      { sessionToken, siteId, page: pageId }
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page File Edit ----- */
export async function pageFileEditAction({
  request,
  cookies,
  getClientAddress
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageFileEditSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const { siteId, pageId, lastRevisionId, fileId, file, name, comments } = form.data
    const res = await pageFileEdit(
      siteId,
      pageId,
      session?.user_id,
      fileId,
      name === "" ? undefined : name,
      file,
      lastRevisionId,
      comments,
      getClientAddress(),
      { sessionToken, siteId, page: pageId }
    )

    return withFiles({ form, res })
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageFileEditSchema = object({
  ...baseSchema,
  fileId: number(),
  file: optional(file()),
  name: string(),
  comments: string()
})

/* ----- Page File Move ----- */
export async function pageFileMoveAction({ request, cookies }: RequestEvent) {
  const form = await superValidate(request, valibot(pageFileMoveSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const { siteId, pageId, lastRevisionId, fileId, destinationPage, name, comments } =
      form.data
    const res = await pageFileMove(
      siteId,
      pageId,
      destinationPage,
      session?.user_id,
      fileId,
      lastRevisionId,
      name === "" ? undefined : name,
      comments,
      { sessionToken, siteId, page: pageId }
    )

    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageFileMoveSchema = object({
  ...baseSchema,
  fileId: number(),
  destinationPage: string(),
  name: string(),
  comments: string()
})

/* ----- Page File Restore ----- */
export async function pageFileRestoreAction({
  request,
  cookies,
  getClientAddress
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageFileRestoreSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const { siteId: requestSiteId, siteSlug } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")

  try {
    const session = sessionToken ? await authGetSession(sessionToken) : undefined
    const { siteId, pageId, fileId, newPage, newName, comments } = form.data
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
    const res = await pageFileRestore(
      siteId,
      pageId,
      userId,
      fileId,
      newPage === "" ? undefined : newPage,
      newName === "" ? undefined : newName,
      comments,
      getClientAddress(),
      { sessionToken, siteId, page: pageId }
    )

    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageFileRestoreSchema = object({
  ...baseSchema,
  fileId: number(),
  newPage: string(),
  newName: string(),
  comments: string()
})

/* ----- Page File History ----- */
export async function pageFileHistoryAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      fileId: number
      revisionNumber: Optional<number>
      limit: Optional<number>
    } = await request.json()

    const { siteId, pageId, fileId, revisionNumber, limit } = requestData

    const res = await pageFileHistory(siteId, pageId, fileId, revisionNumber, limit)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page File Rollback ----- */
export async function pageFileRollbackAction({
  request,
  cookies,
  getClientAddress
}: RequestEvent) {
  const { siteId: requestSiteId, siteSlug } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")

  try {
    const session = sessionToken ? await authGetSession(sessionToken) : undefined
    const requestData: {
      siteId: number
      pageId: number
      fileId: number
      revisionNumber: number
      lastRevisionId: number
      comments: Optional<string>
    } = await request.json()

    const { siteId, pageId, fileId, revisionNumber, lastRevisionId, comments } =
      requestData
    const userId = resolvePageMutationUserId(
      session?.user_id,
      siteSlug,
      requestSiteId,
      siteId
    )
    if (userId === undefined) {
      return fail(403, {
        message: "Permission denied."
      })
    }
    const res = await pageFileRollback(
      siteId,
      pageId,
      userId,
      fileId,
      lastRevisionId,
      revisionNumber,
      comments,
      getClientAddress(),
      { sessionToken, siteId, page: pageId },
      false
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page History ----- */
export async function pageHistoryAction({ request, locals }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: Optional<number>
      limit: Optional<number>
    } = await request.json()

    const { siteId, pageId, revisionNumber, limit } = requestData

    const res = await pageHistory(
      siteId,
      pageId,
      revisionNumber,
      limit,
      getRequestContext(locals)
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Revision ----- */
export async function pageRevisionAction({ request, locals }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: number
      compiledHtml: Optional<boolean>
      wikitext: Optional<boolean>
    } = await request.json()

    const { siteId, pageId, revisionNumber, compiledHtml, wikitext } = requestData

    const res = await pageRevision(
      siteId,
      pageId,
      revisionNumber,
      compiledHtml ?? true,
      wikitext ?? true,
      getRequestContext(locals)
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Rollback ----- */
export async function pageRollbackAction({
  request,
  params,
  getClientAddress,
  cookies
}: RequestEvent) {
  const { slug } = params
  const ipAddress = getClientAddress()
  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: number
      comments: Optional<string>
      lastRevisionId: number
    } = await request.json()

    const { siteId, pageId, revisionNumber, comments, lastRevisionId } = requestData

    const res = await pageRollback(
      siteId,
      pageId,
      session?.user_id,
      ipAddress,
      slug,
      lastRevisionId,
      revisionNumber,
      comments ?? "",
      { sessionToken, siteId, page: pageId ?? slug }
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Layout ----- */
export async function layoutAction({ request, cookies, getClientAddress }: RequestEvent) {
  const form = await superValidate(request, valibot(layoutSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)
  const ipAddress = getClientAddress()

  try {
    const { siteId, pageId, layout } = form.data
    await pageLayout(siteId, pageId, session?.user_id, ipAddress, layout, {
      sessionToken,
      siteId,
      page: pageId
    })

    return { form }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const layoutSchema = object({
  ...baseSchema,
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
  const session = await authGetSession(sessionToken)
  const ipAddress = getClientAddress()
  const { slug } = params

  try {
    const { siteId, pageId, lastRevisionId, newSlug, comments } = form.data
    const res = await pageMove(
      siteId,
      pageId,
      session?.user_id,
      ipAddress,
      slug,
      lastRevisionId,
      newSlug,
      comments,
      { sessionToken, siteId, page: pageId ?? slug }
    )
    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageMoveSchema = object({
  ...baseSchema,
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
  const session = await authGetSession(sessionToken)

  try {
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
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageParentSchema = object({
  ...baseSchema,
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
    } = await request.json()
    const { siteId, pageId, slug } = requestData
    const res = await pageParentGet(siteId, pageId, slug)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Vote Get ----- */
export async function pageVoteGetAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await request.json()
    const { siteId, pageId } = requestData
    const res = await pageVoteList(siteId, pageId)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Vote Cast ----- */
export async function pageVoteCastAction({ request, cookies }: RequestEvent) {
  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const requestData: {
      siteId: number
      pageId: number
      value: number
    } = await request.json()
    const { siteId, pageId, value } = requestData
    const res = await pageVoteCast(siteId, pageId, session?.user_id, value)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Vote Cancel ----- */
export async function pageVoteCancelAction({ request, cookies }: RequestEvent) {
  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await request.json()
    const { siteId, pageId } = requestData
    const res = await pageVoteRemove(siteId, pageId, session?.user_id)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Score ----- */
export async function pageScoreAction({ request, params }: RequestEvent) {
  const { slug } = params

  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await request.json()
    const { siteId, pageId } = requestData
    const res = await pageScore(siteId, pageId, slug)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Deleted Get ----- */
export async function pageDeletedGetAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      slug: string
    } = await request.json()
    const { siteId, slug } = requestData
    const res = await pageDeletedGet(siteId, slug)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error)
  }
}

/* ----- Page Restore ----- */
export async function pageRestoreAction({
  request,
  cookies,
  getClientAddress
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageRestoreSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)
  const ipAddress = getClientAddress()

  try {
    const { siteId, pageId, comments } = form.data
    const res = await pageRestore(siteId, pageId, session?.user_id, ipAddress, comments, {
      sessionToken,
      siteId,
      page: pageId
    })
    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return failForDeepwellError(error, { form })
  }
}

const pageRestoreSchema = object({
  ...baseSchema,
  comments: string()
})
