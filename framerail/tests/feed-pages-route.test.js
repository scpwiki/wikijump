// @ts-nocheck
import assert from "node:assert/strict"
import { createServer as createHttpServer } from "node:http"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

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

const missingArticleView = {
  site: {
    site_id: 17,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: null,
    deleted_at: null,
    from_wikidot: true,
    slug: "test",
    name: "Route test site",
    tagline: "",
    description: "",
    locale: "en",
    default_page: "start",
    top_bar_page: null,
    side_bar_page: null,
    preferred_domain: null,
    layout: "wikidot",
    license: "cc-by-sa-3.0"
  },
  site_settings: {},
  site_file_domain: "test.wjfiles.localhost",
  license_name: "CC BY-SA 3.0",
  license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
  license_kind: "standard",
  license_html: null,
  user_session: null,
  article_page_cache_key: null,
  public_content_cache_fence: null,
  anonymous_permission_cache_fence: null,
  page: {
    type: "missing",
    data: {
      options: {
        edit: false,
        title: null,
        parent: null,
        tags: null,
        no_redirect: false,
        no_render: false,
        debug: false,
        renderer: false,
        comments: false,
        history: false,
        offset: null,
        data: ""
      },
      redirect_page: null,
      wikitext: "",
      compiled_body_html: "",
      compiled_body_styles: [],
      compiled_top_bar_html: null,
      compiled_side_bar_html: null,
      theme: { type: "default" },
      new_page_wikitext: null,
      page_templates: [],
      selected_template_page_id: null,
      data_form: null
    }
  }
}

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()
  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  originalClientRequest = client.request
  client.request = async (method) => {
    if (method === "article_view") return structuredClone(missingArticleView)
    if (method === "translate") return {}
    if (method === "wikidot_list_pages_feed") {
      throw new Error("Exact /feed/pages must not use the feed endpoint")
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

test("anonymous exact feed/pages renders the generic missing-page shell", async () => {
  const missingResponse = await fetch(`${baseUrl}/feed/pages`, {
    headers: siteHeaders
  })
  const missingBody = await missingResponse.text()

  assert.equal(missingResponse.status, 404)
  // Refs #2090: one coherent missing-page shell — create guidance plus the
  // site _404 body, with no duplicated hardcoded message and no Restore
  // control for a viewer without server-confirmed recoverability.
  assert.match(missingBody, /id="create-it-now-link"/u)
  assert.doesNotMatch(missingBody, /id="404-message"/u)
  assert.doesNotMatch(missingBody, /id="restore-button"/u)
})

test("empty selector feeds retain the Wikidot root separator", async () => {
  const rejectingClientRequest = client.request
  client.request = async (method, ...args) => {
    if (method === "wikidot_list_pages_feed") return { items: [] }
    return rejectingClientRequest(method, ...args)
  }

  try {
    const feedResponse = await fetch(`${baseUrl}/feed/pages/category/news`, {
      headers: siteHeaders
    })
    const feedBody = await feedResponse.text()

    assert.equal(feedResponse.status, 200)
    assert.equal(feedResponse.headers.get("content-type"), "text/xml;charset=utf-8")
    assert.ok(
      feedBody.startsWith(
        '<?xml version="1.0" encoding="UTF-8" ?>\n' +
          '<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:wikidot="http://www.wikidot.com/rss-namespace">\n\n' +
          "\t<channel>\n"
      )
    )
  } finally {
    client.request = rejectingClientRequest
  }
})

test("populated feed items retain the multiline content CDATA envelope", async () => {
  const bodyHtml = "<p>body ]]> & text</p>"
  const createdByHtml = '<span class="printuser">A & B</span>'
  const rejectingClientRequest = client.request
  client.request = async (method, ...args) => {
    if (method === "wikidot_list_pages_feed") {
      return {
        items: [
          {
            slug: "news:item",
            title: "Item",
            created_at: "2026-07-22T23:23:22Z",
            body_html: bodyHtml,
            created_by_html: createdByHtml
          }
        ]
      }
    }
    return rejectingClientRequest(method, ...args)
  }

  try {
    const feedResponse = await fetch(`${baseUrl}/feed/pages/category/news`, {
      headers: siteHeaders
    })
    const feedBody = await feedResponse.text()
    const expectedEnvelope = [
      "\t\t\t\t\t\t\t\t\t\t\t\t<content:encoded>",
      "\t\t\t\t\t<![CDATA[",
      "\t\t\t\t\t\t <p>body ]]]]><![CDATA[> & text</p>",
      '<p>by <span class="printuser">A & B</span></p> ',
      "\t\t\t\t \t]]>",
      "\t\t\t\t</content:encoded>"
    ].join("\n")

    assert.ok(feedBody.includes(expectedEnvelope))

    const encodedElement = /<content:encoded>([\s\S]*?)<\/content:encoded>/u.exec(
      feedBody
    )?.[1]
    assert.ok(encodedElement)
    const parsedContent = Array.from(
      encodedElement.matchAll(/<!\[CDATA\[([\s\S]*?)\]\]>/gu),
      (match) => match[1]
    ).join("")
    const prefix = "\n\t\t\t\t\t\t "
    const suffix = " \n\t\t\t\t \t"
    assert.equal(
      parsedContent.slice(prefix.length, -suffix.length),
      '<p>body ]]> & text</p>\n<p>by <span class="printuser">A & B</span></p>'
    )
  } finally {
    client.request = rejectingClientRequest
  }
})
