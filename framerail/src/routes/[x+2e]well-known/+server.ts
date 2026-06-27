import type { RequestHandler } from "./$types"

const WELL_KNOWN_NOT_CONFIGURED =
  "No .well-known resource is configured for this Framerail target.\n"

const WELL_KNOWN_HEADERS = {
  "content-type": "text/plain; charset=utf-8"
}

export const GET: RequestHandler = () =>
  new Response(WELL_KNOWN_NOT_CONFIGURED, {
    status: 404,
    headers: WELL_KNOWN_HEADERS
  })

export const HEAD: RequestHandler = () =>
  new Response(null, {
    status: 404,
    headers: WELL_KNOWN_HEADERS
  })
