import { strict as assert } from "node:assert"
import test from "node:test"

import { uploadToPresignUrl } from "../src/lib/server/deepwell/presigned-upload.ts"

test("presigned uploads reject fulfilled HTTP error responses", async (t) => {
  const originalFetch = globalThis.fetch
  t.after(() => {
    globalThis.fetch = originalFetch
  })
  globalThis.fetch = async () => new Response(null, { status: 503 })

  const file = new File(["content"], "example.txt")
  await assert.rejects(
    uploadToPresignUrl("https://uploads.example.test/pending", file),
    /HTTP status 503/
  )
})

test("presigned uploads fulfill only after an accepted PUT", async (t) => {
  const originalFetch = globalThis.fetch
  t.after(() => {
    globalThis.fetch = originalFetch
  })
  let request
  globalThis.fetch = async (url, init) => {
    request = { url, init }
    return new Response(null, { status: 200 })
  }

  const file = new File(["content"], "example.txt")
  await uploadToPresignUrl("https://uploads.example.test/pending", file)

  assert.equal(request.url, "https://uploads.example.test/pending")
  assert.equal(request.init.method, "PUT")
  assert.equal(request.init.body, file)
})
