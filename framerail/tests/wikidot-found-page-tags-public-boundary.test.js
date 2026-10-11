// @ts-nocheck
import assert from "node:assert/strict"
import { fileURLToPath } from "node:url"
import { after, before, test } from "node:test"
import { parse } from "svelte/compiler"

import { createTestViteServer } from "./vite-test-server.js"

const root = fileURLToPath(new URL("..", import.meta.url))

let previousWorkingDirectory
let vite
let render
let pageComponent
let pageTagsComponent
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
  ;({ default: pageTagsComponent } = await vite.ssrLoadModule(
    "/src/routes/[slug]/[...extra]/WikidotFoundPageTags.svelte"
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

const foundPageData = {
  page: {
    page_id: 42,
    slug: "tag-holder",
    from_wikidot: true,
    created_at: "2026-07-27T08:00:00Z",
    updated_at: "2026-07-27T08:00:00Z",
    discussion_thread_id: null
  },
  page_revision: {
    revision_id: 9,
    revision_number: 1,
    title: "ListPages tag holder",
    tags: ["_lp-holder-hidden", "lp-same-a-20260727", "lp-same-b-20260727"]
  },
  site: { site_id: 1, name: "Sandbox For Codex", locale: "en" },
  wikidot_snapshot: null,
  wikidot_breadcrumbs: [],
  page_rating: { enabled: false },
  page_discussion: { enabled: false },
  options: {},
  wikitext: "Holder body",
  compiled_body_html: "<p>Holder <!--page-source-note-->body</p>",
  compiled_body_styles: [],
  theme: {},
  legacy_actions: [],
  rate_actions: null,
  membership_actions: [],
  meta_tags: [],
  attributions: [],
  data_form: null,
  internationalization: {}
}

const renderFoundPage = (data) =>
  render(pageComponent, {
    props: { data },
    context: new Map([[PAGE_LAYOUT_CONTEXT_KEY, { current: Layout.WIKIDOT }]])
  }).body

const assertTagTree = (body, tags, hidden = false) => {
  const tree = body.match(
    /<div class="page-tags(?: hidden)?"><span>[\s\S]*?<\/span><\/div>/u
  )
  assert.ok(tree, "tag links must retain their native div/span wrappers")
  const root = parse(tree[0]).html.children
  assert.equal(root.length, 1)
  assert.equal(root[0].type, "Element")
  assert.equal(root[0].name, "div")
  assert.equal(
    root[0].attributes.find((attribute) => attribute.name === "class")?.value[0]?.data,
    `page-tags${hidden ? " hidden" : ""}`
  )

  const spans = root[0].children.filter((child) => child.type === "Element")
  assert.equal(spans.length, 1)
  assert.equal(spans[0].name, "span")
  assert.ok(
    spans[0].children.every(
      (child) => child.type === "Comment" || child.type === "Element"
    )
  )

  const links = spans[0].children.filter((child) => child.type === "Element")
  assert.deepEqual(
    links.map((link) => ({
      href: link.attributes.find((attribute) => attribute.name === "href")?.value[0]
        ?.data,
      label: link.children
        .filter((child) => child.type === "Text")
        .map((child) => child.data)
        .join("")
    })),
    tags.map((tag) => ({ href: `/system:page-tags/tag/${tag}#pages`, label: tag }))
  )
}

test("default Wikidot found-page route SSR preserves the native span around ordered tag links", () => {
  const body = renderFoundPage(foundPageData)

  assert.match(body, /<!--page-source-note-->/u)
  assertTagTree(body, foundPageData.page_revision.tags)
})

test("default Wikidot tag leaf SSR preserves the span around supplied revision tags", () => {
  const body = render(pageTagsComponent, {
    props: { tags: ["lp-range-20260727", "older-revision-tag"], hidden: false }
  }).body

  assertTagTree(body, ["lp-range-20260727", "older-revision-tag"])
})

test("tagless found-page route SSR omits page-tags", () => {
  const body = renderFoundPage({
    ...foundPageData,
    page_revision: { ...foundPageData.page_revision, tags: [] }
  })

  assert.doesNotMatch(body, /class="page-tags(?:\s|")/u)
})

test("editing found-page route SSR preserves the hidden page-tags state", () => {
  const body = renderFoundPage({
    ...foundPageData,
    options: { edit: true },
    data_form: { fields: [] }
  })

  assertTagTree(body, foundPageData.page_revision.tags, true)
})

// Svelte SSR leaves hydration comments between siblings; drop them (and
// whitespace) so sibling order can be compared without a backtracking regex.
const withoutMarkupNoise = (html) =>
  html
    .split("<!--")
    .map((part, index) => (index === 0 ? part : part.slice(part.indexOf("-->") + 3)))
    .join("")
    .trim()

const assertPageInfoBreakOrder = (body) => {
  const breakTag = '<div id="page-info-break"></div>'
  const breakIndex = body.indexOf(breakTag)
  assert.notEqual(breakIndex, -1, "found pages must render Wikidot's #page-info-break")
  assert.equal(body.indexOf('id="page-info-break"', breakIndex + breakTag.length), -1)

  const tagsIndex = body.indexOf('<div class="page-tags')
  if (tagsIndex !== -1) {
    assert.ok(tagsIndex < breakIndex, ".page-tags precedes #page-info-break")
  }

  const optionsIndex = body.indexOf('<div id="page-options-container"')
  assert.ok(
    breakIndex < optionsIndex,
    "#page-info-break precedes #page-options-container"
  )
  assert.equal(
    withoutMarkupNoise(body.slice(breakIndex, optionsIndex)),
    breakTag,
    "#page-info-break sits directly between page tags and page options"
  )
}

test("found-page route SSR places #page-info-break between page tags and page options", () => {
  const body = renderFoundPage(foundPageData)

  const tagsEnd = body.indexOf("</span></div>") + "</span></div>".length
  const breakIndex = body.indexOf('<div id="page-info-break"></div>')
  assert.ok(tagsEnd > 0 && tagsEnd <= breakIndex)
  assert.equal(withoutMarkupNoise(body.slice(tagsEnd, breakIndex)), "")
  assertPageInfoBreakOrder(body)
})

test("tagless found-page route SSR still renders #page-info-break before page options", () => {
  const body = renderFoundPage({
    ...foundPageData,
    page_revision: { ...foundPageData.page_revision, tags: [] }
  })

  assertPageInfoBreakOrder(body)
})

test("editing found-page route SSR keeps #page-info-break before page options", () => {
  const body = renderFoundPage({
    ...foundPageData,
    options: { edit: true },
    data_form: { fields: [] }
  })

  assertPageInfoBreakOrder(body)
})
