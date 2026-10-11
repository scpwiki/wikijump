import assert from "node:assert/strict"
import test from "node:test"

import { formatWikidotFeedDate } from "../src/lib/server/list-pages-feed.ts"
import { buildWikidotSiteChangesFeedXml } from "../src/lib/server/site-changes-feed.ts"

const output = {
  site_name: 'SCP & "Test"',
  items: [
    {
      revision_id: 3_000_000_123,
      revision_number: 7,
      is_new_page: false,
      is_file: false,
      created_at: "2026-10-07T16:53:08Z",
      slug: "forum:page-discussion",
      title: 'A "page" & <title>',
      change_type: "source",
      user_id: 42,
      user_name: "Author & User",
      body_html: "<p>One ]]> two</p>"
    },
    {
      revision_id: 3_000_000_124,
      revision_number: 1,
      is_new_page: true,
      is_file: false,
      created_at: "2026-10-07T15:00:00Z",
      slug: "new-page",
      title: "New page",
      change_type: "new page",
      user_id: 43,
      user_name: "Another <author>",
      body_html: "<p>Fresh</p>"
    }
  ]
}

test("site changes RSS reports the frozen channel and newest-first bounded rows", () => {
  const xml = buildWikidotSiteChangesFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/site-changes.xml",
    output,
    new Date("2026-10-07T16:54:02Z")
  )

  assert.match(xml, /^<\?xml version="1\.0" encoding="UTF-8" \?>\n<rss version="2\.0" xmlns:content=/)
  assert.match(
    xml,
    /<title>Recent page changes from site "SCP &amp; &quot;Test&quot;" \(a Wikidot site\)<\/title>/
  )
  assert.match(xml, /<link>https:\/\/scp-wiki\.wikijump\.localhost<\/link>/)
  assert.match(xml, /<lastBuildDate>Wed, 07 Oct 2026 16:54:02 \+0000<\/lastBuildDate>/)
  assert.equal((xml.match(/<item>/gu) ?? []).length, 2)
  // newest first, with the revision identity in the GUID
  assert.match(
    xml,
    /<guid>http:\/\/scp-wiki\.wikijump\.localhost\/forum:page-discussion#revision-3000000123<\/guid>/
  )
  assert.match(xml, /<title>\&quot;A \&quot;page\&quot; \&amp; \&lt;title\&gt;\&quot; - source change<\/title>/)
  assert.match(
    xml,
    /<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum:page-discussion<\/link>/
  )
  assert.match(
    xml,
    /<description>Page: &lt;a href=&quot;https:\/\/scp-wiki\.wikijump\.localhost\/forum:page-discussion&quot;&gt;A &quot;page&quot; &amp; &lt;title&gt;&lt;\/a&gt;/
  )
  assert.match(xml, /Current revision number: rev\. 7/)
  assert.match(xml, /Date changed: Wed, 07 Oct 2026 16:53:08 \+0000/)
  assert.match(xml, /Change type: source/)
  assert.match(xml, /By: Author &amp; User/)
  assert.match(xml, /Page content preview: &lt;p&gt;One \]\]&gt; two&lt;\/p&gt;/)
  assert.match(xml, /<pubDate>Wed, 07 Oct 2026 16:53:08 \+0000<\/pubDate>/)
  // the preview body is carried as CDATA and stays well formed
  assert.match(
    xml,
    /<!\[CDATA\[\n\t\t\t\t<p>One \]\]\]\]><!\[CDATA\[> two<\/p>\n\t\t\t\t\]\]>/
  )
})

test("site changes RSS item titles use the per-row change type", () => {
  const xml = buildWikidotSiteChangesFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/site-changes.xml",
    output,
    new Date("2026-10-07T16:54:02Z")
  )

  assert.match(xml, /<title>\&quot;New page\&quot; - new page change<\/title>/)
  assert.match(xml, /Current revision number: new/)
  assert.match(xml, /<guid>http:\/\/scp-wiki\.wikijump\.localhost\/new-page#revision-3000000124<\/guid>/)
  assert.match(xml, /By: Another &lt;author&gt;/)
})

test("an empty site still produces the frozen channel shape", () => {
  const xml = buildWikidotSiteChangesFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/site-changes.xml",
    { site_name: "Empty", items: [] },
    new Date("2026-10-07T16:54:02Z")
  )

  assert.equal((xml.match(/<item>/gu) ?? []).length, 0)
  assert.match(xml, /<title>Recent page changes from site "Empty" \(a Wikidot site\)<\/title>/)
  // the shared Wikidot RSS date format is reused by the site changes feed
  assert.equal(
    formatWikidotFeedDate(new Date("2026-10-07T16:53:08Z")),
    "Wed, 07 Oct 2026 16:53:08 +0000"
  )
})
