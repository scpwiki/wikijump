import {
  InvalidWikidotListPagesFeedPath,
  WIKIDOT_LIST_PAGES_FEED_HEADERS,
  buildWikidotListPagesFeedXml,
  parseWikidotListPagesFeedPath,
  wikidotListPagesFeedErrorBody,
  wikidotListPagesFeedSelectorError
} from "$lib/server/list-pages-feed"
import { client } from "$lib/server/deepwell"
import { loadSiteInfo } from "$lib/server/load/site-info"

import type { WikidotListPagesFeedOutput } from "$lib/server/list-pages-feed"
import type { RequestHandler } from "./$types"

export const GET: RequestHandler = async ({ request }) => {
  let path
  try {
    path = parseWikidotListPagesFeedPath(request.url)
  } catch (error) {
    if (error instanceof InvalidWikidotListPagesFeedPath) {
      return new Response(error.message, { status: 400 })
    }
    throw error
  }
  if (!path) return new Response("Not found", { status: 404 })

  const selectorError = wikidotListPagesFeedSelectorError(path.selectors)
  if (selectorError) {
    return new Response(wikidotListPagesFeedErrorBody(selectorError), {
      headers: WIKIDOT_LIST_PAGES_FEED_HEADERS
    })
  }

  const { siteId } = loadSiteInfo(request.headers)
  const output = (await client.request(
    "wikidot_list_pages_feed",
    {
      site_id: siteId,
      ...path.selectors
    },
    { siteId }
  )) as WikidotListPagesFeedOutput
  return new Response(buildWikidotListPagesFeedXml(request.url, path, output), {
    headers: WIKIDOT_LIST_PAGES_FEED_HEADERS
  })
}
