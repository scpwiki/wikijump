import { strict as assert } from "node:assert"
import test from "node:test"

import { createFramerailHttpServer } from "../server.js"

const listen = async (server) => {
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  return `http://127.0.0.1:${address.port}`
}

test("Framerail HTTP server delegates requests and closes its fence cache", async () => {
  let fenceCacheCloseCount = 0
  const lifecycle = createFramerailHttpServer({
    fastPathHandler: async (_request, response) => {
      response.statusCode = 204
      response.end()
    },
    fenceCache: {
      close: () => {
        fenceCacheCloseCount += 1
      }
    }
  })
  const baseUrl = await listen(lifecycle.server)

  const response = await fetch(baseUrl)
  assert.equal(response.status, 204)

  lifecycle.closeServer()
  assert.equal(fenceCacheCloseCount, 1)
})

test("Framerail HTTP server turns handler failures into 500 responses", async (t) => {
  const lifecycle = createFramerailHttpServer({
    fastPathHandler: async () => {
      throw new Error("fast path failed")
    },
    fenceCache: { close: () => {} }
  })
  t.after(lifecycle.closeServer)
  const baseUrl = await listen(lifecycle.server)

  const response = await fetch(baseUrl)
  assert.equal(response.status, 500)
  assert.equal(await response.text(), "fast path failed")
})
