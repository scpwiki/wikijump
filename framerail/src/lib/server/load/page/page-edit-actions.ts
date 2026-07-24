import {
  pageDelete,
  pageEdit,
  pageEditPermission,
  pageLayout,
  pageMove
} from "$lib/server/deepwell/page"
import {
  failForActionError,
  pageMutationBaseSchema
} from "$lib/server/load/page/page-action-shared"
import {
  requirePageMutationUserId,
  resolvePageActionRequestContext
} from "$lib/server/load/page/page-action-context"
import { DeleteOptions, Layout } from "$lib/types"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  literal,
  nullable,
  object,
  optional,
  string,
  variant,
  enum as vEnum
} from "valibot"

import type { RequestEvent } from "@sveltejs/kit"

export async function pageDeleteAction(event: RequestEvent) {
  const { request, params, getClientAddress } = event
  const form = await superValidate(request, valibot(pageDeleteSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const { slug } = params
  const ipAddress = getClientAddress()

  try {
    const { siteId, pageId, lastRevisionId, option, comments } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "optional"
    })
    const userId = requirePageMutationUserId(context, siteId)
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
        context.requestContext
      )
      return { form, res, option: DeleteOptions.Move }
    }

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
      context.requestContext
    )
    return { form, res, option: DeleteOptions.Delete }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageDeleteSchema = variant("option", [
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

export async function pageEditPermissionAction(event: RequestEvent) {
  try {
    const context = await resolvePageActionRequestContext(event)
    const res = await pageEditPermission(context.requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageEditAction(event: RequestEvent) {
  const { request, params, getClientAddress } = event
  const form = await superValidate(request, valibot(pageEditSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const { slug } = params
  const ipAddress = getClientAddress()

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
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "optional"
    })
    const userId = requirePageMutationUserId(context, siteId)
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
      context.requestContext
    )

    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageEditSchema = object({
  ...pageMutationBaseSchema,
  title: string(),
  altTitle: string(),
  wikitext: string(),
  tags: string(),
  comments: string(),
  layout: optional(nullable(vEnum(Layout)))
})

export async function pageLayoutAction(event: RequestEvent) {
  const { request, getClientAddress } = event
  const form = await superValidate(request, valibot(layoutSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const ipAddress = getClientAddress()

  try {
    const { siteId, pageId, layout } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    await pageLayout(
      siteId,
      pageId,
      context.sessionUserId,
      ipAddress,
      layout,
      context.requestContext
    )

    return { form }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const layoutSchema = object({
  ...pageMutationBaseSchema,
  layout: nullable(vEnum(Layout))
})

export async function pageMoveAction(event: RequestEvent) {
  const { request, params, getClientAddress } = event
  const form = await superValidate(request, valibot(pageMoveSchema))
  if (!form.valid) {
    return fail(400, { form })
  }
  const ipAddress = getClientAddress()
  const { slug } = params

  try {
    const { siteId, pageId, lastRevisionId, newSlug, comments } = form.data
    const context = await resolvePageActionRequestContext(event, {
      submittedSiteId: siteId,
      session: "required"
    })
    const res = await pageMove(
      {
        siteId,
        pageId,
        userId: context.sessionUserId,
        userIpAddr: ipAddress,
        slug,
        lastRevisionId,
        newSlug,
        revisionComments: comments
      },
      context.requestContext
    )
    return { form, res }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const pageMoveSchema = object({
  ...pageMutationBaseSchema,
  newSlug: string(),
  comments: string()
})
