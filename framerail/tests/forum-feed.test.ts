import assert from "node:assert/strict"
import test from "node:test"

import {
  buildWikidotForumFeedXml,
  formatWikidotForumFeedDate,
  parseWikidotForumFeed,
  wikidotForumFeedHeaders
} from "../src/lib/server/forum-feed.ts"

const output = {
  scope: "site" as const,
  site_name: 'SCP & "Test"',
  site_description: "Secure <safe> & sound",
  items: [
    {
      forum_post_id: 91,
      forum_thread_id: 42,
      created_at: "2026-10-07T12:21:30Z",
      title: "Post & <title>",
      thread_title: "Thread's title",
      thread_slug: "thread-title",
      page_slug: null,
      forum_group_id: 2,
      forum_group_name: "Group",
      forum_category_id: 8,
      forum_category_name: "Category",
      forum_category_slug: "category",
      author_user_id: 17,
      author_name: "Author & User",
      content_html: "<p>One ]]> two</p>"
    }
  ]
}

test("forum RSS uses local identifiers, public channel metadata, and safe CDATA", () => {
  const xml = buildWikidotForumFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/forum/posts.xml",
    "posts",
    output,
    new Date("2026-10-07T16:54:02Z")
  )

  assert.match(xml, /^<\?xml version="1\.0" encoding="UTF-8" \?>\n<rss version="2\.0" xmlns:content=/)
  assert.match(xml, /<title>SCP &amp; &quot;Test&quot; - new forum posts<\/title>/)
  assert.match(xml, /<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/start<\/link>/)
  assert.match(xml, /<description>Posts in forums of the site &quot;SCP &amp; &quot;Test&quot;&quot; - Secure &lt;safe&gt; &amp; sound<\/description>/)
  assert.match(xml, /<guid>http:\/\/scp-wiki\.wikijump\.localhost\/forum\/t-42#post-91<\/guid>/)
  assert.match(xml, /<title\/>\n\t\t\t<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/t-42\/thread-title#post-91<\/link>\n\t\t\t<description\/>/)
  assert.match(xml, /<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/t-42\/thread-title#post-91<\/link>/)
  assert.match(xml, /<wikidot:authorUserId>17<\/wikidot:authorUserId>/)
  assert.match(xml, /<pubDate>Wed, 07 Oct 2026 12:21:30 \+0000<\/pubDate>/)
  assert.match(xml, /<!\[CDATA\[\n\t\t\t\t<p>One \]\]\]\]><!\[CDATA\[> two<\/p>/)
  assert.match(xml, /Forum category: <a href="https:\/\/scp-wiki\.wikijump\.localhost\/forum\/c-8">Group \/ Category<\/a>/)
  assert.match(xml, /Forum thread: <a href="https:\/\/scp-wiki\.wikijump\.localhost\/forum\/t-42\/thread-title">Thread&#039;s title<\/a>/)
})

test("thread RSS uses the thread identity and UTC dates", () => {
  const xml = buildWikidotForumFeedXml(
    "https://example.wikijump.localhost/feed/forum/threads.xml",
    "threads",
    output,
    new Date("2026-10-07T16:53:15Z")
  )

  assert.match(xml, /<title>Thread&#039;s title<\/title>/)
  assert.match(xml, /<guid>http:\/\/example\.wikijump\.localhost\/forum\/t-42<\/guid>/)
  assert.match(xml, /<link>https:\/\/example\.wikijump\.localhost\/forum\/t-42\/thread-title<\/link>/)
  assert.equal(formatWikidotForumFeedDate(new Date("2026-10-07T16:53:15Z")), "Wed, 07 Oct 2026 16:53:15 +0000")
})

test("forum RSS responses use the observed XML content type and cache policy", () => {
  const headers = wikidotForumFeedHeaders()
  assert.equal(headers.get("content-type"), "text/xml;charset=utf-8")
  assert.equal(headers.get("cache-control"), "no-cache, must-revalidate")
  assert.equal(headers.get("expires"), "Mon, 26 Jul 1997 05:00:00 GMT")
})

const scopedCategoryOutput = {
  ...output,
  scope: "category" as const,
  forum_group_name: "Group",
  forum_category_id: 50864,
  forum_category_name: "Per page discussions"
}

const scopedThreadOutput = {
  ...output,
  scope: "thread" as const,
  forum_thread_id: 17768232,
  forum_thread_title: "National Fog Safety Initiative"
}

test("scoped category and thread RSS links resolve legacy Wikidot feed names", () => {
  assert.deepEqual(parseWikidotForumFeed("threads.xml"), { kind: "threads" })
  assert.deepEqual(parseWikidotForumFeed("posts.xml"), { kind: "posts" })
  assert.deepEqual(parseWikidotForumFeed("ct-50864.xml"), { kind: "threads", categoryId: 50864 })
  assert.deepEqual(parseWikidotForumFeed("cp-50864.xml"), { kind: "posts", categoryId: 50864 })
  assert.deepEqual(parseWikidotForumFeed("t-17768232.xml"), { kind: "posts", threadId: 17768232 })
  // The route family owns every /feed/forum/* name: unsupported spellings fail
  // closed instead of falling through to the wiki-page view.
  assert.equal(parseWikidotForumFeed("ct-.xml"), null)
  assert.equal(parseWikidotForumFeed("ct-abc.xml"), null)
  assert.equal(parseWikidotForumFeed("ct-1"), null)
  assert.equal(parseWikidotForumFeed("t-0.xml"), null)
  assert.equal(parseWikidotForumFeed("t--5.xml"), null)
  assert.equal(parseWikidotForumFeed("other.xml"), null)
  assert.equal(parseWikidotForumFeed(""), null)
})

test("scoped category RSS names the category and stays on the local origin", () => {
  const xml = buildWikidotForumFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/forum/ct-50864.xml",
    "threads",
    scopedCategoryOutput,
    new Date("2026-10-07T16:54:02Z")
  )

  assert.match(xml, /<title>SCP &amp; &quot;Test&quot; - new forum threads in category &quot;Group \/ Per page discussions&quot;<\/title>/)
  assert.match(xml, /<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/c-50864<\/link>/)
  assert.match(xml, /<description>Threads in forum category &quot;Group \/ Per page discussions&quot; of the site/)
  assert.match(xml, /<guid>http:\/\/scp-wiki\.wikijump\.localhost\/forum\/t-42<\/guid>/)
})

test("scoped category post RSS uses the posts channel spelling", () => {
  const xml = buildWikidotForumFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/forum/cp-50864.xml",
    "posts",
    scopedCategoryOutput,
    new Date("2026-10-07T16:54:02Z")
  )

  assert.match(xml, /<title>SCP &amp; &quot;Test&quot; - new forum posts in category &quot;Group \/ Per page discussions&quot;<\/title>/)
  assert.match(xml, /<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/c-50864<\/link>/)
  assert.match(xml, /<title\/>\n\t\t\t<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/t-42\/thread-title#post-91<\/link>/)
})

test("thread RSS uses the observed page discussion channel title", () => {
  const xml = buildWikidotForumFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/forum/t-17768232.xml",
    "posts",
    scopedThreadOutput,
    new Date("2026-10-07T16:54:02Z")
  )

  assert.match(xml, /<title>Comments for page &quot;National Fog Safety Initiative&quot;<\/title>/)
  assert.match(xml, /<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/t-17768232<\/link>/)
  assert.match(xml, /<description>Posts in forum thread &quot;National Fog Safety Initiative&quot; of the site/)
  assert.match(xml, /#post-91<\/link>/)
})

test("site RSS channel stays the observed site-wide pair", () => {
  const xml = buildWikidotForumFeedXml(
    "https://scp-wiki.wikijump.localhost/feed/forum/posts.xml",
    "posts",
    output,
    new Date("2026-10-07T16:54:02Z")
  )

  assert.match(xml, /<title>SCP &amp; &quot;Test&quot; - new forum posts<\/title>/)
  assert.match(xml, /<link>https:\/\/scp-wiki\.wikijump\.localhost\/forum\/start<\/link>/)
  assert.match(xml, /<description>Posts in forums of the site &quot;SCP &amp; &quot;Test&quot;&quot; - Secure &lt;safe&gt; &amp; sound<\/description>/)
})
