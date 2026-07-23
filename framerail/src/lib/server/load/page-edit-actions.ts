import { authGetSession } from "$lib/server/auth/get-session"
import {
  pageDelete,
  pageEdit,
  pageEditPermission,
  pageLayout,
  pageMove
} from "$lib/server/deepwell/page"
import { preloadView } from "$lib/server/deepwell/views"
import { resolvePageMutationUserId } from "$lib/server/load/local-authoring-actor"
import {
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import {
  failForActionError,
  failForMissingSession,
  pageMutationBaseSchema
} from "$lib/server/load/page-action-shared"
import { loadSiteInfo } from "$lib/server/load/site-info"
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
import { getRequestContext, withDefaultPageContext } from "./request-ctx"

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
  const { siteId: requestSiteId, siteSlug } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")
  const ipAddress = getClientAddress()

  try {
    const session = sessionToken ? await authGetSession(sessionToken) : undefined
    const { siteId, pageId, lastRevisionId, option, comments } = form.data
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
        { sessionToken, siteId, page: pageId ?? slug }
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
      { sessionToken, siteId, page: pageId ?? slug }
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

export async function pageEditPermissionAction({
  request,
  cookies,
  locals
}: RequestEvent) {
  try {
    let requestContext = getRequestContext(locals)

    if (requestContext?.page === undefined) {
      const { siteId } = loadSiteInfo(request.headers)
      const requestLocales = getPreloadRequestLocales(request)
      const backendLocales = getPreloadBackendLocales(requestLocales)
      const sessionToken = cookies.get("wikijump_token")
      const { site } = await preloadView(siteId, backendLocales, sessionToken)
      requestContext = withDefaultPageContext(requestContext, site.default_page)
    }

    const res = await pageEditPermission(requestContext)
    return { res }
  } catch (error) {
    return failForActionError(error)
  }
}

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
  const { siteId: requestSiteId, siteSlug } = loadSiteInfo(request.headers)
  const sessionToken = cookies.get("wikijump_token")
  const ipAddress = getClientAddress()

  try {
    const session = sessionToken ? await authGetSession(sessionToken) : undefined
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
      { sessionToken, siteId, page: pageId ?? slug }
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

export async function layoutAction({ request, cookies, getClientAddress }: RequestEvent) {
  const form = await superValidate(request, valibot(layoutSchema))
  if (!form.valid) {
    return fail(400, { form })
  }

  const sessionToken = cookies.get("wikijump_token")
  if (!sessionToken) return failForMissingSession({ form })
  const ipAddress = getClientAddress()

  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, layout } = form.data
    await pageLayout(siteId, pageId, session.user_id, ipAddress, layout, {
      sessionToken,
      siteId,
      page: pageId
    })

    return { form }
  } catch (error) {
    return failForActionError(error, { form })
  }
}

export const layoutSchema = object({
  ...pageMutationBaseSchema,
  layout: nullable(vEnum(Layout))
})

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
  if (!sessionToken) return failForMissingSession({ form })
  const ipAddress = getClientAddress()
  const { slug } = params

  try {
    const session = await authGetSession(sessionToken)
    const { siteId, pageId, lastRevisionId, newSlug, comments } = form.data
    const res = await pageMove(
      {
        siteId,
        pageId,
        userId: session.user_id,
        userIpAddr: ipAddress,
        slug,
        lastRevisionId,
        newSlug,
        revisionComments: comments
      },
      { sessionToken, siteId, page: pageId ?? slug }
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
