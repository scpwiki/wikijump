import {
  pageDeletedGet,
  pageHistory,
  pageRestore,
  pageRevision,
  pageRollback
} from "$lib/server/deepwell/page"
import {
  failForActionError,
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page/page-action-shared"
import { resolvePageActionRequestContext } from "$lib/server/load/page/page-action-context"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { object, string } from "valibot"

import type { Optional } from "$lib/types"
import type { RequestEvent } from "@sveltejs/kit"

export async function pageHistoryAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: Optional<number>
      limit: Optional<number>
    } = await readActionJson(request)

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

export async function pageRevisionAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: number
      compiledHtml: Optional<boolean>
      wikitext: Optional<boolean>
    } = await readActionJson(request)

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

export async function pageRollbackAction(event: RequestEvent) {
  const { request, params, getClientAddress } = event
  const { slug } = params
  const ipAddress = getClientAddress()
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: number
      comments: Optional<string>
      lastRevisionId: number
    } = await readActionJson(request)

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

export async function pageDeletedGetAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      slug: string
    } = await readActionJson(request)
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
