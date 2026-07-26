import { isSignedI64String } from "./context.js"
import { forumPostsByPage } from "./data.js"
import { sendRpcError } from "./response.js"

/**
 * @typedef {{
 *   content: string
 *   created_at: string
 *   created_by: string
 *   html: string
 *   id: number
 *   reply_to: number | null
 *   title: string
 * }} FixtureForumPost
 */

/**
 * @param {{
 *   rpcRequest: any
 *   response: import("node:http").ServerResponse
 * }} input
 * @returns {{ result?: number[]; responded?: boolean } | undefined}
 */
export const handleForumSelectRpc = ({ rpcRequest, response }) => {
  if (
    rpcRequest.method !== "forum_post_select" ||
    rpcRequest.params?.site_id !== 6000005 ||
    (rpcRequest.params.page !== undefined &&
      typeof rpcRequest.params.page !== "string") ||
    (rpcRequest.params.reply_to !== undefined &&
      typeof rpcRequest.params.reply_to !== "string") ||
    (rpcRequest.params.created_by !== undefined &&
      typeof rpcRequest.params.created_by !== "string")
  ) {
    return undefined
  }

  if (
    rpcRequest.params.reply_to !== undefined &&
    rpcRequest.params.reply_to !== "-" &&
    !isSignedI64String(rpcRequest.params.reply_to)
  ) {
    sendRpcError(
      response,
      rpcRequest.id,
      -32602,
      "Unexpected fixture forum_post_select params"
    )
    return { responded: true }
  }

  const posts =
    rpcRequest.params.page === undefined
      ? Object.values(forumPostsByPage).flat()
      : (forumPostsByPage[rpcRequest.params.page] ?? [])
  const result = posts
    .filter(
      /** @param {FixtureForumPost} post */
      (post) => {
        if (rpcRequest.params.reply_to === undefined) return true
        if (rpcRequest.params.reply_to === "-") return post.reply_to === null
        return String(post.reply_to) === rpcRequest.params.reply_to
      }
    )
    .filter(
      /** @param {FixtureForumPost} post */
      (post) =>
        rpcRequest.params.created_by === undefined ||
        post.created_by === rpcRequest.params.created_by
    )
    .map(
      /** @param {FixtureForumPost} post */
      (post) => post.id
    )

  return { result }
}
