import type { PlaywrightTestConfig } from "@playwright/test"

const config: PlaywrightTestConfig = {
  webServer: {
    command: "pnpm build && pnpm preview",
    env: {
      XML_RPC_PASSWORD: "test-key",
      XML_RPC_USERNAME: "test-app"
    },
    port: 4173
  }
}

export default config
