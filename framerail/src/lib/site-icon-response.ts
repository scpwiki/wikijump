import {
  IOS_ICON_DECLARATIONS,
  SITE_ICON_CACHE_CONTROL,
  WINDOWS_TILE_FILENAME
} from "./site-icons.ts"
import {
  isSiteLocalFileSource,
  siteIconRedirectLocation,
  type SiteIconSourceKind
} from "./site-icon-source.ts"

import type { Nullable, SiteModel } from "$lib/types"

export interface StoredSiteIcon {
  mime: string
  sha256Hex: string
  data: Uint8Array
}

type IconRouteSite = Pick<SiteModel, "from_wikidot" | "slug">

const NOT_CONFIGURED_HEADERS = {
  "cache-control": SITE_ICON_CACHE_CONTROL,
  "content-type": "text/plain; charset=utf-8"
}

/** The only filenames Wikidot declares (and serves) per icon route. */
const CANONICAL_FILENAMES: Record<SiteIconSourceKind, readonly string[]> = {
  favicon: ["favicon.gif"],
  ios: IOS_ICON_DECLARATIONS.map((declaration) => declaration.filename),
  windows: [WINDOWS_TILE_FILENAME]
}

function notConfigured(): Response {
  return new Response("This site has no icon configured.\n", {
    status: 404,
    headers: NOT_CONFIGURED_HEADERS
  })
}

/**
 * Serve materialized icon bytes with Wikidot's header shape.
 *
 * The content type keeps Wikidot's `charset=utf-8` suffix (observed on both
 * the favicon and Windows tile routes), and the ETag is the stored SHA-256
 * so refreshes change the cache identity.
 */
export function storedSiteIconResponse(stored: StoredSiteIcon): Response {
  return new Response(stored.data, {
    status: 200,
    headers: {
      "cache-control": SITE_ICON_CACHE_CONTROL,
      "content-type": `${stored.mime}; charset=utf-8`,
      etag: `"${stored.sha256Hex}"`
    }
  })
}

/**
 * Select the compatibility-route response for a site icon.
 *
 * Stored bytes win so imported mirrors never depend on Wikidot at request
 * time. Without bytes, only same-origin local file sources redirect; a
 * Wikidot or WDFiles source with nothing materialized is a 404 rather than
 * a hotlink back to the source origin.
 */
export function selectSiteIconResponse(
  site: Nullable<IconRouteSite>,
  configuredSource: Nullable<string>,
  stored: Nullable<StoredSiteIcon>,
  kind: SiteIconSourceKind,
  filename: string
): Response {
  if (!CANONICAL_FILENAMES[kind].includes(filename)) return notConfigured()
  if (stored !== null) return storedSiteIconResponse(stored)

  const location =
    site !== null ? siteIconRedirectLocation(site, configuredSource, kind) : null
  if (location !== null && isSiteLocalFileSource(location)) {
    return new Response(null, {
      status: 302,
      headers: { "cache-control": SITE_ICON_CACHE_CONTROL, location }
    })
  }

  return notConfigured()
}
