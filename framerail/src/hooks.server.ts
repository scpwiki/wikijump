// Hook that runs on every request, including form actions.

import { runWithRequestContext } from "$lib/server/load/request-ctx"
import { loadSiteInfo } from "$lib/server/load/site-info"
import type { Handle } from "@sveltejs/kit"

export const handle: Handle = async ({ event, resolve }) => {
  const { request, cookies, params } = event

  // Gather common request metadata into a shared context.
  const { siteId } = loadSiteInfo(request.headers)
  const requestContext = {
    sessionToken: cookies.get("wikijump_token"),
    siteId,
    page: params.slug
  }

  // Continue processing the request within its isolated context.
  return runWithRequestContext(requestContext, () => resolve(event))
}
