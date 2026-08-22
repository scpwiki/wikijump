import defaults from "$lib/defaults"

import { authGetSession } from "$lib/server/auth/getSession"
import {
  pageDelete,
  pageDeletedGet,
  pageEdit,
  pageEditPermission,
  pageHistory,
  pageLayout,
  pageLockCreate,
  pageLockHistory,
  pageLockRemove,
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
import { pageView } from "$lib/server/deepwell/views"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { type DeepwellError, DeleteOptions, Layout, PageLockType } from "$lib/types"
import { error, redirect } from "@sveltejs/kit"
import { fail, superValidate, withFiles } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  array,
  boolean,
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

import type { PageView, PreloadDataAsync } from "$lib/server/deepwell/views"
import type { Optional, TranslateKeys } from "$lib/types"
import type { Cookies, RequestEvent } from "@sveltejs/kit"
import { getRequestContext } from "./request-ctx"

export async function loadPage(
  slug: Optional<string>,
  extra: Optional<string>,
  request: Request,
  cookies: Cookies,
  preloadData: PreloadDataAsync
) {
  // Set up parameters
  const { siteId } = loadSiteInfo(request.headers)
  const route = slug || extra ? { slug, extra } : null
  const sessionToken = cookies.get("wikijump_token")

  const parentData = await preloadData()
  const locales = parentData.locales

  // Request data from backend
  const response = await pageView(siteId, locales, route, sessionToken)

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
    "wiki-page-layout.toast": {},
    "wiki-page-edit.toast": {},

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
        date: new Date(updatedAt).toLocaleString(locales),
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
      "wiki-page-vote.toast-set": {},
      "wiki-page-vote.toast-remove": {},

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
      "wiki-page-delete.toast": {},
      "wiki-page-lock": {},
      "wiki-page-lock.permission-only": {},
      "wiki-page-lock.permission-only-text": {},
      "wiki-page-lock.author-or-permission-only": {},
      "wiki-page-lock.author-or-permission-only-text": {},
      "wiki-page-lock.reason": {},
      "wiki-page-lock.expires-at": {},
      "wiki-page-lock.override": {},
      "wiki-page-lock.history": {},
      "wiki-page-lock.history-type": {},
      "wiki-page-lock.history-user": {},
      "wiki-page-lock.history-reason": {},
      "wiki-page-lock.history-created": {},
      "wiki-page-lock.history-expires": {},
      "wiki-page-lock.history-status": {},
      "wiki-page-lock.history-active": {},
      "wiki-page-lock.history-expired": {},
      "wiki-page-lock.history-removed": {},
      "wiki-page-lock.history-overridden": {},
      "wiki-page-lock.history-none": {},
      "wiki-page-lock.remove": {},
      "wiki-page-move": {},
      "wiki-page-move.new-slug": {},
      "wiki-page-move.toast": {},
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
      "wiki-page-restore.toast": {},
      "wiki-page-create": {},
      "wiki-page-deleted": {
        // To be determined lazily
        datetime: "{$datetime}"
      }
    }
  }

  const internationalization = await translate(locales, translateKeys)

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
    pageLockForm: await superValidate(request, valibot(pageLockSchema)),
    // added here for type checking
    pageRestoreForm: await superValidate(request, valibot(pageRestoreSchema))
  }

  const errorForms = {
    pageEditForm: await superValidate(request, valibot(pageEditSchema)),
    pageRestoreForm: await superValidate(request, valibot(pageRestoreSchema))
  }

  const viewData = { ...responseData, view: responseType, internationalization }

  if (errorStatus !== null) {
    error(errorStatus, { ...viewData, forms: errorForms })
  }

  // TODO remove checkRedirect when errorStatus is fixed
  if (checkRedirect) {
    runRedirect(responseData, slug, extra)
  }

  // Return to page for rendering
  return { ...viewData, forms }
}

function runRedirect(
  viewData: PageView["data"],
  originalSlug: Optional<string>,
  extra: Optional<string>
): void {
  if (!viewData.redirect_page) {
    // Nothing to do
    return
  }

  const slug: Optional<string> = viewData.redirect_page || originalSlug
  const route: string = buildRoute(slug, extra)
  redirect(308, `/${route}`)
}

function buildRoute(slug: Optional<string>, extra: Optional<string>): string {
  // Combines a nullable slug and extra to form a route for redirection.
  //
  // Test cases:
  // null, null => ''
  // 'start', null => 'start'
  // 'start', '' => 'start'
  // 'start', 'comments/show' => 'start/comments/show'
  // null, 'xyz' => (impossible)

  if (slug === null) {
    return ""
  } else if (!extra) {
    return slug ?? ""
  } else {
    return `${slug}/${extra}`
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
  const sessionToken = cookies.get("wikijump_token")
  const ipAddress = getClientAddress()

  const session = await authGetSession(sessionToken)

  try {
    const { siteId, pageId, lastRevisionId, option, comments } = form.data
    if (option === DeleteOptions.Move) {
      const { newSlug } = form.data
      const res = await pageMove(
        siteId,
        pageId,
        session?.user_id,
        ipAddress,
        slug,
        lastRevisionId,
        newSlug,
        comments
      )
      return { form, res, option: DeleteOptions.Move }
    } else {
      const res = await pageDelete(
        siteId,
        pageId,
        session?.user_id,
        ipAddress,
        slug,
        lastRevisionId,
        comments
      )
      return { form, res, option: DeleteOptions.Delete }
    }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
export async function pageEditPermissionAction({ locals }: RequestEvent) {
  try {
    const res = await pageEditPermission(getRequestContext(locals))
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
  const sessionToken = cookies.get("wikijump_token")
  const ipAddress = getClientAddress()

  const session = await authGetSession(sessionToken)

  try {
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
    const tags = tagsStr.split(" ").filter((tag) => tag.length)
    const res = await pageEdit(
      siteId,
      pageId,
      session?.user_id,
      ipAddress,
      slug,
      lastRevisionId,
      comments,
      wikitext,
      title,
      altTitle,
      tags,
      layout
    )

    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

/* ----- Page File Upload ----- */
export async function pageFileUploadAction({ request, cookies }: RequestEvent) {
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
      comments
    )

    return withFiles({ form, res })
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
      comments ?? ""
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

/* ----- Page File Edit ----- */
export async function pageFileEditAction({ request, cookies }: RequestEvent) {
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
      comments
    )

    return withFiles({ form, res })
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
      comments
    )

    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
export async function pageFileRestoreAction({ request, cookies }: RequestEvent) {
  const form = await superValidate(request, valibot(pageFileRestoreSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
    const { siteId, pageId, fileId, newPage, newName, comments } = form.data
    const res = await pageFileRestore(
      siteId,
      pageId,
      session?.user_id,
      fileId,
      newPage === "" ? undefined : newPage,
      newName === "" ? undefined : newName,
      comments
    )

    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

/* ----- Page File Rollback ----- */
export async function pageFileRollbackAction({ request, cookies }: RequestEvent) {
  const sessionToken = cookies.get("wikijump_token")
  const session = await authGetSession(sessionToken)

  try {
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
    const res = await pageFileRollback(
      siteId,
      pageId,
      session?.user_id,
      fileId,
      lastRevisionId,
      revisionNumber,
      comments,
      false
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

/* ----- Page History ----- */
export async function pageHistoryAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: Optional<number>
      limit: Optional<number>
    } = await request.json()

    const { siteId, pageId, revisionNumber, limit } = requestData

    const res = await pageHistory(siteId, pageId, revisionNumber, limit)
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

/* ----- Page Revision ----- */
export async function pageRevisionAction({ request }: RequestEvent) {
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
      wikitext ?? true
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
      comments ?? ""
    )
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    await pageLayout(siteId, pageId, session?.user_id, ipAddress, layout)

    return { form }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
      comments
    )
    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
      removeParents
    )
    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
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
    const res = await pageRestore(siteId, pageId, session?.user_id, ipAddress, comments)
    return { form, res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

const pageRestoreSchema = object({
  ...baseSchema,
  comments: string()
})

/* ----- Page Lock Create ----- */
export async function pageLockCreateAction({
  request,
  getClientAddress,
  locals
}: RequestEvent) {
  const form = await superValidate(request, valibot(pageLockSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const ipAddress = getClientAddress()

  try {
    const { pageId, lockType, reason, expiresAt, overrideExisting } = form.data
    await pageLockCreate(
      pageId,
      lockType,
      reason,
      expiresAt,
      overrideExisting,
      ipAddress,
      getRequestContext(locals)
    )
    return { form }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      form,
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

const pageLockSchema = object({
  pageId: number(),
  lockType: optional(vEnum(PageLockType), PageLockType.PermissionOnly),
  reason: string(),
  expiresAt: optional(string()),
  overrideExisting: optional(boolean(), false)
})

/* ----- Page Lock Remove ----- */
export async function pageLockRemoveAction({
  request,
  getClientAddress,
  locals
}: RequestEvent) {
  const ipAddress = getClientAddress()

  try {
    const { pageId }: { pageId: number } = await request.json()
    await pageLockRemove(pageId, ipAddress, getRequestContext(locals))
    return {}
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}

/* ----- Page Lock History ----- */
export async function pageLockHistoryAction({ request, locals }: RequestEvent) {
  try {
    const { pageId }: { pageId: number } = await request.json()
    const res = await pageLockHistory(pageId, getRequestContext(locals))
    return { res }
  } catch (e) {
    const error = e as DeepwellError
    return fail(500, {
      message: error.message,
      code: error.code,
      data: error.data
    })
  }
}
