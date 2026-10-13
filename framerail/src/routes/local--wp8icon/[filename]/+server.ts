import type { RequestHandler } from "./$types"
import { siteIconResponse } from "$lib/server/site-icon-response"

export const GET: RequestHandler = ({ params, request }) =>
  siteIconResponse(request, (site) => site.windows_tile_source, "windows", params.filename)

export const HEAD: RequestHandler = ({ params, request }) =>
  siteIconResponse(request, (site) => site.windows_tile_source, "windows", params.filename)
