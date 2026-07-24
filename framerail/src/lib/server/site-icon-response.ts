import { loadSiteInfo } from "$lib/server/load/site-info"
import { preloadView } from "$lib/server/deepwell/views"

const NOT_CONFIGURED_HEADERS = {
  "content-type": "text/plain; charset=utf-8"
}

/**
 * Redirects a Wikidot icon route to the site's configured icon source.
 *
 * The configured source is fetched by the browser rather than proxied
 * here, so a site-configured external URL cannot turn this route into a
 * server-side fetcher of author-supplied addresses.
 */
export async function siteIconResponse(
  request: Request,
  select: (site: {
    favicon_source: string | null
    ios_icon_source: string | null
  }) => string | null
): Promise<Response> {
  const { siteId } = loadSiteInfo(request.headers)
  const preload = await preloadView(siteId, [], undefined)
  const source = select(preload?.site ?? { favicon_source: null, ios_icon_source: null })

  if (!source) {
    return new Response("This site has no icon configured.\n", {
      status: 404,
      headers: NOT_CONFIGURED_HEADERS
    })
  }

  return new Response(null, {
    status: 302,
    headers: { location: source }
  })
}
