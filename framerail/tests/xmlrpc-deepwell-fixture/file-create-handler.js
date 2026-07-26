import {
  createFixtureFile,
  fixtureState,
  hasExactKeys,
  pageById,
  requestContextHeaders
} from "./context.js"
import { sendRpcError } from "./response.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handleFileCreateRpc = ({ rpcRequest, request, response }) => {
  const { fileRequests, filesByPageId, pendingUploads } = fixtureState

  if (
    rpcRequest.method !== "file_create" ||
    !hasExactKeys(rpcRequest.params, [
      "bypass_filter",
      "ip_address",
      "name",
      "page_id",
      "revision_comments",
      "site_id",
      "uploaded_blob_id",
      "user_id"
    ]) ||
    rpcRequest.params.site_id !== 6000005 ||
    typeof rpcRequest.params.page_id !== "number" ||
    !pageById(rpcRequest.params.page_id) ||
    typeof rpcRequest.params.name !== "string" ||
    typeof rpcRequest.params.uploaded_blob_id !== "string" ||
    !pendingUploads[rpcRequest.params.uploaded_blob_id] ||
    typeof rpcRequest.params.revision_comments !== "string" ||
    rpcRequest.params.user_id !== 123 ||
    typeof rpcRequest.params.ip_address !== "string" ||
    rpcRequest.params.bypass_filter !== true ||
    request.headers["x-deepwell-session-token"] !== "fixture-session-token" ||
    request.headers["x-deepwell-site-id"] !== "6000005" ||
    request.headers["x-deepwell-page"] !== pageById(rpcRequest.params.page_id)?.slug
  ) {
    return undefined
  }

  fileRequests.fileCreate.push({
    headers: requestContextHeaders(request),
    params: rpcRequest.params
  })
  const pageFiles = (filesByPageId[rpcRequest.params.page_id] ??= {})
  if (pageFiles[rpcRequest.params.name]) {
    sendRpcError(
      response,
      rpcRequest.id,
      -32602,
      "Unexpected fixture duplicate file_create target"
    )
    return { responded: true }
  }
  const content = pendingUploads[rpcRequest.params.uploaded_blob_id]
  delete pendingUploads[rpcRequest.params.uploaded_blob_id]
  const file = createFixtureFile(
    rpcRequest.params.name,
    content,
    rpcRequest.params.revision_comments,
    rpcRequest.params.user_id,
    false
  )
  pageFiles[file.name] = file
  return { result: { file_id: file.file_id, revision_id: file.revision_id } }
}
