import { fixtureState, pageById, requestContextHeaders } from "./context.js"
import { pages } from "./data.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageEditRpc = ({ rpcRequest, request }) => {
  if (
    rpcRequest.method !== "page_edit" ||
    rpcRequest.params.site_id !== 6000005 ||
    !(
      (typeof rpcRequest.params.page === "string" && pages[rpcRequest.params.page]) ||
      (typeof rpcRequest.params.page === "number" && pageById(rpcRequest.params.page))
    ) ||
    typeof rpcRequest.params.last_revision_id !== "number" ||
    typeof rpcRequest.params.revision_comments !== "string" ||
    typeof rpcRequest.params.user_id !== "number" ||
    typeof rpcRequest.params.ip_address !== "string" ||
    request.headers["x-deepwell-session-token"] !== "fixture-session-token" ||
    request.headers["x-deepwell-site-id"] !== "6000005" ||
    request.headers["x-deepwell-page"] !==
      (typeof rpcRequest.params.page === "string"
        ? rpcRequest.params.page
        : pageById(rpcRequest.params.page)?.slug)
  ) {
    return undefined
  }

  const { counters, pageWriteRequests } = fixtureState
  pageWriteRequests.pageEdit.push({
    headers: requestContextHeaders(request),
    params: rpcRequest.params
  })
  const page =
    typeof rpcRequest.params.page === "string"
      ? pages[rpcRequest.params.page]
      : pageById(rpcRequest.params.page)
  if (!page) return undefined
  if (typeof rpcRequest.params.wikitext === "string") {
    page.wikitext = rpcRequest.params.wikitext
    page.compiled_body_html = `<p>${rpcRequest.params.wikitext}</p>`
  }
  if (typeof rpcRequest.params.title === "string") page.title = rpcRequest.params.title
  if (Array.isArray(rpcRequest.params.tags)) page.tags = rpcRequest.params.tags
  page.page_updated_at = "2026-06-29T00:01:00Z"
  page.revision_created_at = "2026-06-29T00:01:00Z"
  page.revision_user_id = rpcRequest.params.user_id
  page.revision_id = counters.nextRevisionId++
  page.page_revision_count += 1
  return {
    result: {
      revision_id: page.revision_id,
      revision_number: page.page_revision_count - 1
    }
  }
}
