// TODO refactor into proper TS service

import * as jsonrpc from "json-rpc-client"

const DEEPWELL_HOST = process.env.DEEPWELL_HOST || "localhost"
const DEEPWELL_PORT = 2747
const DEEPWELL_CLIENT = new jsonrpc({
  host: DEEPWELL_HOST,
  port: DEEPWELL_PORT,
  keepalive: true
})
DEEPWELL_CLIENT.connect()

export interface JsonRpcSuccess<T> {
  jsonrpc: "2.0"
  result: T
  id: number
}

export interface JsonRpcFailure {
  jsonrpc: "2.0"
  error: JsonRpcError
  id: number
}

export interface JsonRpcError {
  code: number
  message: string
  data?: any
}

export type JsonRpcResponse = JsonRpcSuccess | JsonRpcFailure

export async function wellcall<T>(method: string, params?: any, notification?: bool): T {
  const reply = await DEEPWELL_CLIENT.send(method, params, notification)
  if (reply.error) {
    throw new Error(reply.error)
  }

  return reply.response
}

export async function ping(): void {
  try {
    await wellcall.send("ping")
  } catch {
    throw new Error("Cannot ping DEEPWELL!")
  }
}
