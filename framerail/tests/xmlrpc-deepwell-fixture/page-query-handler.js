import { fixtureState, hasExactKeys, requestContextHeaders } from "./context.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageQueryRpc = ({ rpcRequest, request }) => {
  const { pageReadRequests } = fixtureState
  /** @type {string[]} */
  let result

  if (
    rpcRequest.method === "page_select" &&
    hasExactKeys(rpcRequest.params, ["parent", "site"]) &&
    rpcRequest.params.site === "scp-wiki" &&
    rpcRequest.params.parent === "scp-173"
  ) {
    pageReadRequests.pageSelect.push(rpcRequest.params)
    result = ["scp-173-child-a", "scp-173-child-b"]
  } else if (
    rpcRequest.method === "page_select" &&
    hasExactKeys(rpcRequest.params, ["parent", "site"]) &&
    rpcRequest.params.site === "scp-wiki" &&
    typeof rpcRequest.params.parent === "string"
  ) {
    pageReadRequests.pageSelect.push(rpcRequest.params)
    result = []
  } else if (
    rpcRequest.method === "page_tags_select" &&
    rpcRequest.params?.site === "scp-wiki" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    (rpcRequest.params.categories === undefined ||
      rpcRequest.params.categories === null ||
      (Array.isArray(rpcRequest.params.categories) &&
        rpcRequest.params.categories.length <= 100 &&
        rpcRequest.params.categories.every(
          /** @param {unknown} category */
          (category) => typeof category === "string"
        ))) &&
    (rpcRequest.params.pages === undefined ||
      rpcRequest.params.pages === null ||
      (Array.isArray(rpcRequest.params.pages) &&
        rpcRequest.params.pages.length <= 100 &&
        rpcRequest.params.pages.every(
          /** @param {unknown} page */
          (page) => typeof page === "string"
        )))
  ) {
    fixtureState.lastPageTagsSelectRequest = {
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    }
    result = ["_cc", "tale"]
  } else if (
    rpcRequest.method === "page_select" &&
    rpcRequest.params?.site === "scp-wiki" &&
    rpcRequest.params?.pagetype === "normal" &&
    Array.isArray(rpcRequest.params.categories) &&
    rpcRequest.params.categories.length === 1 &&
    rpcRequest.params.categories[0] === "_default" &&
    rpcRequest.params?.created_by === "-1" &&
    rpcRequest.params?.rating === ">=0" &&
    rpcRequest.params?.order === "created_at desc"
  ) {
    fixtureState.lastPageSelectParams = rpcRequest.params
    result = ["scp-173", "scp-anthology-2024", "scp-8566"]
  } else {
    return undefined
  }

  return { result }
}
