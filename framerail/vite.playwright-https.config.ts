import { readFileSync } from "node:fs"

import type { UserConfig } from "vite"

import baseConfig from "./vite.config.ts"

const tlsKey = process.env.WIKIJUMP_PLAYWRIGHT_TLS_KEY
const tlsCert = process.env.WIKIJUMP_PLAYWRIGHT_TLS_CERT

if (!tlsKey || !tlsCert) {
  throw new Error(
    "Playwright HTTPS Vite fixture requires an ephemeral TLS key and certificate"
  )
}

const config: UserConfig = {
  ...baseConfig,
  server: {
    ...baseConfig.server,
    host: "127.0.0.1",
    port: Number(process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"),
    strictPort: true,
    https: {
      key: readFileSync(tlsKey),
      cert: readFileSync(tlsCert)
    }
  }
}

export default config
