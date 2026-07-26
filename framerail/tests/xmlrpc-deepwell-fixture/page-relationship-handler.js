import { fixtureState, hasExactKeys } from "./context.js"
import { forumPostsByPage, pages, parentBySlug } from "./data.js"

/** @param {{ rpcRequest: any }} input */
export const handlePageRelationshipRpc = ({ rpcRequest }) => {
  const { pageReadRequests } = fixtureState
  let result

  if (
    rpcRequest.method === "parent_relationships_get" &&
    hasExactKeys(rpcRequest.params, ["page", "relationship_type", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    rpcRequest.params.relationship_type === "parents"
  ) {
    pageReadRequests.parentRelationshipsGet.push(rpcRequest.params)
    const parentSlug = parentBySlug[rpcRequest.params.page]
    const child = pages[rpcRequest.params.page]
    const parent = parentSlug ? pages[parentSlug] : null
    result =
      child && parent
        ? [{ child_page_id: child.page_id, parent_page_id: parent.page_id }]
        : []
  } else if (
    rpcRequest.method === "forum_post_page_summary" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string"
  ) {
    pageReadRequests.forumPostPageSummary.push(rpcRequest.params)
    const posts = forumPostsByPage[rpcRequest.params.page] ?? []
    const latest = posts.at(-1)
    result = {
      comments: posts.length,
      commented_at: latest?.created_at ?? null,
      commented_by: latest?.created_by ?? null
    }
  } else {
    return undefined
  }

  return { result }
}
