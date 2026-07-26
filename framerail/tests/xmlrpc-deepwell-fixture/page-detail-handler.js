import { handlePageLookupRpc } from "./page-lookup-handler.js"
import { handlePageRelationshipRpc } from "./page-relationship-handler.js"

/** @param {{ rpcRequest: any }} input */
export const handlePageDetailRpc = (input) => {
  return handlePageLookupRpc(input) ?? handlePageRelationshipRpc(input)
}
