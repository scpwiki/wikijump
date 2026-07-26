import { handleForumGetRpc } from "./forum-get-handler.js"
import { handleForumSelectRpc } from "./forum-select-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handleForumRpc = (input) => {
  return handleForumSelectRpc(input) ?? handleForumGetRpc(input)
}
