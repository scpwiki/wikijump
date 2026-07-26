import { handleIdentityRpc } from "./identity-handler.js"
import { handleSiteRpc } from "./site-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handleAuthRpc = (input) => {
  return handleIdentityRpc(input) ?? handleSiteRpc(input)
}
