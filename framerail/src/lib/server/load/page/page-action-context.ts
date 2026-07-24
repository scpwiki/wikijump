import { authGetSession } from "$lib/server/auth/get-session"
import { preloadView } from "$lib/server/deepwell/views"
import { resolvePageMutationUserId } from "$lib/server/load/local-authoring-actor"
import {
  MissingActionSessionError,
  PageActionContextMismatchError
} from "$lib/server/load/action-error"
import {
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import { getRequestContext, withDefaultPageContext } from "$lib/server/request-context"
import { loadSiteInfo } from "$lib/server/load/site-info"

import type { RequestContext } from "$lib/server/request-context"
import type { RequestEvent } from "@sveltejs/kit"

type SessionRequirement = "none" | "optional" | "required"

interface PageActionContextOptions {
  submittedSiteId?: number
  session?: SessionRequirement
}

export interface PageActionRequestContext {
  requestContext: Exclude<RequestContext, void>
  sessionToken?: string
  sessionUserId?: number
  siteId: number
  siteSlug: string
}

export interface AuthenticatedPageActionRequestContext extends PageActionRequestContext {
  sessionToken: string
  sessionUserId: number
}

export function requirePageMutationUserId(
  context: PageActionRequestContext,
  mutationSiteId: number
): number {
  const userId = resolvePageMutationUserId(
    context.sessionUserId,
    context.siteSlug,
    context.siteId,
    mutationSiteId
  )
  if (userId === undefined) {
    throw new PageActionContextMismatchError("Permission denied.")
  }
  return userId
}

export function resolvePageActionRequestContext(
  event: RequestEvent,
  options: PageActionContextOptions & { session: "required" }
): Promise<AuthenticatedPageActionRequestContext>
export function resolvePageActionRequestContext(
  event: RequestEvent,
  options?: PageActionContextOptions
): Promise<PageActionRequestContext>
export async function resolvePageActionRequestContext(
  event: RequestEvent,
  { submittedSiteId, session: sessionRequirement = "none" }: PageActionContextOptions = {}
): Promise<PageActionRequestContext> {
  const { request, locals } = event
  const { siteId, siteSlug } = loadSiteInfo(request.headers)
  const storedContext = getRequestContext(locals)

  if (
    !storedContext ||
    storedContext.siteId !== siteId ||
    (submittedSiteId !== undefined && submittedSiteId !== siteId)
  ) {
    throw new PageActionContextMismatchError("Permission denied.")
  }

  const sessionToken = storedContext.sessionToken
  if (sessionRequirement === "required" && !sessionToken) {
    throw new MissingActionSessionError("Authentication required.")
  }

  const session =
    sessionRequirement !== "none" && sessionToken
      ? await authGetSession(sessionToken)
      : undefined
  let requestContext = storedContext

  if (requestContext.page === undefined) {
    const requestLocales = getPreloadRequestLocales(request)
    const backendLocales = getPreloadBackendLocales(requestLocales)
    const { site } = await preloadView(siteId, backendLocales, sessionToken)
    requestContext =
      withDefaultPageContext(requestContext, site.default_page) ?? requestContext
  }

  return {
    requestContext,
    sessionToken,
    sessionUserId: session?.user_id,
    siteId,
    siteSlug
  }
}
