import { handlePageCreateEditRpc } from "./page-create-edit-handler.js"
import { handlePageRevisionRpc } from "./page-revision-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageWriteRpc = (input) => {
  return handlePageCreateEditRpc(input) ?? handlePageRevisionRpc(input)
}
