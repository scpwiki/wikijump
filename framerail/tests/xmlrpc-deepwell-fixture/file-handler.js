import { handleFileCreateRpc } from "./file-create-handler.js"
import { handleFileEditRpc } from "./file-edit-handler.js"
import { handleFileReadRpc } from "./file-read-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 *   port: number
 * }} input
 */
export const handleFileRpc = (input) => {
  return (
    handleFileReadRpc(input) ?? handleFileCreateRpc(input) ?? handleFileEditRpc(input)
  )
}
