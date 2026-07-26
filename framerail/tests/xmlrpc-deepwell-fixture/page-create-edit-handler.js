import { handlePageCreateRpc } from "./page-create-handler.js"
import { handlePageEditRpc } from "./page-edit-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageCreateEditRpc = (input) => {
  return handlePageCreateRpc(input) ?? handlePageEditRpc(input)
}
