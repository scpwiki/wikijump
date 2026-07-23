import type { RequestContext } from "$lib/server/deepwell/request-context"

export function storeRequestContext(
  locals: App.Locals,
  sessionToken?: string,
  siteId?: number,
  page?: string | number
) {
  locals.requestContext = {
    sessionToken,
    siteId,
    page
  }
}

export function getRequestContext(locals: App.Locals): RequestContext {
  return locals.requestContext
}

export function withDefaultPageContext(
  requestContext: RequestContext,
  defaultPage: string
): RequestContext {
  if (requestContext?.page !== undefined || defaultPage.length === 0) {
    return requestContext
  }

  return {
    ...(requestContext ?? {}),
    page: defaultPage
  }
}
