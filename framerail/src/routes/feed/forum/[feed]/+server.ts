import {
  parseWikidotForumFeed,
  wikidotForumFeedHeaders,
  buildWikidotForumFeedXml
} from "$lib/server/forum-feed"
import { wikidotForumFeed } from "$lib/server/deepwell/forum"
import { loadSiteInfo } from "$lib/server/load/site-info"

import type { RequestHandler } from "./$types"

export const GET: RequestHandler = async ({ request, params, cookies }) => {
  const target = parseWikidotForumFeed(params.feed)
  // The route family owns every `/feed/forum/*` path, so an unsupported or
  // malformed name must not fall through to the wiki-page view.
  if (!target) return new Response("Not found", { status: 404 })

  const { siteId } = loadSiteInfo(request.headers)
  const output = await wikidotForumFeed(
    siteId,
    target.kind,
    {
      siteId,
      sessionToken: cookies.get("wikijump_token")
    },
    {
      categoryId: target.categoryId,
      threadId: target.threadId
    }
  )
  // A hidden, deleted or nonexistent category/thread is indistinguishable
  // from an unknown one.
  if (!output) return new Response("Not found", { status: 404 })

  return new Response(buildWikidotForumFeedXml(request.url, target.kind, output), {
    headers: wikidotForumFeedHeaders()
  })
}
