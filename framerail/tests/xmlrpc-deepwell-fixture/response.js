/**
 * @param {import("node:http").ServerResponse} response
 * @param {unknown} id
 * @param {number} code
 * @param {string} message
 */
export const sendRpcError = (response, id, code, message) => {
  response.writeHead(200, { "content-type": "application/json" }).end(
    JSON.stringify({
      error: { code, message },
      id,
      jsonrpc: "2.0"
    })
  )
}

/**
 * @param {import("node:http").ServerResponse} response
 * @param {unknown} id
 * @param {unknown} result
 */
export const sendRpcResult = (response, id, result) => {
  response
    .writeHead(200, { "content-type": "application/json" })
    .end(JSON.stringify({ id, jsonrpc: "2.0", result }))
}
