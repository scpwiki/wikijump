import { wikidotSitemap } from "$lib/server/deepwell/page"
import { loadSiteInfo } from "$lib/server/load/site-info"
import {
  buildSitemapUrlsetXml,
  InvalidSitemapShardPath,
  parseSitemapShardNumber,
  shardSitemapEntries,
  SITEMAP_XML_HEADERS
} from "$lib/server/sitemap"

import type { RequestHandler } from "./$types"

export const GET: RequestHandler = async ({ request, cookies, params }) => {
  let shard: number
  try {
    shard = parseSitemapShardNumber(params.n)
  } catch (error) {
    if (error instanceof InvalidSitemapShardPath) {
      return new Response("Not found", { status: 404 })
    }
    throw error
  }

  const { siteId } = loadSiteInfo(request.headers)
  const output = await wikidotSitemap(siteId, {
    siteId,
    sessionToken: cookies.get("wikijump_token")
  })
  // A site the viewer cannot see has no public sitemap, so the route answers
  // the same way as any other missing page.
  if (!output) return new Response("Not found", { status: 404 })

  const shards = shardSitemapEntries(output.entries)
  const entries = shards[shard - 1]
  if (!entries) return new Response("Not found", { status: 404 })

  const origin = new URL(request.url).origin
  return new Response(buildSitemapUrlsetXml(origin, entries), {
    headers: SITEMAP_XML_HEADERS
  })
}
