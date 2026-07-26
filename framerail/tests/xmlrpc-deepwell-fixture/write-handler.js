import { handlePageWriteRpc } from "./page-write-handler.js"
import { handleParentWriteRpc } from "./parent-write-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handleWriteRpc = (input) => {
  return handlePageWriteRpc(input) ?? handleParentWriteRpc(input)
}
