// @ts-nocheck
import assert from "node:assert/strict"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))
let previousWorkingDirectory
let vite
let client
let originalClientRequest
let requests
let pageRoute
let printerRoute

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()
  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  originalClientRequest = client.request
  client.request = async (method) => {
    requests.push(method)
    throw new Error(`Unexpected Deepwell request: ${method}`)
  }
  pageRoute = await vite.ssrLoadModule("/src/routes/[slug]/[...extra]/+page.server.ts")
  printerRoute = await vite.ssrLoadModule(
    "/src/routes/printer--friendly/[...path]/+page.server.ts"
  )
})

after(async () => {
  if (client && originalClientRequest) client.request = originalClientRequest
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

const request = new Request("https://site.example.test/_admin", {
  headers: {
    "X-Wikijump-Site-Id": "17",
    "X-Wikijump-Site-Slug": "test"
  }
})
const cookies = { get: () => undefined }
const locals = {}

async function assertRedirect(load, params) {
  await assert.rejects(load({ params, request, cookies, locals }), (thrown) => {
    assert.equal(thrown.status, 303)
    assert.equal(thrown.location, "/_admin")
    return true
  })
}

test("reserved admin no-render aliases redirect before article lookup", async () => {
  for (const slug of ["_admin", "_ADMIN"]) {
    requests = []
    await assertRedirect(pageRoute.load, { slug, extra: "norender/true" })
    assert.deepEqual(requests, [], slug)
  }
})

test("reserved admin print aliases redirect before article lookup", async () => {
  for (const path of ["/_admin", "//_ADMIN"]) {
    requests = []
    await assertRedirect(printerRoute.load, { path })
    assert.deepEqual(requests, [], path)
  }
})

test("ordinary page no-render and print routes are not classified as admin aliases", async () => {
  for (const [load, params] of [
    [pageRoute.load, { slug: "ordinary-page", extra: "norender/true" }],
    [printerRoute.load, { path: "//ordinary-page" }]
  ]) {
    requests = []
    await assert.rejects(
      load({ params, request, cookies, locals }),
      /Unexpected Deepwell request: article_view/u
    )
    assert.ok(requests.includes("article_view"), JSON.stringify(params))
  }
})
