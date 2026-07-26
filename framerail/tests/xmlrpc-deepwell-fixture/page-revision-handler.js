import { fixtureState, hasExactKeys, pageById, requestContextHeaders } from "./context.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageRevisionRpc = ({ rpcRequest, request }) => {
  const { counters, pageWriteRequests } = fixtureState
  let result

  if (
    rpcRequest.method === "page_rollback" &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "number" &&
    pageById(rpcRequest.params.page) &&
    typeof rpcRequest.params.last_revision_id === "number" &&
    typeof rpcRequest.params.revision_number === "number" &&
    typeof rpcRequest.params.revision_comments === "string" &&
    rpcRequest.params.user_id === 123 &&
    typeof rpcRequest.params.ip_address === "string" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page)?.slug
  ) {
    pageWriteRequests.pageRollback.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    const page = pageById(rpcRequest.params.page)
    if (!page) return undefined
    page.revision_id = counters.nextRevisionId++
    page.page_revision_count += 1
    result = {
      revision_id: page.revision_id,
      revision_number: page.page_revision_count - 1
    }
  } else if (
    rpcRequest.method === "vote_set" &&
    hasExactKeys(rpcRequest.params, ["page_id", "user_id", "value"]) &&
    pageById(rpcRequest.params.page_id) &&
    rpcRequest.params.user_id === 123 &&
    (rpcRequest.params.value === -1 || rpcRequest.params.value === 1) &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page_id)?.slug
  ) {
    pageWriteRequests.voteSet.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    result = {
      page_vote_id: 7000001,
      page_id: rpcRequest.params.page_id,
      user_id: rpcRequest.params.user_id,
      value: rpcRequest.params.value
    }
  } else {
    return undefined
  }

  return { result }
}
