import { handleAjaxModuleConnectorRequest } from "$lib/server/ajax-module-connector.js"
import { client } from "$lib/server/deepwell"
import { loadSiteInfo } from "$lib/server/load/site-info"

import type { RequestHandler } from "./$types"

interface WikidotListPagesModuleOutput {
  body: string
}

export const POST: RequestHandler = async ({ request }) => {
  const { siteId } = loadSiteInfo(request.headers)
  return handleAjaxModuleConnectorRequest(request, {
    siteId,
    renderListPages: ({
      siteId,
      moduleBody,
      parameters
    }: {
      siteId: number
      moduleBody: string
      parameters: Record<string, string>
    }) =>
      client.request<WikidotListPagesModuleOutput>(
        "wikidot_list_pages_module",
        {
          site_id: siteId,
          module_body: moduleBody,
          parameters
        },
        { siteId }
      )
  })
}
