import {
  buildWikidotInterwikiFrameHtml,
  buildWikidotInterwikiSourceUrl,
  fetchCromInterwikiPage
} from "$lib/wikidot/wikidot-interwiki"
import type { RequestHandler } from "./$types"

const FRAME_HEADERS = {
  "content-type": "text/html; charset=utf-8",
  "cache-control": "no-store"
}

export const GET: RequestHandler = async ({ url, fetch }) => {
  const lang = url.searchParams.get("lang") ?? ""
  const community = url.searchParams.get("community") ?? ""
  const pagename = url.searchParams.get("pagename") ?? ""
  const sourceUrl = buildWikidotInterwikiSourceUrl({
    community,
    lang,
    sourcePath: pagename
  })
  const page = sourceUrl ? await fetchCromInterwikiPage(fetch, sourceUrl) : null
  const body = buildWikidotInterwikiFrameHtml({ community, lang, pagename, page })

  return new Response(body, { headers: FRAME_HEADERS })
}

export const HEAD: RequestHandler = () => new Response(null, { headers: FRAME_HEADERS })
