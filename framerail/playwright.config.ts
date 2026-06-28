import type { PlaywrightTestConfig } from "@playwright/test"

const config: PlaywrightTestConfig = {
  webServer: {
    command:
      "sh -c 'node tests/xmlrpc-deepwell-fixture-server.js & fixture=$!; trap \"kill $fixture\" EXIT INT TERM; pnpm build && DEEPWELL_HOST=127.0.0.1 DEEPWELL_PORT=42747 pnpm preview'",
    env: {
      XML_RPC_PASSWORD: "test-key",
      XML_RPC_USERNAME: "test-app"
    },
    port: 4173
  }
}

export default config
