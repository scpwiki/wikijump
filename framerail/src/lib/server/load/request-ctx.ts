// Helper functions and types for request context, a set of common metadata for each request to Deepwell.
import { AsyncLocalStorage } from "node:async_hooks"

export type RequestContext = {
  sessionToken?: string
  siteId?: number
  page?: string | number
}

export type RequestContextOptional = RequestContext | void

const requestContextStore = new AsyncLocalStorage<RequestContext>()

export function runWithRequestContext<T>(
  requestContext: RequestContext,
  callback: () => T
): T {
  return requestContextStore.run(requestContext, callback)
}

export function getRequestContextFromStore(): RequestContext | undefined {
  return requestContextStore.getStore()
}
