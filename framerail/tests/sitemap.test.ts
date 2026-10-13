import assert from "node:assert/strict"
import test from "node:test"

import {
  buildSitemapIndexXml,
  buildSitemapUrlsetXml,
  InvalidSitemapEntry,
  InvalidSitemapShardPath,
  formatSitemapLastmod,
  parseSitemapShardNumber,
  shardSitemapEntries,
  SITEMAP_ENTRIES_PER_SHARD
} from "../src/lib/server/sitemap.ts"

test("parses shard numbers starting at 1 and rejects anything else", () => {
  assert.equal(parseSitemapShardNumber("1"), 1)
  assert.equal(parseSitemapShardNumber("12"), 12)
  for (const bad of ["0", "01", "-1", "1.5", "1.xml", "", "one"]) {
    assert.throws(() => parseSitemapShardNumber(bad), InvalidSitemapShardPath)
  }
})

test("formats RFC 3339 timestamps as sitemap lastmod dates", () => {
  assert.equal(formatSitemapLastmod("2026-05-22T00:00:00+00:00"), "2026-05-22")
  assert.equal(formatSitemapLastmod("2026-07-22T14:03:10Z"), "2026-07-22")
  assert.throws(() => formatSitemapLastmod("2026-05-22"), InvalidSitemapEntry)
  assert.throws(() => formatSitemapLastmod("not a date"), InvalidSitemapEntry)
})

test("builds a sitemap index pointing at each shard page", () => {
  assert.equal(
    buildSitemapIndexXml("https://scp-wiki.example", 2),
    `<?xml version="1.0" encoding="UTF-8"?>\n` +
      `<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n` +
      `\t<sitemap>\n\t\t<loc>https://scp-wiki.example/sitemap_page_1.xml</loc>\n\t</sitemap>\n` +
      `\t<sitemap>\n\t\t<loc>https://scp-wiki.example/sitemap_page_2.xml</loc>\n\t</sitemap>\n` +
      `</sitemapindex>`
  )
})

test("builds a shard urlset with escaped locations and lastmod dates", () => {
  assert.equal(
    buildSitemapUrlsetXml("https://scp-wiki.example", [
      { slug: "scp-9506", last_modified: "2026-05-22T00:00:00+00:00" },
      { slug: "a&b", last_modified: "2026-07-22T14:03:10+00:00" }
    ]),
    `<?xml version="1.0" encoding="UTF-8"?>\n` +
      `<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n` +
      `\t<url>\n\t\t<loc>https://scp-wiki.example/scp-9506</loc>\n\t\t<lastmod>2026-05-22</lastmod>\n\t</url>\n` +
      `\t<url>\n\t\t<loc>https://scp-wiki.example/a&amp;b</loc>\n\t\t<lastmod>2026-07-22</lastmod>\n\t</url>\n` +
      `</urlset>`
  )
})

test("splits entries into bounded shards", () => {
  const entries = Array.from({ length: SITEMAP_ENTRIES_PER_SHARD + 1 }, (_, index) => ({
    slug: `page-${index}`,
    last_modified: "2026-05-22T00:00:00+00:00"
  }))
  const shards = shardSitemapEntries(entries)
  assert.equal(shards.length, 2)
  assert.equal(shards[0].length, SITEMAP_ENTRIES_PER_SHARD)
  assert.equal(shards[1].length, 1)
  assert.deepEqual(shardSitemapEntries([]), [])
})
