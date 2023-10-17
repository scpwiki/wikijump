// TODO refactor into proper TS service

import * as jsonrpc from "json-rpc-client"

const DEEPWELL_HOST = process.env.DEEPWELL_HOST || "localhost"
const DEEPWELL_PORT = 2747
const DEEPWELL_CLIENT = new jsonrpc({ host: DEEPWELL_HOST, port: DEEPWELL_PORT })
DEEPWELL_CLIENT.connect()

export async function ping(): void {
  const reply = DEEPWELL_CLIENT.send("ping")
  if (reply.error) {
    throw new Error("Cannot ping DEEPWELL!")
  }
}
