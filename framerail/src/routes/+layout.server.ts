import { loadPreload, pageRouteProvidesPreload } from "$lib/server/load/preload"

export async function load({ request, cookies, route }) {
  if (pageRouteProvidesPreload(route.id)) {
    return {}
  }

  return loadPreload(request, cookies)
}
