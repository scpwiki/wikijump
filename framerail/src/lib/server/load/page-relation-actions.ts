import { authGetSession } from "$lib/server/auth/get-session"
import {
  pageParentGet,
  pageParentUpdate,
  pageScore,
  pageVoteCast,
  pageVoteList,
  pageVoteRemove
} from "$lib/server/deepwell/page"
import {
  failForActionError,
  failForMissingSession,
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page-action-shared"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { array, object, optional, string } from "valibot"

import type { RequestEvent } from "@sveltejs/kit"

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
      session.user_id,
      addParents,
      removeParents,
      { sessionToken, siteId, page: pageId }
    )
    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageParentSchema = object({
  ...pageMutationBaseSchema,
  parents: string(),
  addParents: optional(array(string())),
  removeParents: optional(array(string()))
})

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
