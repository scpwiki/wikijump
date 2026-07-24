import { randomUUID } from "node:crypto"

import type { PlaywrightTestConfig } from "@playwright/test"

const appPort = Number(process.env.PLAYWRIGHT_APP_PORT ?? "4173")
const fixturePort = Number(process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747")
const wikidotAppName = process.env.WIKIDOT_APP_NAME ?? "test-app"
const wikidotApiKey = process.env.WIKIDOT_API_KEY ?? randomUUID()
const xmlRpcWriteUsername = process.env.XML_RPC_WRITE_USERNAME ?? "admin@wikijump"
const xmlRpcWritePassword = process.env.XML_RPC_WRITE_PASSWORD ?? randomUUID()

Object.assign(process.env, {
  WIKIDOT_API_KEY: wikidotApiKey,
  WIKIDOT_APP_NAME: wikidotAppName,
  XML_RPC_WRITE_PASSWORD: xmlRpcWritePassword,
  XML_RPC_WRITE_USERNAME: xmlRpcWriteUsername
})

const config: PlaywrightTestConfig = {
  testDir: "./tests",
  testMatch: "**/*.spec.ts",
  webServer: {
    command: `sh -c 'node tests/xmlrpc-deepwell-fixture-server.js & fixture=$!; trap "kill $fixture" EXIT INT TERM; pnpm build && NODE_ENV=development DEEPWELL_HOST=127.0.0.1 DEEPWELL_PORT=${fixturePort} pnpm preview --host 127.0.0.1 --port ${appPort}'`,
    env: {
      PLAYWRIGHT_FIXTURE_PORT: String(fixturePort),
      XML_RPC_WRITE_PASSWORD: xmlRpcWritePassword,
      XML_RPC_WRITE_USERNAME: xmlRpcWriteUsername,
      WIKIDOT_API_KEY: wikidotApiKey,
      WIKIDOT_APP_NAME: wikidotAppName,
      WIKIDOT_XMLRPC_OWNER_USERNAME: "rokurokubi",
      WIKIJUMP_XMLRPC_LOCAL_FILE_UPLOAD: "1"
    },
    port: appPort
  }
}

export default config
