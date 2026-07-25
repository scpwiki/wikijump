import { loadPreload, pageRouteProvidesPreload } from "$lib/server/load/preload"

import type { PreloadData } from "$lib/server/deepwell/views"
import type { LayoutServerLoad } from "./$types"

export const load: LayoutServerLoad<PreloadData> = async ({
  request,
  cookies,
  route,
  locals
}) => {
  if (pageRouteProvidesPreload(route.id)) {
    return {} as PreloadData
  }

  const preload = await loadPreload(request, cookies)
  locals.siteLocale = preload.site.locale
  return preload
}
