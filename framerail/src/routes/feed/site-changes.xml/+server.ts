import {
  WIKIDOT_SITE_CHANGES_FEED_HEADERS,
  buildWikidotSiteChangesFeedXml
} from "$lib/server/site-changes-feed"
import { wikidotSiteChangesFeed } from "$lib/server/deepwell/page"
import { loadSiteInfo } from "$lib/server/load/site-info"

import type { WikidotSiteChangesFeedOutput } from "$lib/server/site-changes-feed"
import type { RequestHandler } from "./$types"

export const GET: RequestHandler = async ({ request, cookies }) => {
  const { siteId } = loadSiteInfo(request.headers)
  const output: WikidotSiteChangesFeedOutput | null = await wikidotSiteChangesFeed(
    siteId,
    {
      siteId,
      sessionToken: cookies.get("wikijump_token")
    }
  )
  // A site the viewer cannot see has no public feed, so the route answers the
  // same way as any other missing feed.
  if (!output) return new Response("Not found", { status: 404 })

  return new Response(buildWikidotSiteChangesFeedXml(request.url, output), {
    headers: WIKIDOT_SITE_CHANGES_FEED_HEADERS
  })
}
