// @ts-nocheck
import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))

let previousWorkingDirectory
let vite
let render
let appendPane
let backlinksPane
let client
let originalClientRequest
let pageEditAction
let pageBacklinksAction

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()

  ;({ render } = await vite.ssrLoadModule("svelte/server"))
  ;({ client } = await vite.ssrLoadModule("/src/lib/server/deepwell/index.ts"))
  originalClientRequest = client.request
  ;({ pageEditAction } = await vite.ssrLoadModule(
    "/src/lib/server/load/page/page-edit-actions.ts"
  ))
  ;({ pageBacklinksAction } = await vite.ssrLoadModule(
    "/src/lib/server/load/page/page-relation-actions.ts"
  ))
  ;({ default: appendPane } = await vite.ssrLoadModule(
    "/src/routes/[slug]/[...extra]/AppendPane.svelte"
  ))
  ;({ default: backlinksPane } = await vite.ssrLoadModule(
    "/src/routes/[slug]/[...extra]/BacklinksPane.svelte"
  ))
})

after(async () => {
  if (client && originalClientRequest) client.request = originalClientRequest
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

test("Append SSR exposes only new source while preserving the current revision fields", () => {
  const body = render(appendPane, {
    props: {
      data: {
        site: { site_id: 17 },
        page: { page_id: 42 },
        page_revision: {
          revision_id: 9,
          title: "Fixture title",
          alt_title: "Fixture alternate title",
          tags: ["alpha", "_hidden"]
        },
        wikitext: "Existing source\n",
        internationalization: {
          save: "Save",
          cancel: "Cancel"
        },
        wikidot_page_actions: { append: "Append" }
      },
      pagePaneState: "append"
    }
  }).body

  assert.match(body, /<form id="page-append"[^>]*action="\?\/edit"[^>]*method="POST"/u)
  assert.match(body, /<textarea[^>]*id="page-append-input"/u)
  assert.doesNotMatch(body, /Existing source<\/textarea>/u)
  assert.match(body, /name="title" type="hidden" value="Fixture title"/u)
  assert.match(body, /name="altTitle" type="hidden" value="Fixture alternate title"/u)
  assert.match(body, /name="tags" type="hidden" value="alpha _hidden"/u)
  assert.match(body, /name="lastRevisionId" type="hidden" value="9"/u)
  assert.match(body, /name="wikitext" type="hidden" value="Existing source\n"/u)
  assert.match(body, /value="Cancel"/u)
  assert.match(body, /value="Save"/u)
})

test("Append submission preserves metadata and commits only the appended source at the revision-bound edit action", async () => {
  const calls = []
  client.request = async (method, params, context) => {
    calls.push({ method, params, context })
    if (method === "session_get") {
      return {
        session_token: "append-session",
        user_id: 23,
        created_at: "2026-08-10T00:00:00Z",
        expires_at: "2026-08-11T00:00:00Z",
        ip_address: "192.0.2.42",
        user_agent: "append test",
        restricted: false
      }
    }
    if (method === "page_edit") {
      return { revision_id: 10, revision_number: 4, parser_errors: [] }
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  const fields = new FormData()
  for (const [name, value] of Object.entries({
    siteId: 17,
    pageId: 42,
    lastRevisionId: 9,
    title: "Fixture title",
    altTitle: "Fixture alternate title",
    wikitext: "Existing source\nAppended source",
    tags: "alpha _hidden",
    comments: ""
  })) {
    fields.set(name, String(value))
  }

  const result = await pageEditAction({
    request: new Request("https://wikijump.test/fixture?/edit", {
      method: "POST",
      body: fields,
      headers: {
        "X-Wikijump-Site-Id": "17",
        "X-Wikijump-Site-Slug": "test"
      }
    }),
    params: { slug: "fixture" },
    cookies: { get: () => "append-session" },
    getClientAddress: () => "192.0.2.42",
    locals: {
      requestContext: {
        sessionToken: "append-session",
        siteId: 17,
        page: "fixture"
      }
    }
  })

  assert.equal(result.res.revision_id, 10)
  assert.deepEqual(
    calls.map(({ method }) => method),
    ["session_get", "page_edit"]
  )
  assert.equal(calls[1].params.wikitext, "Existing source\r\nAppended source")
  assert.equal(calls[1].params.title, "Fixture title")
  assert.equal(calls[1].params.alt_title, "Fixture alternate title")
  assert.deepEqual(calls[1].params.tags, ["alpha", "_hidden"])
  assert.equal(calls[1].params.page, 42)
  assert.equal(calls[1].params.last_revision_id, 9)
  assert.equal(calls[1].params.user_id, 23)
})

test("saved Wikidot options expose Append and Backlinks in captured relative order", async () => {
  const source = await readFile(
    new URL("../src/routes/[slug]/[...extra]/PageView.svelte", import.meta.url),
    "utf8"
  )
  const appendIndex = source.indexOf('id="edit-append-button"')
  const backlinksIndex = source.indexOf('id="backlinks-button"')
  const sourceIndex = source.indexOf('id="view-source-button"')

  assert.ok(appendIndex >= 0)
  assert.ok(backlinksIndex > appendIndex)
  assert.ok(sourceIndex > backlinksIndex)
  assert.match(source, /edit-append-button[\s\S]*PagePane\.Append/u)
  assert.match(source, /backlinks-button[\s\S]*PagePane\.Backlinks/u)
})

test("saved Wikidot options expose More Options and View Source controls", async () => {
  const source = await readFile(
    new URL("../src/routes/[slug]/[...extra]/PageView.svelte", import.meta.url),
    "utf8"
  )
  const moreOptionsIndex = source.indexOf('id="more-options-button"')
  const viewSourceIndex = source.indexOf('id="view-source-button"')

  assert.ok(moreOptionsIndex >= 0)
  assert.ok(viewSourceIndex > moreOptionsIndex)
  assert.match(source, /more-options-button[\s\S]*toggleShowPageOptions/u)
  assert.match(source, /view-source-button[\s\S]*showSource\s*=\s*true/u)
})

test("Backlinks SSR exposes a lazy read pane without page identifiers", () => {
  const body = render(backlinksPane, {
    props: {
      data: {
        site: { site_id: 17 },
        page: { page_id: 42, slug: "fixture" },
        wikidot_page_actions: { backlinks: "Backlinks" }
      }
    }
  }).body

  assert.match(body, /<h1[^>]*>Backlinks<\/h1>/u)
  assert.match(body, /id="page-backlinks-list"/u)
  assert.match(body, /aria-live="polite"/u)
  assert.doesNotMatch(body, /page[_-]id/u)
})

test("Backlinks handler delegates the current route to the typed public Deepwell view", async () => {
  const calls = []
  client.request = async (method, params, context) => {
    calls.push({ method, params, context })
    if (method === "page_backlinks_view") {
      return [
        { slug: "alpha-linker", title: "Alpha Linker" },
        { slug: "beta-linker", title: "Beta Linker" }
      ]
    }
    throw new Error(`Unexpected Deepwell method ${method}`)
  }

  // What the pane actually submits is fetch("?/backlinks", { method: "POST",
  // body: "" }). A string body makes the browser send this content type, which
  // SvelteKit accepts for form actions. The read carries no page identifier:
  // the action resolves the route slug and trusted site context server-side.
  const request = new Request("https://wikijump.test/fixture?/backlinks", {
    method: "POST",
    body: "",
    headers: {
      "X-Wikijump-Site-Id": "17",
      "X-Wikijump-Site-Slug": "test"
    }
  })
  assert.equal(request.headers.get("content-type"), "text/plain;charset=UTF-8")

  const result = await pageBacklinksAction({
    request,
    params: { slug: "fixture" },
    locals: {
      requestContext: {
        siteId: 17,
        page: "fixture"
      }
    }
  })

  assert.deepEqual(result.res, [
    { slug: "alpha-linker", title: "Alpha Linker" },
    { slug: "beta-linker", title: "Beta Linker" }
  ])
  assert.deepEqual(calls, [
    {
      method: "page_backlinks_view",
      params: { site_id: 17, page: "fixture" },
      context: { siteId: 17, page: "fixture" }
    }
  ])
  assert.doesNotMatch(JSON.stringify(result.res), /page_id/u)
})
