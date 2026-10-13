import type { RequestHandler } from "./$types"
import { siteIconResponse } from "$lib/server/site-icon-response"

export const GET: RequestHandler = ({ params, request }) =>
  siteIconResponse(request, (site) => site.ios_icon_source, "ios", params.filename)

export const HEAD: RequestHandler = ({ params, request }) =>
  siteIconResponse(request, (site) => site.ios_icon_source, "ios", params.filename)
