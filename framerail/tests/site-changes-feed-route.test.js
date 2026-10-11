// @ts-nocheck
import assert from "node:assert/strict"
import { createServer as createHttpServer } from "node:http"
import { fileURLToPath } from "node:url"
import { after, before, beforeEach, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))
const siteHeaders = {
  "X-Wikijump-Site-Id": "17",
  "X-Wikijump-Site-Slug": "test"
}

let previousWorkingDirectory
let vite
let server
let baseUrl
let client
let originalClientRequest
let feedCalls
let feedOutput

const feedItem = {
  revision_id: 3_000_000_123,
  revision_number: 7,
  is_new_page: false,
  is_file: false,
  created_at: "2026-10-07T16:53:08Z",
  slug: "fixture-site-changes-page",
  title: "Fixture Site Changes Page",
  change_type: "source",
  user_id: 42,
  user_name: "Author",
  body_html: "<p>Local site change body</p>"
}

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()
  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  originalClientRequest = client.request
  client.request = async (method, params) => {
    if (method === "wikidot_site_changes_feed") {
      feedCalls.push(params.site_id)
      return feedOutput === null ? null : structuredClone(feedOutput)
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  server = createHttpServer((request, response) => vite.middlewares(request, response))
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert.equal(typeof address, "object")
  baseUrl = `http://127.0.0.1:${address.port}`
})

after(async () => {
  if (client && originalClientRequest) client.request = originalClientRequest
  if (server) await new Promise((resolve) => server.close(resolve))
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

beforeEach(() => {
  feedCalls = []
  feedOutput = {
    site_name: "Route test site",
    items: [feedItem]
  }
})

test("site changes feed serves RSS with the recent-page-changes channel", async () => {
  const response = await fetch(`${baseUrl}/feed/site-changes.xml`, {
    headers: siteHeaders
  })
  const body = await response.text()

  assert.equal(response.status, 200)
  assert.equal(response.headers.get("content-type"), "text/xml;charset=utf-8")
  assert.deepEqual(feedCalls, [17])
  assert.match(body, /^<\?xml version="1\.0" encoding="UTF-8" \?>/u)
  assert.match(
    body,
    /<title>Recent page changes from site "Route test site" \(a Wikidot site\)<\/title>/u
  )
  assert.match(body, /<link>https?:\/\/[^<]+<\/link>/u)
  assert.match(body, /<lastBuildDate>[A-Z][a-z]{2}, \d{2} [A-Z][a-z]{2} \d{4}/u)
  assert.match(
    body,
    /<title>&quot;Fixture Site Changes Page&quot; - source change<\/title>/u
  )
  assert.match(
    body,
    /<guid>http:\/\/[^<]+\/fixture-site-changes-page#revision-3000000123<\/guid>/u
  )
  assert.match(
    body,
    /<link>https?:\/\/[^<]+\/fixture-site-changes-page<\/link>/u
  )
})

test("site changes feed is independently reachable without the wiki page", async () => {
  const response = await fetch(`${baseUrl}/feed/site-changes.xml`, {
    headers: siteHeaders
  })
  const body = await response.text()

  assert.equal(response.status, 200)
  // The route family owns the path, so the missing `feed` wiki page never renders.
  assert.doesNotMatch(body, /id="page-title"/u)
  assert.doesNotMatch(body, /<html/u)
})

test("site changes feed returns 404 when the site is not viewable", async () => {
  feedOutput = null

  const response = await fetch(`${baseUrl}/feed/site-changes.xml`, {
    headers: siteHeaders
  })
  await response.text()

  assert.equal(response.status, 404)
  assert.deepEqual(feedCalls, [17])
})

test("other feed paths are not claimed by the site changes route", async () => {
  for (const shape of ["/feed/site-changes.json", "/feed/site-changes", "/feed/other.xml"]) {
    const response = await fetch(`${baseUrl}${shape}`, { headers: siteHeaders })
    await response.text()
    assert.notEqual(response.status, 200, shape)
  }
  assert.deepEqual(feedCalls, [])
})
