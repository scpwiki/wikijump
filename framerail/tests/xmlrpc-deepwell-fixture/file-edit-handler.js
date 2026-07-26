import {
  fixtureState,
  hasExactKeys,
  pageById,
  requestContextHeaders,
  updateFixtureFile
} from "./context.js"
import { sendRpcError } from "./response.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handleFileEditRpc = ({ rpcRequest, request, response }) => {
  const { counters, fileRequests, filesByPageId, pendingUploads } = fixtureState

  if (
    rpcRequest.method === "file_edit" &&
    hasExactKeys(rpcRequest.params, [
      "bypass_filter",
      "file_id",
      "ip_address",
      "last_revision_id",
      "page_id",
      "revision_comments",
      "site_id",
      "uploaded_blob_id",
      "user_id"
    ]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page_id === "number" &&
    typeof rpcRequest.params.file_id === "number" &&
    typeof rpcRequest.params.last_revision_id === "number" &&
    typeof rpcRequest.params.uploaded_blob_id === "string" &&
    pendingUploads[rpcRequest.params.uploaded_blob_id] &&
    typeof rpcRequest.params.revision_comments === "string" &&
    rpcRequest.params.user_id === 123 &&
    typeof rpcRequest.params.ip_address === "string" &&
    rpcRequest.params.bypass_filter === true &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page_id)?.slug
  ) {
    const pageFiles = filesByPageId[rpcRequest.params.page_id] ?? {}
    const existing = Object.values(pageFiles).find(
      (file) =>
        file.file_id === rpcRequest.params.file_id &&
        file.revision_id === rpcRequest.params.last_revision_id
    )
    if (!existing) {
      sendRpcError(response, rpcRequest.id, -32602, "Unexpected fixture file_edit target")
      return { responded: true }
    }
    fileRequests.fileEdit.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    const content = pendingUploads[rpcRequest.params.uploaded_blob_id]
    delete pendingUploads[rpcRequest.params.uploaded_blob_id]
    updateFixtureFile(
      existing,
      content,
      rpcRequest.params.revision_comments,
      rpcRequest.params.user_id
    )
    return { result: { file_id: existing.file_id, revision_id: existing.revision_id } }
  }

  if (
    rpcRequest.method === "file_restore" &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page_id === "number" &&
    pageById(rpcRequest.params.page_id) &&
    typeof rpcRequest.params.file_id === "number" &&
    typeof rpcRequest.params.revision_comments === "string" &&
    rpcRequest.params.user_id === 123 &&
    typeof rpcRequest.params.ip_address === "string" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    request.headers["x-deepwell-page"] === pageById(rpcRequest.params.page_id)?.slug
  ) {
    fileRequests.fileRestore.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    return {
      result: {
        file_id: rpcRequest.params.file_id,
        page_id: rpcRequest.params.page_id,
        revision_id: counters.nextRevisionId++
      }
    }
  }

  return undefined
}
