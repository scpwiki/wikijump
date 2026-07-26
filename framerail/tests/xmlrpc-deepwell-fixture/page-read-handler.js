import { handlePageDetailRpc } from "./page-detail-handler.js"
import { handlePageQueryRpc } from "./page-query-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageReadRpc = (input) => {
  return handlePageDetailRpc(input) ?? handlePageQueryRpc(input)
}
