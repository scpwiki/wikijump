import { authGetSession } from "$lib/server/auth/get-session"
import {
  pageDeletedGet,
  pageHistory,
  pageRestore,
  pageRevision,
  pageRollback
} from "$lib/server/deepwell/page"
import {
  failForActionError,
  failForMissingSession,
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page-action-shared"
import { getRequestContext } from "$lib/server/load/request-ctx"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { object, string } from "valibot"

import type { Optional } from "$lib/types"
import type { RequestEvent } from "@sveltejs/kit"

export async function pageHistoryAction({ request, locals }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: Optional<number>
      limit: Optional<number>
    } = await readActionJson(request)

    const { siteId, pageId, revisionNumber, limit } = requestData
    const res = await pageHistory(
      siteId,
      pageId,
      revisionNumber,
      limit,
      getRequestContext(locals)
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageRevisionAction({ request, locals }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: number
      compiledHtml: Optional<boolean>
      wikitext: Optional<boolean>
    } = await readActionJson(request)

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
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageRollbackAction({
  request,
  params,
  getClientAddress,
  cookies
}: RequestEvent) {
  const { slug } = params
  const ipAddress = getClientAddress()
  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession()
  try {
    const session = await authGetSession(sessionToken)
    const requestData: {
      siteId: number
      pageId: number
      revisionNumber: number
      comments: Optional<string>
      lastRevisionId: number
    } = await readActionJson(request)

    const { siteId, pageId, revisionNumber, comments, lastRevisionId } = requestData
    const res = await pageRollback(
      {
        siteId,
        pageId,
        userId: session.user_id,
        userIpAddr: ipAddress,
        slug,
        lastRevisionId,
        revisionNumber,
        revisionComments: comments ?? ""
      },
      { sessionToken, siteId, page: pageId ?? slug }
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageDeletedGetAction({ request }: RequestEvent) {
  try {
    const requestData: {
      siteId: number
      slug: string
    } = await readActionJson(request)
    const { siteId, slug } = requestData
    const res = await pageDeletedGet(siteId, slug)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

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
  if (!sessionToken) return failForMissingSession({ form })
  const ipAddress = getClientAddress()

  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, comments } = form.data
    const res = await pageRestore(siteId, pageId, session.user_id, ipAddress, comments, {
      sessionToken,
      siteId,
      page: pageId
    })
    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageRestoreSchema = object({
  ...pageMutationBaseSchema,
  comments: string()
})
