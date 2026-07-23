import type { PlaywrightTestConfig } from "@playwright/test"

const appPort = Number(process.env.PLAYWRIGHT_APP_PORT ?? "4173")
const fixturePort = Number(process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747")

const config: PlaywrightTestConfig = {
  testDir: "./tests",
  testMatch: "**/*.spec.ts",
  webServer: {
    command: `sh -c 'node tests/xmlrpc-deepwell-fixture-server.js & fixture=$!; trap "kill $fixture" EXIT INT TERM; pnpm build && NODE_ENV=development DEEPWELL_HOST=127.0.0.1 DEEPWELL_PORT=${fixturePort} pnpm preview --host 127.0.0.1 --port ${appPort}'`,
    env: {
      PLAYWRIGHT_FIXTURE_PORT: String(fixturePort),
      XML_RPC_WRITE_PASSWORD: "wikijumpadmin1",
      XML_RPC_WRITE_USERNAME: "admin@wikijump",
      WIKIDOT_API_KEY: "test-key",
      WIKIDOT_APP_NAME: "test-app",
      WIKIDOT_XMLRPC_OWNER_USERNAME: "rokurokubi",
      WIKIJUMP_XMLRPC_LOCAL_FILE_UPLOAD: "1"
    },
    port: appPort
  }
}

export default config
