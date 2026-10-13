import { wikidotSitemap } from "$lib/server/deepwell/page"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { buildSitemapIndexXml, shardSitemapEntries, SITEMAP_XML_HEADERS } from "$lib/server/sitemap"

import type { RequestHandler } from "./$types"

export const GET: RequestHandler = async ({ request, cookies }) => {
  const { siteId } = loadSiteInfo(request.headers)
  const output = await wikidotSitemap(siteId, {
    siteId,
    sessionToken: cookies.get("wikijump_token")
  })
  // A site the viewer cannot see has no public sitemap, so the route answers
  // the same way as any other missing page.
  if (!output) return new Response("Not found", { status: 404 })

  const shards = shardSitemapEntries(output.entries)
  const origin = new URL(request.url).origin
  return new Response(buildSitemapIndexXml(origin, shards.length), {
    headers: SITEMAP_XML_HEADERS
  })
}
