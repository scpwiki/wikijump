import { JSONRPCClient } from "json-rpc-2.0"

import type { Nullable } from "$lib/types"
import type { JSONRPCRequest } from "json-rpc-2.0"
import type { RequestContext } from "../request-context"
import { stripPrivateDeepwellErrorData } from "./public-error.js"

export const DEEPWELL_HOST = process.env.DEEPWELL_HOST || "localhost"
export const DEEPWELL_PORT = process.env.DEEPWELL_PORT || 2747
export const DEEPWELL_URL = `http://${DEEPWELL_HOST}:${DEEPWELL_PORT}/jsonrpc`

export const client = new JSONRPCClient<RequestContext>(sendJsonRpcRequest)

async function sendJsonRpcRequest(
  request: JSONRPCRequest,
  reqContext: RequestContext = {}
): Promise<void> {
  const headers: Record<string, string> = { "content-type": "application/json" }

  // Populate request context in the headers
  if (reqContext?.sessionToken) {
    headers["X-Deepwell-Session-Token"] = reqContext.sessionToken
  }
  if (reqContext?.siteId) {
    headers["X-Deepwell-Site-Id"] = reqContext.siteId.toString()
  }
  if (reqContext?.page) {
    headers["X-Deepwell-Page"] = reqContext.page.toString()
  }

  const response = await fetch(DEEPWELL_URL, {
    method: "POST",
    headers: headers,
    body: JSON.stringify(request)
  })

  const data = stripPrivateDeepwellErrorData(await response.json())
  client.receive(data)
}

export async function ping(): Promise<void> {
  await client.request("ping", {})
}

/* ----- INFO ----- */
interface Info {
  package: PackageInfo
  compile_info: CompileInfo

  current_time: string
  hostname: string
  config_path: string
}

interface PackageInfo {
  name: string
  description: string
  license: string
  repository: string
  version: string
}

interface CompileInfo {
  built_at: string
  rustc_version: string
  endian: string
  target: string
  threads: number
  git_commit: Nullable<string>
}

export async function info(): Promise<Info> {
  return client.request("info", {})
}

console.info(`Using DEEPWELL service at ${DEEPWELL_URL}`)
