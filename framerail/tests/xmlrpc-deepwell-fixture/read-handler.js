import { handleArticleRpc } from "./article-handler.js"
import { handleAuthRpc } from "./auth-handler.js"
import { handleForumRpc } from "./forum-handler.js"
import { handlePageReadRpc } from "./page-read-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handleReadRpc = (input) => {
  return (
    handleAuthRpc(input) ??
    handleArticleRpc(input) ??
    handlePageReadRpc(input) ??
    handleForumRpc(input)
  )
}
