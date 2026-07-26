import { fixtureState, hasExactKeys, requestContextHeaders } from "./context.js"
import { pages, parentBySlug } from "./data.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handleParentWriteRpc = ({ rpcRequest, request }) => {
  const { counters, pageWriteRequests } = fixtureState
  let result

  if (
    rpcRequest.method === "parent_get_all" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === rpcRequest.params.page
  ) {
    pageWriteRequests.parentGetAll.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    result = parentBySlug[rpcRequest.params.page]
      ? [parentBySlug[rpcRequest.params.page]]
      : []
  } else if (
    rpcRequest.method === "parent_update" &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.child === "string" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === rpcRequest.params.child
  ) {
    pageWriteRequests.parentUpdate.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    if (Array.isArray(rpcRequest.params.add) && rpcRequest.params.add.length > 0) {
      parentBySlug[rpcRequest.params.child] = rpcRequest.params.add[0]
    }
    if (Array.isArray(rpcRequest.params.remove)) {
      for (const parent of rpcRequest.params.remove) {
        if (parentBySlug[rpcRequest.params.child] === parent) {
          delete parentBySlug[rpcRequest.params.child]
        }
      }
    }
    result = { added: [], removed: [] }
  } else if (
    rpcRequest.method === "page_move" &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    pages[rpcRequest.params.page] &&
    typeof rpcRequest.params.new_slug === "string" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === rpcRequest.params.page
  ) {
    pageWriteRequests.pageMove.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    const page = pages[rpcRequest.params.page]
    delete pages[rpcRequest.params.page]
    page.slug = rpcRequest.params.new_slug
    page.revision_id = counters.nextRevisionId++
    page.page_revision_count += 1
    pages[page.slug] = page
    if (parentBySlug[rpcRequest.params.page]) {
      parentBySlug[page.slug] = parentBySlug[rpcRequest.params.page]
      delete parentBySlug[rpcRequest.params.page]
    }
    result = { new_slug: page.slug, old_slug: rpcRequest.params.page }
  } else {
    return undefined
  }

  return { result }
}
