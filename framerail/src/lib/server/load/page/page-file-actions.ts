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
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page/page-action-shared"
import { withPageFileClientAddress } from "$lib/server/deepwell/page-file-mutation-payloads"
import { resolvePageActionRequestContext } from "$lib/server/load/page/page-action-context"
import { fail } from "@sveltejs/kit"
import { superValidate, withFiles } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { file, number, object, optional, string } from "valibot"

import type { Optional } from "$lib/types"
import type { RequestEvent } from "@sveltejs/kit"

export async function pageFileListAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: { siteId: number; pageId: number; deleted: Optional<boolean> } =
      await readActionJson(request)
    const { siteId, pageId, deleted } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageFileList(siteId, pageId, deleted, context.requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageFileUploadAction(event: RequestEvent) {
  const { request, getClientAddress } = event
  const form = await superValidate(request, valibot(pageFileUploadSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  try {
    const { siteId, pageId, file, name, comments } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageFileCreate(
      withPageFileClientAddress(getClientAddress, {
        siteId,
        pageId,
        userId: context.sessionUserId,
        name: name === "" ? undefined : name,
        file,
        revisionComments: comments
      }),
      context.requestContext
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

export async function pageFileDeleteAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
      fileId: number
      lastRevisionId: number
      comments: Optional<string>
    } = await readActionJson(request)

    const { siteId, pageId, fileId, lastRevisionId, comments } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageFileDelete(
      {
        siteId,
        pageId,
        userId: context.sessionUserId,
        fileId,
        lastRevisionId,
        revisionComments: comments ?? ""
      },
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageFileEditAction(event: RequestEvent) {
  const { request, getClientAddress } = event
  const form = await superValidate(request, valibot(pageFileEditSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  try {
    const { siteId, pageId, lastRevisionId, fileId, file, name, comments } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageFileEdit(
      withPageFileClientAddress(getClientAddress, {
        siteId,
        pageId,
        userId: context.sessionUserId,
        fileId,
        name: name === "" ? undefined : name,
        file,
        lastRevisionId,
        revisionComments: comments
      }),
      context.requestContext
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

export async function pageFileMoveAction(event: RequestEvent) {
  const { request } = event
  const form = await superValidate(request, valibot(pageFileMoveSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  try {
    const { siteId, pageId, lastRevisionId, fileId, destinationPage, name, comments } =
      form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageFileMove(
      {
        siteId,
        currentPageId: pageId,
        destinationPage,
        userId: context.sessionUserId,
        fileId,
        lastRevisionId,
        name: name === "" ? undefined : name,
        revisionComments: comments
      },
      context.requestContext
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

export async function pageFileRestoreAction(event: RequestEvent) {
  const { request, getClientAddress } = event
  const form = await superValidate(request, valibot(pageFileRestoreSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  try {
    const { siteId, pageId, fileId, newPage, newName, comments } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "optional"
    })
    const userId = resolvePageMutationUserId(
      context.sessionUserId,
      context.siteSlug,
      context.siteId,
      siteId
    )
    if (userId === undefined) {
      return fail(403, {
        form,
        message: "Permission denied."
      })
    }
    const res = await pageFileRestore(
      withPageFileClientAddress(getClientAddress, {
        siteId,
        pageId,
        userId,
        fileId,
        newPage: newPage === "" ? undefined : newPage,
        newName: newName === "" ? undefined : newName,
        revisionComments: comments
      }),
      context.requestContext
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

export async function pageFileHistoryAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
      fileId: number
      revisionNumber: Optional<number>
      limit: Optional<number>
    } = await readActionJson(request)

    const { siteId, pageId, fileId, revisionNumber, limit } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageFileHistory(
      siteId,
      pageId,
      fileId,
      revisionNumber,
      limit,
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageFileRollbackAction(event: RequestEvent) {
  const { request, getClientAddress } = event
  try {
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
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "optional"
    })
    const userId = resolvePageMutationUserId(
      context.sessionUserId,
      context.siteSlug,
      context.siteId,
      siteId
    )
    if (userId === undefined) {
      return fail(403, {
        message: "Permission denied."
      })
    }
    const res = await pageFileRollback(
      withPageFileClientAddress(getClientAddress, {
        siteId,
        pageId,
        userId,
        fileId,
        lastRevisionId,
        revisionNumber,
        revisionComments: comments
      }),
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}
