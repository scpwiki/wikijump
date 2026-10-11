import { fixtureState, hasExactKeys, isFixtureSiteId, pageById } from "./context.js"
import { pages, toPageResult } from "./data.js"
import { sendRpcError, sendRpcResult } from "./response.js"

const HISTORY_WORKFLOW_PAGE_ID = 3000345
const HISTORY_NAVIGATION_PAGE_ID = 3000173

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handlePageLookupRpc = ({ rpcRequest, request, response }) => {
  const { pageReadRequests } = fixtureState
  let result

  const pageReference = rpcRequest.params?.page
  const isStringPageViewPermission =
    typeof pageReference === "string" &&
    isFixtureSiteId(Number(request.headers["x-deepwell-site-id"])) &&
    request.headers["x-deepwell-page"] === pageReference &&
    (request.headers["x-deepwell-session-token"] === undefined ||
      request.headers["x-deepwell-session-token"] === "fixture-session-token")
  const isXmlRpcParentViewPermission =
    typeof pageReference === "number" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token"

  if (
    rpcRequest.method === "page_view_permission" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    isFixtureSiteId(rpcRequest.params.site_id) &&
    (isStringPageViewPermission || isXmlRpcParentViewPermission)
  ) {
    if (isXmlRpcParentViewPermission) {
      pageReadRequests.pageViewPermission.push({
        headers: {
          sessionToken: request.headers["x-deepwell-session-token"]
        },
        params: rpcRequest.params
      })
    }
    const page =
      typeof pageReference === "number" ? pageById(pageReference) : pages[pageReference]
    result = page !== undefined && page !== null && page.slug !== "private-page"
  } else if (
    rpcRequest.method === "page_edit_permission" &&
    hasExactKeys(rpcRequest.params, []) &&
    request.headers["x-deepwell-session-token"] === undefined &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    typeof request.headers["x-deepwell-page"] === "string" &&
    pages[request.headers["x-deepwell-page"]]
  ) {
    // Production answers an anonymous actor with the permission decision, not
    // a transport error, so Hash Magic and option-bar edit clicks both reach
    // the localized denial dialog instead of an RPC failure message.
    result = { can_edit: false }
  } else if (
    rpcRequest.method === "page_edit_permission" &&
    hasExactKeys(rpcRequest.params, []) &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    typeof request.headers["x-deepwell-page"] === "string" &&
    pages[request.headers["x-deepwell-page"]]
  ) {
    result = { can_edit: true }
  } else if (
    rpcRequest.method === "page_lifecycle_identity" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === rpcRequest.params.page
  ) {
    pageReadRequests.pageLifecycleIdentity.push({
      headers: {
        page: request.headers["x-deepwell-page"],
        sessionToken: request.headers["x-deepwell-session-token"],
        siteId: request.headers["x-deepwell-site-id"]
      },
      params: rpcRequest.params
    })
    const page = pages[rpcRequest.params.page]
    result =
      page && page.slug !== "private-page"
        ? {
            created_by:
              page.creator_display_name ?? displayNameForUserId(page.creator_user_id),
            updated_by:
              page.updater_display_name ?? displayNameForUserId(page.revision_user_id)
          }
        : null
  } else if (
    rpcRequest.method === "page_get" &&
    hasExactKeys(rpcRequest.params, ["details", "page", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
    typeof rpcRequest.params.details.compiled_html === "boolean" &&
    typeof rpcRequest.params.details.wikitext === "boolean"
  ) {
    pageReadRequests.pageGet.push(rpcRequest.params)
    result = toPageResult(
      pages[rpcRequest.params.page] ?? null,
      rpcRequest.params.details
    )
  } else if (
    rpcRequest.method === "page_get_direct" &&
    hasExactKeys(rpcRequest.params, ["allow_deleted", "details", "page_id", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    pageById(rpcRequest.params.page_id) &&
    rpcRequest.params.allow_deleted === false &&
    hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
    rpcRequest.params.details.compiled_html === false &&
    rpcRequest.params.details.wikitext === false
  ) {
    pageReadRequests.pageGetDirect.push(rpcRequest.params)
    result = toPageResult(pageById(rpcRequest.params.page_id), rpcRequest.params.details)
  } else if (
    rpcRequest.method === "page_get_score" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page === "string" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === rpcRequest.params.page &&
    pages[rpcRequest.params.page]
  ) {
    pageReadRequests.pageGetScore.push({
      headers: {
        page: request.headers["x-deepwell-page"],
        siteId: request.headers["x-deepwell-site-id"]
      },
      params: rpcRequest.params
    })
    const page = pages[rpcRequest.params.page]
    result = {
      page_id: page.page_id,
      score: fixtureState.ratingScores[page.page_id] ?? 0
    }
  } else if (
    rpcRequest.method === "page_who_rated" &&
    hasExactKeys(rpcRequest.params, ["page_id", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    rpcRequest.params.page_id === 3000342 &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === "3000342"
  ) {
    const value = fixtureState.voteValues[3000342]
    result = value
      ? [
          {
            user: {
              "user-id": 123,
              "user-slug": "guest",
              "user-name": "Guest",
              "user-karma": 0
            },
            value
          }
        ]
      : []
  } else if (
    rpcRequest.method === "page_revision_count" &&
    hasExactKeys(rpcRequest.params, ["page", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    (rpcRequest.params.page === HISTORY_WORKFLOW_PAGE_ID ||
      rpcRequest.params.page === HISTORY_NAVIGATION_PAGE_ID)
  ) {
    const page = pageById(rpcRequest.params.page)
    const revisionCount = page?.page_revision_count ?? 1
    result = {
      revision_count: revisionCount,
      first_revision: 0,
      last_revision: revisionCount - 1
    }
  } else if (
    rpcRequest.method === "page_revision_range" &&
    hasExactKeys(rpcRequest.params, [
      "limit",
      "page_id",
      "revision_direction",
      "revision_number",
      "site_id"
    ]) &&
    rpcRequest.params.site_id === 6000005 &&
    (rpcRequest.params.page_id === HISTORY_WORKFLOW_PAGE_ID ||
      rpcRequest.params.page_id === HISTORY_NAVIGATION_PAGE_ID) &&
    rpcRequest.params.revision_direction === "before" &&
    rpcRequest.params.revision_number === -1 &&
    rpcRequest.params.limit === 20
  ) {
    result = [
      {
        revision_id: 9000341,
        revision_type: "regular",
        revision_number: 1,
        created_at: "2026-08-15T00:00:00Z",
        user_id: 12345,
        author: {
          "user-id": 12345,
          "user-slug": "history-author",
          "user-name": "History Author",
          "user-karma": 0,
          "user-avatar-data":
            "http://www.wikidot.com/avatar.php?userid=12345&amp;size=small",
          "user-profile-url": "http://www.wikidot.com/user:info/history-author"
        },
        comments: "old revision"
      },
      {
        revision_id: 9000342,
        revision_type: "regular",
        revision_number: 2,
        created_at: "2026-08-15T00:00:00Z",
        user_id: 12345,
        author: {
          "user-id": 12345,
          "user-slug": "history-author",
          "user-name": "History Author",
          "user-karma": 0,
          "user-avatar-data":
            "http://www.wikidot.com/avatar.php?userid=12345&amp;size=small",
          "user-profile-url": "http://www.wikidot.com/user:info/history-author"
        },
        comments: "new revision"
      }
    ]
  } else if (
    rpcRequest.method === "page_revision_diff" &&
    hasExactKeys(rpcRequest.params, [
      "from_revision_number",
      "page_id",
      "site_id",
      "to_revision_number"
    ]) &&
    rpcRequest.params.site_id === 6000005 &&
    rpcRequest.params.page_id === HISTORY_WORKFLOW_PAGE_ID &&
    ((rpcRequest.params.from_revision_number === 1 &&
      rpcRequest.params.to_revision_number === 2) ||
      (rpcRequest.params.from_revision_number === 2 &&
        rpcRequest.params.to_revision_number === 1))
  ) {
    pageReadRequests.pageRevisionDiff.push(rpcRequest.params)
    result = {
      site_id: 6000005,
      page_id: rpcRequest.params.page_id,
      from_revision_number: rpcRequest.params.from_revision_number,
      to_revision_number: rpcRequest.params.to_revision_number,
      lines:
        rpcRequest.params.from_revision_number === 1
          ? [{ kind: "removed", text: "OLD STALE DIFF" }]
          : [
              { kind: "added", text: "NEW CURRENT DIFF" },
              { kind: "removed", text: "OLD REVISION TEXT" },
              { kind: "unchanged", text: "SHARED FIXTURE TEXT" }
            ]
    }
    if (rpcRequest.params.from_revision_number === 1) {
      fixtureState.pendingPageRevisionDiffResponse = (outcome = "failure") => {
        if (outcome === "success") {
          sendRpcResult(response, rpcRequest.id, result)
        } else {
          sendRpcError(response, rpcRequest.id, -32603, "OLD STALE ERROR")
        }
      }
      return { responded: true }
    }
  } else if (
    rpcRequest.method === "page_revision_get" &&
    hasExactKeys(rpcRequest.params, [
      "details",
      "page_id",
      "revision_number",
      "site_id"
    ]) &&
    rpcRequest.params.site_id === 6000005 &&
    rpcRequest.params.page_id === HISTORY_WORKFLOW_PAGE_ID &&
    [1, 2].includes(rpcRequest.params.revision_number) &&
    hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
    (rpcRequest.params.details.compiled_html || rpcRequest.params.details.wikitext)
  ) {
    pageReadRequests.pageRevisionGet.push(rpcRequest.params)
    result = {
      revision_id: 9000340 + rpcRequest.params.revision_number,
      revision_type: "regular",
      revision_number: rpcRequest.params.revision_number,
      site_id: 6000005,
      page_id: HISTORY_WORKFLOW_PAGE_ID,
      user_id: 123,
      created_at: "2026-08-15T00:00:00Z",
      changes: ["wikitext"],
      comments: rpcRequest.params.revision_number === 1 ? "old revision" : "new revision",
      wikitext: rpcRequest.params.details.wikitext
        ? `Source for revision ${rpcRequest.params.revision_number}`
        : null,
      compiled_body_html: rpcRequest.params.details.compiled_html
        ? `<p>Historical version ${rpcRequest.params.revision_number}</p>`
        : null,
      compiled_body_styles: [],
      tags: []
    }
  } else if (
    rpcRequest.method === "page_revision_get" &&
    hasExactKeys(rpcRequest.params, [
      "details",
      "page_id",
      "revision_number",
      "site_id"
    ]) &&
    rpcRequest.params.site_id === 6000005 &&
    pageById(rpcRequest.params.page_id) &&
    rpcRequest.params.revision_number === 0 &&
    hasExactKeys(rpcRequest.params.details, ["compiled_html", "wikitext"]) &&
    rpcRequest.params.details.compiled_html === false &&
    rpcRequest.params.details.wikitext === false
  ) {
    pageReadRequests.pageRevisionGet.push(rpcRequest.params)
    result = {
      revision_number: 0,
      user_id: pageById(rpcRequest.params.page_id)?.creator_user_id
    }
  } else {
    return undefined
  }

  return { result }
}

const displayNameForUserId = (userId) => {
  if (userId === 123) return "Rokurokubi"
  if (userId === 456) return "Fixture Updater"
  return null
}
