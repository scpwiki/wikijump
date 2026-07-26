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
 * @returns {{
 *       result?: (FixtureForumPost & { fullname: string })[]
 *       responded?: boolean
 *     }
 *   | undefined}
 */
export const handleForumGetRpc = ({ rpcRequest, response }) => {
  if (
    rpcRequest.method !== "forum_post_get" ||
    rpcRequest.params?.site_id !== 6000005 ||
    !Array.isArray(rpcRequest.params.posts) ||
    !rpcRequest.params.posts.every(
      /** @param {unknown} post */
      (post) => typeof post === "string"
    )
  ) {
    return undefined
  }

  if (
    rpcRequest.params.posts.length > 10 ||
    rpcRequest.params.posts.some(
      /** @param {string} post */
      (post) => !isSignedI64String(post)
    )
  ) {
    sendRpcError(
      response,
      rpcRequest.id,
      -32602,
      "Unexpected fixture forum_post_get params"
    )
    return { responded: true }
  }

  const postsById = new Map(
    Object.entries(forumPostsByPage).flatMap(([page, posts]) =>
      posts.map(
        /** @param {FixtureForumPost} post */
        (post) => [String(post.id), { ...post, fullname: page }]
      )
    )
  )
  const result = rpcRequest.params.posts.flatMap(
    /** @param {string} post */
    (post) => {
      const fixturePost = postsById.get(post)
      return fixturePost === undefined ? [] : [fixturePost]
    }
  )

  return { result }
}
