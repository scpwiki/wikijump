import { fixtureState, hasExactKeys, requestContextHeaders } from "./context.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
export const handleIdentityRpc = ({ rpcRequest, request }) => {
  const { pageWriteRequests } = fixtureState
  let result

  if (
    rpcRequest.method === "login" &&
    hasExactKeys(rpcRequest.params, [
      "ip_address",
      "name_or_email",
      "password",
      "user_agent"
    ]) &&
    rpcRequest.params.name_or_email === process.env.XML_RPC_WRITE_USERNAME &&
    rpcRequest.params.password === process.env.XML_RPC_WRITE_PASSWORD &&
    typeof rpcRequest.params.ip_address === "string" &&
    rpcRequest.params.user_agent === "wikijump-xmlrpc-api/0.1"
  ) {
    pageWriteRequests.login.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    result = { needs_mfa: false, session_token: "fixture-session-token" }
  } else if (
    rpcRequest.method === "session_get" &&
    Array.isArray(rpcRequest.params) &&
    rpcRequest.params.length === 1 &&
    rpcRequest.params[0] === "fixture-session-token"
  ) {
    pageWriteRequests.sessionGet.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    result = { user_id: 123 }
  } else if (
    rpcRequest.method === "user_get" &&
    hasExactKeys(rpcRequest.params, ["user"]) &&
    ((rpcRequest.params.user === 123 &&
      request.headers["x-deepwell-session-token"] === "fixture-session-token") ||
      rpcRequest.params.user === "rokurokubi")
  ) {
    pageWriteRequests.userGet.push({
      headers: requestContextHeaders(request),
      params: rpcRequest.params
    })
    result = {
      aliases: [],
      user_id: 123,
      name: "Rokurokubi",
      slug: "rokurokubi"
    }
  } else {
    return undefined
  }

  return { result }
}
