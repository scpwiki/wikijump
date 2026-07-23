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
  pageMutationBaseSchema,
  readActionJson
} from "$lib/server/load/page-action-shared"
import { resolvePageActionRequestContext } from "$lib/server/load/page-action-context"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import { array, object, optional, string } from "valibot"

import type { RequestEvent } from "@sveltejs/kit"

export async function pageParentSetAction(event: RequestEvent) {
  const { request } = event
  const form = await superValidate(request, valibot(pageParentSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  try {
    const { siteId, pageId, addParents, removeParents } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageParentUpdate(
      siteId,
      pageId,
      context.sessionUserId,
      addParents,
      removeParents,
      context.requestContext
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

export async function pageParentGetAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
      slug: string
    } = await readActionJson(request)
    const { siteId, pageId, slug } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageParentGet(siteId, pageId, slug, context.requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageVoteGetAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await readActionJson(request)
    const { siteId, pageId } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageVoteList(pageId, context.requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageVoteCastAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
      value: number
    } = await readActionJson(request)
    const { siteId, pageId, value } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageVoteCast(
      pageId,
      context.sessionUserId,
      value,
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageVoteCancelAction(event: RequestEvent) {
  const { request } = event
  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await readActionJson(request)
    const { siteId, pageId } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageVoteRemove(
      pageId,
      context.sessionUserId,
      context.requestContext
    )
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageScoreAction(event: RequestEvent) {
  const { request, params } = event
  const { slug } = params

  try {
    const requestData: {
      siteId: number
      pageId: number
    } = await readActionJson(request)
    const { siteId, pageId } = requestData
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId
    })
    const res = await pageScore(siteId, pageId, slug, context.requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}
