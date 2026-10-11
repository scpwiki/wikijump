import type { RequestHandler } from "./$types"
import { buildWikidotStyleFrameHtml } from "$lib/wikidot/wikidot-styleframe"

const FRAME_HEADERS = {
  "content-type": "text/html; charset=utf-8",
  "cache-control": "no-store"
}

export const GET: RequestHandler = () =>
  new Response(buildWikidotStyleFrameHtml(), { headers: FRAME_HEADERS })

export const HEAD: RequestHandler = () => new Response(null, { headers: FRAME_HEADERS })
