import { handleFileRpc } from "./file-handler.js"
import { handleReadRpc } from "./read-handler.js"
import { sendRpcError, sendRpcResult } from "./response.js"
import { handleWriteRpc } from "./write-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 *   port: number
 * }} input
 */
export const dispatchFixtureRpc = ({ rpcRequest, request, response, port }) => {
  const input = { rpcRequest, request, response }
  const outcome =
    handleReadRpc(input) ?? handleWriteRpc(input) ?? handleFileRpc({ ...input, port })

  if (outcome && "responded" in outcome && outcome.responded) return
  if (outcome) {
    sendRpcResult(response, rpcRequest.id, outcome.result)
    return
  }

  const requestShape =
    rpcRequest.method === "article_view"
      ? ` ${JSON.stringify({
          paramKeys: Object.keys(rpcRequest.params ?? {}).sort(),
          route: rpcRequest.params?.route,
          sessionTokenType:
            rpcRequest.params?.session_token === null
              ? "null"
              : typeof rpcRequest.params?.session_token,
          siteId: rpcRequest.params?.site_id,
          headerSiteId: request.headers["x-deepwell-site-id"],
          hasHeaderSessionToken: Boolean(request.headers["x-deepwell-session-token"])
        })}`
      : ""
  sendRpcError(
    response,
    rpcRequest.id,
    -32601,
    `Unexpected Deepwell fixture request: ${rpcRequest.method}${requestShape}`
  )
}
