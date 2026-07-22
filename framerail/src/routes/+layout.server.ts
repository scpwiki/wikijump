import { loadPreload, pageRouteProvidesPreload } from "$lib/server/load/preload"

export async function load({ request, cookies, route, locals }) {
  if (pageRouteProvidesPreload(route.id)) {
    return {}
  }

  const preload = await loadPreload(request, cookies)
  locals.siteLocale = preload.site.locale
  return preload
}
