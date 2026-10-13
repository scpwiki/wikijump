import {
  pageDelete,
  pageDeletePermission,
  pageEdit,
  pageEditPermission,
  pageLayout,
  pageMove,
  pageMovePermission,
  pageParentUpdate
} from "$lib/server/deepwell/page"
import { pageView } from "$lib/server/deepwell/views"
import {
  failForActionError,
  pageMutationBaseSchema
} from "$lib/server/load/page/page-action-shared"
import {
  requirePageMutationUserId,
  resolvePageActionRequestContext
} from "$lib/server/load/page/page-action-context"
import {
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import { DeleteOptions, Layout } from "$lib/types"
import { fail, superValidate } from "sveltekit-superforms"
import { valibot } from "sveltekit-superforms/adapters"
import {
  literal,
  nullable,
  object,
  optional,
  strictObject,
  string,
  variant,
  enum as vEnum
} from "valibot"

import type { RequestEvent } from "@sveltejs/kit"

export async function pageDeleteAction(event: RequestEvent) {
  const { request, params, getClientAddress } = event
  const requestData = await request.formData()
  const isNativeForm =
    request.headers.get("content-type")?.split(";", 1)[0].trim() ===
      "application/x-www-form-urlencoded" && !requestData.has("__superform_json")

  if (isNativeForm) {
    const submittedData = Object.fromEntries(requestData)
    const form = await superValidate(submittedData, valibot(pageDeleteNativeSchema), {
      strict: true
    })
    if (!form.valid) {
      return fail(400, { form })
    }

    const ipAddress = getClientAddress()

    try {
      const context = await resolvePageActionRequestContext(event, {
        session: "optional"
      })
      const userId = requirePageMutationUserId(context, context.siteId)
      const requestLocales = getPreloadRequestLocales(request)
      const backendLocales = getPreloadBackendLocales(requestLocales)
      const view = await pageView(
        context.siteId,
        backendLocales,
        { slug: params.slug, extra: params.extra },
        context.sessionToken
      )
      if (view.type !== "found") throw new Error("Page not found.")

      if (form.data.option === DeleteOptions.Move) {
        const res = await pageMove(
          {
            siteId: context.siteId,
            pageId: view.data.page.page_id,
            userId,
            userIpAddr: ipAddress,
            slug: params.slug,
            lastRevisionId: view.data.page_revision.revision_id,
            newSlug: form.data["new-slug"],
            revisionComments: form.data.comments
          },
          context.requestContext
        )
        return { form, res, option: DeleteOptions.Move }
      }

      const res = await pageDelete(
        {
          siteId: context.siteId,
          pageId: view.data.page.page_id,
          userId,
          userIpAddr: ipAddress,
          slug: params.slug,
          lastRevisionId: view.data.page_revision.revision_id,
          revisionComments: form.data.comments
        },
        context.requestContext
      )
      return { form, res, option: DeleteOptions.Delete }
    } catch (error) {
      return failForActionError(error, { form })
    }
  }

  const form = await superValidate(requestData, valibot(pageDeleteSchema))
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

const pageDeleteNativeMoveSchema = strictObject({
  option: literal(DeleteOptions.Move),
  "new-slug": string(),
  comments: string()
})

const pageDeleteNativeDeleteSchema = strictObject({
  option: literal(DeleteOptions.Delete),
  comments: optional(string())
})

const pageDeleteNativeSchema = variant("option", [
  pageDeleteNativeMoveSchema,
  pageDeleteNativeDeleteSchema
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

export async function pageMovePermissionAction(event: RequestEvent) {
  try {
    const context = await resolvePageActionRequestContext(event)
    const res = await pageMovePermission(context.requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

export async function pageDeletePermissionAction(event: RequestEvent) {
  try {
    const context = await resolvePageActionRequestContext(event)
    const res = await pageDeletePermission(context.requestContext)
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
      parent,
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
    if (!pageId && parent) {
      if (!res.slug) throw new Error("Page creation did not return its assigned slug.")
      await pageParentUpdate(siteId, res.slug, userId, [parent], [], {
        ...context.requestContext,
        page: res.slug
      })
    }

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
  parent: optional(string()),
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
  const requestData = await request.formData()
  const isNativeForm =
    request.headers.get("content-type")?.split(";", 1)[0].trim() ===
      "application/x-www-form-urlencoded" && !requestData.has("__superform_json")
  const { slug } = params

  if (isNativeForm) {
    const form = await superValidate(
      Object.fromEntries(requestData),
      valibot(pageMoveNativeSchema)
    )
    if (!form.valid) {
      return fail(400, { form })
    }
    const ipAddress = getClientAddress()

    try {
      const context = await resolvePageActionRequestContext(event, {
        session: "required"
      })
      const requestLocales = getPreloadRequestLocales(request)
      const backendLocales = getPreloadBackendLocales(requestLocales)
      const view = await pageView(
        context.siteId,
        backendLocales,
        { slug: params.slug, extra: params.extra },
        context.sessionToken
      )
      if (view.type !== "found") throw new Error("Page not found.")

      const res = await pageMove(
        {
          siteId: context.siteId,
          pageId: view.data.page.page_id,
          userId: context.sessionUserId,
          userIpAddr: ipAddress,
          slug,
          lastRevisionId: view.data.page_revision.revision_id,
          newSlug: form.data["new-slug"],
          revisionComments: form.data.comments
        },
        context.requestContext
      )
      return { form, res }
    } catch (error) {
      return failForActionError(error, { form })
    }
  }

  const form = await superValidate(requestData, valibot(pageMoveSchema))
  if (!form.valid) {
    return fail(400, { form })
  }
  const ipAddress = getClientAddress()

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

const pageMoveNativeSchema = strictObject({
  "new-slug": string(),
  comments: string()
})
