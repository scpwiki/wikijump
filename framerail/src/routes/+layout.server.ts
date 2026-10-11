import { readNativeFooterDestinationOverrides } from "$lib/platform-footer-links"
import { loadPreload, pageRouteProvidesPreload } from "$lib/server/load/preload"

import type { PreloadData } from "$lib/server/deepwell/views"
import type { LayoutServerLoad } from "./$types"

export const load: LayoutServerLoad<PreloadData> = async ({
  request,
  cookies,
  route,
  locals
}) => {
  // The native footer's Docs / Terms / Privacy / Security destinations are
  // instance-level, not per-wiki. They are resolved once here so every route
  // that renders the native layout, including the routes whose own load owns
  // the preload, exposes the same platform destinations.
  const nativeFooterDestinations = readNativeFooterDestinationOverrides(process.env)

  if (pageRouteProvidesPreload(route.id)) {
    return { native_footer_destinations: nativeFooterDestinations } as PreloadData
  }

  const preload = await loadPreload(request, cookies)
  locals.siteLocale = preload.site.locale
  return { ...preload, native_footer_destinations: nativeFooterDestinations }
}
