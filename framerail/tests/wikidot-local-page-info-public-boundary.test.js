// @ts-nocheck
import assert from "node:assert/strict"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))

let previousWorkingDirectory
let vite
let render
let pageComponent
let buildWikidotPagePresentation
let PAGE_LAYOUT_CONTEXT_KEY
let Layout

before(async () => {
  previousWorkingDirectory = process.cwd()
  process.chdir(root)
  vite = await createTestViteServer()

  ;({ render } = await vite.ssrLoadModule("svelte/server"))
  ;({ default: pageComponent } = await vite.ssrLoadModule(
    "/src/routes/[slug]/[...extra]/PageView.svelte"
  ))
  ;({ buildWikidotPagePresentation } = await vite.ssrLoadModule(
    "/src/lib/server/load/page/page-presentation.ts"
  ))
  ;({ PAGE_LAYOUT_CONTEXT_KEY } = await vite.ssrLoadModule(
    "/src/lib/layout/page-layout-context.ts"
  ))
  ;({ Layout } = await vite.ssrLoadModule("/src/lib/types.ts"))
})

after(async () => {
  if (vite) await vite.close()
  if (previousWorkingDirectory) process.chdir(previousWorkingDirectory)
})

const localFoundPage = {
  type: "found",
  data: {
    page: {
      page_id: 42,
      slug: "local-page",
      from_wikidot: false,
      created_at: "2026-08-12T01:02:03Z",
      updated_at: "2026-08-13T04:05:06Z",
      discussion_thread_id: null
    },
    page_revision: {
      revision_id: 9,
      revision_number: 3,
      title: "Local page",
      tags: []
    },
    wikidot_snapshot: null,
    wikidot_breadcrumbs: [],
    page_rating: { enabled: false },
    page_discussion: { enabled: false },
    options: {},
    wikitext: "Local body",
    compiled_body_html: "<p>Local body</p>",
    compiled_body_styles: [],
    theme: {},
    legacy_actions: [],
    rate_actions: null,
    membership_actions: [],
    meta_tags: [],
    attributions: [],
    data_form: null
  }
}

test("Wikidot-layout SSR gives a locally authored found page Wikidot page-info grammar", () => {
  const presentation = buildWikidotPagePresentation(localFoundPage, {
    hasSession: false,
    siteLocale: "en",
    now: Date.parse("2026-08-13T06:05:06Z")
  })
  const body = render(pageComponent, {
    props: {
      data: {
        ...localFoundPage.data,
        ...presentation,
        site: { locale: "en" },
        internationalization: {
          "wiki-page-revision": "legacy revision",
          "wiki-page-last-edit": "legacy edit"
        }
      }
    },
    context: new Map([[PAGE_LAYOUT_CONTEXT_KEY, { current: Layout.WIKIDOT }]])
  }).body

  assert.match(
    body,
    /<div id="page-info"[^>]*>[\s\S]*?page revision: 3, last edited: 13 Aug 2026, 13:05 \(2 hours ago\)[\s\S]*?<\/div>/u
  )
})

test("Wikidot-layout SSR uses local page creation time when it has no update time", () => {
  const response = {
    ...localFoundPage,
    data: {
      ...localFoundPage.data,
      page: { ...localFoundPage.data.page, updated_at: null }
    }
  }
  const presentation = buildWikidotPagePresentation(response, {
    hasSession: false,
    siteLocale: "en",
    now: Date.parse("2026-08-12T03:02:03Z")
  })
  const body = render(pageComponent, {
    props: {
      data: {
        ...response.data,
        ...presentation,
        site: { locale: "en" },
        internationalization: {}
      }
    },
    context: new Map([[PAGE_LAYOUT_CONTEXT_KEY, { current: Layout.WIKIDOT }]])
  }).body

  assert.match(
    body,
    /<div id="page-info"[^>]*>[\s\S]*?page revision: 3, last edited: 12 Aug 2026, 10:02 \(2 hours ago\)[\s\S]*?<\/div>/u
  )
})

test("imported page-info remains sourced from its retained Wikidot snapshot", () => {
  const response = {
    ...localFoundPage,
    data: {
      ...localFoundPage.data,
      page: { ...localFoundPage.data.page, from_wikidot: true },
      wikidot_snapshot: {
        source_site: "scp-wiki",
        source_revision_count: 17,
        source_updated_at: "2024-03-27T20:18:44Z",
        imported_rating: 4,
        comments: 2
      }
    }
  }

  const presentation = buildWikidotPagePresentation(response, {
    hasSession: false,
    siteLocale: "en",
    now: Date.parse("2024-03-28T07:18:44+09:00")
  })

  assert.equal(
    presentation.wikidot_page_info,
    "page revision: 17, last edited: 28 Mar 2024, 05:18 (2 hours ago)"
  )
  assert.equal(presentation.wikidot_page_actions?.discuss, "Discuss (2)")
  assert.equal(presentation.wikidot_page_actions?.showDiscuss, true)

  const body = render(pageComponent, {
    props: {
      data: {
        ...response.data,
        ...presentation,
        site: { locale: "en" },
        internationalization: {}
      }
    },
    context: new Map([[PAGE_LAYOUT_CONTEXT_KEY, { current: Layout.WIKIDOT }]])
  }).body

  assert.match(body, /<a[^>]*id="discuss-button"[^>]*>Discuss \(2\)<\/a>/u)
  assert.doesNotMatch(body, /id="discuss-button"[^>]*onclick=/u)
})

test("imported Discuss is a plain link to an existing page discussion thread", () => {
  const response = {
    ...localFoundPage,
    data: {
      ...localFoundPage.data,
      page: {
        ...localFoundPage.data.page,
        slug: "scp-9506",
        from_wikidot: true,
        discussion_thread_id: 17768232
      },
      wikidot_snapshot: {
        source_site: "scp-wiki",
        source_revision_count: 49,
        source_updated_at: "2026-10-04T21:03:51Z",
        imported_rating: 461,
        comments: 95
      }
    }
  }

  const presentation = buildWikidotPagePresentation(response, {
    hasSession: false,
    siteLocale: "en",
    now: Date.parse("2026-10-12T00:00:00Z")
  })
  assert.equal(
    presentation.wikidot_page_actions?.discussHref,
    "/forum/t-17768232/scp-9506"
  )

  const body = render(pageComponent, {
    props: {
      data: {
        ...response.data,
        ...presentation,
        site: { locale: "en" },
        internationalization: {}
      }
    },
    context: new Map([[PAGE_LAYOUT_CONTEXT_KEY, { current: Layout.WIKIDOT }]])
  }).body

  assert.match(
    body,
    /<a href="\/forum\/t-17768232\/scp-9506" class="btn btn-default(?: svelte-[a-z0-9]+)?" id="discuss-button">Discuss \(95\)<\/a>/u
  )
  assert.doesNotMatch(body, /id="discuss-button"[^>]*onclick=/u)
})
