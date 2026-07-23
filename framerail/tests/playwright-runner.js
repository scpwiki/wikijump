import { spawn } from "node:child_process"
import { createServer } from "node:net"

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
const child = spawn("pnpm", ["exec", "playwright", "test", ...process.argv.slice(2)], {
  env: {
    ...process.env,
    PLAYWRIGHT_APP_PORT: String(appPort),
    PLAYWRIGHT_FIXTURE_PORT: String(fixturePort)
  },
  stdio: "inherit"
})

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.once(signal, () => child.kill(signal))
}

child.once("error", (error) => {
  console.error(error)
  process.exitCode = 1
})
child.once("exit", (code, signal) => {
  process.exitCode = signal ? 1 : (code ?? 1)
})
