import { fixtureState, hasExactKeys, isFixtureSiteId } from "./context.js"
import { backlinksByPage, forumPostsByPage, pages, parentBySlug } from "./data.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handlePageRelationshipRpc = ({ rpcRequest, request }) => {
  const { pageReadRequests } = fixtureState
  let result

  if (
    rpcRequest.method === "parent_get_direct_metadata" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token"
  ) {
    pageReadRequests.parentDirectMetadata.push({
      headers: {
        sessionToken: request.headers["x-deepwell-session-token"]
      },
      params: rpcRequest.params
    })
    const parentReference = parentBySlug[rpcRequest.params.page]
    if (typeof parentReference !== "string" || parentReference === "private-page") {
      result = null
    } else {
      const parent = pages[parentReference]
      result = parent ? { slug: parent.slug, title: parent.title } : null
    }
  } else if (
    rpcRequest.method === "parent_relationships_get" &&
    hasExactKeys(rpcRequest.params, ["page", "relationship_type", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    rpcRequest.params.relationship_type === "parents"
  ) {
    pageReadRequests.parentRelationshipsGet.push(rpcRequest.params)
    const parentReference = parentBySlug[rpcRequest.params.page]
    const parentSlugs = Array.isArray(parentReference)
      ? parentReference
      : parentReference
        ? [parentReference]
        : []
    const child = pages[rpcRequest.params.page]
    result = child
      ? parentSlugs.flatMap((parentSlug) => {
          const parent = pages[parentSlug]
          return parent
            ? [{ child_page_id: child.page_id, parent_page_id: parent.page_id }]
            : []
        })
      : []
  } else if (
    rpcRequest.method === "page_backlinks_view" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    isFixtureSiteId(rpcRequest.params.site_id) &&
    typeof rpcRequest.params.page === "string"
  ) {
    pageReadRequests.backlinksView.push(rpcRequest.params)
    result = (backlinksByPage[rpcRequest.params.page] ?? []).map((backlink) => ({
      ...backlink
    }))
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
