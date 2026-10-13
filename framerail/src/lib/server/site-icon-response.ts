import { getSiteIcon } from "$lib/server/deepwell/site-icon"
import { loadSiteInfo } from "$lib/server/load/site-info"
import { preloadView } from "$lib/server/deepwell/views"
import {
  getPreloadBackendLocales,
  getPreloadRequestLocales
} from "$lib/server/load/preload"
import {
  selectSiteIconResponse,
  type StoredSiteIcon
} from "$lib/site-icon-response"
import type { SiteIconSourceKind } from "$lib/site-icon-source"
import type { Nullable, SiteModel } from "$lib/types"

/**
 * Serves a Wikidot icon route from materialized per-site bytes.
 *
 * Stored bytes are served directly so imported mirrors never fetch from
 * Wikidot at request time. Sources are never proxied here, so a
 * site-configured external URL cannot turn this route into a server-side
 * fetcher of author-supplied addresses.
 */
export async function siteIconResponse(
  request: Request,
  select: (site: SiteModel) => Nullable<string>,
  kind: SiteIconSourceKind,
  filename: string
): Promise<Response> {
  const { siteId } = loadSiteInfo(request.headers)
  // Deepwell rejects a preload with no locales, so this resolves them the same
  // way an ordinary page request does rather than sending an empty list.
  const locales = getPreloadBackendLocales(getPreloadRequestLocales(request))
  const preload = await preloadView(siteId, locales, undefined)
  const site = preload?.site ?? null

  let stored: Nullable<StoredSiteIcon> = null
  if (site !== null) {
    try {
      const icon = await getSiteIcon(siteId, kind)
      if (icon !== null) {
        stored = {
          mime: icon.mime,
          sha256Hex: icon.sha256_hex,
          data: new Uint8Array(icon.data)
        }
      }
    } catch {
      // Fall through to the source redirect or 404 below; a transient
      // store read must not turn the icon route into a 500.
      stored = null
    }
  }

  return selectSiteIconResponse(
    site,
    site ? select(site) : null,
    stored,
    kind,
    filename
  )
}
