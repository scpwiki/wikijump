import type { PlaywrightTestConfig } from "@playwright/test"

const config: PlaywrightTestConfig = {
  testDir: "./tests",
  testMatch: "**/*.spec.ts",
  webServer: {
    command:
      "sh -c 'node tests/xmlrpc-deepwell-fixture-server.js & fixture=$!; trap \"kill $fixture\" EXIT INT TERM; pnpm build && DEEPWELL_HOST=127.0.0.1 DEEPWELL_PORT=42747 pnpm preview'",
    env: {
      XML_RPC_WRITE_PASSWORD: "wikijumpadmin1",
      XML_RPC_WRITE_USERNAME: "admin@wikijump",
      WIKIDOT_API_KEY: "test-key",
      WIKIDOT_APP_NAME: "test-app",
      WIKIDOT_XMLRPC_OWNER_USERNAME: "rokurokubi",
      WIKIJUMP_XMLRPC_LOCAL_FILE_UPLOAD: "1"
    },
    port: 4173
  }
}

export default config
