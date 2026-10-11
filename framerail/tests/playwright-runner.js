import { randomUUID } from "node:crypto"
import { spawn } from "node:child_process"
import { createServer } from "node:net"
import { fileURLToPath } from "node:url"

import { cleanupOwnedTlsDirectory } from "./playwright-https-tls.js"

const playwrightCli = fileURLToPath(import.meta.resolve("@playwright/test/cli"))

const allocatePort = () =>
  new Promise((resolve, reject) => {
    const server = createServer()
    server.once("error", reject)
    server.listen(0, "127.0.0.1", () => {
      const address = server.address()
      if (!address || typeof address === "string") {
        server.close()
        reject(new Error("Failed to allocate a Playwright port"))
        return
      }
      server.close((error) => (error ? reject(error) : resolve(address.port)))
    })
  })

const appPort = await allocatePort()
const fixturePort = await allocatePort()
const browserSupportHttps = process.argv.includes("playwright.browser-support.config.ts")
const httpsPort = browserSupportHttps ? await allocatePort() : undefined
const tlsOwnerToken = browserSupportHttps ? randomUUID() : undefined
const tlsDirectory = browserSupportHttps
  ? (process.env.WIKIJUMP_PLAYWRIGHT_TLS_DIR ??
    `/tmp/wikijump-playwright-tls-${tlsOwnerToken}`)
  : undefined
const cleanupBrowserSupportHttps = () => {
  if (browserSupportHttps && tlsDirectory && tlsOwnerToken) {
    try {
      cleanupOwnedTlsDirectory(tlsDirectory, tlsOwnerToken)
    } catch (error) {
      console.error(
        `Failed to clean this run's Playwright HTTPS fixture: ${error.message}`
      )
      process.exitCode = 1
    }
  }
}
const child = spawn(process.execPath, [playwrightCli, "test", ...process.argv.slice(2)], {
  env: {
    ...process.env,
    PLAYWRIGHT_APP_PORT: String(appPort),
    PLAYWRIGHT_FIXTURE_PORT: String(fixturePort),
    ...(browserSupportHttps
      ? {
          PLAYWRIGHT_HTTPS_APP_PORT:
            process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? String(httpsPort),
          WIKIJUMP_PLAYWRIGHT_TLS_DIR: tlsDirectory,
          WIKIJUMP_PLAYWRIGHT_TLS_OWNER_TOKEN: tlsOwnerToken
        }
      : {})
  },
  stdio: "inherit"
})

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.once(signal, () => child.kill(signal))
}

child.once("error", (error) => {
  cleanupBrowserSupportHttps()
  console.error(error)
  process.exitCode = 1
})
child.once("exit", (code, signal) => {
  cleanupBrowserSupportHttps()
  process.exitCode = signal ? 1 : (code ?? 1)
})
