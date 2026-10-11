import { handlePageLookupRpc } from "./page-lookup-handler.js"
import { handlePageRelationshipRpc } from "./page-relationship-handler.js"
import { hasExactKeys } from "./context.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageDetailRpc = (input) => {
  const { rpcRequest } = input
  if (
    rpcRequest.method === "page_meta_tags" &&
    hasExactKeys(rpcRequest.params, ["site_id", "page_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    Number.isSafeInteger(rpcRequest.params.page_id)
  ) {
    return { result: [] }
  }

  return handlePageLookupRpc(input) ?? handlePageRelationshipRpc(input)
}
