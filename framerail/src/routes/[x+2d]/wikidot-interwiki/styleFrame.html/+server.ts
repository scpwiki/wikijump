import type { RequestHandler } from "./$types"
import { buildWikidotStyleFrameHtml } from "$lib/wikidot/wikidot-styleframe"

const FRAME_HEADERS = {
  "content-type": "text/html; charset=utf-8",
  "cache-control": "no-store"
}

export const GET: RequestHandler = ({ url }) => {
  const css = url.searchParams.get("css") ?? ""
  const themes = url.searchParams.getAll("theme")
  const priority = url.searchParams.get("priority") ?? ""
  const body = buildWikidotStyleFrameHtml({
    priority,
    themes,
    css,
    origin: url.origin
  })

  return new Response(body, { headers: FRAME_HEADERS })
}

export const HEAD: RequestHandler = () => new Response(null, { headers: FRAME_HEADERS })
