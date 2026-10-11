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
let requestedScopeOutput

const feedItem = {
  forum_post_id: 91,
  forum_thread_id: 42,
  created_at: "2026-10-07T12:21:30Z",
  title: "Post title",
  thread_title: "Thread title",
  thread_slug: "thread-title",
  page_slug: null,
  forum_group_id: 2,
  forum_group_name: "Group",
  forum_category_id: 8,
  forum_category_name: "Category",
  forum_category_slug: "category",
  author_user_id: 17,
  author_name: "Author",
  content_html: "<p>Local forum body</p>"
}

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()
  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  originalClientRequest = client.request
  client.request = async (method, params) => {
    if (method === "wikidot_forum_feed") {
      feedCalls.push([params.kind, params.category_id, params.thread_id])
      if (feedOutput === null) return null
      return structuredClone({ ...feedOutput, ...requestedScopeOutput })
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
    site_description: "Route description",
    items: [feedItem]
  }
  requestedScopeOutput = { scope: "site" }
})

test("forum threads feed serves RSS with the forum-threads channel", async () => {
  const response = await fetch(`${baseUrl}/feed/forum/threads.xml`, {
    headers: siteHeaders
  })
  const body = await response.text()

  assert.equal(response.status, 200)
  assert.equal(response.headers.get("content-type"), "text/xml;charset=utf-8")
  assert.deepEqual(feedCalls, [["threads", undefined, undefined]])
  assert.match(body, /^<\?xml version="1\.0" encoding="UTF-8" \?>/u)
  assert.match(body, /<title>Route test site - new forum threads<\/title>/u)
  assert.match(body, /<link>https?:\/\/[^<]+\/forum\/start<\/link>/u)
  assert.match(body, /<title>Thread title<\/title>/u)
})

test("forum posts feed serves RSS with the forum-posts channel", async () => {
  const response = await fetch(`${baseUrl}/feed/forum/posts.xml`, {
    headers: siteHeaders
  })
  const body = await response.text()

  assert.equal(response.status, 200)
  assert.equal(response.headers.get("content-type"), "text/xml;charset=utf-8")
  assert.deepEqual(feedCalls, [["posts", undefined, undefined]])
  assert.match(body, /<title>Route test site - new forum posts<\/title>/u)
  assert.match(body, /<link>https?:\/\/[^<]+\/forum\/start<\/link>/u)
  assert.match(body, /<guid>http:\/\/[^<]+\/forum\/t-42#post-91<\/guid>/u)
})

test("forum feed returns 404 when Deepwell yields no feed output", async () => {
  feedOutput = null

  const response = await fetch(`${baseUrl}/feed/forum/threads.xml`, {
    headers: siteHeaders
  })

  assert.equal(response.status, 404)
  assert.deepEqual(feedCalls, [["threads", undefined, undefined]])
})

test("category and thread feed scopes resolve the legacy Wikidot feed names", async () => {
  const scopedCases = [
    {
      path: "/feed/forum/ct-50864.xml",
      call: ["threads", 50864, undefined],
      scope: "category",
      category: 50864,
      channel: /<link>https?:\/\/[^<]+\/forum\/c-50864<\/link>/u
    },
    {
      path: "/feed/forum/cp-50864.xml",
      call: ["posts", 50864, undefined],
      scope: "category",
      category: 50864,
      channel: /<link>https?:\/\/[^<]+\/forum\/c-50864<\/link>/u
    },
    {
      path: "/feed/forum/t-17768232.xml",
      call: ["posts", undefined, 17768232],
      scope: "thread",
      thread: 17768232,
      channel: /<link>https?:\/\/[^<]+\/forum\/t-17768232<\/link>/u
    }
  ]

  for (const scopedCase of scopedCases) {
    feedCalls = []
    requestedScopeOutput = {
      scope: scopedCase.scope,
      forum_group_name: "Group",
      forum_category_id: scopedCase.category,
      forum_category_name: "Category",
      forum_thread_id: scopedCase.thread,
      forum_thread_title: "Thread title"
    }
    const response = await fetch(`${baseUrl}${scopedCase.path}`, { headers: siteHeaders })
    const body = await response.text()

    assert.equal(response.status, 200, scopedCase.path)
    assert.equal(response.headers.get("content-type"), "text/xml;charset=utf-8")
    assert.deepEqual(feedCalls, [scopedCase.call])
    assert.match(body, scopedCase.channel, scopedCase.path)
  }
})

test("a hidden category or thread feed scope answers 404 like an unknown one", async () => {
  feedOutput = null

  const hiddenScopes = ["/feed/forum/ct-50864.xml", "/feed/forum/t-17768232.xml"]
  for (const shape of hiddenScopes) {
    const response = await fetch(`${baseUrl}${shape}`, { headers: siteHeaders })
    await response.text()
    assert.equal(response.status, 404, shape)
  }
})

test("unknown forum feed shapes remain 404 and never reach Deepwell", async () => {
  const unknownShapes = [
    "/feed/forum/threads",
    "/feed/forum/threads.rss",
    "/feed/forum/posts.atom",
    "/feed/forum/ct-.xml",
    "/feed/forum/ct-abc.xml",
    "/feed/forum/t--5.xml"
  ]

  for (const shape of unknownShapes) {
    const response = await fetch(`${baseUrl}${shape}`, { headers: siteHeaders })
    await response.text()
    assert.equal(response.status, 404, shape)
  }
  assert.deepEqual(feedCalls, [])
})
