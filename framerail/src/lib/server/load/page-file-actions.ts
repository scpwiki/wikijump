import { authGetSession } from "$lib/server/auth/get-session"
import {
  pageFileCreate,
  pageFileDelete,
  pageFileEdit,
  pageFileHistory,
  pageFileList,
  pageFileMove,
  pageFileRestore,
  pageFileRollback
} from "$lib/server/deepwell/page-file"
import { resolvePageMutationUserId } from "$lib/server/load/local-authoring-actor"
import {
  failForActionError,
  failForMissingSession,
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page-action-shared"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { fail } from "@sveltejs/kit"
import { superValidate, withFiles } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { file, number, object, optional, string } from "valibot"

import type { Optional } from "$lib/types"
import type { RequestEvent } from "@sveltejs/kit"

export async function pageFileListAction({ request }: RequestEvent) {
  try {
    const requestData: { siteId: number; pageId: number; deleted: Optional<boolean> } =
      await readActionJson(request)
    const { siteId, pageId, deleted } = requestData
    const res = await pageFileList(siteId, pageId, deleted)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

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
  if (!sessionToken) return failForMissingSession({ form })
  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, file, name, comments } = form.data
    const res = await pageFileCreate(
      {
        siteId,
        pageId,
        userId: session.user_id,
        name: name === "" ? undefined : name,
        file,
        revisionComments: comments,
        ipAddress: getClientAddress()
      },
      { sessionToken, siteId, page: pageId }
    )

    return withFiles({ form, res })
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageFileUploadSchema = object({
  ...pageMutationBaseSchema,
  file: file(),
  name: string(),
  comments: string()
})

export async function pageFileDeleteAction({ request, cookies }: RequestEvent) {
  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession()
  try {
    const session = await authGetSession(sessionToken)
    const requestData: {
      siteId: number
      pageId: number
      fileId: number
      lastRevisionId: number
      comments: Optional<string>
    } = await readActionJson(request)

    const { siteId, pageId, fileId, lastRevisionId, comments } = requestData
    const res = await pageFileDelete(
      {
        siteId,
        pageId,
        userId: session.user_id,
        fileId,
        lastRevisionId,
        revisionComments: comments ?? ""
      },
      { sessionToken, siteId, page: pageId }
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

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
  if (!sessionToken) return failForMissingSession({ form })
  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, lastRevisionId, fileId, file, name, comments } = form.data
    const res = await pageFileEdit(
      {
        siteId,
        pageId,
        userId: session.user_id,
        fileId,
        name: name === "" ? undefined : name,
        file,
        lastRevisionId,
        revisionComments: comments,
        ipAddress: getClientAddress()
      },
      { sessionToken, siteId, page: pageId }
    )

    return withFiles({ form, res })
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageFileEditSchema = object({
  ...pageMutationBaseSchema,
  fileId: number(),
  file: optional(file()),
  name: string(),
  comments: string()
})

export async function pageFileMoveAction({ request, cookies }: RequestEvent) {
  const form = await superValidate(request, valibot(pageFileMoveSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession({ form })
  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, lastRevisionId, fileId, destinationPage, name, comments } =
      form.data
    const res = await pageFileMove(
      {
        siteId,
        currentPageId: pageId,
        destinationPage,
        userId: session.user_id,
        fileId,
        lastRevisionId,
        name: name === "" ? undefined : name,
        revisionComments: comments
      },
      { sessionToken, siteId, page: pageId }
    )

    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageFileMoveSchema = object({
  ...pageMutationBaseSchema,
  fileId: number(),
  destinationPage: string(),
  name: string(),
  comments: string()
})

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
      {
        siteId,
        pageId,
        userId,
        fileId,
        newPage: newPage === "" ? undefined : newPage,
        newName: newName === "" ? undefined : newName,
        revisionComments: comments,
        ipAddress: getClientAddress()
      },
      { sessionToken, siteId, page: pageId }
    )

    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageFileRestoreSchema = object({
  ...pageMutationBaseSchema,
  fileId: number(),
  newPage: string(),
  newName: string(),
  comments: string()
})

export async function pageFileHistoryAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      fileId: number
      revisionNumber: Optional<number>
      limit: Optional<number>
    } = await readActionJson(request)

    const { siteId, pageId, fileId, revisionNumber, limit } = requestData
    const res = await pageFileHistory(siteId, pageId, fileId, revisionNumber, limit)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

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
    } = await readActionJson(request)

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
      {
        siteId,
        pageId,
        userId,
        fileId,
        lastRevisionId,
        revisionNumber,
        revisionComments: comments,
        ipAddress: getClientAddress()
      },
      { sessionToken, siteId, page: pageId }
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}
