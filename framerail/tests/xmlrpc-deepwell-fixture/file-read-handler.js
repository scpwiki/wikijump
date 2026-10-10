import {
  fixtureState,
  hasExactKeys,
  requestContextHeaders,
  toFileResult,
  toFileResultWithoutData
} from "./context.js"
import { pages } from "./data.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   port: number
 * }} input
 */
export const handleFileReadRpc = ({ rpcRequest, request, port }) => {
  const { counters, fileRequests, filesByPageId } = fixtureState
  let result

  if (rpcRequest.method === "blob_get" && rpcRequest.params === "112233") {
    // Deterministic avatar bytes for the synthetic profile fixture only.
    result = {
      data: Array.from(
        Buffer.from(
          "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=",
          "base64"
        )
      ),
      mime: "image/png"
    }
  } else if (
    rpcRequest.method === "page_get_files" &&
    hasExactKeys(rpcRequest.params, ["deleted", "page_id", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page_id === "number" &&
    rpcRequest.params.deleted === false
  ) {
    fileRequests.pageGetFiles.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    result = Object.values(filesByPageId[rpcRequest.params.page_id] ?? {}).map(
      toFileResultWithoutData
    )
  } else if (
    rpcRequest.method === "page_get_files" &&
    hasExactKeys(rpcRequest.params, ["deleted", "page_id", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page_id === "number" &&
    rpcRequest.params.deleted === true
  ) {
    result = Object.values(
      fixtureState.deletedFilesByPageId[rpcRequest.params.page_id] ?? {}
    ).map(toFileResultWithoutData)
  } else if (
    rpcRequest.method === "file_revision_page_history" &&
    hasExactKeys(rpcRequest.params, ["limit", "page_id", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    (rpcRequest.params.page_id === 3000345 || rpcRequest.params.page_id === 3000173) &&
    rpcRequest.params.limit === 20
  ) {
    result = { revision_count: 0, revisions: [] }
  } else if (
    rpcRequest.method === "file_get" &&
    hasExactKeys(rpcRequest.params, ["details", "file", "page_id", "site_id"]) &&
    rpcRequest.params.site_id === 6000005 &&
    typeof rpcRequest.params.page_id === "number" &&
    typeof rpcRequest.params.file === "string" &&
    hasExactKeys(rpcRequest.params.details, ["data"]) &&
    typeof rpcRequest.params.details.data === "boolean"
  ) {
    fileRequests.fileGet.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    const file =
      filesByPageId[rpcRequest.params.page_id]?.[rpcRequest.params.file] ?? null
    result = file ? toFileResult(file, Boolean(rpcRequest.params.details.data)) : null
  } else if (
    rpcRequest.method === "blob_upload" &&
    hasExactKeys(rpcRequest.params, ["blob_size", "scope", "user_id"]) &&
    rpcRequest.params.user_id === 123 &&
    typeof rpcRequest.params.blob_size === "number" &&
    rpcRequest.params.scope === "page" &&
    request.headers["x-deepwell-session-token"] === "fixture-session-token" &&
    request.headers["x-deepwell-site-id"] === "6000005" &&
    typeof request.headers["x-deepwell-page"] === "string" &&
    Object.hasOwn(pages, request.headers["x-deepwell-page"])
  ) {
    fileRequests.blobUpload.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    const pendingBlobId = `fixture-blob-${counters.nextPendingBlobId++}`
    result = {
      pending_blob_id: pendingBlobId,
      presign_url: `http://127.0.0.1:${port}/upload/${encodeURIComponent(pendingBlobId)}`
    }
  } else {
    return undefined
  }

  return { result }
}
