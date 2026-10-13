import type { SiteModel } from "$lib/types"

export type SiteIconSourceKind = "favicon" | "ios" | "windows"

type SiteIconSourceSite = Pick<SiteModel, "from_wikidot" | "slug">

const MAX_ICON_SOURCE_LENGTH = 2048
const SITE_SLUG = /^[a-z0-9-]+$/u
const ENCODED_LINE_BREAK = /%(?:0a|0d)/iu

const WIKIDOT_ROUTE_PREFIX: Partial<Record<SiteIconSourceKind, string>> = {
  favicon: "/local--favicon/",
  ios: "/local--iosicon/",
  windows: "/local--wp8icon/"
}

/**
 * Whether an icon source is a same-origin local file route.
 *
 * Only these sources can be served without depending on Wikidot at request
 * time; anything else needs materialized bytes to avoid hotlinking.
 */
export function isSiteLocalFileSource(source: string): boolean {
  return safeLocalFileSource(source)
}

function safeLocalFileSource(source: string): boolean {
  if (!source.startsWith("/local--files/")) return false

  let url: URL
  try {
    url = new URL(source, "https://wikijump.invalid")
  } catch {
    return false
  }

  return (
    url.pathname.startsWith("/local--files/") &&
    url.pathname.length > "/local--files/".length &&
    url.search === "" &&
    url.hash === ""
  )
}

function pathHasContent(pathname: string, prefix: string): boolean {
  return pathname.startsWith(prefix) && pathname.length > prefix.length
}

function sourceHasUnsafeText(source: string): boolean {
  for (const character of source) {
    const codePoint = character.codePointAt(0) ?? 0
    if (codePoint <= 0x1f || codePoint === 0x7f || character === "\\") return true
  }
  return ENCODED_LINE_BREAK.test(source)
}

/**
 * Return a redirect location only when an icon source belongs to the site.
 *
 * Local sites may refer to a same-origin file route. Imported sites may
 * also refer to the matching Wikidot icon route or the matching wdfiles
 * file host. Arbitrary external URLs remain unsupported until the
 * configured image is downloaded into local file storage.
 */
export function siteIconRedirectLocation(
  site: SiteIconSourceSite,
  source: string | null,
  kind: SiteIconSourceKind
): string | null {
  if (!source || source.length > MAX_ICON_SOURCE_LENGTH || sourceHasUnsafeText(source)) {
    return null
  }

  if (safeLocalFileSource(source)) return source
  if (!site.from_wikidot || !SITE_SLUG.test(site.slug)) return null

  let url: URL
  try {
    url = new URL(source)
  } catch {
    return null
  }

  if (
    url.protocol !== "https:" ||
    url.username !== "" ||
    url.password !== "" ||
    url.port !== "" ||
    url.search !== "" ||
    url.hash !== ""
  ) {
    return null
  }

  const slug = site.slug.toLowerCase()
  if (
    url.hostname === `${slug}.wdfiles.com` &&
    pathHasContent(url.pathname, "/local--files/")
  ) {
    return source
  }

  const wikidotPrefix = WIKIDOT_ROUTE_PREFIX[kind]
  if (
    wikidotPrefix &&
    url.hostname === `${slug}.wikidot.com` &&
    pathHasContent(url.pathname, wikidotPrefix)
  ) {
    return source
  }

  return null
}
