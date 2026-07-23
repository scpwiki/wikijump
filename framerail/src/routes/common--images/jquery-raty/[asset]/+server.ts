import { wikidotRatyAsset } from "$lib/server/wikidot-raty-assets"

import type { RequestHandler } from "./$types"

export const GET: RequestHandler = ({ params }) => {
  const body = wikidotRatyAsset(params.asset)
  if (!body) return new Response("Unknown Wikidot rating asset", { status: 404 })

  return new Response(body, {
    headers: {
      "cache-control": "public, max-age=31536000, immutable",
      "content-type": "image/png"
    }
  })
}
