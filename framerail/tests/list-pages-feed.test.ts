import assert from "node:assert/strict"
import test from "node:test"

import {
  InvalidWikidotListPagesFeedPath,
  buildWikidotListPagesFeedXml,
  formatWikidotFeedDate,
  parseWikidotListPagesFeedPath,
  wikidotListPagesFeedErrorBody,
  wikidotListPagesFeedSelectorError
} from "../src/lib/server/list-pages-feed.ts"

test("parses Wikidot feed path pairs with form-style spaces and last-value wins", () => {
  assert.deepEqual(
    parseWikidotListPagesFeedPath(
      "https://sandbox.example/feed/pages/t/First/t/Last+Title/d/A%26B/h/blog%3A_start/category/%2A%2C-deleted/tags/%2Bred%2C-blue/unknown/value"
    ),
    {
      title: "Last Title",
      description: "A&B",
      home: "blog:_start",
      selectors: {
        category: "*,-deleted",
        tags: "+red,-blue"
      }
    }
  )
})

test("pairs arguments sequentially and treats an orphan key as empty", () => {
  assert.deepEqual(
    parseWikidotListPagesFeedPath("https://sandbox.example/feed/pages/t/d/foo"),
    {
      title: "d",
      description: "",
      home: null,
      selectors: {}
    }
  )
  assert.deepEqual(
    parseWikidotListPagesFeedPath("https://sandbox.example/feed/pages/t"),
    {
      title: "",
      description: "",
      home: null,
      selectors: {}
    }
  )
})

test("rejects malformed percent escapes and paths outside the feed route", () => {
  assert.equal(parseWikidotListPagesFeedPath("https://sandbox.example/feed/pages"), null)
  assert.throws(
    () => parseWikidotListPagesFeedPath("https://sandbox.example/feed/pages/t/%ZZ"),
    InvalidWikidotListPagesFeedPath
  )
})

test("recognizes the feed endpoint's narrower selector grammar", () => {
  for (const pagetype of ["*", "hidden", "normal"]) {
    assert.equal(wikidotListPagesFeedSelectorError({ pagetype }), null)
  }
  assert.equal(
    wikidotListPagesFeedSelectorError({ pagetype: "all" }),
    "Invalid pagetype attribute."
  )

  for (const rating of ["0", "-1", ">0", ">=0", "<0", "<=0", "<>0", "=0"]) {
    assert.equal(wikidotListPagesFeedSelectorError({ rating }), null)
  }
  for (const rating of ["+1", "1.5", "!=0", "0foo"]) {
    assert.equal(
      wikidotListPagesFeedSelectorError({ rating }),
      "Invalid rating argument."
    )
  }

  assert.equal(wikidotListPagesFeedSelectorError({ range: "." }), null)
  assert.equal(
    wikidotListPagesFeedSelectorError({ range: "before" }),
    "Invalid range argument."
  )
})

test("renders RSS metadata, items, escaped descriptions, and safe CDATA", () => {
  const path = parseWikidotListPagesFeedPath(
    "https://sandbox.example/feed/pages/t/%3CFeed%26/d/%22Description%22/h/blog%3A_start"
  )
  assert.ok(path)
  const xml = buildWikidotListPagesFeedXml(
    "https://sandbox.example/feed/pages/t/test",
    path,
    {
      items: [
        {
          slug: "category:one&two",
          title: "<One>",
          created_at: "2026-07-22T23:23:22Z",
          body_html: "<p>body ]]> & text</p>",
          created_by_html: '<span class="printuser">A & B</span>'
        }
      ]
    },
    new Date("2026-07-27T09:53:12Z")
  )

  assert.ok(xml.includes("<title>&lt;Feed&amp;</title>"))
  assert.ok(xml.includes("<description>&quot;Description&quot;</description>"))
  assert.ok(xml.includes("<link>blog:_start</link>"))
  assert.ok(xml.includes("<guid>https://sandbox.example/category:one&amp;two</guid>"))
  assert.ok(xml.includes("<title>&lt;One&gt;</title>"))
  assert.ok(xml.includes("Wed, 22 Jul 2026 23:23:22 +0000"))
  assert.ok(xml.includes("Mon, 27 Jul 2026 09:53:12 +0000"))
  assert.ok(xml.includes("&lt;p&gt;body ]]&gt; &amp; text&lt;/p&gt;"))
  assert.ok(xml.includes("]]]]><![CDATA[>"))
  assert.doesNotMatch(xml, /<script>/)
})

test("formats the live generic feed error without exposing a stack trace", () => {
  assert.equal(
    wikidotListPagesFeedErrorBody("Invalid rating argument."),
    "A nasty error has occurred. If the problem repeats, please fill (if possible) a bug report.<br/><br/>Invalid rating argument."
  )
  assert.equal(
    formatWikidotFeedDate(new Date("2026-01-02T03:04:05Z")),
    "Fri, 02 Jan 2026 03:04:05 +0000"
  )
})
