// TODO refactor into proper TS service

import { JSONRPCClient, type JSONRPCRequest } from "json-rpc-2.0"

export const DEEPWELL_HOST = process.env.DEEPWELL_HOST || "localhost"
export const DEEPWELL_PORT = 2747
export const DEEPWELL_URL = `http://${DEEPWELL_HOST}:${DEEPWELL_PORT}/jsonrpc`
export const client = new JSONRPCClient(processRawRequest)

async function processRawRequest(request: JSONRPCRequest): Promise<void> {
  const response = await fetch(DEEPWELL_URL, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(request)
  })

  const data = await response.json()
  client.receive(data)
}

export async function ping(): void {
  await client.request("ping")
}

console.info(`Using DEEPWELL service at ${DEEPWELL_URL}`)
