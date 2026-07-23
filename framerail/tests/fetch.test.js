import { strict as assert } from "node:assert"
import test from "node:test"

import { wjfetch } from "../src/lib/fetch.ts"

test("wjfetch applies its timeout without mutating caller options", async (t) => {
  const originalFetch = globalThis.fetch
  t.after(() => {
    globalThis.fetch = originalFetch
  })

  const options = {
    timeout: 25,
    headers: { accept: "application/json" }
  }
  let forwardedOptions
  globalThis.fetch = async (_url, init) => {
    forwardedOptions = init
    return new Response(null, { status: 204 })
  }

  const response = await wjfetch("https://example.test/resource", options)

  assert.equal(response.status, 204)
  assert.deepEqual(options, {
    timeout: 25,
    headers: { accept: "application/json" }
  })
  assert.equal("timeout" in forwardedOptions, false)
  assert.ok(forwardedOptions.signal instanceof AbortSignal)
})
