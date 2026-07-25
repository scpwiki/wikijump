import {
  pageDeletedGet,
  pageHistory,
  pageRestore,
  pageRevision,
  pageRollback
} from "$lib/server/deepwell/page"
import {
  failForActionError,
  pageActionBaseSchema,
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page/page-action-shared"
import { resolvePageActionRequestContext } from "$lib/server/load/page/page-action-context"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { boolean, number, object, optional, string } from "valibot"
import type { RequestEvent } from "@sveltejs/kit"

export async function pageHistoryAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData = await readActionJson(request, pageHistorySchema)

    const { siteId, pageId, revisionNumber, limit } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageHistory(
      siteId,
      pageId,
      revisionNumber,
      limit,
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

const pageHistorySchema = object({
  ...pageActionBaseSchema,
  revisionNumber: optional(number()),
  limit: optional(number())
})

export async function pageRevisionAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData = await readActionJson(request, pageRevisionSchema)

    const { siteId, pageId, revisionNumber, compiledHtml, wikitext } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageRevision(
      siteId,
      pageId,
      revisionNumber,
      compiledHtml ?? true,
      wikitext ?? true,
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

const pageRevisionSchema = object({
  ...pageActionBaseSchema,
  revisionNumber: number(),
  compiledHtml: optional(boolean()),
  wikitext: optional(boolean())
})

export async function pageRollbackAction(event: RequestEvent) {
  const { request, params, getClientAddress } = event
  const { slug } = params
  const ipAddress = getClientAddress()
  try {
    const requestData = await readActionJson(request, pageRollbackSchema)

    const { siteId, pageId, revisionNumber, comments, lastRevisionId } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageRollback(
      {
        siteId,
        pageId,
        userId: context.sessionUserId,
        userIpAddr: ipAddress,
        slug,
        lastRevisionId,
        revisionNumber,
        revisionComments: comments ?? ""
      },
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

const pageRollbackSchema = object({
  ...pageMutationBaseSchema,
  revisionNumber: number(),
  comments: optional(string())
})

export async function pageDeletedGetAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData = await readActionJson(request, pageDeletedGetSchema)
    const { siteId, slug } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageDeletedGet(siteId, slug, context.requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

const pageDeletedGetSchema = object({
  siteId: number(),
  slug: string()
})

export async function pageRestoreAction(event: RequestEvent) {
  const { request, getClientAddress } = event
  const form = await superValidate(request, valibot(pageRestoreSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const ipAddress = getClientAddress()

  try {
    const { siteId, pageId, comments } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageRestore(
      siteId,
      pageId,
      context.sessionUserId,
      ipAddress,
      comments,
      context.requestContext
    )
    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageRestoreSchema = object({
  ...pageMutationBaseSchema,
  comments: string()
})
