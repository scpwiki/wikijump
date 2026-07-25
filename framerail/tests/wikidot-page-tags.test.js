import { strict as assert } from "node:assert"
import test from "node:test"

import { buildWikidotPageTagsHtml } from "../src/lib/wikidot/wikidot-page-tags.js"

test("renders imported Wikidot page tag links without visible separators", () => {
  const html = buildWikidotPageTagsHtml(["_cc", "_licensebox", "alive"])

  assert.equal(
    html,
    '<a href="/system:page-tags/tag/_cc#pages">_cc</a><a href="/system:page-tags/tag/_licensebox#pages">_licensebox</a><a href="/system:page-tags/tag/alive#pages">alive</a>'
  )
  assert.equal(/>\s+</.test(html), false)
})

test("escapes imported Wikidot page tag labels and hrefs", () => {
  assert.equal(
    buildWikidotPageTagsHtml(["tag&<\"'"], (tag) => `/tags/${tag}#pages`),
    '<a href="/tags/tag&amp;&lt;&quot;&#39;#pages">tag&amp;&lt;&quot;&#39;</a>'
  )
})
